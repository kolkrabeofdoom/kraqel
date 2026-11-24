"""Internal helper for check-p3-readonly-volume.ps1. Uses only the synthetic fixture."""
import base64
import ctypes
from ctypes import wintypes
from contextlib import ExitStack, closing, contextmanager
import hashlib
import hmac
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


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


P3_SERVICE = Path(__file__).resolve().parent / "native-window/target/release/pipe-service.exe"
P3_WINDOW = P3_SERVICE.with_name("native-leaf.exe")
P3_SCENE = ("Szene A", "Text aus der Testkopie.", 0)


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


def capture_writable_fixture(root, selected_root, read_oplock, during_copy=None):
    """Capture one bounded logical instant. Return bytes only after every outstanding lease passes."""
    checks, files, snapshot = [], [], {}
    remaining = 32 * 1024 * 1024
    with ExitStack() as pins:
        def visit(path, depth):
            require(depth <= 32 and len(checks) < 256, "fixture tree budget exceeded")
            before = path.lstat()
            require(not before.st_file_attributes & 0x400, "reparse point")
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
            require(os.path.samestat(before, info) and not info.st_file_attributes & 0x400,
                    "changed capture identity")
            if depth == 0:
                require(os.path.samestat(selected_root, info), "selected project directory changed")
            checks.append(pins.enter_context(read_oplock(raw)))
            check_entry(path)
            name = path.relative_to(root).as_posix()
            if directory:
                if depth:
                    snapshot[name] = None
                for child in path.iterdir():
                    visit(child, depth + 1)
            else:
                require(info.st_nlink == 1 and path.is_file(), "unsupported capture file")
                reader = pins.enter_context(path.open("rb"))
                require(os.path.samestat(info, os.fstat(reader.fileno())), "changed capture file")
                files.append((name, reader, info.st_size))

        visit(root, 0)
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


def selection_digest(selected, candidate, metadata):
    require(len(selected) == 2 and all(type(n) is int and n > 0 for n in selected),
            "missing selected directory identity")
    payload = json.dumps([selected, object_id(candidate), metadata], ensure_ascii=True,
                         sort_keys=True, separators=(",", ":")).encode("ascii")
    return hashlib.sha256(b"KRAQEL-P3-SELECTED-SOURCE-v1\0" + payload).digest()


def verify_selection(fixture, candidate, metadata, selected):
    with native_authority(fixture / "selection-install", False) as authority:
        authority.attest(NativeAuthority.domains[0], selection_digest(selected, candidate, metadata[:4]), False)


def confirm_fixture_source():
    dialog = ctypes.WinDLL("user32", use_last_error=True).MessageBoxW
    dialog.argtypes = [wintypes.HWND, wintypes.LPCWSTR, wintypes.LPCWSTR, wintypes.UINT]
    dialog.restype = ctypes.c_int
    result = dialog(None,
                    "Den vorbereiteten synthetischen Projektordner „Szene A“ für diese lokale "
                    "Testkopie auswählen?",
                    "KRAQEL – P3-Quellauswahl", 0x1 | 0x100)  # OK/Cancel, default Cancel
    if result == 0:
        raise ctypes.WinError(ctypes.get_last_error())
    return result == 1


