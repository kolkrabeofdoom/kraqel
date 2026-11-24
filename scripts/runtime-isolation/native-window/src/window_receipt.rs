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

pub const INLINE_ABI: [u8; 32] = *b"KRAQEL-SCENE-INLINE-V1..........";

pub const COMPARE_ABI: [u8; 32] = *b"KRAQEL-SCENE-COMPARE-V1.........";

pub const MERGE_ABI: [u8; 32] = *b"KRAQEL-SCENE-MERGE-V1...........";

pub const REFRESH_ABI: [u8; 32] = *b"KRAQEL-SCENE-REFRESH-V1.........";

pub const GAP_ABI: [u8; 32] = *b"KRAQEL-SCENE-GAP-V1.............";
pub const REVIEWED_EXIT: u32 = 47;
pub const SCENE_REVISION_LIMIT: u32 = 256;

pub const REPEAT_ABI: [u8; 32] = *b"KRAQEL-SCENE-REPEAT-V1..........";

/// Fixed synthetic row: project UUID, scene UUID, revision LE, title NUL and text.
pub const P3_READ: &str = "--managed-p3-preview-v2";
pub const P3_CONTENT: &[u8] = b"Szene A\0Text aus der Testkopie.";
pub const P3_LEN: usize = 36 + P3_CONTENT.len();

pub const P3_COMPARE: &str = "--managed-p3-comparison-v2";
const MAX_PAIR: usize = 2 * 1024 * 1024;

pub fn p3_comparison(argument: Option<&str>) -> Option<bool> {
    match argument {
        Some(P3_COMPARE) => Some(true),
        Some(P3_READ) => Some(false),
        _ => None,
    }
}

// Comparison frames carry a little-endian length; the single-scene ABI stays fixed.
pub fn read_p3(input: &mut impl std::io::Read, comparison: bool) -> std::io::Result<Vec<u8>> {
    let size = if comparison {
        let mut length = [0; 4];
        input.read_exact(&mut length)?;
        let size = u32::from_le_bytes(length) as usize;
        if !(110..=MAX_PAIR).contains(&size) {
            return Err(std::io::Error::other("invalid comparison length"));
        }
        size
    } else {
        P3_LEN
    };
    let mut row = vec![0; size];
    input.read_exact(&mut row)?;
    Ok(row)
}

pub fn write_p3(
    output: &mut impl std::io::Write,
    comparison: bool,
    row: &[u8],
) -> std::io::Result<()> {
    if comparison {
        if !(110..=MAX_PAIR).contains(&row.len()) {
            return Err(std::io::Error::other("invalid comparison length"));
        }
        output.write_all(&(row.len() as u32).to_le_bytes())?;
    }
    output.write_all(row)
}

pub fn pair_parts(row: &[u8]) -> Option<(&[u8], &[u8])> {
    let left_len = u32::from_le_bytes(row.get(96..100)?.try_into().ok()?) as usize;
    let (left, right) = row.get(100..)?.split_at_checked(left_len)?;
    Some((left, right))
}

pub fn valid_p3_row(row: &[u8], comparison: bool) -> bool {
    if row.len() < 36 || row[..16] == [0; 16] || row[16..32] == [0; 16] || row[..16] == row[16..32]
    {
        return false;
    }
    if comparison {
        let Some((left, right)) = pair_parts(row) else {
            return false;
        };
        // This research installation may render only known synthetic text.
        let Some(text) = left.strip_prefix(b"\x01\0\0\0Szene B\0") else {
            return false;
        };
        let phrase = b"Die Flut erreicht den Hafen.";
        return row[32..64] != [0; 32]
            && row[64..96] != [0; 32]
            && row[32..64] != row[64..96]
            && matches!(text.len(), n if n == phrase.len() || n == 1024 * phrase.len())
            && text.chunks(phrase.len()).all(|chunk| chunk == phrase)
            && right == b"\x01\0\0\0Szene A\0Der Hafen liegt im Nebel.";
    }
    row.len() == P3_LEN
        && row[32..36] == [0; 4]
        && matches!(&row[36..], P3_CONTENT | b"Szene B\0Wind an der Suedkueste.")
}

pub fn p3_receipt(row: &[u8]) -> [u8; 32] {
    Sha256::new()
        .chain_update(b"KRAQEL-P3-ROW-RECEIVED-v1")
        .chain_update(row)
        .finalize()
        .into()
}
