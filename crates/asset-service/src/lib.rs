//! Sicherer Import, Prüfsummen, relative Pfade und Vorschaubilder.

use std::path::{Component, Path};

#[must_use]
pub fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_safe_relative_path;

    #[test]
    fn rejects_parent_traversal() {
        assert!(!is_safe_relative_path(Path::new(
            "../private/manuscript.md"
        )));
    }

    #[test]
    fn accepts_project_relative_asset() {
        assert!(is_safe_relative_path(Path::new("assets/portrait.webp")));
    }
}