def prepare_captured_project(target, fixture, interactive_selection=False):
    import runpy
    read_oplock = runpy.run_path(str(Path(__file__).with_name("p3_directory_oplock.py")))["read_oplock"]
    with tempfile.TemporaryDirectory(prefix="kraqel-p3-writable-") as temp:
        source_fixture = Path(temp)
        root = source_fixture / "project"
        seed_writable_project(root, source_fixture)
        metadata = json.loads((source_fixture / "expected.json").read_text())

        selected_root = root.lstat()
        planned = dict(metadata[3])

        def capture(after_first=None):
            snapshot = capture_writable_fixture(root, selected_root, read_oplock, after_first)
            expected = {name: None if data is None else hashlib.sha256(data).hexdigest()
                        for name, data in snapshot.items()}
            require(expected == planned, "incomplete writable capture")
            with tempfile.TemporaryDirectory(dir=target.parent) as work:
                staged = Path(work) / "project"
                staged.mkdir()
                for name, data in snapshot.items():
                    if data is None:
                        (staged / name).mkdir()
                    else:
                        with (staged / name).open("xb") as dst:
                            dst.write(data)
                require(inventory(staged) == expected, "capture staging changed")
                # Register while the original selection and the staged candidate are still bound.
                with pinned_directory(staged):
                    home = fixture / "selection-install"
                    home.mkdir()
                    shutil.copyfile(Path(__file__).parent / "probe/target/debug/kraqel-isolation-probe.exe",
                                    home / "p3-starter.exe")
                    with native_authority(home, True) as authority:
                        authority.attest(NativeAuthority.domains[0],
                                         selection_digest(selected, staged, [*metadata[:3], expected]), True)
                staged.rename(target)
            return expected

        def add_asset():
            subprocess.run([sys.executable, "-I", "-c",
                            "import sys; open(sys.argv[1], 'xb').write(b'late asset')",
                            str(root / "assets/nested/late.png")], check=True, timeout=10)
        with pinned_directory(root):
            selected = object_id(root)
            require(selected == [selected_root.st_dev, selected_root.st_ino], "selected root replaced")
            if interactive_selection:
                if not confirm_fixture_source():
                    return False
            else:
                planned["assets/nested/late.png"] = hashlib.sha256(b"late asset").hexdigest()
                try:
                    capture(add_asset)
                except RuntimeError as exc:
                    require(str(exc) == "project changed during capture" and not target.exists(),
                            "uncertain capture released")
                else:
                    raise RuntimeError("concurrent asset omitted")
            metadata[3] = capture()
        metadata.append(selected)
        (fixture / "expected.json").write_text(json.dumps(metadata))
        return True


def writable_fixture(interactive_selection=False):
    fixture = Path(tempfile.gettempdir()) / ("kraqel-p3-" + uuid.uuid4().hex)
    fixture.mkdir()
    try:
        target = fixture / "captured"
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
        with fixture_database(target, metadata[3]) as db:
            rows = db.execute("SELECT title, plain_text, revision FROM document WHERE project_id=? AND id=?",
                              metadata[:2]).fetchall()
            require(rows == [P3_SCENE], "lost committed WAL scene")
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
    subprocess.run([sys.executable, "-I", __file__, "writer", str(root.parent),
                    "ro" if readonly else "rw"], check=True, timeout=15)


def commit_fixture_wal(root, document):
    db = sqlite3.connect(root / "kraqel.sqlite3")
    db.execute("PRAGMA wal_autocheckpoint=0")
    updated = db.execute("UPDATE document SET plain_text=? WHERE id=?", (P3_SCENE[1], document))
    require(updated.rowcount == 1, "missing WAL scene")
    db.commit()
    os._exit(82)  # Keep the committed WAL and its matching shared-memory index.


def seed_writable_project(root, fixture):
    root.mkdir()
    nested = root / "assets" / "nested"
    nested.mkdir(parents=True)
    (root / "empty").mkdir()
    png = base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jZQAAAABJRU5ErkJggg==")
    (nested / "cover.png").write_bytes(png)
    thumbnails = root / "assets" / "thumbnails"
    thumbnails.mkdir()
    (thumbnails / "cover.png").write_bytes(png)
    identity, document, asset = (str(uuid.uuid4()) for _ in range(3))
    with closing(sqlite3.connect(root / "kraqel.sqlite3")) as db:
        migrations = sorted((Path(__file__).resolve().parents[2] / "crates/storage/migrations").glob("*.sql"))
        require([m.name[:4] for m in migrations] == [f"{i:04d}" for i in range(1, 7)], "unexpected fixture schema")
        for migration in migrations:
            db.executescript(migration.read_text(encoding="utf-8"))
        db.execute("PRAGMA foreign_keys=ON")
        db.execute("PRAGMA user_version=6")
        db.execute("INSERT INTO project(id,name,schema_version,created_at,updated_at) VALUES(?, 'P3',1,'t','t')", (identity,))
        db.execute("INSERT INTO document(id,project_id,kind,title,plain_text,position,created_at,updated_at) "
                   "VALUES(?,?,'scene',?,?,0,'t','t')", (document, identity, P3_SCENE[0], "Vor dem WAL-Commit"))
        db.execute("INSERT INTO media_asset(id,project_id,kind,relative_path,thumbnail_path,original_name,"
                   "title,mime_type,checksum,byte_count,width,height,provenance,created_at,updated_at) "
                   "VALUES(?,?,'image','assets/nested/cover.png','assets/thumbnails/cover.png','cover.png',"
                   "'Cover','image/png',?,?,1,1,'imported','t','t')",
                   (asset, identity, hashlib.sha256(png).hexdigest(), len(png)))
        db.execute("INSERT INTO asset_link(project_id,asset_id,target_type,target_id,created_at) "
                   "VALUES(?,?,'document',?,'t')", (identity, asset, document))
        db.commit()
        require(db.execute("PRAGMA journal_mode=WAL").fetchone() == ("wal",), "WAL mode unavailable")
    main_before = hashlib.sha256((root / "kraqel.sqlite3").read_bytes()).digest()
    run_to_exit(commit_fixture_wal, (root, document), 82)
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
    (fixture / "expected.json").write_text(json.dumps([identity, document, asset, inventory(root)]))


