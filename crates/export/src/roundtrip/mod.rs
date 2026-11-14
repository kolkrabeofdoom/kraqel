//! Native, bounded manuscript exchange. ADR 0027 / `DOCX_ROUNDTRIP.md`.
mod inventory;
mod numbering;
mod package;
mod reader;
#[cfg(test)]
mod tests;
mod writer;

use crate::Manuscript;
use std::fmt;

/// Stable error without document contents, paths or XML diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocxRoundtripError;
impl fmt::Display for DocxRoundtripError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Das DOCX-Manuskript ist ungültig, nicht unterstützt oder überschreitet die Austauschgrenzen.")
    }
}
impl std::error::Error for DocxRoundtripError {}

/// Reads only the versioned KRAQEL manuscript profile, entirely offline.
///
/// # Errors
/// Rejects invalid packages, unsupported profiles/content, ambiguous structure or exceeded limits.
pub fn read_docx_roundtrip(bytes: &[u8]) -> Result<Manuscript, DocxRoundtripError> {
    reader::read(bytes).map_err(|()| DocxRoundtripError)
}

/// Reads a regular, bounded local DOCX file and validates its complete manuscript profile.
///
/// # Errors
/// Rejects unsafe/non-DOCX file inputs and any unsupported or malformed package.
pub fn read_docx_roundtrip_file(path: &std::path::Path) -> Result<Manuscript, DocxRoundtripError> {
    let bytes = crate::docx_scene::read_bytes(path).map_err(|()| DocxRoundtripError)?;
    read_docx_roundtrip(&bytes)
}

/// Produces a deterministic manuscript DOCX and verifies a native semantic self-import.
///
/// # Errors
/// Rejects invalid/oversized models or packages whose semantic self-import differs.
pub fn write_docx_roundtrip(manuscript: &Manuscript) -> Result<Vec<u8>, DocxRoundtripError> {
    let manuscript = manuscript.canonicalized()?;
    let bytes = writer::write(&manuscript).map_err(|()| DocxRoundtripError)?;
    if read_docx_roundtrip(&bytes)? != manuscript {
        return Err(DocxRoundtripError);
    }
    Ok(bytes)
}
