"""Internal helper for check-p3-readonly-volume.ps1. Uses only the synthetic fixture."""
import base64
import ctypes
from ctypes import wintypes
from contextlib import ExitStack, closing, contextmanager
import hashlib
import hmac
import io
import multiprocessing
import msvcrt
import os
import secrets
import json
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
import uuid
import zipfile


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


P3_SERVICE = Path(__file__).resolve().parent / "native-window/target/release/pipe-service.exe"
P3_WINDOW = P3_SERVICE.with_name("native-leaf.exe")
P3_SCENE = ("Szene A", "Text aus der Testkopie.", 0)


P3_SOUTH = ("Szene B", "Wind an der Suedkueste.", 0)


# Known synthetic examples. A real project's working set must come from its verified database.
EXTRA_FILES = {
    "backups/alt.txt": (1, b"synthetic backup"),
    "exports/entwurf.txt": (2, b"synthetic export"),
    ".git/HEAD": (4, b"ref: refs/heads/synthetic\n"),
    "notiz.txt": (8, b"synthetic private note"),
}
ARCHIVE_HELPER = Path(__file__).resolve().parents[2] / "target/release/kraqel-archiv.exe"


def archive_name(metadata):
    encoding = metadata[6].get("format") if len(metadata) == 7 else None
    require(encoding in (None, "age-v1-scrypt"), "unsupported archive format")
    return "zusatzbestand.zip.age" if encoding else "zusatzbestand.zip"


def expected_scene(metadata):
    if len(metadata) == 5:
        return P3_SCENE
    require(len(metadata) in (6, 7) and metadata[5] in ("north", "south"), "unknown fixture scene")
    return P3_SOUTH if metadata[5] == "south" else P3_SCENE


def scene_frame(metadata, row):
    title, text, revision = row
    return (uuid.UUID(metadata[0]).bytes + uuid.UUID(metadata[1]).bytes + revision.to_bytes(4, "little")
            + title.encode("utf-8") + b"\0" + text.encode("utf-8"))


kernel = ctypes.WinDLL("kernel32", use_last_error=True)
open_file = kernel.CreateFileW
open_file.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                          wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
open_file.restype = wintypes.HANDLE

@contextmanager
def pinned_directory(path):
    handle = open_file(str(path), 0x80, 3, None, 3, 0x02000000 | 0x00200000, None)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        yield handle
    finally:
        kernel.CloseHandle(wintypes.HANDLE(handle))