def stage_tag(bundle, expected, approved, key, register=False):
    stage = bundle / "project"
    layout = []
    for name, digest in sorted(expected.items()):
        path = stage / name
        info = check_entry(path)
        require(info.st_ino and path.is_dir() == (digest is None), "invalid staging object")
        if digest is not None:
            require(path.is_file() and info.st_nlink == 1, "invalid staging file")
        layout.append([name, info.st_dev, info.st_ino])
    return layout_tag(bundle, approved, key, layout, register)


def layout_tag(bundle, approved, key, layout, register=False):
    return tag(key, b"KRAQEL-P3-FIXTURE-STAGING-v1",
               [approved, object_id(bundle), object_id(bundle / "project"), layout], register).encode("ascii")


def create_stage(bundle, expected, approved, key):
    bundle.mkdir()
    stage = bundle / "project"
    stage.mkdir()
    for name, digest in sorted(expected.items()):
        if digest is None:
            (stage / name).mkdir()
        else:
            with (stage / name).open("xb"):
                pass
    with (bundle / "reservation").open("xb") as proof:
        proof.write(stage_tag(bundle, expected, approved, key, register=True))


@contextmanager
def reserved_file(path):
    raw = open_file(str(path), 0xc0000000, 0, None, 3, 0x00200000, None)
    if raw == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    with os.fdopen(msvcrt.open_osfhandle(raw, os.O_RDWR | os.O_BINARY), "r+b") as file:
        yield file


