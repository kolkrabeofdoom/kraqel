//! Fixed, guest-only SQLite fixture. Neither paths nor SQL are supplied by the channel.
use crate::*;
use kraqel_native_scene_model::{Mode, Reply, Scene, SyntheticCatalog};
use rusqlite::{Connection, OpenFlags, params};
use std::{
    fs::{File, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, Isolation::*, *},
    Storage::FileSystem::*,
};

pub const READER: &str = "org.kraqel.bp4.store-reader";
const WRITER: &str = "org.kraqel.bp4.data";
#[derive(Clone, Copy, PartialEq)]
pub enum StorePhase {
    Partial,
    ReadPartial,
    Timeout,
    ReadTimeout,
    WindowClose,
    ReadClosed,
    Before,
    Recover,
    ReadOld,
    After,
    ReadNew,
}
impl StorePhase {
    pub fn scope(self) -> &'static str {
        match self {
            Self::WindowClose => "A/edit/window-close",
            Self::ReadClosed => "A/read/window-close",
            Self::Partial => "A/edit/partial",
            Self::ReadPartial => "A/read/partial",
            Self::Timeout => "A/edit/timeout",
            Self::ReadTimeout => "A/read/timeout",
            Self::Before => "A/edit/crash-before",
            Self::After => "A/edit/crash-after",
            Self::Recover => "A/recovery/crash-before",
            Self::ReadOld => "A/read/crash-before",
            Self::ReadNew => "A/read/crash-after",
        }
    }
    pub const ALL: [Self; 11] = [
        Self::WindowClose,
        Self::ReadClosed,
        Self::Partial,
        Self::ReadPartial,
        Self::Timeout,
        Self::ReadTimeout,
        Self::Before,
        Self::Recover,
        Self::ReadOld,
        Self::After,
        Self::ReadNew,
    ];
    pub fn reader(self) -> bool {
        matches!(
            self,
            Self::ReadClosed
                | Self::ReadOld
                | Self::ReadNew
                | Self::ReadPartial
                | Self::ReadTimeout
        )
    }
    pub fn writer(self) -> bool {
        matches!(
            self,
            Self::WindowClose | Self::Before | Self::After | Self::Partial | Self::Timeout
        )
    }
    pub fn exits(self) -> (u32, u32) {
        match self {
            Self::WindowClose => (97, 98),
            Self::ReadClosed => (99, 100),
            Self::Partial => (89, 90),
            Self::ReadPartial => (91, 92),
            Self::Timeout => (93, 94),
            Self::ReadTimeout => (95, 96),
            Self::Before => (80, 82),
            Self::After => (81, 82),
            Self::Recover => (83, 84),
            Self::ReadOld => (85, 86),
            Self::ReadNew => (87, 88),
        }
    }
}
fn directory(phase: Option<StorePhase>) -> std::path::PathBuf {
    Path::new(DIR).join(match phase {
        None => "scene-store-v1",
        Some(StorePhase::WindowClose | StorePhase::ReadClosed) => "scene-store-close",
        Some(StorePhase::After | StorePhase::ReadNew) => "scene-store-after",
        Some(StorePhase::Partial | StorePhase::ReadPartial) => "scene-store-partial",
        Some(StorePhase::Timeout | StorePhase::ReadTimeout) => "scene-store-timeout",
        Some(StorePhase::Before | StorePhase::Recover | StorePhase::ReadOld) => {
            "scene-store-before"
        }
    })
}
fn sid_text(profile: &str) -> io::Result<String> {
    unsafe {
        let mut sid = null_mut();
        require(
            DeriveAppContainerSidFromAppContainerName(wide(profile).as_ptr(), &mut sid) >= 0,
            "fixture SID",
        )?;
        let mut ptr = null_mut();
        let result = check(ConvertSidToStringSidW(sid, &mut ptr));
        FreeSid(sid);
        result?;
        let len = (0..184).find(|i| *ptr.add(*i) == 0);
        let text = len.and_then(|n| String::from_utf16(std::slice::from_raw_parts(ptr, n)).ok());
        LocalFree(ptr.cast());
        text.ok_or(io::Error::other("SID length"))
    }
}

fn pin(path: &Path, is_dir: bool) -> io::Result<File> {
    OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if is_dir {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        )
        .open(path)
}

pub fn pin_existing_v1() -> io::Result<(File, File)> {
    let root = directory(None);
    Ok((
        pin(&root, true)?,
        pin(&root.join("synthetic.sqlite"), false)?,
    ))
}

