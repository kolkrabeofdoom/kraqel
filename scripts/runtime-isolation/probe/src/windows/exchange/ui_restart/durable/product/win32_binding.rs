//! Bind a provisioned native role package to one activated broker in the MACed history.
use super::*;
use crate::native_win32_package::{IMAGES, PinnedPackage};

impl Store {
    pub(super) fn win32_binding(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        enrollment: Option<(&Path, [u8; 32])>,
    ) -> Result<Value> {
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