@contextmanager
def checked_staging(root, bundle, expected, approved, key):
    stage = bundle / "project"
    entries = inventory(root)
    require(entries == expected, "source inventory mismatch")
    check_entry(bundle)
    require({p.name for p in bundle.iterdir()} == {"project", "reservation"}, "unexpected staging evidence")
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
        require(hmac.compare_digest(layout_tag(bundle, approved, key, layout), attestation),
                "unbound reservation")
        yield files, os.fstat(reservation_file.fileno())


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
    identity, document, asset, expected = metadata[:4]
    stage = bundle / "project"
    entries = expected
    with checked_staging(root, bundle, expected, approved, key) as (files, _):
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
        require(db.execute("SELECT title, plain_text, revision FROM document WHERE project_id=? AND id=?",
                           (identity, document)).fetchall() == [P3_SCENE], "incomplete scene revision")
        require(db.execute("pragma integrity_check").fetchall() == [("ok",)], "invalid copy")
        require(db.execute("select id from project").fetchall() == [(identity,)], "wrong project")
        require(not db.execute("pragma foreign_key_check").fetchall(), "invalid foreign key")
        links = db.execute("SELECT d.id,m.id,m.relative_path,m.thumbnail_path,m.checksum FROM document d "
                           "JOIN asset_link a ON a.target_type='document' AND a.target_id=d.id "
                           "AND a.project_id=d.project_id JOIN media_asset m ON m.id=a.asset_id "
                           "AND m.project_id=a.project_id WHERE d.project_id=?", (identity,)).fetchall()
        image = "assets/nested/cover.png"
        thumb = "assets/thumbnails/cover.png"
        require(links == [(document, asset, image, thumb, entries[image])]
                and entries[thumb] == entries[image], "incomplete scene asset binding")


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
def native_authority(home, create, cleanup=False, read_only=False):
    binary = home / "p3-starter.exe"
    operation = "authorize-p3-fixture" if create else "reopen-p3-fixture"
    if read_only:
        require(not create and not cleanup, "read requires existing publication")
        operation = "read-p3-fixture"
    if cleanup:
        require(not create, "cleanup requires existing installation")
        operation = "confirm-p3-fixture-cleanup"
    with subprocess.Popen([binary, "broker-store", operation], env={**os.environ, "USERPROFILE": str(home)},
                          stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE) as proc:
        try:
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
    with volume_guard(str(root.parent), True), pinned_directory(root), pinned_directory(destination):
        with native_authority(home, False, cleanup=True) as authority:
            require(hmac.compare_digest(decision(authority, root, destination, metadata), approved),
                    "unbound decision")
            project_id = str(uuid.UUID(metadata[0]))
            with target_lock(destination, project_id):
                require(not os.path.lexists(destination / project_id), "published destination")
                pending = destination / (".pending-" + project_id)
                with checked_staging(root, pending, metadata[3], approved, authority) as (files, info):
                    # Bind the decision to the exact verified prefixes and reservation file.
                    lengths = {name: dst.tell() for name, _, dst in files}
                    ids = {name: [os.fstat(dst.fileno()).st_dev, os.fstat(dst.fileno()).st_ino]
                           for name, _, dst in files}
                    layout = [[name, *(object_id(pending / "project" / name) if digest is None
                                       else ids[name])] for name, digest in sorted(metadata[3].items())]
                    plan = [approved, object_id(pending), object_id(pending / "project"), layout]
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
    with volume_guard(str(root.parent), True), pinned_directory(root), pinned_directory(destination):
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
    with volume_guard(str(root.parent), True), pinned_directory(root), pinned_directory(target):
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
                    require(len(row) == len(scene_frame(metadata, P3_SCENE)), "unsupported fixture row")
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
            with native_authority(home, False, read_only=True) as reader:
                require(decision(reader, root, target, metadata) == ticket, "wrong read decision")
                verify_bundle(final, ticket, metadata, reader)
                wrong_scene = [metadata[0], metadata[2]]  # Use the asset UUID in the scene field.
                try:
                    reader.deliver(scene_frame(wrong_scene, P3_SCENE))
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
            require(read_published_scene(root, target, metadata, ticket, home, interactive) == P3_SCENE,
                    "wrong published scene")
            with native_authority(home, False, read_only=True) as reader:
                require(decision(reader, root, target, metadata) == ticket, "wrong restart decision")
                verify_bundle(final, ticket, metadata, reader)
                reader.start_delivery(scene_frame(metadata, P3_SCENE), interactive=True)
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
                create_stage(pending, metadata[3], ticket, authority)
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
                                    + scene_frame(fields[-1], P3_SCENE)).digest()
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
    with volume_guard(str(root.parent), True), pinned_directory(root), pinned_directory(destination):
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
        create_stage(bundle, metadata[3], approved, key)
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
    require({p.name for p in final.iterdir()} == {"receipt", "reservation", "project"}, "unexpected bundle")
    proof = final / "receipt"
    require(check_entry(proof).st_nlink == 1 and proof.is_file() and proof.stat().st_size == 64,
            "invalid receipt")
    reservation = final / "reservation"
    require(check_entry(reservation).st_nlink == 1 and reservation.is_file()
            and reservation.stat().st_size == 64 and hmac.compare_digest(
                reservation.read_bytes(), stage_tag(final, metadata[3], approved, key)), "unbound reservation")
    require(hmac.compare_digest(proof.read_bytes(), receipt(key, approved, final).encode("ascii")),
            "unbound receipt")
    require(inventory(final / "project") == metadata[3], "changed destination")
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
    if sys.argv[1:] in (["writable"], ["select-writable"]):
        writable_fixture(interactive_selection=sys.argv[1] == "select-writable")
        sys.exit(0)
    mode, volume, arg = sys.argv[1:]
    require(re.fullmatch(r"\\\\\?\\Volume\{[0-9a-fA-F-]{36}\}\\?", volume), "invalid fixture volume")
    volume = volume.rstrip("\\") + "\\"
    root = Path(volume) / "project"
    if mode == "writer":
        require(arg in ("ro", "rw"), "invalid writer mode")
        with volume_guard(volume, arg == "ro"):
            writer(root, arg == "ro")
    else:
        fixture = Path(arg)
        require(fixture.parent.resolve() == Path(tempfile.gettempdir()).resolve()
                and re.fullmatch(r"kraqel-p3-[0-9a-f]{32}", fixture.name), "invalid fixture home")
        require(mode in ("prepare", "capture", "preview"), "invalid fixture mode")
        with volume_guard(volume, mode != "prepare"):
            if mode == "prepare":
                prepare_captured_project(root, fixture)
                challenge(root, False)
            else:
                capture(root, fixture, interactive=mode == "preview")
        if mode != "prepare":
            require(inventory(root) == json.loads((fixture / "expected.json").read_text())[3],
                    "source changed during capture")
            shutil.rmtree(fixture / "selection-install")
            print("PASS: committed WAL scene read from published copy; native controller restart resumed partial transfer and confirmed publication; foreign installation refused; cleanup recovered; bound scene shown read-only; substituted scene and window image refused")
