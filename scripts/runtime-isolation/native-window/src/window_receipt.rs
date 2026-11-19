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

pub const SCENE_ARG: &str = "--managed-scene-a-read-v1";