/// Called under the starter's exclusive lock. Existing roots are refused. Both
/// freshly created objects remain pinned against rename/deletion through both runs.
pub fn prepare(phase: Option<StorePhase>) -> io::Result<(File, File)> {
    guest_image("pipe-check.exe")?;
    let acl = format!(
        "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;0x1301bf;;;{})(A;OICI;0x1200a9;;;{})S:(ML;OICI;NW;;;LW)",
        sid_text(WRITER)?,
        sid_text(READER)?
    );
    unsafe {
        let mut sd = null_mut();
        check(ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide(acl).as_ptr(),
            1,
            &mut sd,
            null_mut(),
        ))?;
        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd,
            bInheritHandle: 0,
        };
        let result = check(CreateDirectoryW(wide(directory(phase)).as_ptr(), &sa));
        LocalFree(sd);
        result?;
    }
    let root = pin(&directory(phase), true)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(directory(phase).join("synthetic.sqlite"))?;
    let db = pin(&directory(phase).join("synthetic.sqlite"), false)?;
    drop(file);
    Ok((root, db))
}

pub struct Store {
    managed: bool,
    db: Connection,
    mode: Mode,
    phase: Option<StorePhase>,
}
fn failure(_: rusqlite::Error) -> io::Error {
    io::Error::other("fixture SQLite")
}

