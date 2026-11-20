//! Bind a provisioned native role package to one activated broker in the MACed history.
use super::*;
use crate::native_win32_package::{IMAGES, PinnedPackage};

mod suspended;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Snapshot {
    volume: u64,
    file_id: [u8; 16],
    sha256: [u8; 32],
}

struct SnapshotFile {
    file: fs::File,
    _dirs: Vec<fs::File>,
}
impl SnapshotFile {
    fn open(store: &Store, name: &'static str, create: bool) -> Result<Self> {
        let dir = store.root.join("data");
        let dirs = pin_directories(&dir)?;
        private_acl(&dir, true)?;
        let path = dir.join(name);
        let file = io(
            "scene-snapshot-open",
            OpenOptions::new()
                .read(true)
                .write(create)
                .create_new(create)
                .share_mode(0)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path),
        )?;
        let info = file_info(&file)?;
        if info.dwFileAttributes & (BAD_ATTRIBUTES | FILE_ATTRIBUTE_DIRECTORY) != 0
            || info.nNumberOfLinks != 1
        {
            return Err(invalid("scene-snapshot-file"));
        }
        private_acl(&path, false)?;
        Ok(Self { file, _dirs: dirs })
    }
    fn identity(&mut self) -> Result<Snapshot> {
        use std::io::{Read, Seek};
        let size = io("scene-snapshot-size", self.file.metadata())?.len();
        if !(1..=crate::window_receipt::MAX_SCENE_DB).contains(&size) {
            return Err(invalid("scene-snapshot-size"));
        }
        io("scene-snapshot-rewind", self.file.rewind())?;
        let mut bytes = Vec::with_capacity(size as usize);
        io(
            "scene-snapshot-read",
            (&mut self.file).take(size + 1).read_to_end(&mut bytes),
        )?;
        if bytes.len() as u64 != size {
            return Err(invalid("scene-snapshot-size"));
        }
        io("scene-snapshot-rewind", self.file.rewind())?;
        let (volume, file_id) = secure_fs::file_id(&self.file)?;
        Ok(Snapshot {
            volume,
            file_id,
            sha256: Sha256::digest(bytes).into(),
        })
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Session {
    #[default]
    Unused,
    Open([u8; 32]), // Digest of the unique, authenticated begin record.
    Closed,
}
impl Session {
    pub(super) fn require_closed(self) -> Result<()> {
        if matches!(self, Self::Open(_)) {
            return Err(invalid("win32-window-session-open"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum WindowMode {
    Suspended,
    Receipt,
    Surface,
    Interactive,
    Scene(crate::window_receipt::SceneOperation),
}
pub(super) enum Operation<'a> {
    Bind(&'a Path, [u8; 32]),
    Verify,
    CheckWindow(WindowMode),
}

impl Store {
    pub(super) fn win32_binding(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        operation: Operation,
    ) -> Result<Value> {
        let enrollment = match operation {
            Operation::Bind(path, digest) => Some((path, digest)),
            _ => None,
        };
        self.append_guard(actor, head)?;
        if self.state.schema != 4 || self.state.pending.is_some() || self.state.activated.len() < 2
        {
            return Err(invalid("win32-generation-unavailable"));
        }
        let active = self
            .state
            .win32_anchor
            .filter(|(owner, _)| *owner == actor)
            .map(|(_, anchor)| anchor);
        let expected = match (enrollment, active) {
            (Some((_, digest)), None) | (None, Some(digest)) if digest != [0; 32] => digest,
            _ => return Err(invalid("win32-package-binding-state")),
        };
        let broker = self.package(actor);
        let (_parents, mut managed, actual) = binary_pin(&broker)?;
        private_acl(&broker, false)?;
        if actual != actor {
            return Err(invalid("win32-managed-broker-mismatch"));
        }
        package::verify_win32_binding(&mut managed)?;
        let root = self.root.join("bin").join(format!("win32-{}", hex(&actor)));
        if let Some((source, _)) = enrollment {
            let _source_dirs = pin_directories(source)?;
            let _source_pins = io(
                "win32-provision-source",
                PinnedPackage::open(source, Some(expected)),
            )?;
            if !io("win32-managed-directory", root.try_exists())? {
                publish(source, &root)?;
            }
        }
        let _dirs = pin_directories(&root)?;
        private_acl(&root, true)?;
        let pins = io(
            "win32-package-pin",
            PinnedPackage::open(&root, Some(expected)),
        )?;
        for name in IMAGES.into_iter().chain(["anchor.json"]) {
            private_acl(&root.join(name), false)?;
        }
        if enrollment.is_some() {
            self.append(actor, head, Action::BindWin32Package { anchor: expected })?;
        }
        if let Operation::CheckWindow(mode) = operation {
            let image = pins
                .image(suspended::WINDOW)
                .ok_or(invalid("win32-window-not-pinned"))?;
            package::verify_window_session(&mut managed)?;
            if self.state.sequence + 2 >= MAX_RECORDS {
                return Err(invalid("win32-window-record-budget"));
            }
            use crate::window_receipt::SceneOperation::{ReadPersistent, SavePersistent};
            let saving = mode == WindowMode::Scene(SavePersistent);
            let mut snapshot = None;
            if saving || mode == WindowMode::Scene(ReadPersistent) {
                package::verify_scene_snapshot(&mut managed)?;
                if saving {
                    if self.state.scene_snapshot.is_some() {
                        return Err(invalid("scene-snapshot-already-bound"));
                    }
                } else {
                    let (owner, bound) = self
                        .state
                        .scene_snapshot
                        .ok_or(invalid("scene-snapshot-unbound"))?;
                    if owner != actor {
                        return Err(invalid("scene-snapshot-owner"));
                    }
                    let mut opened = SnapshotFile::open(self, "scene-a-v1.sqlite", false)?;
                    if opened.identity()? != bound {
                        return Err(invalid("scene-snapshot-mismatch"));
                    }
                    snapshot = Some(opened);
                }
            }
            self.append(actor, head, Action::BeginWin32Window { anchor: expected })?;
            let begin = self.state.head;
            let baseline = if saving {
                snapshot = Some(SnapshotFile::open(self, "scene-a-v1.sqlite", true)?);
                Some(SnapshotFile::open(self, "scene-a-base-v2.sqlite", true)?)
            } else {
                None
            };
            let mut result = if let WindowMode::Scene(scene_mode) = mode {
                suspended::read_scene(
                    self,
                    &root,
                    image,
                    pins.image("pipe-service.exe")
                        .ok_or(invalid("win32-service-not-pinned"))?,
                    scene_mode,
                    [
                        snapshot.as_ref().map(|s| &s.file),
                        baseline.as_ref().map(|s| &s.file),
                    ],
                )?
            } else {
                suspended::check_window(self, &root, image, mode)?
            };
            // Only the call holding the original lease reaches here after signaled process
            // end, empty job and profile cleanup. There is no external finish command.
            let end = match snapshot.as_mut() {
                Some(file) if saving => Action::EndWin32SceneSave {
                    begin,
                    snapshot: file.identity()?,
                },
                _ => Action::EndWin32Window { begin },
            };
            self.append_with_checked_context(actor, begin, end)?;
            result["headSha256"] = json!(hex(&self.state.head));
            return Ok(result);
        }
        Ok(
            json!({"nativePreviewOnly":true,"roleStarted":false,"dataAllowed":false,
            "actorSha256":hex(&actor),"anchorSha256":hex(&pins.anchor_digest),"headSha256":hex(&self.state.head)}),
        )
    }
}

// Publish a complete directory atomically. An interrupted staging directory cannot
// become active; a retry checks any already published bytes before recording the binding.
fn publish(source: &Path, target: &Path) -> Result<()> {
    let stage = target.with_file_name(format!("win32-stage-{}", Uuid::new_v4().simple()));
    secure_directory(&stage)?;
    let result = (|| {
        for name in IMAGES.into_iter().chain(["anchor.json"]) {
            let mut input = io("win32-stage-source", fs::File::open(source.join(name)))?;
            let mut output = io(
                "win32-stage-create",
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .share_mode(0)
                    .open(stage.join(name)),
            )?;
            io("win32-stage-copy", std::io::copy(&mut input, &mut output))?;
            io("win32-stage-sync", output.sync_all())?;
        }
        check(
            unsafe {
                MoveFileExW(
                    path_wide(&stage)?.as_ptr(),
                    path_wide(target)?.as_ptr(),
                    MOVEFILE_WRITE_THROUGH,
                )
            },
            "win32-stage-publish",
        )
    })();
    if result.is_err() {
        for name in IMAGES.into_iter().chain(["anchor.json"]) {
            let _ = fs::remove_file(stage.join(name));
        }
        let _ = fs::remove_dir(stage);
    }
    result
}
