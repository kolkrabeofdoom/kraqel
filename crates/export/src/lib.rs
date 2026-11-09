//! Portable, deterministische Exportformate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Html,
    Docx,
    Pdf,
}