impl Store {
    pub fn open(read_only: bool, phase: Option<StorePhase>) -> io::Result<Self> {
        if read_only {
            require(
                matches!(OpenOptions::new().write(true).open(directory(phase).join("synthetic.sqlite")), Err(ref e) if e.kind()==io::ErrorKind::PermissionDenied),
                "reader write access",
            )?;
        }
        let flags = if read_only {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        };
        let db = Connection::open_with_flags(
            directory(phase).join("synthetic.sqlite"),
            flags | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(failure)?;
        Self::configure(db, read_only, phase, SyntheticCatalog::default)
    }

    /// The managed service gets only this file handle; SQLite works in memory and
    /// never opens a caller-supplied path or a filesystem journal.
    pub fn managed(file: &mut File, reopening: bool) -> io::Result<Self> {
        if reopening {
            return Self::from_snapshot(file, Mode::Read);
        }
        require(file.metadata()?.len() == 0, "new snapshot")?;
        let db = Connection::open_in_memory().map_err(failure)?;
        let mut store = Self::configure(db, false, None, SyntheticCatalog::with_marked_scene_a)?;
        store.managed = true;
        store.flush_managed(file, false)?;
        Ok(store)
    }

    /// Always load through a read-only file handle. Edits affect only the in-memory copy.
    pub fn from_snapshot(file: &mut File, mode: Mode) -> io::Result<Self> {
        let size = file.metadata()?.len();
        require(
            (1..=crate::window_receipt::MAX_SCENE_DB).contains(&size),
            "snapshot size",
        )?;
        let mut db = Connection::open_in_memory().map_err(failure)?;
        db.deserialize_read_exact("main", file, size as usize, mode == Mode::Read)
            .map_err(failure)?;
        db.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY;")
            .map_err(failure)?;
        if mode == Mode::Read {
            db.execute_batch("PRAGMA query_only=ON").map_err(failure)?;
        }
        Ok(Self {
            db,
            mode,
            phase: None,
            managed: true,
        })
    }
    pub fn flush_managed(&self, file: &mut File, saving: bool) -> io::Result<()> {
        use std::io::Write;
        let image = self.db.serialize("main").map_err(failure)?;
        require(
            !image.is_empty() && image.len() as u64 <= crate::window_receipt::MAX_SCENE_DB,
            "snapshot size",
        )?;
        require(file.metadata()?.len() == 0, "new snapshot")?;
        if saving && cfg!(feature = "probe-stop-mid-snapshot") {
            file.write_all(&image[..image.len() / 2])?;
            file.sync_all()?;
            std::process::exit(82);
        }
        file.write_all(&image)?;
        file.sync_all()
    }
    fn configure(
        mut db: Connection,
        read_only: bool,
        phase: Option<StorePhase>,
        initial: fn() -> SyntheticCatalog,
    ) -> io::Result<Self> {
        db.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY;")
            .map_err(failure)?;
        if read_only || phase == Some(StorePhase::Recover) {
            db.execute_batch("PRAGMA query_only=ON").map_err(failure)?;
        } else {
            db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")
                .map_err(failure)?;
            let tx = db.transaction().map_err(failure)?;
            tx.execute_batch("CREATE TABLE scene(id TEXT PRIMARY KEY CHECK(id='A'), document TEXT NOT NULL, plain_text TEXT NOT NULL, revision INTEGER NOT NULL); CREATE VIRTUAL TABLE scene_fts USING fts5(plain_text);").map_err(failure)?;
            let (doc, rev) = initial().snapshot_a();
            tx.execute(
                "INSERT INTO scene VALUES ('A',?1,?2,?3)",
                params![doc, "Rabe\n42", rev],
            )
            .map_err(failure)?;
            tx.execute(
                "INSERT INTO scene_fts(rowid,plain_text) VALUES (1,?1)",
                ["Rabe\n42"],
            )
            .map_err(failure)?;
            tx.commit().map_err(failure)?;
        }
        Ok(Self {
            managed: false,
            db,
            phase,
            mode: if read_only || phase == Some(StorePhase::Recover) {
                Mode::Read
            } else {
                Mode::Edit
            },
        })
    }
    pub fn dispatch(
        &mut self,
        body: &[u8],
        on_window_end: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<Reply> {
        let tx = self.db.transaction().map_err(failure)?;
        let (content, rev): (String, u32) = tx
            .query_row(
                "SELECT document,revision FROM scene WHERE id='A' AND length(document)<=1024",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(failure)?;
        let mut catalog = (if self.managed {
            SyntheticCatalog::restore_managed_a(rev, &content)
        } else {
            SyntheticCatalog::restore_a(rev, &content)
        })
        .ok_or(io::Error::other("invalid stored fixture"))?;
        let mode = if body == kraqel_native_scene_model::channel::COMPARE_READ {
            Mode::Read
        } else {
            self.mode
        };
        let mut scene = catalog.bind(Scene::A, mode);
        let mut handle = |body: &[u8]| {
            if self.managed {
                scene.handle_managed(body)
            } else {
                scene.handle(body)
            }
        };
        let reply = handle(body);
        if let Reply::Saved { revision } = reply {
            let Reply::Read { text, .. } = handle(br#"{"command":"c4_document_read_v1"}"#) else {
                return Err(io::Error::other("invalid saved fixture"));
            };
            let (encoded, _) = catalog.snapshot_a();
            require(tx.execute("UPDATE scene SET document=?1,plain_text=?2,revision=?3 WHERE id='A' AND revision=?4",params![encoded,text,revision,rev]).map_err(failure)?==1,"revision check")?;
            require(
                tx.execute("UPDATE scene_fts SET plain_text=?1 WHERE rowid=1", [text])
                    .map_err(failure)?
                    == 1,
                "index row",
            )?;
        }
        if matches!(reply, Reply::Saved { .. }) && self.phase == Some(StorePhase::Before) {
            // Spill dirty pages while the rollback journal still protects revision zero.
            tx.cache_flush().map_err(failure)?;
            terminate_fixture(80)?;
        }
        if matches!(reply, Reply::Saved { .. }) && self.phase == Some(StorePhase::WindowClose) {
            tx.cache_flush().map_err(failure)?;
            on_window_end()?;
            tx.rollback().map_err(failure)?;
            return Err(io::ErrorKind::ConnectionAborted.into());
        }
        tx.commit().map_err(failure)?;
        if matches!(reply, Reply::Saved { .. }) && self.phase == Some(StorePhase::After) {
            terminate_fixture(81)?;
        }
        Ok(reply)
    }
    pub fn close(self) -> io::Result<()> {
        self.db.close().map_err(|(_, e)| failure(e))
    }
}
fn terminate_fixture(code: u32) -> io::Result<()> {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
    unsafe {
        check(TerminateProcess(GetCurrentProcess(), code))?;
    }
    Err(io::Error::other("service termination returned"))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_index_row_rolls_back_document_and_revision() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE scene(id,document,plain_text,revision); CREATE TABLE scene_fts(plain_text);").unwrap();
        let (doc, rev) = SyntheticCatalog::default().snapshot_a();
        db.execute(
            "INSERT INTO scene VALUES('A',?1,?2,?3)",
            params![doc, "Rabe\n42", rev],
        )
        .unwrap();
        let mut store = Store {
            managed: false,
            db,
            mode: Mode::Edit,
            phase: None,
        };
        assert!(
            store
                .dispatch(
                    br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#,
                    || unreachable!()
                )
                .is_err()
        );
        assert!(matches!(
            store
                .dispatch(br#"{"command":"c4_document_read_v1"}"#, || unreachable!())
                .unwrap(),
            Reply::Read {
                text,
                bold_second: false,
                revision: 0
            } if text == "Rabe\n42"
        ));
    }
}
