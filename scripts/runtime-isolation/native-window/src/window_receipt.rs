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
            Self::Revise => "--managed-scene-a-revise-v2",
            Self::ReadRevision => "--managed-scene-a-revision-read-v1",
            Self::KeyboardEdit => "--managed-scene-a-keyboard-v2",
            Self::SavePersistent => "--managed-scene-a-store-edit-v2",
            Self::ReadPersistent => "--managed-scene-a-store-read-v1",
            Self::Once => "--managed-scene-a-read-v1",
            Self::CheckEditConflict => "--managed-scene-a-edit-conflict-v1",
            Self::CheckDeniedSave => "--managed-scene-a-denied-save-v1",
        }
    }
}

pub const MAX_SCENE_DB: u64 = 128 * 1024;

pub const DISCARDED_EXIT: u32 = 42;
pub fn discard_receipt(material: &[u8; 96], window: bool) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-SCENE-DISCARD-V1");
    hash.update([u8::from(window)]);
    hash.update(material);
    hash.finalize().into()
}

pub fn checkpoint_argument(mode: SceneOperation) -> &'static str {
    match mode {
        SceneOperation::KeyboardEdit => "--managed-scene-a-checkpoint-v1",
        SceneOperation::Revise => "--managed-scene-a-checkpoint-revise-v1",
        _ => unreachable!("checkpoint requires a keyboard edit"),
    }
}

pub fn publication_receipt(frame: &[u8; 36]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-SCENE-PUBLICATION-V1");
    hash.update(frame);
    hash.finalize().into()
}

pub fn continue_argument(mode: SceneOperation) -> &'static str {
    match mode {
        SceneOperation::KeyboardEdit => "--managed-scene-a-continue-v1",
        SceneOperation::Revise => "--managed-scene-a-continue-revise-v1",
        _ => unreachable!("continuation requires a keyboard edit"),
    }
}

pub fn continue_receipt(frame: &[u8; 36]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-SCENE-CONTINUE-V1");
    hash.update(frame);
    hash.finalize().into()
}

// Exact read-only PE metadata for journal-budgeted continuation over the private bootstrap pipe.
pub const AUTOSAVE_ABI: [u8; 32] = *b"KRAQEL-WIN32-AUTOSAVE-V2........";

pub fn budgeted_argument(mode: SceneOperation) -> &'static str {
    match mode {
        SceneOperation::KeyboardEdit => "--managed-scene-a-continue-v2",
        SceneOperation::Revise => "--managed-scene-a-continue-revise-v2",
        _ => unreachable!("continuation requires a keyboard edit"),
    }
}

pub const COMPARE_ABI: [u8; 32] = *b"KRAQEL-SCENE-COMPARE-V1.........";
