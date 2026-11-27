//! The title contains only fixed labels and the last confirmed revision.
use super::*;
use std::cell::Cell;

pub(super) const REFRESH: u32 = WM_APP + 41;
thread_local! { static QUEUED: Cell<bool> = const { Cell::new(false) }; }

pub(super) unsafe fn changed(hwnd: HWND) {
    unsafe {
        if !QUEUED.replace(true) {
            // Also checked at the top of the loop if a modal dialog consumes the wakeup.
            PostMessageW(hwnd, REFRESH, 0, 0);
        }
    }
}

pub(super) fn take_changed() -> bool {
    QUEUED.replace(false)
}

pub(super) enum Phase<'a> {
    Idle(&'a str),
    Pending(Option<&'a str>),
    Failed,
    Conflict,
    Unknown,
}

impl Phase<'_> {
    fn label(&self, matches: Option<bool>, automatic: bool) -> &'static str {
        match self {
            Self::Failed => "Speichern fehlgeschlagen",
            Self::Conflict => "Revisionskonflikt",
            Self::Unknown => "Speicherstand unklar",
            Self::Pending(Some(_)) if matches != Some(true) => {
                "Speichert · Weitere Änderungen offen"
            }
            Self::Pending(_) => "Speichert",
            Self::Idle(_) if matches.is_none() => {
                "Ungültiger Entwurf · Text oder Speichergrenze prüfen"
            }
            Self::Idle(_) if matches == Some(true) => "Gespeichert",
            Self::Idle(_) if automatic => "Ungespeicherte Änderungen · Schreibpause",
            Self::Idle(_) => "Ungespeicherte Änderungen",
        }
    }
}

pub(super) unsafe fn render(
    hwnd: HWND,
    revision: u64,
    phase: Phase<'_>,
    previous: &mut Option<(u64, &'static str)>,
) {
    unsafe {
        let matches = match phase {
            Phase::Idle(expected) | Phase::Pending(Some(expected)) => {
                project_metrics::matches_draft(GetDlgItem(hwnd, EDIT), expected)
            }
            _ => None,
        };
        let label = phase.label(matches, project_autosave::enabled());
        if *previous != Some((revision, label)) {
            SetWindowTextW(
                hwnd,
                wide(format!(
                    "Szene · Revision {revision} · {label} – Bearbeiten"
                ))
                .as_ptr(),
            );
            *previous = Some((revision, label));
        }
    }
}
