//! Fixed receipt for a broker-issued window ticket; no scene or command payload.
use sha2::{Digest, Sha256};

pub const ARG: &str = "--c4b-window-receipt";

pub fn receipt(ticket: &[u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-C4B-WINDOW-RECEIPT\0\x01");
    hash.update(ticket);
    hash.finalize().into()
}

pub const UI_ARG: &str = "--c4b-window-ui";

pub fn commitment(grant: &[u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-C4B-WINDOW-OPEN\0\x01");
    hash.update(grant);
    hash.finalize().into()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneOperation {
    Once,
    CheckDeniedSave,
    CheckEditConflict,
    Revise,
    ReadRevision,
    KeyboardEdit,
    SavePersistent,
    ReadPersistent,
}
impl SceneOperation {
    pub fn revision_bound(self) -> bool {
        matches!(self, Self::Revise | Self::ReadRevision)
    }
    pub fn argument(self) -> &'static str {
        match self {
            Self::Revise => "--managed-scene-a-revise-v1",
            Self::ReadRevision => "--managed-scene-a-revision-read-v1",
            Self::KeyboardEdit => "--managed-scene-a-keyboard-v1",
            Self::SavePersistent => "--managed-scene-a-store-edit-v2",
            Self::ReadPersistent => "--managed-scene-a-store-read-v1",
            Self::Once => "--managed-scene-a-read-v1",
            Self::CheckEditConflict => "--managed-scene-a-edit-conflict-v1",
            Self::CheckDeniedSave => "--managed-scene-a-denied-save-v1",
        }
    }
}

pub const MAX_SCENE_DB: u64 = 128 * 1024;
