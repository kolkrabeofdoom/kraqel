//! Fixed receipt for a broker-issued window ticket; no scene or command payload.
use sha2::{Digest, Sha256};

pub const ARG: &str = "--c4b-window-receipt";

pub fn receipt(ticket: &[u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"KRAQEL-C4B-WINDOW-RECEIPT\0\x01");
    hash.update(ticket);
    hash.finalize().into()
}