@contextmanager
def volume_guard(volume, readonly):
    with pinned_directory(volume) as handle:
        flags = wintypes.DWORD()
        label = ctypes.create_unicode_buffer(64)
        if not kernel.GetVolumeInformationByHandleW(wintypes.HANDLE(handle), label, len(label),
                                                   None, None, ctypes.byref(flags), None, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        require(label.value == "KRAQEL-P3", "foreign volume")
        require(bool(flags.value & 0x80000) == readonly, "unexpected volume write protection")
        yield


class StreamInfo(ctypes.Structure):
    _fields_ = [("size", ctypes.c_longlong), ("name", wintypes.WCHAR * 296)]


def check_entry(path):
    info = path.lstat()
    require(not info.st_file_attributes & 0x400, "reparse point")
    # A plain-file inventory would silently omit NTFS alternate data streams.
    first = kernel.FindFirstStreamW
    first.restype = wintypes.HANDLE
    stream = StreamInfo()
    handle = first(str(path), 0, ctypes.byref(stream), 0)
    if handle == ctypes.c_void_p(-1).value:
        if ctypes.get_last_error() == 38 and path.is_dir():  # ERROR_HANDLE_EOF
            return info
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        while True:
            require(stream.name == "::$DATA", "unaccounted named stream")
            if not kernel.FindNextStreamW(wintypes.HANDLE(handle), ctypes.byref(stream)):
                if ctypes.get_last_error() != 38:
                    raise ctypes.WinError(ctypes.get_last_error())
                break
    finally:
        kernel.FindClose(wintypes.HANDLE(handle))
    return info


def inventory(root):
    entries = {}
    check_entry(root)

    def visit(directory):
        for item in directory.iterdir():
            info = check_entry(item)
            name = item.relative_to(root).as_posix()
            if item.is_dir():
                entries[name] = None
                visit(item)
            else:
                require(item.is_file() and info.st_nlink == 1, "unexpected fixture object")
                entries[name] = hashlib.sha256(item.read_bytes()).hexdigest()
    visit(root)
    return entries


def capture_writable_fixture(root, selected_root, read_oplock, during_copy=None, selected_entries=None, include=None, exact=False):
    """Capture one bounded logical instant. Return bytes only after every outstanding lease passes."""
    checks, files, snapshot = [], [], {}
    observed = set()
    remaining = 32 * 1024 * 1024
    with ExitStack() as pins:
        def visit(path, depth):
            require(depth <= 32 and len(checks) < 256, "fixture tree budget exceeded")
            before = path.lstat()
            require(not before.st_file_attributes & (0x400 | 0x1000 | 0x40000 | 0x400000),
                    "reparse or remotely backed entry")
            directory = bool(before.st_file_attributes & 0x10)
            # Deny rename/deletion on each opened object; acquire a directory lease before listing.
            raw = open_file(str(path), 0x80, 3 if directory else 1, None, 3, 0x42200000, None)
            if raw == ctypes.c_void_p(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
            try:
                fd = msvcrt.open_osfhandle(raw, os.O_RDONLY | os.O_BINARY)
            except OSError:
                kernel.CloseHandle(wintypes.HANDLE(raw))
                raise
            pins.callback(os.close, fd)
            if depth == 0:
                filesystem = ctypes.create_unicode_buffer(32)
                require(len(path.drive) == 2 and kernel.GetDriveTypeW(path.anchor) == 3,
                        "fixture requires a local fixed drive")
                if not kernel.GetVolumeInformationByHandleW(wintypes.HANDLE(raw), None, 0, None, None,
                                                           None, filesystem, len(filesystem)):
                    raise ctypes.WinError(ctypes.get_last_error())
                require(filesystem.value.upper() == "NTFS", "fixture requires NTFS")
            info = os.fstat(fd)
            require(os.path.samestat(before, info) and not info.st_file_attributes & (0x400 | 0x1000 | 0x40000 | 0x400000),
                    "changed capture identity")
            if depth == 0:
                require(os.path.samestat(selected_root, info), "selected project directory changed")
            elif selected_entries is not None:
                require(path in selected_entries and os.path.samestat(selected_entries[path], info),
                        "selected source entry changed")
            checks.append(pins.enter_context(read_oplock(raw)))
            check_entry(path)
            name = path.relative_to(root).as_posix()
            observed.add(path)
            included = include is None or name in include
            if exact and depth:
                require(included, "unexpected prepared entry")
            if directory:
                if depth and included:
                    snapshot[name] = None
                for child in path.iterdir():
                    visit(child, depth + 1)
            else:
                require(info.st_nlink == 1 and path.is_file(), "unsupported capture file")
                if not included:
                    return
                reader = pins.enter_context(path.open("rb"))
                require(os.path.samestat(info, os.fstat(reader.fileno())), "changed capture file")
                files.append((name, reader, info.st_size))

        visit(root, 0)
        if selected_entries is not None:
            require(observed == set(selected_entries) | {root}, "selected source entry missing")
        for index, (name, reader, length) in enumerate(files):
            require(length <= min(remaining, 8 * 1024 * 1024), "fixture byte budget exceeded")
            data = reader.read(length + 1)
            require(len(data) == length, "changed capture size")
            snapshot[name] = data
            remaining -= length
            if index == 0 and during_copy:
                during_copy()
        # The snapshot instant is after the last read, before the first final lease check.
        for verify in checks:
            verify()
    return snapshot


def initialize_fixture_schema(db):
    migrations = sorted((Path(__file__).resolve().parents[2] / "crates/storage/migrations").glob("*.sql"))
    require([m.name[:4] for m in migrations] == [f"{i:04d}" for i in range(1, 7)], "unexpected fixture schema")
    for migration in migrations:
        db.executescript(migration.read_text(encoding="utf-8"))


def check_working_schema(db):
    # Only objects produced by the six supported migrations belong in this preview.
    query = "SELECT type,name,tbl_name,sql FROM main.sqlite_schema ORDER BY type,name"
    with closing(sqlite3.connect(":memory:")) as reference:
        initialize_fixture_schema(reference)
        expected = reference.execute(query).fetchall()
    require(db.execute("PRAGMA user_version").fetchone() == (6,)
            and db.execute(query).fetchmany(len(expected) + 1) == expected,
            "unsupported project schema")


def rebuild_working_database(source):
    """Re-encode the live rows using only the schema created by our migrations."""
    with closing(sqlite3.connect(":memory:")) as target:
        target.execute("PRAGMA temp_store=MEMORY")
        initialize_fixture_schema(target)
        page_size = target.execute("PRAGMA page_size").fetchone()[0]
        target.execute(f"PRAGMA max_page_count={8 * 1024 * 1024 // page_size}")
        target.execute("PRAGMA foreign_keys=ON")
        target.execute("BEGIN")
        target.execute("PRAGMA defer_foreign_keys=ON")
        # Schema creation order puts media and style rows before their validation triggers need them.
        # The trusted target supplies every identifier. FTS shadow tables are derived data.
        tables = target.execute("SELECT s.name FROM sqlite_schema s JOIN pragma_table_list p "
                                "ON p.name=s.name AND p.schema='main' WHERE p.type='table' "
                                "AND s.type='table' AND s.name NOT GLOB 'sqlite_*' ORDER BY s.rowid").fetchall()
        for (table,) in tables:
            query = f'SELECT rowid,* FROM "{table}" ORDER BY rowid'
            rows = source.execute(query)
            fields = target.execute(query + " LIMIT 0").description
            parameters = ",".join("?" for _ in fields)
            columns = ",".join(f'"{c[0]}"' for c in fields)
            target.executemany(f'INSERT INTO "{table}" ({columns}) VALUES ({parameters})', rows)
        target.commit()
        for (table,) in tables:
            query = f'SELECT rowid,* FROM "{table}" ORDER BY rowid'
            require(all(a == b for a, b in zip(source.execute(query), target.execute(query), strict=True)),
                    "logical row mismatch")
        # Document insert triggers built the search index exclusively from current text.
        target.execute("INSERT INTO document_fts(document_fts,rank) VALUES('integrity-check',1)")
        target.commit()
        require(target.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
                and target.execute("PRAGMA foreign_key_check").fetchone() is None, "invalid logical copy")
        return target.serialize()


def selected_working_files(root, directory_id, entries, read_oplock, metadata):
    # Resolve rows against already enumerated entries. Never use a database string to open a path.
    database_names = {"kraqel.sqlite3", "kraqel.sqlite3-wal", "kraqel.sqlite3-shm"}
    snapshot = capture_writable_fixture(root, directory_id, read_oplock, selected_entries=entries,
                                        include=database_names)
    require("kraqel.sqlite3" in snapshot and all(isinstance(b, bytes) for b in snapshot.values()),
            "invalid database inventory")
    files = {path.relative_to(root).as_posix(): stat for path, stat in entries.items()
             if not stat.st_file_attributes & 0x10}
    require("kraqel.sqlite3-journal" not in files, "unsupported SQLite sidecars")
    database_hashes = {name: hashlib.sha256(data).hexdigest() for name, data in snapshot.items()}
    working = set(database_hashes)
    hashes = dict(database_hashes)
    # Read the hash-bound WAL view with write/delete denied on all SQLite files.
    del snapshot
    with fixture_database(root, database_hashes) as db:
        check_working_schema(db)
        require(db.execute("SELECT id FROM project LIMIT 2").fetchall() == [(root.name,)],
                "unsupported media inventory")
        rows = db.execute("SELECT project_id,relative_path,thumbnail_path,checksum,byte_count "
                          "FROM media_asset LIMIT 257").fetchall()
        require(len(rows) <= 256, "media inventory budget exceeded")
        for project, image, thumb, digest, size in rows:
            require(project == root.name and image in files and thumb in files
                    and image.startswith("assets/") and not image.startswith("assets/thumbnails/")
                    and thumb.startswith("assets/thumbnails/") and image not in hashes
                    and files[image].st_size == size and re.fullmatch("[a-f0-9]{64}", digest),
                    "invalid media reference")
            working.update((image, thumb))
            hashes[image] = digest
        verify_scene_view(db, metadata, metadata[3])
        rebuilt = rebuild_working_database(db)
    # The picker lists these exact supplemental names. A new unlisted file requires a new decision.
    require(set(files) - working == set(EXTRA_FILES), "unreviewed supplemental inventory")
    for name in tuple(working):
        working.update(parent.as_posix() for parent in Path(name).parents if parent != Path("."))
    if (root / "empty") in entries and entries[root / "empty"].st_file_attributes & 0x10:
        working.add("empty")
    return working, hashes, rebuilt


def selection_digest(selected, candidate, metadata):
    require(len(selected) == 2 and all(type(n) is int and n > 0 for n in selected),
            "missing selected directory identity")
    payload = json.dumps([selected, object_id(candidate), metadata], ensure_ascii=True,
                         sort_keys=True, separators=(",", ":")).encode("ascii")
    return hashlib.sha256(b"KRAQEL-P3-SELECTED-SOURCE-v1\0" + payload).digest()


def verify_selection(fixture, candidate, metadata, selected):
    with native_authority(fixture / "selection-install", False) as authority:
        authority.attest(NativeAuthority.domains[0], selection_digest(selected, candidate, metadata[:4] + metadata[5:]), False)

    if len(metadata) == 7:
        approved_archive(candidate.parent, metadata)


def require_logical_copy(metadata):
    # A historical byte-copy receipt cannot authorize a new logical preview.
    record = metadata[6].get("workingCopy", {}) if len(metadata) == 7 else {}
    digest_field = {"schema6-logical-v1": "sourceInventorySha256",
                    "desktop-prepared-v1": "manifestSha256"}.get(record.get("format"))
    require(digest_field and re.fullmatch("[0-9a-f]{64}", record.get(digest_field, ""))
            and not {"kraqel.sqlite3-wal", "kraqel.sqlite3-shm"} & metadata[3].keys(),
            "new preview requires logical copy evidence")


def approved_archive(parent, metadata):
    path = parent / archive_name(metadata)
    record = metadata[6] if len(metadata) == 7 else None
    if not record or not record["groups"]:
        require(not os.path.lexists(path), "unapproved archive")
        return None
    limit = 32 * 1024 * 1024 + (64 * 1024 if record.get("format") else 0)
    before = check_entry(path)
    require(before.st_nlink == 1 and path.is_file(), "archive changed")
    with path.open("rb") as stream:
        require(os.path.samestat(before, os.fstat(stream.fileno())), "archive replaced")
        data = stream.read(limit + 1)
    require(len(data) <= limit and hashlib.sha256(data).hexdigest() == record["sha256"],
            "archive changed")
    if record.get("format") == "age-v1-scrypt":
        require(data.startswith(b"age-encryption.org/v1\n"), "invalid encrypted envelope")
    else:
        verify_zip(data, record["groups"])
    return data


def verify_zip(data, groups):
    expected = {"files/" + name: content for name, (group, content) in EXTRA_FILES.items()
                if groups & group}
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        require(sorted(archive.namelist()) == sorted(expected), "archive selection mismatch")
        for name, content in expected.items():
            require(archive.read(name) == content, "archive entry mismatch")


class UnownedFixtureFile(RuntimeError):
    """The file at this name no longer matches the generated fixture."""


def only_default_stream(raw):
    # FILE_STREAM_INFO contains a linked list; a generated file has just ::$DATA.
    buffer = ctypes.create_string_buffer(512)
    if not kernel.GetFileInformationByHandleEx(wintypes.HANDLE(raw), 7, buffer, len(buffer)):
        return False
    data = buffer.raw
    name = "::$DATA".encode("utf-16-le")
    return (int.from_bytes(data[:4], "little") == 0
            and int.from_bytes(data[4:8], "little") == len(name)
            and data[24:24 + len(name)] == name)


@contextmanager
def pending_fixture_deletion(files):
    """Windows closes these exact synthetic files if the controller dies in the picker."""
    with ExitStack() as pins:
        handles = []
        for path, identity, digest in files:
            raw = open_file(str(path), 0x80010000, 0, None, 3, 0x00200000, None)
            if raw == ctypes.c_void_p(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
            try:
                fd = msvcrt.open_osfhandle(raw, os.O_RDONLY | os.O_BINARY)
            except OSError:
                kernel.CloseHandle(wintypes.HANDLE(raw))
                raise
            file = pins.enter_context(os.fdopen(fd, "rb"))
            info = os.fstat(fd)
            if (not os.path.samestat(identity, info) or info.st_nlink != 1
                    or info.st_file_attributes & 0x410 or not only_default_stream(raw)
                    or hashlib.file_digest(file, "sha256").hexdigest() != digest):
                raise UnownedFixtureFile("source cleanup identity mismatch")
            handles.append(wintypes.HANDLE(raw))

        for handle in handles:
            set_delete_disposition(handle, True)

        def retain():
            for handle in handles:
                set_delete_disposition(handle, False)

        yield retain


def set_delete_disposition(handle, delete):
    flag = ctypes.c_ubyte(delete)
    if not kernel.SetFileInformationByHandle(handle, 4, ctypes.byref(flag), ctypes.sizeof(flag)):
        raise ctypes.WinError(ctypes.get_last_error())


def remove_empty_fixture_directory(path, identity):
    raw = open_file(str(path), 0x10080, 3, None, 3, 0x02200000, None)
    if raw == ctypes.c_void_p(-1).value:
        if ctypes.get_last_error() in (2, 3):
            return
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        fd = msvcrt.open_osfhandle(raw, os.O_RDONLY)
    except OSError:
        kernel.CloseHandle(wintypes.HANDLE(raw))
        raise
    try:
        info = os.fstat(fd)
        if os.path.samestat(info, identity) and info.st_file_attributes & 0x410 == 0x10:
            try:
                set_delete_disposition(wintypes.HANDLE(raw), True)
            except OSError as exc:
                if exc.winerror != 145:  # A new entry keeps this directory and its ancestors alive.
                    raise
    finally:
        os.close(fd)


@contextmanager
def selection_sources(interactive):
    if not interactive:
        with tempfile.TemporaryDirectory(prefix="kraqel-p3-writable-") as temp:
            yield Path(temp), {}, []
        return
    temp = Path(tempfile.gettempdir()) / ("kraqel-p3-writable-" + uuid.uuid4().hex)
    temp.mkdir()
    files = {}
    directories = [(temp, temp.lstat())]
    try:
        yield temp, files, directories
    finally:
        for entries in files.values():
            for entry in entries:
                try:
                    with pending_fixture_deletion([entry]):
                        pass
                except (FileNotFoundError, UnownedFixtureFile):
                    pass
        for directory, identity in reversed(directories):
            remove_empty_fixture_directory(directory, identity)


def report_native_root(process, image):
    if os.environ.get("KRAQEL_P3_ETW_ROOTS") == "1":
        print(f"P3-ETW {process.pid} {image.name}", flush=True)


def prepare_captured_project(target, fixture, interactive_selection=False):
    import runpy
    read_oplock = runpy.run_path(str(Path(__file__).with_name("p3_directory_oplock.py")))["read_oplock"]
    with selection_sources(interactive_selection) as (temp, generated_files, directories), ExitStack() as pins:
        pins.enter_context(pinned_directory(temp))
        choices = {}
        groups = 0
        handles = []
        catalog = temp / "projects"
        catalog.mkdir()
        if interactive_selection:
            directories.append((catalog, catalog.lstat()))
        pins.enter_context(pinned_directory(catalog))
        for name in (("Projekt Nord", "Projekt Sued") if interactive_selection else ("project",)):
            identity = str(uuid.uuid4())
            root = catalog / identity
            metadata = seed_writable_project(root, identity, P3_SOUTH if name == "Projekt Sued" else P3_SCENE)
            if interactive_selection:
                for relative, (_, content) in EXTRA_FILES.items():
                    path = root / relative
                    path.parent.mkdir(exist_ok=True)
                    with path.open("xb") as stream:
                        stream.write(content)
                for directory in (root, root / "empty", root / "assets",
                                  root / "assets/nested", root / "assets/thumbnails",
                                  root / "backups", root / "exports", root / ".git"):
                    directories.append((directory, directory.lstat()))
                generated_files[name] = []
                for relative, digest in metadata[3].items():
                    if digest is None:
                        continue
                    path = root / relative
                    generated_files[name].append((path, path.lstat(), digest))
                for relative, (_, content) in EXTRA_FILES.items():
                    path = root / relative
                    generated_files[name].append((path, path.lstat(), hashlib.sha256(content).hexdigest()))
            handles.append(pins.enter_context(pinned_directory(root)))
            choices[name] = (metadata, root, root.lstat())
        if interactive_selection:
            picker = Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe"
            with subprocess.Popen([picker, "broker-store", "inspect-p3-fixture-catalog",
                                   temp.name.removeprefix("kraqel-p3-writable-")],
                                  stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                  stderr=subprocess.DEVNULL) as probe:
                report_native_root(probe, picker)
                output, _ = probe.communicate()
                require(probe.returncode == 0 and json.loads(output) ==
                        {"candidates": sorted(choice[0][0] for choice in choices.values())},
                        "uncertain project catalog")
            import _winapi
            process = _winapi.GetCurrentProcess()
            with ExitStack() as protected:
                retain = {name: protected.enter_context(pending_fixture_deletion(files))
                          for name, files in generated_files.items()}
                with ExitStack() as inherited:
                    duplicates = []
                    for handle in handles:
                        duplicate = _winapi.DuplicateHandle(process, handle, process, 0x80, True, 0)
                        inherited.callback(_winapi.CloseHandle, duplicate)
                        duplicates.append(duplicate)
                    owner = _winapi.DuplicateHandle(process, process, process, 0x100000, True, 0)  # SYNCHRONIZE
                    inherited.callback(_winapi.CloseHandle, owner)
                    duplicates.append(owner)
                    startup = subprocess.STARTUPINFO(lpAttributeList={"handle_list": duplicates})
                    dialog = subprocess.Popen([picker, "select-p3-fixture", *map(str, duplicates)],
                                              stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                              stdin=subprocess.DEVNULL, startupinfo=startup, close_fds=True)
                with dialog:
                    report_native_root(dialog, picker)
                    output, _ = dialog.communicate()
                    require(dialog.returncode == 0, "source selection unavailable")
                reply = json.loads(output)
                if reply == {"selection": "cancel"}:
                    return False
                names = {"north": "Projekt Nord", "south": "Projekt Sued"}
                require(isinstance(reply, dict) and reply.get("selection") in names, "unknown source selection")
                name = names[reply["selection"]]
                stat = choices[name][2]
                groups = reply.get("archiveGroups")
                require(type(groups) is int and 0 <= groups <= 15, "invalid archive selection")
                require(reply == {"selection": reply["selection"], "archiveGroups": groups,
                                  "directory": [stat.st_dev, f"{stat.st_ino:032x}"]},
                        "selected directory identity mismatch")
                retain[name]()
        else:
            name = "project"
        metadata, root, selected_root = choices[name]

        selected_entries = None
        if interactive_selection:
            selected_entries = {path: identity for path, identity, _ in generated_files[name]}
            selected_entries.update((path, identity) for path, identity in directories
                                    if root in path.parents)
        variant = [("south" if name == "Projekt Sued" else "north"),
                   {"groups": groups, "sha256": None, "format": "age-v1-scrypt"}] if interactive_selection else []
        extras = {relative for relative, (group, _) in EXTRA_FILES.items() if groups & group}
        planned = dict(metadata[3])

        selected = object_id(root)
        require(selected == [selected_root.st_dev, selected_root.st_ino], "selected root replaced")
        if interactive_selection:
            working, required_hashes, rebuilt = selected_working_files(
                root, selected_root, selected_entries, read_oplock, [*metadata, selected, *variant])
        else:
            working, required_hashes = set(planned), {}

        def capture(after_first=None):
            include = working | extras if interactive_selection else None
            snapshot = capture_writable_fixture(root, selected_root, read_oplock, after_first,
                                                selected_entries, include)
            archive_bytes = {relative: snapshot.pop(relative) for relative in sorted(extras)}
            require(all(data == EXTRA_FILES[relative][1] for relative, data in archive_bytes.items()),
                    "selected archive entry changed")
            expected = {name: None if data is None else hashlib.sha256(data).hexdigest()
                        for name, data in snapshot.items()}
            if interactive_selection:
                require(set(expected) == working and expected == planned and all(expected.get(n) == digest
                        for n, digest in required_hashes.items()), "working inventory changed")
                # Bind the verified physical input and the distinct logical output to one selection.
                variant[1]["workingCopy"] = {
                    "format": "schema6-logical-v1",
                    "sourceInventorySha256": hashlib.sha256(json.dumps(expected, sort_keys=True,
                            separators=(",", ":")).encode("ascii")).hexdigest(),
                }
                for sidecar in ("kraqel.sqlite3-wal", "kraqel.sqlite3-shm"):
                    snapshot.pop(sidecar, None)
                    expected.pop(sidecar, None)
                snapshot["kraqel.sqlite3"] = rebuilt
                expected["kraqel.sqlite3"] = hashlib.sha256(rebuilt).hexdigest()
            else:
                require(expected == planned, "incomplete writable capture")
            parent = target.parent.parent if interactive_selection else target.parent
            with tempfile.TemporaryDirectory(dir=parent) as work:
                package = Path(work) / "package"
                package.mkdir()
                staged = package / "project"
                staged.mkdir()
                for name, data in snapshot.items():
                    if data is None:
                        (staged / name).mkdir()
                    else:
                        with (staged / name).open("xb") as dst:
                            dst.write(data)
                require(inventory(staged) == expected, "capture staging changed")
                if archive_bytes:
                    require(ARCHIVE_HELPER.is_file(), "Zuerst cargo build --locked --release -p kraqel-archiv ausführen.")
                    archive = package / archive_name([*metadata, selected, *variant])
                    with io.BytesIO() as plaintext:
                        with zipfile.ZipFile(plaintext, "w", compression=zipfile.ZIP_STORED) as output:
                            for relative, data in archive_bytes.items():
                                output.writestr("files/" + relative, data)
                        verify_zip(plaintext.getvalue(), groups)
                        subprocess.run([ARCHIVE_HELPER, "pack-stdin", archive],
                                       input=plaintext.getvalue(), check=True)
                    with archive.open("rb") as stream:
                        variant[1]["sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
                # Register while the original selection and the staged candidate are still bound.
                with pinned_directory(staged):
                    register_selection(fixture, staged, [*metadata[:3], expected, selected, *variant])
                if interactive_selection:
                    require(target.name == "project" and target.parent.name == "selected",
                            "unexpected selection target")
                    package.rename(target.parent)
                else:
                    staged.rename(target)
            return expected

        def add_asset():
            subprocess.run([sys.executable, "-I", "-c",
                            "import sys; open(sys.argv[1], 'xb').write(b'late asset')",
                            str(root / "assets/nested/late.png")], check=True, timeout=10)
        if not interactive_selection:
            planned["assets/nested/late.png"] = hashlib.sha256(b"late asset").hexdigest()
            try:
                capture(add_asset)
            except RuntimeError as exc:
                require(str(exc) == "project changed during capture" and not target.exists(),
                        "uncertain capture released")
            else:
                raise RuntimeError("concurrent asset omitted")
        metadata[3] = capture()
        metadata.extend([selected, *variant])
        (fixture / "expected.json").write_text(json.dumps(metadata))
        return True


def writable_fixture(interactive_selection=False, export_to=None):
    fixture = Path(tempfile.gettempdir()) / ("kraqel-p3-" + uuid.uuid4().hex)
    fixture.mkdir()
    try:
        target = fixture / "selected/project" if interactive_selection else fixture / "captured"
        if not prepare_captured_project(target, fixture, interactive_selection):
            require(not any(fixture.iterdir()), "cancelled selection produced an artifact")
            print("CANCELLED: no candidate or selection record")
            return
        metadata = json.loads((fixture / "expected.json").read_text())
        selected = metadata[4]
        try:
            verify_selection(fixture, target, metadata, object_id(fixture))
        except RuntimeError as exc:
            require(str(exc) == "native evidence refused", "wrong source substitution refusal")
        else:
            raise RuntimeError("substituted selection accepted")
        verify_selection(fixture, target, metadata, selected)
        if interactive_selection:
            require_logical_copy(metadata)
        with fixture_database(target, metadata[3]) as db:
            rows = db.execute("SELECT title, plain_text, revision FROM document WHERE project_id=? AND id=?",
                              metadata[:2]).fetchall()
            require(rows == [expected_scene(metadata)], "lost committed WAL scene")
            if interactive_selection:
                # The original database contains the overwritten paragraph; the new file and FTS must not.
                require(not any((target / ("kraqel.sqlite3" + suffix)).exists() for suffix in ("-wal", "-shm"))
                        and b"Vor dem WAL-Commit" not in (target / "kraqel.sqlite3").read_bytes(),
                        "physical database history in logical copy")
                search = "SELECT rowid FROM document_fts WHERE document_fts MATCH ?"
                rowid, = db.execute("SELECT rowid FROM document WHERE id=?", (metadata[1],)).fetchone()
                require(db.execute(search, ("Commit",)).fetchall() == []
                        and db.execute(search, (rows[0][1].split()[-1].rstrip("."),)).fetchall() == [(rowid,)],
                        "search index does not match current scene")
        if export_to is not None:
            data = approved_archive(target.parent, metadata)
            require(data is not None, "Keine Zusatzbestände zum Export ausgewählt.")
            # Same-volume publication, Windows rename refuses an existing destination.
            with tempfile.TemporaryDirectory(dir=export_to.parent) as work:
                staged = Path(work) / "sealed.age"
                with staged.open("xb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
                staged.rename(export_to)
            print("Passphrasenverschlüsseltes Zusatzarchiv gespeichert.")
    finally:
        shutil.rmtree(fixture)
    print("PASS: substituted source refused after native restart; selected candidate contains committed WAL scene")


def writer(root, readonly):
    # Same process image, access and paths before and during capture.
    for directory in (root, root / "assets" / "nested"):
        target = directory / "volume-write-probe.png"
        handle = open_file(str(target), 0x40000000, 0, None, 1, 0, None)
        if handle == ctypes.c_void_p(-1).value:
            code = ctypes.get_last_error()
            if not readonly or code != 19:  # ERROR_WRITE_PROTECT
                raise ctypes.WinError(code)
        else:
            kernel.CloseHandle(wintypes.HANDLE(handle))
            target.unlink()
            require(not readonly, "source admitted new asset")


def challenge(root, readonly):
    subprocess.run([sys.executable, "-I", __file__, ("selected-writer" if root.parent.name == "selected" else "writer"), root.anchor,
                    "ro" if readonly else "rw"], check=True, timeout=15)


def commit_fixture_wal(root, document, text):
    db = sqlite3.connect(root / "kraqel.sqlite3")
    db.execute("PRAGMA wal_autocheckpoint=0")
    updated = db.execute("UPDATE document SET plain_text=? WHERE id=?", (text, document))
    require(updated.rowcount == 1, "missing WAL scene")
    db.commit()
    os._exit(82)  # Keep the committed WAL and its matching shared-memory index.


def seed_writable_project(root, identity, scene):
    root.mkdir()
    nested = root / "assets" / "nested"
    nested.mkdir(parents=True)
    (root / "empty").mkdir()
    png = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jZQAAAABJRU5ErkJggg==")
    (nested / "cover.png").write_bytes(png)
    thumbnails = root / "assets" / "thumbnails"
    thumbnails.mkdir()
    thumbnail_name = "vorschau.png" if scene == P3_SOUTH else "cover.png"
    (thumbnails / thumbnail_name).write_bytes(png)
    document, asset = (str(uuid.uuid4()) for _ in range(2))
    with closing(sqlite3.connect(root / "kraqel.sqlite3")) as db:
        initialize_fixture_schema(db)
        db.execute("PRAGMA foreign_keys=ON")
        db.execute("PRAGMA user_version=6")
        db.execute("INSERT INTO project(id,name,schema_version,created_at,updated_at) VALUES(?, 'P3',1,'t','t')", (identity,))
        db.execute("INSERT INTO document(id,project_id,kind,title,plain_text,position,created_at,updated_at) "
                   "VALUES(?,?,'scene',?,?,0,'t','t')", (document, identity, scene[0], "Vor dem WAL-Commit"))
        db.execute("INSERT INTO media_asset(id,project_id,kind,relative_path,thumbnail_path,original_name,"
                   "title,mime_type,checksum,byte_count,width,height,provenance,created_at,updated_at) "
                   "VALUES(?,?,'image','assets/nested/cover.png',?,'cover.png',"
                   "'Cover','image/png',?,?,1,1,'imported','t','t')",
                   (asset, identity, "assets/thumbnails/" + thumbnail_name, hashlib.sha256(png).hexdigest(), len(png)))
        db.execute("INSERT INTO asset_link(project_id,asset_id,target_type,target_id,created_at) "
                   "VALUES(?,?,'document',?,'t')", (identity, asset, document))
        if scene == P3_SOUTH:
            db.execute("INSERT INTO image_prompt_draft(id,project_id,name,source_document_id,"
                       "source_document_revision,prompt_text,created_at,updated_at) VALUES(?,?,?,?,0,?,'t','t')",
                       (str(uuid.uuid4()), identity, "Hafenlicht", document, "Grün über der Südküste"))
        db.commit()
        require(db.execute("PRAGMA journal_mode=WAL").fetchone() == ("wal",), "WAL mode unavailable")
    physical = (root / "kraqel.sqlite3").read_bytes()
    require(b"Vor dem WAL-Commit" in physical, "missing physical history in synthetic source")
    main_before = hashlib.sha256(physical).digest()
    run_to_exit(commit_fixture_wal, (root, document, scene[1]), 82)
    require(hashlib.sha256((root / "kraqel.sqlite3").read_bytes()).digest() == main_before,
            "scene was checkpointed into the main file")
    # The normal stream stays identical, so a hash-only probe would miss this.
    alternate = Path(str(nested / "cover.png") + ":fixture")
    alternate.write_bytes(b"synthetic")
    try:
        inventory(root)
    except RuntimeError as exc:
        require(str(exc) == "unaccounted named stream", "wrong inventory refusal")
    else:
        raise RuntimeError("named stream omitted")
    alternate.unlink()
    return [identity, document, asset, inventory(root)]


def stage_tag(bundle, metadata, approved, key, register=False):
    expected = metadata[3]
    stage = bundle / "project"
    layout = []
    for name, digest in sorted(expected.items()):
        path = stage / name
        info = check_entry(path)
        require(info.st_ino and path.is_dir() == (digest is None), "invalid staging object")
        if digest is not None:
            require(path.is_file() and info.st_nlink == 1, "invalid staging file")
        layout.append([name, info.st_dev, info.st_ino])
    archive_id = None
    if has_archive(metadata):
        info = check_entry(bundle / archive_name(metadata))
        require(info.st_nlink == 1 and (bundle / archive_name(metadata)).is_file(), "invalid staging archive")
        archive_id = [info.st_dev, info.st_ino]
    plan = staging_plan(bundle, approved, layout, archive_id)
    return tag(key, NativeAuthority.domains[1], plan, register).encode("ascii")


def has_archive(metadata):
    return len(metadata) == 7 and metadata[6]["groups"] != 0


def staging_plan(bundle, approved, layout, archive_id):
    plan = [approved, object_id(bundle), object_id(bundle / "project"), layout]
    if archive_id is not None:
        plan.append(archive_id)
    return plan


def create_stage(root, bundle, metadata, approved, key):
    expected = metadata[3]
    archive_data = approved_archive(root.parent, metadata) if len(metadata) == 7 else None
    bundle.mkdir()
    stage = bundle / "project"
    stage.mkdir()
    for name, digest in sorted(expected.items()):
        if digest is None:
            (stage / name).mkdir()
        else:
            with (stage / name).open("xb"):
                pass
    if archive_data is not None:
        with (bundle / archive_name(metadata)).open("xb") as archive:
            archive.write(archive_data)
            archive.flush()
            os.fsync(archive.fileno())
    with (bundle / "reservation").open("xb") as proof:
        proof.write(stage_tag(bundle, metadata, approved, key, register=True))


@contextmanager
def reserved_file(path):
    raw = open_file(str(path), 0xc0000000, 0, None, 3, 0x00200000, None)
    if raw == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    with os.fdopen(msvcrt.open_osfhandle(raw, os.O_RDWR | os.O_BINARY), "r+b") as file:
        yield file


@contextmanager
def checked_staging(root, bundle, metadata, approved, key):
    expected = metadata[3]
    stage = bundle / "project"
    entries = inventory(root)
    require(entries == expected, "source inventory mismatch")
    check_entry(bundle)
    allowed = {"project", "reservation"}
    if has_archive(metadata):
        allowed.add(archive_name(metadata))
    require({p.name for p in bundle.iterdir()} == allowed, "unexpected staging evidence")
    reservation = bundle / "reservation"
    require(check_entry(reservation).st_nlink == 1 and reservation.is_file()
            and reservation.stat().st_size == 64, "invalid reservation")
    require(inventory(stage).keys() == entries.keys(), "unexpected staging contents")
    with ExitStack() as pins:
        pins.enter_context(pinned_directory(bundle))
        pins.enter_context(pinned_directory(stage))
        reservation_file = pins.enter_context(reserved_file(reservation))
        attestation = reservation_file.read(65)
        files = []
        layout = []
        for name, digest in sorted(entries.items()):
            path = stage / name
            if digest is None:
                pins.enter_context(pinned_directory(path))
                layout.append([name, *object_id(path)])
                continue
            info = check_entry(path)
            require(path.is_file() and info.st_nlink == 1, "invalid staging file")
            dst = pins.enter_context(reserved_file(path))
            require(os.path.samestat(info, os.fstat(dst.fileno())), "replaced staging file")
            layout.append([name, info.st_dev, info.st_ino])
            src = pins.enter_context((root / name).open("rb"))
            while prefix := dst.read(65536):
                require(prefix == src.read(len(prefix)), "staging prefix mismatch")
            files.append((name, src, dst))
        archive_id = None
        if has_archive(metadata):
            path = bundle / archive_name(metadata)
            before = check_entry(path)
            archive = pins.enter_context(reserved_file(path))
            stat = os.fstat(archive.fileno())
            require(os.path.samestat(before, stat) and stat.st_nlink == 1
                    and hashlib.file_digest(archive, "sha256").hexdigest() == metadata[6]["sha256"],
                    "unbound staging archive")
            archive_id = [stat.st_dev, stat.st_ino]
        plan = staging_plan(bundle, approved, layout, archive_id)
        require(hmac.compare_digest(tag(key, NativeAuthority.domains[1], plan).encode("ascii"), attestation),
                "unbound reservation")
        yield files, os.fstat(reservation_file.fileno()), plan


@contextmanager
def fixture_database(project, expected, verify=None):
    main = "kraqel.sqlite3"
    wal, shm = main + "-wal", main + "-shm"
    sidecars = {wal, shm}
    names = {p.name for p in project.iterdir()}
    present = names & sidecars
    require(main + "-journal" not in names and present == expected.keys() & sidecars
            and (not present or present == sidecars), "unsupported SQLite sidecars")
    budget = 8 * 1024 * 1024
    with ExitStack() as held:
        held.enter_context(pinned_directory(project))
        images = {}
        for name in [main, *sorted(present)]:
            path = project / name
            info = check_entry(path)
            raw = open_file(str(path), 0x80000000, 1, None, 3, 0x00200000, None)
            if raw == ctypes.c_void_p(-1).value:
                raise ctypes.WinError(ctypes.get_last_error())
            file = held.enter_context(os.fdopen(msvcrt.open_osfhandle(raw, os.O_RDONLY | os.O_BINARY), "rb"))
            pinned = os.fstat(file.fileno())
            require(os.path.samestat(info, pinned) and not pinned.st_file_attributes & 0x400
                    and pinned.st_nlink == 1 and 0 < pinned.st_size <= budget, "unsupported SQLite file")
            images[name] = file
        if verify is not None:
            verify()
        for name, file in images.items():
            require(hashlib.file_digest(file, "sha256").hexdigest() == expected[name], "changed SQLite file")

        db = held.enter_context(closing(sqlite3.connect(":memory:")))
        if wal in images:
            # SQLite interprets WAL commits. All three on-disk files deny writes and replacement.
            uri = (project / main).as_uri() + "?mode=ro"
            with closing(sqlite3.connect(uri, uri=True, timeout=0)) as source:
                source.execute("PRAGMA trusted_schema=OFF")
                source.execute("PRAGMA query_only=ON")
                source.execute("BEGIN")
                pages = source.execute("PRAGMA page_count").fetchone()[0]
                size = source.execute("PRAGMA page_size").fetchone()[0]
                require(source.execute("PRAGMA journal_mode").fetchone() == ("wal",)
                        and 0 < pages * size <= budget, "unsupported WAL view")

                def backup_progress(status, remaining, total):
                    require(status in (sqlite3.SQLITE_OK, sqlite3.SQLITE_DONE)
                            and total * size <= budget, "SQLite backup unavailable")

                source.backup(db, pages=128, progress=backup_progress, sleep=0)
        else:
            images[main].seek(0)
            data = images[main].read(budget + 1)
            require(data[18:20] == b"\x01\x01", "missing WAL evidence")
            db.deserialize(data)
        db.execute("PRAGMA trusted_schema=OFF")
        db.execute("PRAGMA temp_store=MEMORY")
        db.execute("PRAGMA query_only=ON")
        yield db


def copy_project(root, bundle, metadata, approved, key, interrupt):
    expected = metadata[3]
    stage = bundle / "project"
    with checked_staging(root, bundle, metadata, approved, key) as (files, _, _):
        for name, src, dst in files:
            dst.write(src.read(1))
            if callable(interrupt):
                dst.flush()
                interrupt()
            if interrupt == "first-byte":
                dst.flush()
                os._exit(85)
            if name == "kraqel.sqlite3":
                challenge(root, True)
            shutil.copyfileobj(src, dst)
    require(inventory(stage) == expected == inventory(root), "incomplete copy")
    with fixture_database(stage, expected) as db:
        verify_scene_view(db, metadata, expected)


def verify_scene_view(db, metadata, entries):
    identity, document, asset = metadata[:3]
    require(db.execute("SELECT title, plain_text, revision FROM document WHERE project_id=? AND id=?",
                       (identity, document)).fetchall() == [expected_scene(metadata)], "incomplete scene revision")
    require(db.execute("pragma integrity_check").fetchall() == [("ok",)], "invalid copy")
    require(db.execute("select id from project").fetchall() == [(identity,)], "wrong project")
    require(not db.execute("pragma foreign_key_check").fetchall(), "invalid foreign key")
    links = db.execute("SELECT d.id,m.id,m.relative_path,m.thumbnail_path,m.checksum FROM document d "
                       "JOIN asset_link a ON a.target_type='document' AND a.target_id=d.id "
                       "AND a.project_id=d.project_id JOIN media_asset m ON m.id=a.asset_id "
                       "AND m.project_id=a.project_id WHERE d.project_id=?", (identity,)).fetchall()
    require(len(links) == 1, "incomplete scene asset binding")
    linked_document, linked_asset, image, thumb, checksum = links[0]
    require((linked_document, linked_asset) == (document, asset)
            and image.startswith("assets/") and thumb.startswith("assets/thumbnails/")
            and checksum == entries.get(image) and entries.get(thumb) is not None,
            "incomplete scene asset binding")


def object_id(path):
    info = check_entry(path)
    require(path.is_dir() and info.st_ino != 0, "missing directory identity")
    return [info.st_dev, info.st_ino]


class NativeAuthority:
    domains = (b"KRAQEL-P3-FIXTURE-DECISION-v1", b"KRAQEL-P3-FIXTURE-STAGING-v1",
               b"KRAQEL-P3-FIXTURE-RECEIPT-v1", b"KRAQEL-P3-FIXTURE-CLEANUP-v1")

    def __init__(self, pipe):
        self.pipe = pipe
        self.plan_digest = None

    def attest(self, domain, digest, register):
        self.pipe.stdin.write(bytes((int(register), self.domains.index(domain))) + digest)
        self.pipe.stdin.flush()
        response = self.pipe.stdout.readline(66)
        require(re.fullmatch(b"[0-9a-f]{64}\n", response), "native evidence refused")
        return response[:-1].decode("ascii")

    def bind_delivery(self, row):
        require(self.plan_digest is not None, "missing delivery decision")
        self.pipe.stdin.write(bytes((0, 6)) + self.plan_digest + row)
        self.pipe.stdin.flush()
        if self.pipe.stdout.readline(16) != b"BOUND\n":
            raise RuntimeError("scene binding failed: " + self.pipe.stderr.read(512).decode())

    def start_delivery(self, row, interactive):
        self.bind_delivery(row)
        self.pipe.stdin.write(b"\x02" if interactive else b"\x01")
        self.pipe.stdin.flush()
        if interactive and self.pipe.stdout.readline(16) != b"VISIBLE\n":
            raise RuntimeError("preview failed: " + self.pipe.stderr.read(512).decode())

    def deliver(self, row, interactive=False):
        self.start_delivery(row, interactive)
        if self.pipe.stdout.readline(16) != b"DELIVERED\n":
            raise RuntimeError("scene delivery failed: " + self.pipe.stderr.read(512).decode())

    def removal_step(self, step, advance=False):
        self.pipe.stdin.write(bytes((int(advance), 4)) + step.to_bytes(4, "little") + bytes(28))
        self.pipe.stdin.flush()
        require(self.pipe.stdout.readline(4) == b"OK\n", "unconfirmed cleanup progress")

    def cleanup_finished(self, count, finish=False):
        self.pipe.stdin.write(bytes((int(finish), 5)) + count.to_bytes(4, "little") + bytes(28))
        self.pipe.stdin.flush()
        reply = self.pipe.stdout.readline(6)
        require(reply in (b"OPEN\n", b"DONE\n"), "unconfirmed cleanup end")
        return reply == b"DONE\n"


@contextmanager
def native_authority(home, create, cleanup=False, read_only=False, comparison=False):
    binary = home / "p3-starter.exe"
    operation = "authorize-p3-fixture" if create else "reopen-p3-fixture"
    if read_only:
        require(not create and not cleanup, "read requires existing publication")
        operation = "read-p3-fixture"
    if cleanup:
        require(not create, "cleanup requires existing installation")
        operation = "confirm-p3-fixture-cleanup"
    if comparison:
        require(not cleanup and (create != read_only), "invalid comparison operation")
        operation = "authorize-p3-comparison" if create else "read-p3-comparison"
    with subprocess.Popen([binary, "broker-store", operation], env={**os.environ, "USERPROFILE": str(home)},
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as proc:
        try:
            report_native_root(proc, binary)
            if proc.stdout.readline(16) != b"P3/1\n":
                raise RuntimeError("native store unavailable: " + proc.stderr.read(512).decode())
            yield NativeAuthority(proc)
        finally:
            proc.stdin.close()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()


def confirm_cleanup(root, destination, metadata, approved, home):
    with volume_guard(root.anchor, True), pinned_directory(root), pinned_directory(destination):
        with native_authority(home, False, cleanup=True) as authority:
            require(hmac.compare_digest(decision(authority, root, destination, metadata), approved),
                    "unbound decision")
            project_id = str(uuid.UUID(metadata[0]))
            with target_lock(destination, project_id):
                require(not os.path.lexists(destination / project_id), "published destination")
                pending = destination / (".pending-" + project_id)
                with checked_staging(root, pending, metadata, approved, authority) as (files, info, plan):
                    # Bind the decision to the exact verified prefixes and reservation file.
                    lengths = {name: dst.tell() for name, _, dst in files}
                    record = [plan, [info.st_dev, info.st_ino], lengths]
                    sidecar = destination / (".cleanup-" + project_id)
                    data = json.dumps(record, separators=(",", ":")).encode("ascii")
                    if os.path.lexists(sidecar):
                        require(read_cleanup_record(sidecar) == data, "different cleanup record")
                    else:
                        with sidecar.open("xb") as out:
                            out.write(data)
                            out.flush()
                            os.fsync(out.fileno())
                    return tag(authority, NativeAuthority.domains[3], record, True)


def read_cleanup_record(path):
    info = check_entry(path)
    require(path.is_file() and info.st_nlink == 1 and info.st_size <= 65536, "invalid cleanup record")
    with reserved_file(path) as handle:
        require(os.path.samestat(info, os.fstat(handle.fileno())), "replaced cleanup record")
        return handle.read(65537)


def remove_confirmed(root, destination, metadata, approved, home, after_remove=None):
    project_id = str(uuid.UUID(metadata[0]))
    with volume_guard(root.anchor, True), pinned_directory(root), pinned_directory(destination):
        with native_authority(home, False, cleanup=True) as authority, target_lock(destination, project_id):
            require(not os.path.lexists(destination / project_id), "published destination")
            require(hmac.compare_digest(decision(authority, root, destination, metadata), approved),
                    "unbound decision")
            record = json.loads(read_cleanup_record(destination / (".cleanup-" + project_id)))
            plan, reservation_id, lengths = record
            require(plan[0] == approved, "unbound cleanup plan")
            reservation = tag(authority, NativeAuthority.domains[1], plan).encode("ascii")
            tag(authority, NativeAuthority.domains[3], record)  # Verify only. Never grant here.
            pending = destination / (".pending-" + project_id)
            entries = [("project/" + name, [dev, ino], metadata[3][name] is None)
                       for name, dev, ino in plan[3]]
            if has_archive(metadata):
                entries.append((archive_name(metadata), plan[4], False))
            entries += [("reservation", reservation_id, False), ("project", plan[2], True)]
            entries.sort(key=lambda entry: (-entry[0].count("/"), entry[0]))
            count = len(entries) + 1
            if authority.cleanup_finished(count):
                require(not os.path.lexists(pending), "completed cleanup path reused")
                return
            if not os.path.lexists(pending):
                require(authority.cleanup_finished(count, finish=True), "cleanup still open")
                return
            require(object_id(pending) == plan[1], "replaced cleanup root")
            present = inventory(pending)
            remaining = [entry for entry in entries if entry[0] in present]
            require(len(present) == len(remaining) and remaining == entries[len(entries)-len(remaining):],
                    "unexpected cleanup remainder")
            removed = len(entries) - len(remaining)
            authority.removal_step(removed)
            remaining.append(("", plan[1], True))
            with ExitStack() as pins:
                opened = []
                for name, identity, directory in remaining:
                    path = pending / name
                    info = check_entry(path)
                    require([info.st_dev, info.st_ino] == identity and path.is_dir() == directory,
                            "replaced cleanup object")
                    raw = open_file(str(path), 0x80010000, 1 if directory else 0, None, 3,
                                    0x02200000, None)  # READ | DELETE, OPEN_REPARSE_POINT, BACKUP_SEMANTICS
                    if raw == ctypes.c_void_p(-1).value:
                        raise ctypes.WinError(ctypes.get_last_error())
                    fd = msvcrt.open_osfhandle(raw, os.O_RDONLY | os.O_BINARY)
                    owner = pins.enter_context(ExitStack())
                    owner.callback(os.close, fd)
                    held = os.fstat(fd)
                    require(os.path.samestat(info, held) and not held.st_file_attributes & 0x400,
                            "replaced cleanup handle")
                    if not directory:
                        require(held.st_nlink == 1, "linked cleanup file")
                        if name == "reservation":
                            require(os.read(fd, 65) == reservation, "changed cleanup reservation")
                        elif name == archive_name(metadata):
                            with os.fdopen(os.dup(fd), "rb") as file:
                                require(hashlib.file_digest(file, "sha256").hexdigest() == metadata[6]["sha256"],
                                        "changed cleanup archive")
                        else:
                            relative = name[len("project/"):]
                            require(held.st_size == lengths[relative], "changed cleanup prefix")
                            with (root / relative).open("rb") as source:
                                while chunk := os.read(fd, 65536):
                                    require(source.read(len(chunk)) == chunk, "changed cleanup prefix")
                    opened.append((owner, raw))
                # Delete only through the checked handles. Windows refuses a nonempty directory.
                for step, (owner, raw) in enumerate(opened, start=removed + 1):
                    authority.removal_step(step, advance=True)
                    flag = ctypes.c_ubyte(1)
                    if not kernel.SetFileInformationByHandle(wintypes.HANDLE(raw), 4, ctypes.byref(flag), 1):
                        raise ctypes.WinError(ctypes.get_last_error())
                    owner.close()
                    if after_remove:
                        after_remove()
            require(not os.path.lexists(pending) and authority.cleanup_finished(count, finish=True),
                    "cleanup still open")



def read_published_scene(root, target, metadata, approved, home, interactive=False):
    project_id, scene_id, _, expected = metadata[:4]
    with volume_guard(root.anchor, True), pinned_directory(root), pinned_directory(target):
        with native_authority(home, False, read_only=True) as authority, target_lock(target, project_id):
            require(hmac.compare_digest(approved, decision(authority, root, target, metadata)),
                    "unbound decision")
            require(not os.path.lexists(target / (".pending-" + project_id)), "unresolved staging")
            final = target / project_id
            with pinned_directory(final), pinned_directory(final / "project"):
                def verify_publication():
                    verify_bundle(final, approved, metadata, authority)

                with fixture_database(final / "project", expected, verify_publication) as db:
                    rows = db.execute("SELECT title, plain_text, revision FROM document "
                                      "WHERE project_id=? AND id=? AND kind='scene'",
                                      (project_id, scene_id)).fetchall()
                    require(len(rows) == 1, "bound scene missing")
                    row = scene_frame(metadata, rows[0])
                    require(rows[0] == expected_scene(metadata), "unsupported fixture row")
                    authority.deliver(row, interactive)
                    return rows[0]


def retire_preview_profiles(home):
    deadline = time.monotonic() + 3
    while True:
        result = subprocess.run([home / "p3-starter.exe", "broker-store", "retire-p3-preview-profiles"],
                                env={**os.environ, "USERPROFILE": str(home)}, input=b"",
                                capture_output=True, timeout=10)
        if result.returncode == 0:
            break
        # Kill-on-close can still be draining when the controller process has already exited.
        # Retry only the authenticated broker's specific busy result. Other refusals retain evidence.
        try:
            failure = json.loads(result.stderr)
        except (ValueError, UnicodeError):
            failure = None
        if not (isinstance(failure, dict) and failure.get("stage") == "product-job-active"
                and time.monotonic() < deadline):
            raise RuntimeError("profile cleanup failed: " + result.stderr.decode(errors="replace"))
        time.sleep(0.1)
    summary = json.loads(result.stdout)
    require(summary["profileFoldersAbsent"] and summary["sessionOpen"], "uncertain crash cleanup")
    return summary["deletedProfiles"]


def receive_prepared(folder, trusted_hash, project_id):
    """Capture only externally authorized bytes; return a detached in-memory working set."""
    import runpy
    lease = runpy.run_path(str(Path(__file__).with_name("p3_directory_oplock.py")))
    manifest_name = "uebernahme.json"
    with pinned_directory(folder):
        identity = folder.lstat()

        def acquire(names, exact=False):
            # A broken lease invalidates the whole snapshot. Retry only under the original
            # directory identity and external receipt, before any bytes are staged or authorized.
            for attempt in range(3):
                try:
                    return capture_writable_fixture(folder, identity, lease["read_oplock"],
                                                    include=names, exact=exact)
                except lease["CaptureChanged"]:
                    if attempt == 2:
                        raise
                    time.sleep(0.1)

        raw = acquire({manifest_name}).get(manifest_name)
        require(isinstance(raw, bytes) and len(raw) <= 128 * 1024 and
                hashlib.sha256(raw).hexdigest() == trusted_hash, "unconfirmed prepared manifest")

        def distinct(pairs):
            result = dict(pairs)
            require(len(result) == len(pairs), "ambiguous prepared manifest")
            return result

        manifest = json.loads(raw, object_pairs_hook=distinct)
        require(isinstance(manifest, dict) and manifest.get("projectId") == project_id, "wrong preparation")
        basis_hash = None
        if manifest.get("format") == "kraqel-logical-preparation-v2":
            require(set(manifest) == {"format", "projectId", "files", "sceneBasisSha256"}
                    and isinstance(manifest["sceneBasisSha256"], str)
                    and re.fullmatch(r"[0-9a-f]{64}", manifest["sceneBasisSha256"]), "invalid scene basis receipt")
            basis_hash = manifest["sceneBasisSha256"]
        else:
            require(set(manifest) == {"format", "projectId", "files"}
                    and manifest["format"] == "kraqel-logical-preparation-v1", "wrong preparation")
        files = manifest["files"]
        require(isinstance(files, dict) and 1 <= len(files) <= 250 and "kraqel.sqlite3" in files,
                "invalid prepared inventory")
        allowed = {manifest_name, *files}
        if basis_hash:
            basis_name = f"szenenbasis-{basis_hash}.json"
            allowed.add(basis_name)
        for name, digest in files.items():
            parts = name.split("/")
            require(isinstance(digest, str) and re.fullmatch("[0-9a-f]{64}", digest)
                    and (name == "kraqel.sqlite3" or parts[0] == "assets" and len(parts) >= 2)
                    and all(part and not part.startswith(".") and not part.endswith((" ", "."))
                            and not os.path.isreserved(part) and "\\" not in part for part in parts),
                    "unsafe prepared inventory")
            allowed.update("/".join(parts[:i]) for i in range(1, len(parts)))
        require(len({s.casefold() for s in allowed}) == len(allowed), "ambiguous prepared names")
        snapshot = acquire(allowed, exact=True)
        require(set(snapshot) == allowed and snapshot.pop(manifest_name) == raw,
                "incomplete prepared inventory")
        if basis_hash:
            basis = snapshot.pop(basis_name)
            require(isinstance(basis, bytes) and hashlib.sha256(basis).hexdigest() == basis_hash,
                    "changed scene basis")
    actual = {n: hashlib.sha256(data).hexdigest() for n, data in snapshot.items() if data is not None}
    require(actual == files, "changed prepared file")
    image = snapshot["kraqel.sqlite3"]
    require(image[:16] == b"SQLite format 3\0" and image[18:20] == b"\1\1", "unsupported prepared database")
    with closing(sqlite3.connect(":memory:")) as db:
        db.deserialize(image)
        db.execute("PRAGMA trusted_schema=OFF")
        db.execute("PRAGMA temp_store=MEMORY")
        db.execute("PRAGMA query_only=ON")
        check_working_schema(db)
        page_size = db.execute("PRAGMA page_size").fetchone()[0]
        pages = db.execute("PRAGMA page_count").fetchone()[0]
        require(len(image) == page_size * pages, "invalid prepared database length")
        require(db.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
                and db.execute("PRAGMA foreign_key_check").fetchone() is None
                and db.execute("SELECT id FROM project LIMIT 2").fetchall() == [(project_id,)],
                "invalid prepared database")
        refs = {"kraqel.sqlite3"}
        for owner, original, thumb, checksum, size in db.execute(
                "SELECT project_id,relative_path,thumbnail_path,checksum,byte_count FROM media_asset"):
            require(owner == project_id and original.startswith("assets/") and not original.startswith("assets/thumbnails/")
                    and thumb.startswith("assets/thumbnails/") and original in actual and thumb in actual
                    and actual[original] == checksum and len(snapshot[original]) == size,
                    "invalid prepared media binding")
            refs.update((original, thumb))
        require(refs == set(files), "unreferenced prepared file")
    return snapshot


@contextmanager
def desktop_fixture_source(followup=False, divergent=False, long_scene=False):
    producer = Path(__file__).parent / "probe/target/debug/examples/p3_handoff_source.exe"
    with tempfile.TemporaryDirectory(prefix="kraqel-p3-handoff-") as source:
        args = ["--divergent"] if divergent else ["--followup"] if followup else []
        if long_scene:
            args.append("--long-scene")
        with subprocess.Popen([producer, *args], cwd=source, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as proc:
            report_native_root(proc, producer)
            try:
                output, _ = proc.communicate(timeout=60)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.communicate()
                raise
            require(proc.returncode == 0, "desktop preparation failed")
        reply = json.loads(output)
        project, receipt = reply["project"], reply["receipt"]
        require(str(uuid.UUID(project)) == project, "invalid fixture project")
        with ExitStack() as pins:
            parent = Path(source)
            for child in ("", "catalog", project, "backups"):
                parent = parent / child
                pins.enter_context(pinned_directory(parent))
                require(parent.is_dir() and not parent.lstat().st_file_attributes & (0x400 | 0x1000 | 0x40000 | 0x400000),
                        "unsafe fixture ancestor")
            for evidence in (receipt, reply["followup"]) if followup or divergent else (receipt,):
                require(re.fullmatch(r"uebernahme-[0-9a-f-]{36}", evidence["folder"]), "invalid fixture folder")
                pins.enter_context(pinned_directory(parent / evidence["folder"]))
            yield parent / receipt["folder"], receipt["manifestSha256"], reply


def prepared_scene_view(snapshot, project, scene):
    """Project one bound scene from the already verified, detached image."""
    with closing(sqlite3.connect(":memory:")) as db:
        db.deserialize(snapshot["kraqel.sqlite3"])
        db.execute("PRAGMA trusted_schema=OFF")
        db.execute("PRAGMA temp_store=MEMORY")
        db.execute("PRAGMA query_only=ON")
        return db.execute("SELECT title, plain_text, revision, content_json FROM document "
                          "WHERE project_id=? AND id=? AND kind='scene'", (project, scene)).fetchone()


def deliver_comparison(project, scene, hashes, views, interactive, base_hash):
    # Each role receives only its bootstrap and pipe handles. Both inventory receipts
    # are ordered inputs to the registered decision; neither source path crosses the pipe.
    manifests = [bytes.fromhex(digest) for digest in hashes]
    payload = bytearray(uuid.UUID(project).bytes + uuid.UUID(scene).bytes + b"".join(manifests))
    left, right = (revision.to_bytes(4, "little") + title.encode() + b"\0" + text.encode()
                   for title, text, revision, _ in views)
    payload.extend(len(left).to_bytes(4, "little") + left + right)
    length = len(payload).to_bytes(4, "little")
    root = Path(tempfile.gettempdir()).resolve() / ("kraqel-p3-" + uuid.uuid4().hex)
    home = root / "comparison-install"
    root.mkdir()
    home.mkdir()
    shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe", home / "p3-starter.exe")
    nonce = hashlib.sha256(b"KRAQEL-P3-COMMON-BASIS-v1" + bytes.fromhex(base_hash) + secrets.token_bytes(32)).digest()
    binding = hashlib.sha256(b"KRAQEL-P3-COMPARISON-v2" + nonce + hashlib.sha256(P3_SERVICE.read_bytes()).digest()
                             + hashlib.sha256(P3_WINDOW.read_bytes()).digest() + payload).digest()
    records = (binding, *manifests)
    def verify(authority, register):
        for domain, record in zip(authority.domains, records):
            authority.attest(domain, record, register)
        authority.plan_digest = nonce

    with native_authority(home, True, comparison=True) as authority:
        verify(authority, True)
    for image in (P3_SERVICE, P3_WINDOW):
        shutil.copyfile(image, home / "p3-control/bin" / image.name)
    if not interactive:
        with native_authority(home, False, read_only=True, comparison=True) as authority:
            verify(authority, False)
            swapped = payload[:32] + payload[64:96] + payload[32:64] + payload[96:]
            authority.pipe.stdin.write(bytes((0, 6)) + nonce + length + swapped)
            authority.pipe.stdin.flush()
            require(authority.pipe.stdout.read(16) == b"" and authority.pipe.wait(timeout=5) != 0
                    and b"p3-unbound-row" in authority.pipe.stderr.read(512), "changed comparison accepted")
    with native_authority(home, False, read_only=True, comparison=True) as authority:
        verify(authority, False)
        authority.start_delivery(length + payload, interactive)
        side = authority.pipe.stdout.read(1)
        require(side in (b"\x00", b"\x01", b"\x02") and authority.pipe.stdout.readline(16) == b"DELIVERED\n",
                "missing confirmed comparison outcome")
        require(interactive or side == b"\x00", "closing selected a version")
    # Keep the control journal on uncertain exit, including failed profile retirement.
    require(root.parent == Path(tempfile.gettempdir()).resolve() and root.resolve() == root
            and home.resolve() == home, "unexpected comparison cleanup root")
    shutil.rmtree(root)
    return side[0]


def compare_prepared(interactive):
    # Both receipts arrive from the producer independently of the folders.
    # This preview has no destination, publication ticket or write binding.
    with desktop_fixture_source(divergent=True, long_scene=not interactive) as (folder, digest, reply):
        other = reply["followup"]
        second = folder.parent / other["folder"]
        project, scene = reply["project"], reply["scene"]
        before = (inventory(folder), inventory(second))
        left = receive_prepared(folder, digest, project)
        right = receive_prepared(second, other["manifestSha256"], project)
        views = [prepared_scene_view(snapshot, project, scene) for snapshot in (left, right)]
        # The dialog accepts only the fixed synthetic texts, never arbitrary manuscript content.
        require(all(view is not None for view in views) and
                [view[:3] for view in views] == [("Szene B", "Die Flut erreicht den Hafen." * (1 if interactive else 1024), 1),
                                                 ("Szene A", "Der Hafen liegt im Nebel.", 1)], "unexpected comparison scene")
        changed_files = {name for name in left.keys() | right.keys() if left.get(name) != right.get(name)}
        require(changed_files == {"kraqel.sqlite3"}, "unexpected comparison inventory")
        require(reply["comparison"]["localSha256"] == reply["receipt"]["sceneBasisSha256"]
                and reply["incomingSha256"] == other["sceneBasisSha256"], "comparison basis differs from preparation")
        require(reply["comparison"]["changes"] == [{"sceneId":scene,"difference":"conflict","baseExists":True,
                "localExists":True,"incomingExists":True}], "missing scene conflict")
        side = deliver_comparison(project, scene, (digest, other["manifestSha256"]), views, interactive, reply["baseSha256"])
        require(before == (inventory(folder), inventory(second)), "comparison changed a preparation")
    print(("Keine Auswahl.", "Lokale Fassung ausgewählt.", "Externe Fassung ausgewählt.")[side])


def check_prepared_refusals():
    with desktop_fixture_source() as (folder, trusted_hash, reply):
        def refused(expected, digest):
            try:
                receive_prepared(folder, digest, reply["project"])
            except RuntimeError as error:
                require(str(error) == expected, "wrong preparation refusal")
            else:
                raise RuntimeError("unexpected prepared admission")

        manifest = folder / "uebernahme.json"
        raw = manifest.read_bytes()
        media = folder / "assets/nested/hafen.png"
        original = media.read_bytes()
        media.write_bytes(b"changed content")
        refused("changed prepared file", trusted_hash)
        media.write_bytes(original)
        extra = folder / "assets/unreferenced.png"
        extra.write_bytes(b"synthetic optional item")
        refused("unexpected prepared entry", trusted_hash)
        edited = json.loads(raw)
        edited["files"]["assets/unreferenced.png"] = hashlib.sha256(extra.read_bytes()).hexdigest()
        altered = json.dumps(edited).encode()
        manifest.write_bytes(altered)
        refused("unconfirmed prepared manifest", trusted_hash)
        refused("unreferenced prepared file", hashlib.sha256(altered).hexdigest())
        # An authenticated old-format handoff has no comparison authority.
        extra.unlink()
        manifest.write_bytes(raw)
        legacy = json.loads(raw)
        sidecar = folder / ("szenenbasis-" + legacy.pop("sceneBasisSha256") + ".json")
        sidecar.write_bytes(b"changed basis")
        refused("changed scene basis", trusted_hash)
        sidecar.unlink()
        legacy["format"] = "kraqel-logical-preparation-v1"
        old = json.dumps(legacy).encode()
        manifest.write_bytes(old)
        snapshot = receive_prepared(folder, hashlib.sha256(old).hexdigest(), reply["project"])
        require(snapshot["assets/nested/hafen.png"] == original, "incomplete legacy preparation")
    print("PASS: changed media, extra file, substituted manifest and unreferenced file refused")


def stage_desktop_receipt(folder, manifest_hash, reply, target, scene, selection_home=None):
    selected = object_id(folder)
    snapshot = receive_prepared(folder, manifest_hash, reply["project"])
    expected = {name: None if data is None else hashlib.sha256(data).hexdigest() for name, data in snapshot.items()}
    metadata = [reply["project"], reply["scene"], reply["asset"], expected, selected, scene,
                {"groups": 0, "workingCopy": {"format": "desktop-prepared-v1", "manifestSha256": manifest_hash}}]
    with tempfile.TemporaryDirectory(dir=target.parent.parent) as work:
        package = Path(work) / "package"
        staged = package / "project"
        staged.mkdir(parents=True)
        for name, data in snapshot.items():
            dest = staged / name
            if data is None:
                dest.mkdir()
            else:
                with dest.open("xb") as output:
                    output.write(data)
        require(inventory(staged) == expected, "prepared staging changed")
        if selection_home is not None:
            with pinned_directory(staged):
                register_selection(selection_home, staged, metadata)
        package.rename(target.parent)
    return metadata


def prepare_desktop_fixture(target, fixture):
    with desktop_fixture_source(followup=True) as (folder, manifest_hash, reply):
        metadata = stage_desktop_receipt(folder, manifest_hash, reply, target, "south", fixture)
        other = reply["followup"]
        require(other["manifestSha256"] != manifest_hash, "missing second preparation")
        followup = stage_desktop_receipt(folder.parent / other["folder"], other["manifestSha256"], reply,
                                         target.parent.parent / "followup/project", "north")
        (fixture / "expected.json").write_text(json.dumps(metadata))
        (fixture / "followup.json").write_text(json.dumps(followup))


def register_selection(fixture, staged, metadata):
    home = fixture / "selection-install"
    home.mkdir()
    shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe", home / "p3-starter.exe")
    with native_authority(home, True) as authority:
        authority.attest(NativeAuthority.domains[0],
                         selection_digest(metadata[4], staged, metadata[:4] + metadata[5:]), True)


def preview_selected_project(root, fixture, interactive=True):
    """One selection, publication and read-only window, without fault injection."""
    metadata = json.loads((fixture / "expected.json").read_text())
    require(metadata[3] == inventory(root), "unexpected source")
    verify_selection(fixture, root, metadata, metadata[4])
    require_logical_copy(metadata)
    home = fixture / "install-a"
    home.mkdir()
    shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe",
                    home / "p3-starter.exe")
    with tempfile.TemporaryDirectory(dir=fixture) as temporary:
        target = Path(temporary)
        with native_authority(home, True) as authority:
            ticket = decision(authority, root, target, metadata, register=True)
            for image in (P3_SERVICE, P3_WINDOW):
                shutil.copyfile(image, home / "p3-control/bin" / image.name)
            publish(root, target, metadata, ticket, authority)
        if metadata[6]["workingCopy"]["format"] == "desktop-prepared-v1":
            # A second genuine preparation shares the UUID. A ticket grants exactly one state.
            final = target / metadata[0]
            before = object_id(final)
            next_root = root.parent.parent / "followup/project"
            followup = json.loads((fixture / "followup.json").read_text())
            require(followup[0] == metadata[0] and inventory(next_root) == followup[3]
                    and followup[3] != metadata[3], "missing conflicting generation")
            try:
                with native_authority(home, False) as reopened:
                    publish(next_root, target, followup, ticket, reopened)
            except RuntimeError as exc:
                require(str(exc) == "native evidence refused", "unexpected preparation conflict")
            else:
                raise RuntimeError("different preparation accepted")
            with native_authority(home, False) as reopened:
                same = publish(root, target, metadata, ticket, reopened)
                require(object_id(same) == before and list(target.iterdir()) == [final]
                        and inventory(final / "project") == metadata[3], "prepared replay changed destination")
            print("PASS: changed preparation refused; original publication reopened exactly once")
        read_published_scene(root, target, metadata, ticket, home, interactive=interactive)
    shutil.rmtree(home)


def native_restart(root, fixture, metadata, interactive):
    home = fixture / "install-a"
    foreign = fixture / "install-b"
    for directory in (home, foreign):
        directory.mkdir()
    try:
        for directory in (home, foreign):
            shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe",
                            directory / "p3-starter.exe")
        with tempfile.TemporaryDirectory(dir=fixture) as temp:
            target = Path(temp)
            with native_authority(home, True) as authority:
                ticket = decision(authority, root, target, metadata, register=True)
                for image in (P3_SERVICE, P3_WINDOW):
                    shutil.copyfile(image, home / "p3-control/bin" / image.name)

                class ControllerStopped(Exception):
                    pass

                def stop_controller():
                    authority.pipe.kill()
                    authority.pipe.wait(timeout=5)
                    raise ControllerStopped

                try:
                    publish(root, target, metadata, ticket, authority, interrupt=stop_controller)
                except ControllerStopped:
                    pass
                else:
                    raise RuntimeError("controller interruption missing")
            try:
                read_published_scene(root, target, metadata, ticket, home)
            except RuntimeError as exc:
                require("p3-publication-required" in str(exc), "unexpected partial read refusal")
            else:
                raise RuntimeError("partial copy released for reading")
            if has_archive(metadata):
                archive = target / (".pending-" + metadata[0]) / archive_name(metadata)
                original = target / "held-archive"
                archive.rename(original)
                shutil.copyfile(original, archive)
                try:
                    with native_authority(home, False) as authority:
                        publish(root, target, metadata, ticket, authority, resume=True)
                except RuntimeError as exc:
                    require(str(exc) == "native evidence refused" and archive.is_file(),
                            "unexpected archive replacement result")
                else:
                    raise RuntimeError("replacement archive published")
                archive.unlink()
                original.rename(archive)
            with native_authority(home, False) as authority:
                final = publish(root, target, metadata, ticket, authority, resume=True)
                published_id = object_id(final)
                authority.pipe.kill()
                authority.pipe.wait(timeout=5)
            with native_authority(foreign, True) as other:
                other_ticket = decision(other, root, target, metadata, register=True)
                require(not hmac.compare_digest(ticket, other_ticket), "shared installation proof")
                try:
                    publish(root, target, metadata, ticket, other)
                except RuntimeError as exc:
                    require(str(exc) == "unbound decision", "unexpected foreign refusal")
                else:
                    raise RuntimeError("foreign installation claimed the copy")
            with native_authority(home, False) as restored:
                reopened = publish(root, target, metadata, ticket, restored)
                require(object_id(reopened) == published_id and list(target.iterdir()) == [final],
                        "native restart duplicated the copy")
            scene = expected_scene(metadata)
            other_scene = P3_SOUTH if scene == P3_SCENE else P3_SCENE
            for unbound in (scene_frame([metadata[0], metadata[2]], scene),
                            scene_frame(metadata, other_scene)):
                with native_authority(home, False, read_only=True) as reader:
                    require(decision(reader, root, target, metadata) == ticket, "wrong read decision")
                    verify_bundle(final, ticket, metadata, reader)
                    try:
                        reader.deliver(unbound)
                    except RuntimeError as exc:
                        require('"stage":"p3-unbound-row"' in str(exc), "unexpected handoff refusal")
                    else:
                        raise RuntimeError("unbound scene delivered")
            window = home / "p3-control/bin" / P3_WINDOW.name
            original = window.with_suffix(".original")
            window.rename(original)
            try:
                shutil.copyfile(P3_SERVICE, window)
                try:
                    read_published_scene(root, target, metadata, ticket, home)
                except RuntimeError as exc:
                    require('"stage":"p3-unbound-row"' in str(exc), "unexpected window image refusal")
                else:
                    raise RuntimeError("substituted window image accepted")
            finally:
                window.unlink(missing_ok=True)
                original.rename(window)
            require(read_published_scene(root, target, metadata, ticket, home, interactive) == expected_scene(metadata),
                    "wrong published scene")
            with native_authority(home, False, read_only=True) as reader:
                require(decision(reader, root, target, metadata) == ticket, "wrong restart decision")
                verify_bundle(final, ticket, metadata, reader)
                reader.start_delivery(scene_frame(metadata, expected_scene(metadata)), interactive=True)
                reader.pipe.kill()
                reader.pipe.wait(timeout=5)
            require(retire_preview_profiles(home) == 1 and retire_preview_profiles(home) == 0,
                    "unexpected profile retirement count")
            try:
                with native_authority(home, False, read_only=True):
                    raise AssertionError("delivery reopened without end record")
            except RuntimeError as exc:
                require("win32-window-session-open" in str(exc), "unexpected delivery restart refusal")

    except BaseException as exc:
        exc.add_note(f"P3 control evidence retained in {fixture.name}/install-a")
        raise
    else:
        # Only the successful return path proved profile absence and the restart lock.
        shutil.rmtree(home)
    finally:
        shutil.rmtree(foreign)


def remove_first_then_stop(root, target, metadata, ticket, home):
    remove_confirmed(root, target, metadata, ticket, home, after_remove=lambda: os._exit(88))


def check_cleanup_decision(root, fixture, metadata):
    home = fixture / "install-a"
    home.mkdir()
    try:
        shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe",
                        home / "p3-starter.exe")
        with tempfile.TemporaryDirectory(dir=fixture) as temp:
            target = Path(temp)
            with native_authority(home, True) as authority:
                ticket = decision(authority, root, target, metadata, register=True)
                pending = target / (".pending-" + metadata[0])
                create_stage(root, pending, metadata, ticket, authority)
            unowned = pending / "project" / "foreign.bin"
            unowned.write_bytes(b"foreign")
            try:
                confirm_cleanup(root, target, metadata, ticket, home)
            except RuntimeError as exc:
                require(str(exc) == "unexpected staging contents", "wrong cleanup refusal")
            else:
                raise RuntimeError("foreign staging authorized")
            unowned.unlink()
            proof = confirm_cleanup(root, target, metadata, ticket, home)
            require(confirm_cleanup(root, target, metadata, ticket, home) == proof,
                    "cleanup decision lost on restart")
            run_to_exit(remove_first_then_stop, (root, target, metadata, ticket, home), 88)
            # Remove the next scheduled object without its journal step.
            order = sorted(metadata[3], key=lambda name: (-name.count("/"), name))
            victim = pending / "project" / order[1]
            require(victim.is_file(), "expected pending cleanup file")
            retained = target / "retained"
            victim.rename(retained)
            try:
                remove_confirmed(root, target, metadata, ticket, home)
            except RuntimeError as exc:
                require(str(exc) == "unconfirmed cleanup progress", "missing object counted as removed")
            else:
                raise RuntimeError("missing object accepted")
            victim.write_bytes(retained.read_bytes())
            try:
                remove_confirmed(root, target, metadata, ticket, home)
            except RuntimeError as exc:
                require(str(exc) == "replaced cleanup object" and victim.exists(), "replacement removed")
            else:
                raise RuntimeError("replacement accepted")
            victim.unlink()
            retained.rename(victim)
            remove_confirmed(root, target, metadata, ticket, home)
            remove_confirmed(root, target, metadata, ticket, home)
            require(not pending.exists(), "cleanup incomplete")
            pending.mkdir()
            try:
                remove_confirmed(root, target, metadata, ticket, home)
            except RuntimeError as exc:
                require(str(exc) == "completed cleanup path reused" and pending.is_dir(),
                        "terminal cleanup reused")
            else:
                raise RuntimeError("old decision reused after completion")
            pending.rmdir()  # Fixture-owned replacement.

            try:
                with native_authority(home, False):
                    raise AssertionError("transfer reopened after cleanup decision")
            except RuntimeError as exc:
                require("p3-terminal-decision" in str(exc), "unexpected restart refusal")
    finally:
        shutil.rmtree(home)


def tag(key, domain, fields, register=False):
    encoded = json.dumps(fields, ensure_ascii=True, sort_keys=True,
                         separators=(",", ":")).encode("ascii")
    payload = domain + b"\0" + encoded
    if isinstance(key, NativeAuthority):
        digest = hashlib.sha256(payload).digest()
        if domain == NativeAuthority.domains[0]:
            key.plan_digest = digest
            service = hashlib.sha256(P3_SERVICE.read_bytes()).digest()
            window = hashlib.sha256(P3_WINDOW.read_bytes()).digest()
            digest = hashlib.sha256(b"KRAQEL-P3-PREVIEW-v2" + digest + service + window
                                    + scene_frame(fields[-1], expected_scene(fields[-1]))).digest()
        return key.attest(domain, digest, register)
    return hmac.digest(key, payload, "sha256").hex()


def decision(key, source, destination, metadata, register=False):
    return tag(key, b"KRAQEL-P3-FIXTURE-DECISION-v1",
               [str(source), object_id(source), object_id(destination), metadata], register)


def receipt(key, approved, bundle, register=False):
    return tag(key, b"KRAQEL-P3-FIXTURE-RECEIPT-v1",
               [approved, object_id(bundle), object_id(bundle / "project")], register)


@contextmanager
def target_lock(destination, project_id):
    name = "Local\\KRAQEL-P3-fixture-" + project_id + "-" + "-".join(map(str, object_id(destination)))
    kernel.CreateMutexW.restype = wintypes.HANDLE
    handle = kernel.CreateMutexW(None, False, name)
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    handle = wintypes.HANDLE(handle)
    try:
        # An abandoned mutex establishes exclusive access, never recovery authority.
        require(kernel.WaitForSingleObject(handle, 0) in (0, 0x80), "copy in progress")
        try:
            yield
        finally:
            kernel.ReleaseMutex(handle)
    finally:
        kernel.CloseHandle(handle)


def publish(root, destination, metadata, approved, key, interrupt=None, resume=False):
    with volume_guard(root.anchor, True), pinned_directory(root), pinned_directory(destination):
        require(hmac.compare_digest(decision(key, root, destination, metadata), approved),
                "unbound decision")
        project_id = str(uuid.UUID(metadata[0]))
        with target_lock(destination, project_id):
            return publish_locked(root, destination, project_id, metadata, approved, key, interrupt, resume)


def publish_locked(root, destination, project_id, metadata, approved, key, interrupt, resume):
    final = destination / project_id
    bundle = destination / (".pending-" + project_id)
    pending = os.path.lexists(bundle)
    if pending:
        require(resume and not os.path.lexists(final), "unresolved staging")
    elif resume:
        raise RuntimeError("no pending transfer")
    elif os.path.lexists(final):
        return verify_bundle(final, approved, metadata, key)
    else:
        create_stage(root, bundle, metadata, approved, key)
    if pending and os.path.lexists(bundle / "receipt"):
        verify_bundle(bundle, approved, metadata, key)
    else:
        copy_project(root, bundle, metadata, approved, key, interrupt)
        with (bundle / "receipt").open("xb") as proof:
            proof.write(receipt(key, approved, bundle, register=True).encode("ascii"))
        if interrupt == "before-rename":
            os._exit(87)
    bundle.rename(final)
    return verify_bundle(final, approved, metadata, key)


def verify_bundle(final, approved, metadata, key):
    check_entry(final)
    names = {"receipt", "reservation", "project"}
    if len(metadata) == 7 and metadata[6]["groups"]:
        names.add(archive_name(metadata))
    require({p.name for p in final.iterdir()} == names, "unexpected bundle")
    proof = final / "receipt"
    require(check_entry(proof).st_nlink == 1 and proof.is_file() and proof.stat().st_size == 64,
            "invalid receipt")
    reservation = final / "reservation"
    require(check_entry(reservation).st_nlink == 1 and reservation.is_file()
            and reservation.stat().st_size == 64 and hmac.compare_digest(
                reservation.read_bytes(), stage_tag(final, metadata, approved, key)), "unbound reservation")
    require(hmac.compare_digest(proof.read_bytes(), receipt(key, approved, final).encode("ascii")),
            "unbound receipt")
    require(inventory(final / "project") == metadata[3], "changed destination")
    if len(metadata) == 7:
        approved_archive(final, metadata)
    return final


def publish_without_reply(root, destination, metadata, approved, key):
    publish(root, destination, metadata, approved, key)
    os._exit(86)


def run_to_exit(operation, args, code):
    worker = multiprocessing.get_context("spawn").Process(target=operation, args=args)
    worker.start()
    try:
        worker.join(30)
        require(worker.exitcode == code, "publication boundary not reached")
    finally:
        if worker.is_alive():
            worker.kill()
            worker.join()
        worker.close()


def capture(root, fixture, interactive=False):
    metadata = json.loads((fixture / "expected.json").read_text())
    require(metadata[3] == inventory(root), "unexpected source")
    verify_selection(fixture, root, metadata, metadata[4])
    native_restart(root, fixture, metadata, interactive)
    check_cleanup_decision(root, fixture, metadata)
    if len(metadata) == 7:
        return  # The selected recovery run covers the native publication and cleanup above.
    # The fixture controller supplies one explicit decision. Its key lasts
    # across worker processes but grants no authority after the controller exits.
    key = secrets.token_bytes(32)
    with tempfile.TemporaryDirectory(dir=fixture) as temp:
        destination = Path(temp)
        approved = decision(key, root, destination, metadata)
        with tempfile.TemporaryDirectory(dir=fixture) as other:
            try:
                publish(root, Path(other), metadata, approved, key)
            except RuntimeError as exc:
                require(str(exc) == "unbound decision" and not list(Path(other).iterdir()),
                        "wrong target was touched")
            else:
                raise RuntimeError("foreign destination accepted")

        with tempfile.TemporaryDirectory(dir=fixture) as partial:
            target = Path(partial)
            ticket = decision(key, root, target, metadata)
            run_to_exit(publish, (root, target, metadata, ticket, key, "first-byte"), 85)
            pending = target / (".pending-" + metadata[0])
            prefix = inventory(pending)
            require(any(value is not None for value in prefix.values()), "missing partial transfer")
            try:
                publish(root, target, metadata, ticket, key)
            except RuntimeError as exc:
                require(str(exc) == "unresolved staging" and inventory(pending) == prefix
                        and list(target.iterdir()) == [pending], "partial transfer released")
            else:
                raise RuntimeError("partial staging accepted")

            unexpected = pending / "project" / "unknown.bin"
            unexpected.write_bytes(b"foreign fixture")
            try:
                publish(root, target, metadata, ticket, key, resume=True)
            except RuntimeError as exc:
                require(str(exc) == "unexpected staging contents" and unexpected.read_bytes() == b"foreign fixture",
                        "foreign staging modified")
            else:
                raise RuntimeError("foreign staging accepted")
            unexpected.unlink()  # The fixture creator removes only its injected file.
            require(inventory(pending) == prefix, "partial copy changed after refusal")
            partial_file = pending / "project" / "assets" / "nested" / "cover.png"
            first_byte = partial_file.read_bytes()
            require(len(first_byte) == 1, "missing crash prefix")
            partial_file.write_bytes(b"!")
            changed = inventory(pending)
            try:
                publish(root, target, metadata, ticket, key, resume=True)
            except RuntimeError as exc:
                require(str(exc) == "staging prefix mismatch" and inventory(pending) == changed,
                        "unverified prefix modified")
            else:
                raise RuntimeError("wrong prefix accepted")
            partial_file.write_bytes(first_byte)
            run_to_exit(publish, (root, target, metadata, ticket, key, "before-rename", True), 87)
            completed_id = object_id(pending)
            proof = pending / "receipt"
            genuine = proof.read_bytes()
            proof.write_bytes(b"0" * len(genuine))
            try:
                publish(root, target, metadata, ticket, key, resume=True)
            except RuntimeError as exc:
                require(str(exc) == "unbound receipt" and list(target.iterdir()) == [pending],
                        "invalid receipt released")
            else:
                raise RuntimeError("wrong completion receipt accepted")
            proof.write_bytes(genuine)
            completed = publish(root, target, metadata, ticket, key, resume=True)
            require(object_id(completed) == completed_id, "prepared bundle replaced")
            require(list(target.iterdir()) == [completed] and inventory(completed / "project") == metadata[3],
                    "resume did not finish the original transfer")

        run_to_exit(publish_without_reply, (root, destination, metadata, approved, key), 86)

        final = destination / metadata[0]
        before = object_id(final)
        reopened = publish(root, destination, metadata, approved, key)
        require(object_id(reopened) == before and list(destination.iterdir()) == [final],
                "duplicate publication")
        fresh_key = secrets.token_bytes(32)
        try:
            publish(root, destination, metadata, decision(fresh_key, root, destination, metadata), fresh_key)
        except RuntimeError as exc:
            require(str(exc) == "unbound reservation" and object_id(final) == before,
                    "unexpected controller recovery")
        else:
            raise RuntimeError("new controller claimed the earlier publication")
        (final / "project" / "assets" / "nested" / "cover.png").write_bytes(b"changed fixture")
        try:
            publish(root, destination, metadata, approved, key)
        except RuntimeError as exc:
            require(str(exc) == "changed destination", "unexpected replay refusal")
        else:
            raise RuntimeError("changed destination accepted")


if __name__ == "__main__":
    if sys.argv[1:2] in (["check-prepared"], ["prepared"], ["prepared-preview"], ["compare-prepared"], ["check-prepared-comparison"]) and os.environ.get("KRAQEL_P3_ETW_ROOTS") == "1":
        print(f"P3-ETW-CONTROLLER {os.getpid()}", flush=True)
    if sys.argv[1:] in (["compare-prepared"], ["check-prepared-comparison"]):
        compare_prepared(interactive=sys.argv[1] == "compare-prepared")
        sys.exit(0)
    if sys.argv[1:] == ["check-prepared"]:
        check_prepared_refusals()
        sys.exit(0)
    if len(sys.argv) == 3 and sys.argv[1] == "export-supplement":
        destination = Path(sys.argv[2]).absolute()
        require(destination.name.lower().endswith(".zip.age") and not os.path.lexists(destination),
                "Bitte einen neuen Dateinamen mit Endung .zip.age wählen.")
        writable_fixture(interactive_selection=True, export_to=destination)
        sys.exit(0)
    if sys.argv[1:] in (["writable"], ["select-writable"]):
        writable_fixture(interactive_selection=sys.argv[1] == "select-writable")
        sys.exit(0)
    mode, volume, arg = sys.argv[1:]
    require(re.fullmatch(r"\\\\\?\\Volume\{[0-9a-fA-F-]{36}\}\\?", volume), "invalid fixture volume")
    volume = volume.rstrip("\\") + "\\"
    root = Path(volume) / "project"
    if mode in ("select", "prepared", "prepared-preview", "selected-preview", "selected-recovery", "selected-writer"):
        root = Path(volume) / "selected/project"
    if mode in ("writer", "selected-writer"):
        require(arg in ("ro", "rw"), "invalid writer mode")
        with volume_guard(volume, arg == "ro"):
            writer(root, arg == "ro")
    else:
        fixture = Path(arg)
        require(fixture.parent.resolve() == Path(tempfile.gettempdir()).resolve()
                and re.fullmatch(r"kraqel-p3-[0-9a-f]{32}", fixture.name), "invalid fixture home")
        require(mode in ("prepare", "select", "prepared", "prepared-preview", "selected-preview", "selected-recovery", "capture", "preview"), "invalid fixture mode")
        preparing = mode in ("prepare", "select", "prepared")
        with volume_guard(volume, not preparing):
            if preparing:
                if mode == "prepared":
                    prepare_desktop_fixture(root, fixture)
                elif not prepare_captured_project(root, fixture, interactive_selection=mode == "select"):
                    sys.exit(10)
                challenge(root, False)
            elif mode in ("selected-preview", "prepared-preview"):
                preview_selected_project(root, fixture, interactive=mode == "selected-preview")
            else:
                capture(root, fixture, interactive=mode == "preview")
        if not preparing:
            require(inventory(root) == json.loads((fixture / "expected.json").read_text())[3],
                    "source changed during capture")
            shutil.rmtree(fixture / "selection-install")
            if mode in ("selected-preview", "prepared-preview"):
                print("PASS: selected WAL scene displayed from published copy")
                sys.exit(0)
            print("PASS: committed WAL scene read from published copy; native controller restart resumed partial transfer and confirmed publication; foreign installation refused; cleanup recovered; bound scene shown read-only; substituted scene and window image refused")
