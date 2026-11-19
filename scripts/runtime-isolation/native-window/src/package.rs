//! The guest provisioner owns the protected package directory and its per-build anchor.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
use windows_sys::Win32::Storage::FileSystem::*;

pub(crate) const IMAGES: [&str; 4] = [
    "pipe-check.exe",
    "native-leaf.exe",
    "pipe-service.exe",
    "pipe-other.exe",
];

#[derive(Deserialize)]
struct Anchor {
    files: [Image; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Image {
    name: String,
    hash: String,
}

pub(crate) struct PinnedPackage {
    pub anchor_digest: [u8; 32],
    _pins: Vec<File>,
}

fn pin(path: &Path, directory: bool) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        )
        .open(path)?;
    let meta = file.metadata()?;
    if meta.file_attributes()
        & (FILE_ATTRIBUTE_REPARSE_POINT
            | FILE_ATTRIBUTE_OFFLINE
            | FILE_ATTRIBUTE_RECALL_ON_OPEN
            | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
        != 0
        || (directory && !meta.is_dir())
        || (!directory && !meta.is_file())
    {
        return Err(io::Error::other("unexpected package object"));
    }
    Ok(file)
}

impl PinnedPackage {
    pub fn open(root: &Path, expected: Option<[u8; 32]>) -> io::Result<Self> {
        let mut pins = vec![pin(root, true)?];
        let mut anchor = pin(&root.join("anchor.json"), false)?;
        if anchor.metadata()?.len() > 16 * 1024 {
            return Err(io::Error::other("package anchor too large"));
        }
        let mut bytes = Vec::new();
        anchor.read_to_end(&mut bytes)?;
        let anchor_digest: [u8; 32] = Sha256::digest(&bytes).into();
        if expected.is_some_and(|trusted| trusted != anchor_digest) {
            return Err(io::Error::other("package anchor mismatch"));
        }
        pins.push(anchor);
        let manifest: Anchor = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::other("invalid package anchor"))?;
        for name in IMAGES {
            // Only these constant names are ever used as path components.
            let entry = manifest
                .files
                .iter()
                .find(|item| item.name == name)
                .ok_or(io::Error::other("incomplete package inventory"))?;
            let mut file = pin(&root.join(name), false)?;
            let size = file.metadata()?.len();
            if !(1..=128 * 1024 * 1024).contains(&size) {
                return Err(io::Error::other("invalid package image length"));
            }
            let mut hash = Sha256::new();
            io::copy(&mut file, &mut hash)?;
            if !format!("{:x}", hash.finalize()).eq_ignore_ascii_case(&entry.hash) {
                return Err(io::Error::other("package image mismatch"));
            }
            pins.push(file);
        }
        Ok(Self {
            anchor_digest,
            _pins: pins,
        })
    }
}
