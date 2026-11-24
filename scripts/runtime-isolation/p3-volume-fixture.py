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
import uuid


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


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


def writer(root, readonly):
    # Same process image, access and paths before and during capture.
    for directory in (root, root / "assets" / "nested"):
        target = directory / "late.png"
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


def prepare(root, fixture):
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
        db.execute("INSERT INTO document(id,project_id,kind,title,position,created_at,updated_at) "
                   "VALUES(?,?,'scene','Szene A',0,'t','t')", (document, identity))
        db.execute("INSERT INTO media_asset(id,project_id,kind,relative_path,thumbnail_path,original_name,"
                   "title,mime_type,checksum,byte_count,width,height,provenance,created_at,updated_at) "
                   "VALUES(?,?,'image','assets/nested/cover.png','assets/thumbnails/cover.png','cover.png',"
                   "'Cover','image/png',?,?,1,1,'imported','t','t')",
                   (asset, identity, hashlib.sha256(png).hexdigest(), len(png)))
        db.execute("INSERT INTO asset_link(project_id,asset_id,target_type,target_id,created_at) "
                   "VALUES(?,?,'document',?,'t')", (identity, asset, document))
        db.commit()
    challenge(root, False)
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


def copy_project(root, bundle, metadata, approved, key, interrupt):
    identity, document, asset, expected = metadata
    stage = bundle / "project"
    entries = inventory(root)
    require(entries == expected, "source inventory mismatch")
    check_entry(bundle)
    require({p.name for p in bundle.iterdir()} == {"project", "reservation"}, "unexpected staging evidence")
    reservation = bundle / "reservation"
    require(check_entry(reservation).st_nlink == 1 and reservation.is_file()
            and reservation.stat().st_size == 64, "invalid reservation")
    attestation = reservation.read_bytes()
    require(inventory(stage).keys() == entries.keys(), "unexpected staging contents")
    with ExitStack() as pins:
        pins.enter_context(pinned_directory(bundle))
        pins.enter_context(pinned_directory(stage))
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
        # All prefixes must pass before any destination byte changes.
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
    with closing(sqlite3.connect((stage / "kraqel.sqlite3").as_uri() + "?mode=ro", uri=True)) as db:
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
               b"KRAQEL-P3-FIXTURE-RECEIPT-v1")

    def __init__(self, pipe):
        self.pipe = pipe

    def attest(self, domain, digest, register):
        self.pipe.stdin.write(bytes((int(register), self.domains.index(domain))) + digest)
        self.pipe.stdin.flush()
        response = self.pipe.stdout.readline(66)
        require(re.fullmatch(b"[0-9a-f]{64}\n", response), "native evidence refused")
        return response[:-1].decode("ascii")


@contextmanager
def native_authority(home, create):
    binary = home / "p3-starter.exe"
    operation = "authorize-p3-fixture" if create else "reopen-p3-fixture"
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


def native_restart(root, fixture, metadata):
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
    finally:
        # Both directories were exclusively created by this synthetic run.
        for directory in (home, foreign):
            shutil.rmtree(directory)


def tag(key, domain, fields, register=False):
    encoded = json.dumps(fields, ensure_ascii=True, sort_keys=True,
                         separators=(",", ":")).encode("ascii")
    payload = domain + b"\0" + encoded
    if isinstance(key, NativeAuthority):
        return key.attest(domain, hashlib.sha256(payload).digest(), register)
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


def capture(root, fixture):
    metadata = json.loads((fixture / "expected.json").read_text())
    require(metadata[3] == inventory(root), "unexpected source")
    native_restart(root, fixture, metadata)
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
        require(mode in ("prepare", "capture"), "invalid fixture mode")
        with volume_guard(volume, mode == "capture"):
            (prepare if mode == "prepare" else capture)(root, fixture)
        if mode == "capture":
            print("PASS: native controller restart resumed partial transfer and confirmed publication; foreign installation refused; inventory and prefix checks passed")
