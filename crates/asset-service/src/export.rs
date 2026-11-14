//! Metadata-free deterministic scene-image derivatives, media export profile v1.
use super::{AssetError, MAX_ASSET_BYTES, MAX_DIMENSION, MAX_PIXELS, require_real_directory};
use image::{ImageDecoder, ImageEncoder, ImageFormat, ImageReader};
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Component, Path},
};

#[derive(Debug)]
pub struct ExportImage {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub source_width: u32,
    pub source_height: u32,
}

/// Freezes a bounded regular original without following symlinks in any child component.
/// # Errors
/// Rejects unsafe paths, changed lengths and oversized or non-regular sources.
pub fn read_export_original(root: &Path, relative: &str) -> Result<Vec<u8>, AssetError> {
    require_real_directory(root)?;
    let parts: Vec<_> = Path::new(relative).components().collect();
    if parts.len() != 2
        || parts[0] != Component::Normal("assets".as_ref())
        || !parts.iter().all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(AssetError::UnsafePath);
    }
    require_real_directory(&root.join("assets"))?;
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_ASSET_BYTES as u64
    {
        return Err(AssetError::UnsafePath);
    }
    let file = fs::File::open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() != metadata.len() {
        return Err(AssetError::InvalidImage);
    }
    let mut bytes = Vec::new();
    file.take(MAX_ASSET_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != metadata.len() {
        return Err(AssetError::InvalidImage);
    }
    Ok(bytes)
}

