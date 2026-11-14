//! Sicherer, lokaler Bildimport mit Inhaltsprüfung, Hashes und Vorschaudateien.

use std::{
    error::Error,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Cursor, Write},
    path::{Component, Path, PathBuf},
};

use image::{GenericImageView as _, ImageFormat, ImageReader};
use kraqel_domain::EntityId;
use sha2::{Digest as _, Sha256};

mod export;
pub use export::{ExportImage, prepare_export_image, read_export_original};

pub const MAX_ASSET_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 16_384;
pub const MAX_PIXELS: u64 = 40_000_000;
pub const THUMBNAIL_EDGE: u32 = 512;
pub const MAX_THUMBNAIL_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug)]
pub enum AssetError {
    InvalidName,
    UnsupportedFormat,
    InvalidImage,
    TooLarge,
    UnsafePath,
    AlreadyExists,
    Io(io::Error),
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName => formatter.write_str("Der Mediendateiname ist ungültig."),
            Self::UnsupportedFormat => {
                formatter.write_str("Das Bildformat wird nicht unterstützt.")
            }
            Self::InvalidImage => formatter.write_str("Die Bilddatei ist ungültig."),
            Self::TooLarge => {
                formatter.write_str("Die Bilddatei oder ihre Abmessungen sind zu groß.")
            }
            Self::UnsafePath => formatter.write_str("Der Assetpfad ist unsicher."),
            Self::AlreadyExists => formatter.write_str("Die Assetdatei existiert bereits."),
            Self::Io(error) => write!(
                formatter,
                "Mediendatei konnte nicht verarbeitet werden: {error}"
            ),
        }
    }
}

impl Error for AssetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for AssetError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedImage {
    pub original_name: String,
    pub mime_type: &'static str,
    pub extension: &'static str,
    pub checksum: String,
    pub width: u32,
    pub height: u32,
    pub original_bytes: Vec<u8>,
    pub thumbnail_png: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAssetFiles {
    pub relative_path: String,
    pub thumbnail_path: String,
}

#[must_use]
pub fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Validiert Name, Bytegröße, Signatur, Dekodierbarkeit und Abmessungen.
///
/// # Errors
/// Liefert einen Fehler für nicht unterstützte, beschädigte oder übergroße Bilder.
pub fn prepare_image(original_name: &str, bytes: &[u8]) -> Result<PreparedImage, AssetError> {
    validate_file_name(original_name)?;
    if bytes.is_empty() || bytes.len() > MAX_ASSET_BYTES {
        return Err(AssetError::TooLarge);
    }
    let format = image::guess_format(bytes).map_err(|_| AssetError::UnsupportedFormat)?;
    let (mime_type, extension) = format_contract(format)?;
    require_matching_extension(original_name, format)?;
    let reader = ImageReader::with_format(Cursor::new(bytes), format);
    let (width, height) = reader
        .into_dimensions()
        .map_err(|_| AssetError::InvalidImage)?;
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(AssetError::TooLarge);
    }
    let decoded =
        image::load_from_memory_with_format(bytes, format).map_err(|_| AssetError::InvalidImage)?;
    if decoded.dimensions() != (width, height) {
        return Err(AssetError::InvalidImage);
    }
    let thumbnail = decoded.thumbnail(THUMBNAIL_EDGE, THUMBNAIL_EDGE);
    let mut thumbnail_png = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut thumbnail_png, ImageFormat::Png)
        .map_err(|_| AssetError::InvalidImage)?;
    let thumbnail_png = thumbnail_png.into_inner();
    if thumbnail_png.len() > MAX_THUMBNAIL_BYTES {
        return Err(AssetError::TooLarge);
    }
    Ok(PreparedImage {
        original_name: original_name.to_owned(),
        mime_type,
        extension,
        checksum: format!("{:x}", Sha256::digest(bytes)),
        width,
        height,
        original_bytes: bytes.to_vec(),
        thumbnail_png,
    })
}

