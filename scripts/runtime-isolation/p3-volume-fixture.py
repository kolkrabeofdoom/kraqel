"""Internal helper for check-p3-readonly-volume.ps1. Uses only the synthetic fixture."""
import base64
import ctypes
from ctypes import wintypes
from contextlib import closing, contextmanager
import hashlib
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
def volume_guard(volume, readonly):
    handle = open_file(volume, 0x80, 3, None, 3, 0x02000000, None)
    if handle == ctypes.c_void_p(-1).value:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        flags = wintypes.DWORD()
        label = ctypes.create_unicode_buffer(64)
        if not kernel.GetVolumeInformationByHandleW(wintypes.HANDLE(handle), label, len(label),
                                                   None, None, ctypes.byref(flags), None, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        require(label.value == "KRAQEL-P3", "foreign volume")
        require(bool(flags.value & 0x80000) == readonly, "unexpected volume write protection")
        yield
    finally:
        kernel.CloseHandle(wintypes.HANDLE(handle))


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


def capture(root, fixture):
    identity, document, asset, expected = json.loads((fixture / "expected.json").read_text())
    entries = inventory(root)
    require(entries == expected, "source inventory mismatch")
    with tempfile.TemporaryDirectory(dir=fixture) as temp:
        stage = Path(temp)
        for name, digest in sorted(entries.items()):
            if digest is None:
                (stage / name).mkdir()
            else:
                with (root / name).open("rb") as src, (stage / name).open("xb") as dst:
                    dst.write(src.read(1))
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
            print("PASS: whole fixture copied; root and nested creates denied; inventory and UUID verified")