struct BoundedPng(Vec<u8>);
impl Write for BoundedPng {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > 8 * 1024 * 1024 {
            return Err(std::io::Error::other("image limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Decodes one static bounded source, applies orientation and writes metadata-free RGBA PNG.
/// # Errors
/// Rejects animation, mismatched MIME, malformed sources and resource-limit violations.
pub fn prepare_export_image(bytes: &[u8], mime: &str) -> Result<ExportImage, AssetError> {
    if bytes.is_empty() || bytes.len() > MAX_ASSET_BYTES {
        return Err(AssetError::TooLarge);
    }
    let format = image::guess_format(bytes).map_err(|_| AssetError::InvalidImage)?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(160 * 1024 * 1024);
    let actual = match format {
        ImageFormat::Png => {
            let decoder =
                image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits.clone())
                    .map_err(|_| AssetError::InvalidImage)?;
            if decoder.is_apng().map_err(|_| AssetError::InvalidImage)? {
                return Err(AssetError::UnsupportedFormat);
            }
            "image/png"
        }
        ImageFormat::WebP => {
            let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(bytes))
                .map_err(|_| AssetError::InvalidImage)?;
            if decoder.has_animation() {
                return Err(AssetError::UnsupportedFormat);
            }
            "image/webp"
        }
        ImageFormat::Jpeg => "image/jpeg",
        _ => return Err(AssetError::UnsupportedFormat),
    };
    if actual != mime {
        return Err(AssetError::InvalidImage);
    }
    if format == ImageFormat::Jpeg {
        if !bytes.ends_with(&[0xff, 0xd9]) {
            return Err(AssetError::UnsupportedFormat);
        }
        let mut offset = 2;
        while offset + 4 <= bytes.len() {
            if bytes[offset] != 0xff {
                return Err(AssetError::InvalidImage);
            }
            let marker = bytes[offset + 1];
            if marker == 0xda {
                break;
            }
            let length = usize::from(u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]));
            if length < 2 || offset + 2 + length > bytes.len() {
                return Err(AssetError::InvalidImage);
            }
            if marker == 0xe2 && bytes[offset + 4..offset + 2 + length].starts_with(b"MPF\0") {
                return Err(AssetError::UnsupportedFormat);
            }
            offset += 2 + length;
        }
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| AssetError::InvalidImage)?;
    let (source_width, source_height) = decoder.dimensions();
    if source_width == 0
        || source_height == 0
        || u64::from(source_width) * u64::from(source_height) > MAX_PIXELS
    {
        return Err(AssetError::TooLarge);
    }
    let orientation = decoder
        .orientation()
        .map_err(|_| AssetError::InvalidImage)?;
    let mut image =
        image::DynamicImage::from_decoder(decoder).map_err(|_| AssetError::InvalidImage)?;
    image.apply_orientation(orientation);
    let image = if image.width() > 2400 || image.height() > 2400 {
        image.resize(2400, 2400, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };
    let image = image.to_rgba8();
    let mut output = BoundedPng(Vec::new());
    image::codecs::png::PngEncoder::new_with_quality(
        &mut output,
        image::codecs::png::CompressionType::Default,
        image::codecs::png::FilterType::Adaptive,
    )
    .write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|_| AssetError::TooLarge)?;
    Ok(ExportImage {
        png: output.0,
        width: image.width(),
        height: image.height(),
        source_width,
        source_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png() -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            4,
            2,
            image::Rgba([4, 50, 120, 100]),
        ))
        .write_to(&mut output, ImageFormat::Png)
        .unwrap();
        output.into_inner()
    }
    #[test]
    fn deterministic_rgba_and_format_guards() {
        let source = png();
        let first = prepare_export_image(&source, "image/png").unwrap();
        assert_eq!(
            first.png,
            prepare_export_image(&source, "image/png").unwrap().png
        );
        assert_eq!((first.width, first.height), (4, 2));
        assert_eq!(
            image::load_from_memory(&first.png)
                .unwrap()
                .to_rgba8()
                .get_pixel(0, 0)
                .0,
            [4, 50, 120, 100]
        );
        assert!(prepare_export_image(&source, "image/jpeg").is_err());
        assert!(prepare_export_image(b"private file", "image/png").is_err());
    }
    #[test]
    fn jpeg_orientation_is_applied_and_mpf_rejected() {
        let mut source = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut source)
            .encode(
                &[255, 0, 0, 0, 0, 255],
                2,
                1,
                image::ExtendedColorType::Rgb8,
            )
            .unwrap();
        // Little-endian TIFF, one Orientation tag = rotate 90 clockwise.
        let exif = b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
        let mut oriented = source[..2].to_vec();
        oriented.extend_from_slice(&[0xff, 0xe1]);
        oriented.extend_from_slice(&u16::try_from(exif.len() + 2).unwrap().to_be_bytes());
        oriented.extend_from_slice(exif);
        oriented.extend_from_slice(&source[2..]);
        let exported = prepare_export_image(&oriented, "image/jpeg").unwrap();
        assert_eq!((exported.width, exported.height), (1, 2));
        assert!(!exported.png.windows(4).any(|w| w == b"Exif"));
        let mut multi = source[..2].to_vec();
        multi.extend_from_slice(b"\xff\xe2\0\x06MPF\0");
        multi.extend_from_slice(&source[2..]);
        assert!(prepare_export_image(&multi, "image/jpeg").is_err());
    }
    #[test]
    fn webp_static_and_no_upscale_and_long_edge_limit() {
        let pixels = image::RgbaImage::from_pixel(2401, 2, image::Rgba([0, 60, 200, 255]));
        let mut source = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut source)
            .write_image(pixels.as_raw(), 2401, 2, image::ExtendedColorType::Rgba8)
            .unwrap();
        let result = prepare_export_image(&source, "image/webp").unwrap();
        assert_eq!(result.width, 2400);
        assert!(result.height <= 2);
    }
    #[test]
    fn real_animations_are_rejected_and_metadata_is_removed() {
        assert!(
            prepare_export_image(
                include_bytes!("../tests/fixtures/animated.png"),
                "image/png"
            )
            .is_err()
        );
        assert!(
            prepare_export_image(
                include_bytes!("../tests/fixtures/animated.webp"),
                "image/webp"
            )
            .is_err()
        );
        let output = prepare_export_image(
            include_bytes!("../tests/fixtures/metadata.png"),
            "image/png",
        )
        .unwrap();
        assert!(!output.png.windows(7).any(|w| w == b"PRIVATE"));
    }
    #[test]
    fn reads_only_regular_bounded_asset_paths() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("assets")).unwrap();
        fs::write(root.path().join("assets/ok.png"), png()).unwrap();
        assert!(read_export_original(root.path(), "assets/ok.png").is_ok());
        for path in [
            "../secret",
            "assets/../secret",
            "assets",
            "assets/missing",
            "assets/nested/a.png",
        ] {
            assert!(read_export_original(root.path(), path).is_err());
        }
    }
}