/// Veröffentlicht Original und Thumbnail atomar innerhalb des Projektordners.
///
/// # Errors
/// Liefert einen Fehler bei unsicheren Verzeichnissen, vorhandenen Zielen oder I/O-Problemen.
pub fn store_image(
    project_directory: &Path,
    asset_id: EntityId,
    prepared: &PreparedImage,
) -> Result<StoredAssetFiles, AssetError> {
    require_real_directory(project_directory)?;
    let assets = ensure_real_child_directory(project_directory, "assets")?;
    let thumbnails = ensure_real_child_directory(&assets, "thumbnails")?;
    let short_hash = &prepared.checksum[..16];
    let original_name = format!("{asset_id}-{short_hash}.{}", prepared.extension);
    let thumbnail_name = format!("{asset_id}-{short_hash}.png");
    let original = assets.join(&original_name);
    let thumbnail = thumbnails.join(&thumbnail_name);
    write_new_atomically(&original, &prepared.original_bytes)?;
    if let Err(error) = write_new_atomically(&thumbnail, &prepared.thumbnail_png) {
        let _ = fs::remove_file(&original);
        return Err(error);
    }
    Ok(StoredAssetFiles {
        relative_path: format!("assets/{original_name}"),
        thumbnail_path: format!("assets/thumbnails/{thumbnail_name}"),
    })
}

/// Liest ein begrenztes Thumbnail nach erneuter Pfad- und Symlinkprüfung.
///
/// # Errors
/// Liefert einen Fehler bei unsicheren oder übergroßen Dateien.
pub fn read_thumbnail(project_directory: &Path, relative: &str) -> Result<Vec<u8>, AssetError> {
    if !relative.starts_with("assets/thumbnails/") {
        return Err(AssetError::UnsafePath);
    }
    let path = resolve_existing_file(project_directory, Path::new(relative))?;
    if fs::metadata(&path)?.len() > MAX_THUMBNAIL_BYTES as u64 {
        return Err(AssetError::TooLarge);
    }
    fs::read(path).map_err(AssetError::from)
}

/// Prüft Vorhandensein, Größe und SHA-256 eines gespeicherten Originals.
///
/// # Errors
/// Liefert einen Fehler bei unsicheren, fehlenden, übergroßen oder veränderten Dateien.
pub fn verify_original(
    project_directory: &Path,
    relative: &str,
    expected_checksum: &str,
) -> Result<(), AssetError> {
    if !relative.starts_with("assets/") || relative.starts_with("assets/thumbnails/") {
        return Err(AssetError::UnsafePath);
    }
    let path = resolve_existing_file(project_directory, Path::new(relative))?;
    if fs::metadata(&path)?.len() > MAX_ASSET_BYTES as u64 {
        return Err(AssetError::TooLarge);
    }
    let bytes = fs::read(path)?;
    if format!("{:x}", Sha256::digest(bytes)) == expected_checksum {
        Ok(())
    } else {
        Err(AssetError::InvalidImage)
    }
}

/// Entfernt ausschließlich die beiden zuvor gespeicherten Assetdateien.
///
/// # Errors
/// Liefert einen Fehler bei unsicheren Pfaden oder anderen I/O-Problemen.
pub fn remove_asset_files(
    project_directory: &Path,
    relative_path: &str,
    thumbnail_path: &str,
) -> Result<(), AssetError> {
    for relative in [relative_path, thumbnail_path] {
        if !relative.starts_with("assets/") || !is_safe_relative_path(Path::new(relative)) {
            return Err(AssetError::UnsafePath);
        }
        let candidate = project_directory.join(relative);
        match fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(AssetError::UnsafePath);
            }
            Ok(_) => fs::remove_file(candidate)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(AssetError::Io(error)),
        }
    }
    Ok(())
}

fn validate_file_name(value: &str) -> Result<(), AssetError> {
    let path = Path::new(value);
    if value.trim() != value
        || value.is_empty()
        || value.chars().count() > 240
        || value.chars().any(char::is_control)
        || path.file_name().and_then(|name| name.to_str()) != Some(value)
    {
        return Err(AssetError::InvalidName);
    }
    Ok(())
}

