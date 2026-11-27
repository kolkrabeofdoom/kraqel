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
pub use export::{
    ExportImage, prepare_export_image, prepare_pdf_export_image, read_export_original,
};

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

fn decode_image(
    bytes: &[u8],
    format: ImageFormat,
    max_edge: u32,
) -> Result<image::DynamicImage, AssetError> {
    let (w, h) = ImageReader::with_format(Cursor::new(bytes), format)
        .into_dimensions()
        .map_err(|_| AssetError::InvalidImage)?;
    if w == 0 || h == 0 || w > max_edge || h > max_edge || u64::from(w) * u64::from(h) > MAX_PIXELS
    {
        return Err(AssetError::TooLarge);
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(max_edge);
    limits.max_image_height = Some(max_edge);
    reader.limits(limits);
    let decoded = reader.decode().map_err(|_| AssetError::InvalidImage)?;
    if decoded.dimensions() != (w, h) {
        return Err(AssetError::InvalidImage);
    }
    Ok(decoded)
}

/// Prüft die gebundenen Bildbytes vor einer Übernahme. Akzeptiert die Originalformate
/// des Imports und gespeicherte PNG-Vorschaubilder bis zur regulären Vorschaugröße.
///
/// # Errors
/// Abweichende Formate, Katalogmaße, überschrittene Grenzen oder unlesbare Bilder sperren die Übernahme.
pub fn validate_handoff_image(
    original: &[u8],
    thumbnail: &[u8],
    mime: &str,
    width: u32,
    height: u32,
) -> Result<(), AssetError> {
    if original.is_empty()
        || original.len() > MAX_ASSET_BYTES
        || thumbnail.is_empty()
        || thumbnail.len() > MAX_THUMBNAIL_BYTES
    {
        return Err(AssetError::TooLarge);
    }
    let format = image::guess_format(original).map_err(|_| AssetError::InvalidImage)?;
    if format_contract(format)?.0 != mime
        || image::guess_format(thumbnail).ok() != Some(ImageFormat::Png)
    {
        return Err(AssetError::InvalidImage);
    }
    require_complete_container(original, format)?;
    require_complete_container(thumbnail, ImageFormat::Png)?;
    if decode_image(original, format, MAX_DIMENSION)?.dimensions() != (width, height) {
        return Err(AssetError::InvalidImage);
    }
    decode_image(thumbnail, ImageFormat::Png, THUMBNAIL_EDGE)?;
    Ok(())
}

// Some image decoders can produce all pixels from a truncated container.
fn require_complete_container(bytes: &[u8], format: ImageFormat) -> Result<(), AssetError> {
    let complete = match format {
        ImageFormat::Png => {
            let mut chunks = bytes.get(8..).ok_or(AssetError::InvalidImage)?;
            loop {
                let Some(header) = chunks.get(..8) else {
                    break false;
                };
                let len = u32::from_be_bytes(
                    header[..4]
                        .try_into()
                        .map_err(|_| AssetError::InvalidImage)?,
                ) as usize;
                let Some(chunk_end) = len.checked_add(12) else {
                    break false;
                };
                let Some(tail) = chunks.get(chunk_end..) else {
                    break false;
                };
                if &header[4..8] == b"IEND" {
                    break len == 0 && chunks[8..12] == [0xae, 0x42, 0x60, 0x82];
                }
                chunks = tail;
            }
        }
        ImageFormat::WebP => bytes
            .get(4..8)
            .and_then(|b| b.try_into().ok())
            .is_some_and(|size| u64::from(u32::from_le_bytes(size)) + 8 <= bytes.len() as u64),
        ImageFormat::Jpeg => jpeg_scan_has_end(bytes),
        _ => false,
    };
    if complete {
        Ok(())
    } else {
        Err(AssetError::InvalidImage)
    }
}

// Skip length-delimited segments: EXIF may contain a complete nested JPEG.
fn jpeg_scan_has_end(bytes: &[u8]) -> bool {
    let Some(mut rest) = bytes.strip_prefix(&[0xff, 0xd8]) else {
        return false;
    };
    let mut saw_scan = false;
    while let Some(offset) = rest.iter().position(|&byte| byte == 0xff) {
        rest = &rest[offset + 1..];
        while let Some(tail) = rest.strip_prefix(&[0xff]) {
            rest = tail;
        }
        let Some((&marker, tail)) = rest.split_first() else {
            break;
        };
        rest = tail;
        match marker {
            0xd9 => return saw_scan,
            0x00 | 0x01 | 0xd0..=0xd7 => (),
            _ => {
                let [hi, lo, ..] = *rest else { break };
                let length = usize::from(u16::from_be_bytes([hi, lo]));
                if length < 2 || length > rest.len() {
                    break;
                }
                saw_scan |= marker == 0xda;
                rest = &rest[length..];
            }
        }
    }
    false
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
    let decoded = decode_image(bytes, format, MAX_DIMENSION)?;
    let (width, height) = decoded.dimensions();
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
        AssetError, is_safe_relative_path, jpeg_scan_has_end, prepare_image, read_thumbnail,
        remove_asset_files, store_image, validate_handoff_image, verify_original,
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
        validate_handoff_image(&bytes, &prepared.thumbnail_png, "image/png", 8, 6).unwrap();
        let truncated = &prepared.thumbnail_png[..prepared.thumbnail_png.len() - 1];
        assert!(validate_handoff_image(&bytes, truncated, "image/png", 8, 6).is_err());
        assert!(!jpeg_scan_has_end(&[
            0xff, 0xd8, 0xff, 0xe1, 0, 4, 0xff, 0xd9
        ]));
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
