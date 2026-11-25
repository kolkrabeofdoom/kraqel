use crate::{ExportError, ExportFormat, internal};
use kraqel_domain::EntityId;
use std::{fs, io::Write, path::Path};

/// Holds a verified local directory while its catalog or exports are in use.
/// Windows excludes concurrent rename, deletion and reparse-point modification.
///
/// # Errors
/// Fails if the directory cannot be opened with the required sharing rights or is redirected.
#[cfg(windows)]
pub fn pin_excerpt_project(path: &Path) -> Result<fs::File, ExportError> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const FILE_SHARE_READ: u32 = 1;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

    let directory = fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(internal)?;
    let metadata = directory.metadata().map_err(internal)?;
    if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(ExportError::UnsafeDestination);
    }
    Ok(directory)
}

/// Opens an existing directory without following a final symbolic link.
///
/// # Errors
/// Rejects missing or redirected directories.
#[cfg(not(windows))]
pub fn pin_excerpt_project(path: &Path) -> Result<fs::File, ExportError> {
    let metadata = fs::symlink_metadata(path).map_err(internal)?;
    if !metadata.file_type().is_dir() {
        return Err(ExportError::UnsafeDestination);
    }
    fs::File::open(path).map_err(internal)
}

pub(super) fn publish(
    project: &Path,
    prefix: &str,
    format: ExportFormat,
    bytes: &[u8],
) -> Result<String, ExportError> {
    if !matches!(prefix, "szene" | "kapitel" | "manuskript") {
        return Err(ExportError::UnsafeDestination);
    }
    let _project_handle = pin_excerpt_project(project)?;
    let directory = project.join("exports");
    match fs::create_dir(&directory) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(internal(e)),
    }
    let _directory_handle = pin_excerpt_project(&directory)?;
    let name = format!("{prefix}-{}.{}", EntityId::new(), format.extension());
    let mut staged = tempfile::NamedTempFile::new_in(&directory).map_err(internal)?;
    staged.write_all(bytes).map_err(internal)?;
    staged.as_file().sync_all().map_err(internal)?;
    staged
        .persist_noclobber(directory.join(&name))
        .map_err(|e| internal(e.error))?;
    Ok(name)
}

/// Metadata for a local native output; listing never opens the manuscript file.
pub struct ExcerptFile {
    pub file_name: String,
    pub byte_count: u64,
    pub modified: std::time::SystemTime,
}

fn native_output_name(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    let Some((scope, id)) = stem.split_once('-') else {
        return false;
    };
    matches!(extension, "md" | "html" | "docx" | "pdf")
        && matches!(scope, "manuskript" | "kapitel" | "szene")
        && EntityId::parse(id).is_some_and(|value| value.to_string() == id)
}

/// Lists regular files matching the native output naming scheme, newest first.
///
/// # Errors
/// Rejects redirected directories and filesystem errors without reading any file body.
pub fn list_excerpt_files(project: &Path) -> Result<Vec<ExcerptFile>, ExportError> {
    let _project_handle = pin_excerpt_project(project)?;
    let directory = project.join("exports");
    match fs::symlink_metadata(&directory) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(internal(e)),
        Ok(_) => {}
    }
    let _directory_handle = pin_excerpt_project(&directory)?;
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).map_err(internal)? {
        let entry = entry.map_err(internal)?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !native_output_name(&name) {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(internal(e)),
        };
        #[cfg(windows)]
        if std::os::windows::fs::MetadataExt::file_attributes(&metadata) & 0x400 != 0 {
            continue;
        }
        if metadata.is_file() {
            files.push(ExcerptFile {
                file_name: name,
                byte_count: metadata.len(),
                modified: metadata.modified().map_err(internal)?,
            });
        }
    }
    files.sort_unstable_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.file_name.cmp(&b.file_name))
    });
    Ok(files)
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn redirected_export_directory_is_rejected_for_publication_and_listing() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir(project.path().join("target")).unwrap();
        let exports = project.path().join("exports");
        // A directory junction on local NTFS needs no symlink privilege.
        let result = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J", "exports", "target"])
            .current_dir(project.path())
            .output();
        assert!(result.unwrap().status.success());
        assert!(matches!(
            publish(project.path(), "szene", ExportFormat::Markdown, b"test"),
            Err(ExportError::UnsafeDestination)
        ));
        assert!(matches!(
            list_excerpt_files(project.path()),
            Err(ExportError::UnsafeDestination)
        ));
        assert_eq!(
            fs::read_dir(project.path().join("target")).unwrap().count(),
            0
        );
        fs::remove_dir(exports).unwrap();
    }
}