fn format_contract(format: ImageFormat) -> Result<(&'static str, &'static str), AssetError> {
    match format {
        ImageFormat::Png => Ok(("image/png", "png")),
        ImageFormat::Jpeg => Ok(("image/jpeg", "jpg")),
        ImageFormat::WebP => Ok(("image/webp", "webp")),
        _ => Err(AssetError::UnsupportedFormat),
    }
}

fn require_matching_extension(value: &str, format: ImageFormat) -> Result<(), AssetError> {
    let extension = Path::new(value)
        .extension()
        .and_then(|item| item.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or(AssetError::InvalidName)?;
    let valid = match format {
        ImageFormat::Png => extension == "png",
        ImageFormat::Jpeg => matches!(extension.as_str(), "jpg" | "jpeg"),
        ImageFormat::WebP => extension == "webp",
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AssetError::UnsupportedFormat)
    }
}

fn require_real_directory(path: &Path) -> Result<(), AssetError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AssetError::UnsafePath);
    }
    Ok(())
}

fn ensure_real_child_directory(parent: &Path, name: &str) -> Result<PathBuf, AssetError> {
    let path = parent.join(name);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(AssetError::UnsafePath)
        }
        Ok(_) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir(&path)?;
            Ok(path)
        }
        Err(error) => Err(AssetError::Io(error)),
    }
}

fn write_new_atomically(destination: &Path, bytes: &[u8]) -> Result<(), AssetError> {
    if fs::symlink_metadata(destination).is_ok() {
        return Err(AssetError::AlreadyExists);
    }
    let file_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(AssetError::UnsafePath)?;
    let temporary = destination.with_file_name(format!(".{file_name}.tmp"));
    let result = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, destination)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn resolve_existing_file(project: &Path, relative: &Path) -> Result<PathBuf, AssetError> {
    require_real_directory(project)?;
    if !is_safe_relative_path(relative) {
        return Err(AssetError::UnsafePath);
    }
    let mut current = project.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(AssetError::UnsafePath);
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err(AssetError::UnsafePath);
        }
    }
    if !current.is_file() {
        return Err(AssetError::UnsafePath);
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, path::Path};

    use image::{DynamicImage, ImageFormat, RgbaImage};
    use tempfile::tempdir;

    use super::{
        AssetError, is_safe_relative_path, prepare_image, read_thumbnail, remove_asset_files,
        store_image, verify_original,
    };
    use kraqel_domain::EntityId;

    fn png() -> Vec<u8> {
        let image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 6, image::Rgba([12, 34, 56, 255])));
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(!is_safe_relative_path(Path::new(
            "../private/manuscript.md"
        )));
        assert!(!is_safe_relative_path(Path::new("assets/../private.png")));
    }

    #[test]
    fn accepts_project_relative_asset() {
        assert!(is_safe_relative_path(Path::new("assets/portrait.webp")));
    }

    #[test]
    fn validates_content_and_produces_stable_hash_and_thumbnail() {
        let bytes = png();
        let prepared = prepare_image("portrait.png", &bytes).unwrap();
        assert_eq!((prepared.width, prepared.height), (8, 6));
        assert_eq!(prepared.mime_type, "image/png");
        assert_eq!(prepared.checksum.len(), 64);
        assert_eq!(prepared, prepare_image("portrait.png", &bytes).unwrap());
        assert!(matches!(
            prepare_image("portrait.jpg", &bytes),
            Err(AssetError::UnsupportedFormat)
        ));
    }

    #[test]
    fn stores_reads_and_removes_only_generated_paths() {
        let root = tempdir().unwrap();
        let stored = store_image(
            root.path(),
            EntityId::new(),
            &prepare_image("portrait.png", &png()).unwrap(),
        )
        .unwrap();
        assert_ne!(
            read_thumbnail(root.path(), &stored.thumbnail_path).unwrap(),
            Vec::<u8>::new()
        );
        verify_original(
            root.path(),
            &stored.relative_path,
            &prepare_image("portrait.png", &png()).unwrap().checksum,
        )
        .unwrap();
        remove_asset_files(root.path(), &stored.relative_path, &stored.thumbnail_path).unwrap();
        assert!(!root.path().join(stored.relative_path).exists());
    }
}
