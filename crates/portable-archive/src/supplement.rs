use crate::{MAX_ZIP, Result, SecretString, encrypt_zip};
use std::collections::BTreeSet;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use zeroize::{Zeroize, Zeroizing};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

/// Plaintext stays in memory, including when ZIP finalisation fails.
#[derive(Default)]
struct PlainZip(Cursor<Vec<u8>>);

impl Drop for PlainZip {
    fn drop(&mut self) {
        self.0.get_mut().zeroize();
    }
}

impl Write for PlainZip {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.0.position().saturating_add(data.len() as u64) > MAX_ZIP as u64 {
            return Err(io::Error::other("Größenlimit für Zusatzarchiv erreicht."));
        }
        self.0.write(data)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for PlainZip {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

/// Baut das getrennte Archiv ausschließlich aus den ausdrücklich zugeführten Dateien.
pub struct SupplementBuilder {
    zip: Option<ZipWriter<PlainZip>>,
    names: BTreeSet<String>,
}

impl Default for SupplementBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SupplementBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            zip: Some(ZipWriter::new(PlainZip::default())),
            names: BTreeSet::new(),
        }
    }

    /// Fügt einen geöffneten, geprüften Quelldateihandle unter dem relativen Namen hinzu.
    ///
    /// # Errors
    /// Jeder Lese-, Pfad- oder Größenfehler sperrt den ganzen Kandidaten.
    pub fn push(&mut self, relative: &str, mut source: impl Read) -> Result<()> {
        let mut zip = self.zip.take().ok_or("Zusatzarchiv gesperrt.")?;
        if !portable_name(relative)
            || self.names.len() >= 4096
            || !self.names.insert(relative.to_lowercase())
        {
            return Err("Ungültiger, doppelter oder zusätzlicher Archivpfad.");
        }
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file(format!("files/{relative}"), options)
            .map_err(|_| "Archivdatei kann nicht angelegt werden.")?;
        let mut block = Zeroizing::new([0u8; 16 * 1024]);
        loop {
            let read = match source.read(block.as_mut_slice()) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                result => result.map_err(|_| "Zusatzdatei kann nicht gelesen werden.")?,
            };
            if read == 0 {
                break;
            }
            zip.write_all(&block[..read])
                .map_err(|_| "Größenlimit für Zusatzarchiv erreicht.")?;
        }
        self.zip = Some(zip);
        Ok(())
    }

    /// Liefert ausschließlich das authentisierte, verschlüsselte ZIP zur Veröffentlichung.
    ///
    /// # Errors
    /// Fehlende Auswahl, ungültige Eingabe oder ein Fehler beim Abschluss sperrt die Ausgabe.
    pub fn seal(self, passphrase: SecretString) -> Result<Vec<u8>> {
        if self.names.is_empty() {
            return Err("Keine Zusatzdateien ausgewählt.");
        }
        let plain = self
            .zip
            .ok_or("Zusatzarchiv gesperrt.")?
            .finish()
            .map_err(|_| "Zusatzarchiv konnte nicht abgeschlossen werden.")?;
        encrypt_zip(plain.0.get_ref(), passphrase)
    }
}

fn portable_name(relative: &str) -> bool {
    relative.len() <= 4096
        && relative.split('/').all(|part| {
            let base = part
                .split('.')
                .next()
                .unwrap_or("")
                .trim_end_matches(' ')
                .to_uppercase();
            let device_number = base
                .strip_prefix("COM")
                .or_else(|| base.strip_prefix("LPT"));
            !part.is_empty()
                && !part.ends_with(['.', ' '])
                && !part.contains(['\\', ':', '<', '>', '"', '|', '?', '*'])
                && !part.chars().any(char::is_control)
                && !matches!(
                    base.as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                )
                && !device_number.is_some_and(|suffix| {
                    matches!(
                        suffix,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decrypt_zip;

    #[test]
    fn portable_restore_opens_the_selected_supplement() {
        let mut supplement = SupplementBuilder::new();
        supplement
            .push("backups/Entwurf.txt", &b"lokal"[..])
            .unwrap();
        let password = SecretString::from(String::from("synthetischer Roundtrip"));
        let encrypted = supplement.seal(password.clone()).unwrap();
        let plain = decrypt_zip(&encrypted, password).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(plain.as_slice())).unwrap();
        assert_eq!(archive.len(), 1);
        let mut text = String::new();
        archive
            .by_name("files/backups/Entwurf.txt")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        assert_eq!(text, "lokal");

        let mut invalid = SupplementBuilder::new();
        invalid.push("exports/okay.txt", io::empty()).unwrap();
        assert!(invalid.push("../fremd.txt", io::empty()).is_err());
        assert!(
            invalid
                .seal(SecretString::from(String::from("synthetisch")))
                .is_err()
        );
    }
}
