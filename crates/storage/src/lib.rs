//! Autoritative lokale Speicherung und Migrationen.

pub const CURRENT_SCHEMA_VERSION: u32 = 0;

#[must_use]
pub const fn schema_is_supported(version: u32) -> bool {
    version == CURRENT_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::{CURRENT_SCHEMA_VERSION, schema_is_supported};

    #[test]
    fn future_schema_is_rejected() {
        assert!(!schema_is_supported(CURRENT_SCHEMA_VERSION + 1));
    }
}
