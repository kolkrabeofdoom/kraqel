//! Portable age passphrase envelope for a bounded, opaque P3 ZIP.
use kraqel_archiv::{MAX_ENVELOPE, MAX_ZIP, SecretString, decrypt_zip, encrypt_zip};
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};
use tempfile::NamedTempFile;
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, &'static str>;

fn read_limited(reader: impl Read, limit: usize) -> Result<Zeroizing<Vec<u8>>> {
    let mut bytes = Zeroizing::new(Vec::new());
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Lesen fehlgeschlagen.")?;
    if bytes.len() > limit {
        return Err("Archiv überschreitet das P3-Größenlimit.");
    }
    Ok(bytes)
}

fn password(confirm: bool) -> Result<SecretString> {
    let first = Zeroizing::new(
        rpassword::prompt_password("Archiv-Passphrase: ")
            .map_err(|_| "Für die Passphrase wird ein interaktives Terminal benötigt.")?,
    );
    if first.is_empty() {
        return Err("Passphrase fehlt.");
    }
    if confirm {
        let second = Zeroizing::new(
            rpassword::prompt_password("Passphrase wiederholen: ")
                .map_err(|_| "Bestätigung fehlgeschlagen.")?,
        );
        if first != second {
            return Err("Passphrasen stimmen nicht überein.");
        }
    }
    Ok(SecretString::from(String::from(first.as_str())))
}

fn publish(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = NamedTempFile::new_in(parent).map_err(|_| "Zielverzeichnis nicht verfügbar.")?;
    file.write_all(bytes)
        .and_then(|()| file.as_file().sync_all())
        .map_err(|_| "Schreiben fehlgeschlagen.")?;
    file.persist_noclobber(path)
        .map_err(|_| "Zieldatei existiert bereits oder kann nicht angelegt werden.")?;
    Ok(())
}

fn run() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [cmd, dest] if cmd == "pack-stdin" => {
            let zip = read_limited(io::stdin().lock(), MAX_ZIP)?;
            publish(Path::new(dest), &encrypt_zip(&zip, password(true)?)?)
        }
        [cmd, source, dest] if cmd == "restore" || cmd == "pack" => {
            let file = File::open(source).map_err(|_| "Quelldatei nicht verfügbar.")?;
            if cmd == "restore" {
                let data = read_limited(file, MAX_ENVELOPE)?;
                publish(Path::new(dest), &decrypt_zip(&data, password(false)?)?)
            } else {
                let zip = read_limited(file, MAX_ZIP)?;
                publish(Path::new(dest), &encrypt_zip(&zip, password(true)?)?)
            }
        }
        _ => Err(
            "Aufruf: kraqel-archiv pack QUELLE.zip ZIEL.zip.age | restore QUELLE.zip.age NEUES_ZIEL.zip",
        ),
    }
}

fn main() -> std::process::ExitCode {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::ExitCode::FAILURE
    } else {
        std::process::ExitCode::SUCCESS
    }
}
