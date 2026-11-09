//! Regelbasierte Kontinuitätsbefunde mit Quellenankern.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    Information,
    Warning,
    Conflict,
}
