use std::{error::Error, fmt};

use zeroize::Zeroizing;

use crate::CloudProviderId;

const KEYRING_SERVICE: &str = "de.kraqel.desktop.ai";
const MAX_API_KEY_CHARACTERS: usize = 4_096;

/// Datenarme Fehler des Betriebssystem-Schlüsselspeichers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialError {
    /// Der Schlüssel fehlt.
    Missing,
    /// Der Schlüssel verletzt lokale Grenzen.
    Invalid,
    /// Der Betriebssystemspeicher ist nicht verfügbar.
    Unavailable,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => {
                formatter.write_str("Für den Provider ist kein Schlüssel gespeichert.")
            }
            Self::Invalid => formatter.write_str("Der API-Schlüssel ist ungültig."),
            Self::Unavailable => {
                formatter.write_str("Der Betriebssystem-Schlüsselspeicher ist nicht verfügbar.")
            }
        }
    }
}

impl Error for CredentialError {}

/// Abstraktion für einen Schlüsselspeicher, damit Tests nie reale Zugangsdaten benötigen.
pub trait CredentialStore: Send + Sync {
    /// Speichert oder ersetzt einen Schlüssel.
    ///
    /// # Errors
    ///
    /// Ungültige Werte oder ein nicht verfügbarer Betriebssystemspeicher werden gemeldet.
    fn set(&self, provider: CloudProviderId, api_key: &str) -> Result<(), CredentialError>;

    /// Liest einen Schlüssel ausschließlich innerhalb der privilegierten Rust-Grenze.
    ///
    /// # Errors
    ///
    /// Ein fehlender Eintrag oder ein nicht verfügbarer Betriebssystemspeicher wird gemeldet.
    fn get(&self, provider: CloudProviderId) -> Result<Zeroizing<String>, CredentialError>;

    /// Prüft nur die Existenz, ohne den Wert über IPC offenzulegen.
    ///
    /// # Errors
    ///
    /// Ein ungültiger Eintrag oder ein nicht verfügbarer Betriebssystemspeicher wird gemeldet.
    fn contains(&self, provider: CloudProviderId) -> Result<bool, CredentialError>;

    /// Löscht einen Schlüssel; ein bereits fehlender Eintrag gilt als gelöscht.
    ///
    /// # Errors
    ///
    /// Ein nicht verfügbarer Betriebssystemspeicher wird gemeldet.
    fn delete(&self, provider: CloudProviderId) -> Result<(), CredentialError>;
}

/// Plattformübergreifender Betriebssystem-Schlüsselspeicher.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsCredentialStore;

impl OsCredentialStore {
    fn entry(provider: CloudProviderId) -> Result<keyring::Entry, CredentialError> {
        keyring::Entry::new(KEYRING_SERVICE, provider.id())
            .map_err(|_| CredentialError::Unavailable)
    }
}

impl CredentialStore for OsCredentialStore {
    fn set(&self, provider: CloudProviderId, api_key: &str) -> Result<(), CredentialError> {
        validate_api_key(api_key)?;
        Self::entry(provider)?
            .set_password(api_key)
            .map_err(|_| CredentialError::Unavailable)
    }

    fn get(&self, provider: CloudProviderId) -> Result<Zeroizing<String>, CredentialError> {
        let value = Self::entry(provider)?
            .get_password()
            .map_err(|error| map_keyring_error(&error))?;
        validate_api_key(&value)?;
        Ok(Zeroizing::new(value))
    }

    fn contains(&self, provider: CloudProviderId) -> Result<bool, CredentialError> {
        match Self::entry(provider)?.get_password() {
            Ok(value) => {
                let value = Zeroizing::new(value);
                validate_api_key(&value)?;
                Ok(true)
            }
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(_) => Err(CredentialError::Unavailable),
        }
    }

    fn delete(&self, provider: CloudProviderId) -> Result<(), CredentialError> {
        match Self::entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CredentialError::Unavailable),
        }
    }
}

fn validate_api_key(api_key: &str) -> Result<(), CredentialError> {
    if api_key.trim() != api_key
        || api_key.chars().count() < 8
        || api_key.chars().count() > MAX_API_KEY_CHARACTERS
        || api_key.chars().any(char::is_whitespace)
        || api_key.chars().any(char::is_control)
    {
        return Err(CredentialError::Invalid);
    }
    Ok(())
}

fn map_keyring_error(error: &keyring::Error) -> CredentialError {
    if matches!(error, keyring::Error::NoEntry) {
        CredentialError::Missing
    } else {
        CredentialError::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_keys_that_could_leak_through_whitespace_or_unbounded_input() {
        assert_eq!(validate_api_key("short"), Err(CredentialError::Invalid));
        assert_eq!(
            validate_api_key("sk-valid-but trailing"),
            Err(CredentialError::Invalid)
        );
        assert!(validate_api_key("sk-test-12345678").is_ok());
    }
}
