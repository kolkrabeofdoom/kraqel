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

const IMAGES: [&str; 4] = [
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

pub(super) struct PinnedPackage {
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
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (directory && !meta.is_dir())
        || (!directory && !meta.is_file())
    {
        return Err(io::Error::other("unexpected package object"));
    }
    Ok(file)
}

impl PinnedPackage {
    pub fn open(root: &Path) -> io::Result<Self> {
        let mut pins = vec![pin(root, true)?];
        let mut anchor = pin(&root.join("anchor.json"), false)?;
        if anchor.metadata()?.len() > 16 * 1024 {
            return Err(io::Error::other("package anchor too large"));
        }
        let mut bytes = Vec::new();
        anchor.read_to_end(&mut bytes)?;
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
            let mut hash = Sha256::new();
            io::copy(&mut file, &mut hash)?;
            if !format!("{:x}", hash.finalize()).eq_ignore_ascii_case(&entry.hash) {
                return Err(io::Error::other("package image mismatch"));
            }
            pins.push(file);
        }
        Ok(Self {
            anchor_digest: Sha256::digest(bytes).into(),
            _pins: pins,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_cannot_be_replaced_until_package_release_and_changed_bytes_are_denied() {
        let root = std::env::temp_dir().join(format!("kraqel-package-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut files = Vec::new();
        for name in IMAGES {
            std::fs::write(root.join(name), name).unwrap();
            files.push(
                serde_json::json!({"name":name, "hash":format!("{:x}",Sha256::digest(name))}),
            );
        }
        std::fs::write(
            root.join("anchor.json"),
            serde_json::json!({"files":files}).to_string(),
        )
        .unwrap();
        let package = PinnedPackage::open(&root).unwrap();
        let target = root.join("pipe-service.exe");
        assert_eq!(
            std::fs::rename(&target, root.join("old.exe"))
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(&target)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        drop(package);
        std::fs::write(target, b"changed").unwrap();
        assert!(PinnedPackage::open(&root).is_err());
        for name in IMAGES {
            std::fs::remove_file(root.join(name)).unwrap();
        }
        std::fs::remove_file(root.join("anchor.json")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
