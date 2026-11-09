//! Deterministische Exportsnapshots und kontrollierte Git-Operationen.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingTreeState {
    Clean,
    Changed,
}
