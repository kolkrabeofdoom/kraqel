//! Reproduzierbare, begrenzte Bildableitungen für das gebündelte Beispielprojekt.

use std::{env, fs, io::BufWriter, path::Path};

use image::{GenericImageView as _, ImageFormat, ImageReader, codecs::jpeg::JpegEncoder};

const MAX_EDGE: u32 = 1_280;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    let [source, destination] = arguments.as_slice() else {
        return Err("Verwendung: optimize-example-images <Quelle> <Ziel>".into());
    };
    let source = Path::new(source);
    let destination = Path::new(destination);
    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    fs::create_dir_all(destination)?;
    process_directory(source, source, destination)?;
    Ok(())
}

fn process_directory(
    root: &Path,
    current: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let metadata = entry.file_type()?;
        if metadata.is_symlink() {
            return Err("Symbolische Links sind in Beispielmedien nicht erlaubt.".into());
        }
        if metadata.is_dir() {
            process_directory(root, &entry.path(), destination)?;
            continue;
        }
        if !metadata.is_file() {
            return Err("Nicht regulärer Beispieleintrag.".into());
        }
        let relative = entry.path().strip_prefix(root)?.with_extension("jpg");
        let target = destination.join(relative);
        fs::create_dir_all(target.parent().ok_or("Ziel ohne Elternpfad")?)?;
        optimize(&entry.path(), &target)?;
    }
    Ok(())
}

fn optimize(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let reader = ImageReader::open(source)?.with_guessed_format()?;
    let format = reader.format().ok_or("Unbekanntes Bildformat")?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err("Nur PNG und JPEG sind für das Beispiel vorgesehen.".into());
    }
    let decoded = reader.decode()?;
    let (width, height) = decoded.dimensions();
    let image = if width > MAX_EDGE || height > MAX_EDGE {
        decoded.thumbnail(MAX_EDGE, MAX_EDGE)
    } else {
        decoded
    };
    let file = fs::File::create(destination)?;
    let mut writer = BufWriter::new(file);
    JpegEncoder::new_with_quality(&mut writer, 84).encode_image(&image.to_rgb8())?;
    Ok(())
}
