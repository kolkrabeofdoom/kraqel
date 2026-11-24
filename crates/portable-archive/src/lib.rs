//! Portable Zusatzarchive für ausdrücklich ausgewählte Projektdateien.
use age::scrypt;
pub use age::secrecy::SecretString;
pub use supplement::SupplementBuilder;
use zeroize::Zeroizing;
mod supplement;

pub const MAX_ZIP: usize = 32 * 1024 * 1024;
pub const MAX_ENVELOPE: usize = MAX_ZIP + 64 * 1024;
const WORK_FACTOR: u8 = 18;
type Result<T> = std::result::Result<T, &'static str>;

/// Verschlüsselt ein begrenztes ZIP im portablen age-v1-Format.
///
/// # Errors
/// Ausgabe, Größe oder Verschlüsselung ungültig.
pub fn encrypt_zip(data: &[u8], passphrase: SecretString) -> Result<Vec<u8>> {
    if data.len() > MAX_ZIP {
        return Err("Archiv überschreitet das P3-Größenlimit.");
    }
    let mut recipient = scrypt::Recipient::new(passphrase);
    recipient.set_work_factor(WORK_FACTOR);
    age::encrypt(&recipient, data).map_err(|_| "Verschlüsselung fehlgeschlagen.")
}

/// Authentisiert den vollständigen Inhalt vor der Rückgabe.
///
/// # Errors
/// Fehler bei ungültiger Passphrase, beschädigtem oder übergroßem Archiv.
pub fn decrypt_zip(data: &[u8], passphrase: SecretString) -> Result<Zeroizing<Vec<u8>>> {
    if data.len() > MAX_ENVELOPE {
        return Err("Archiv überschreitet das P3-Größenlimit.");
    }
    let mut identity = scrypt::Identity::new(passphrase);
    identity.set_max_work_factor(WORK_FACTOR);
    let plain = Zeroizing::new(
        age::decrypt(&identity, data)
            .map_err(|_| "Falsche Passphrase oder beschädigtes Archiv.")?,
    );
    if plain.len() > MAX_ZIP {
        return Err("Entschlüsseltes Archiv zu groß.");
    }
    Ok(plain)
}
