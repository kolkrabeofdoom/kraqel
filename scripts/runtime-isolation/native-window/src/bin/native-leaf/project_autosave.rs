//! Debounce only the active edit window. The timer never holds manuscript text.
use super::*;
use std::cell::Cell;

pub(super) const TOGGLE: usize = 219;
pub(super) const TIMER: usize = 7003;

#[derive(Clone, Copy, Default)]
struct Schedule {
    enabled: bool,
    deadline: Option<Instant>,
}

impl Schedule {
    fn changed(&mut self, now: Instant) {
        if self.enabled {
            self.deadline = Some(now + WRITE_PAUSE);
        }
    }

    fn due(&mut self, now: Instant) -> bool {
        if self.deadline.is_some_and(|d| now >= d) {
            self.deadline = None;
            true
        } else {
            false
        }
    }
}

thread_local! { static STATE: Cell<Schedule> = Cell::new(Schedule::default()); }

pub(super) fn enabled() -> bool {
    STATE.get().enabled
}

pub(super) unsafe fn stop(hwnd: HWND) {
    unsafe {
        KillTimer(hwnd, TIMER);
        STATE.set(Schedule::default());
        project_menu::refresh(hwnd);
    }
}

pub(super) unsafe fn resume(hwnd: HWND) {
    unsafe {
        if let Some(deadline) = STATE.get().deadline {
            let delay = deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                + 1;
            if SetTimer(hwnd, TIMER, delay as u32, None) == 0 {
                stop(hwnd);
                status(
                    hwnd,
                    "Automatisches Speichern deaktiviert. Bitte manuell speichern.",
                );
            }
        }
    }
}

pub(super) unsafe fn changed(hwnd: HWND) {
    if enabled() {
        let mut schedule = STATE.get();
        schedule.changed(Instant::now());
        STATE.set(schedule);
        unsafe {
            resume(hwnd);
        }
    }
}

pub(super) unsafe fn toggle(hwnd: HWND) {
    unsafe {
        if enabled() {
            stop(hwnd);
        } else {
            STATE.set(Schedule {
                enabled: true,
                deadline: None,
            });
            changed(hwnd);
            project_menu::refresh(hwnd);
        }
    }
}

pub(super) unsafe fn due(hwnd: HWND, busy: bool) -> bool {
    unsafe {
        KillTimer(hwnd, TIMER);
        if busy {
            return false;
        } // Preserve the deadline until the acknowledgement.
        let mut schedule = STATE.get();
        let due = schedule.due(Instant::now());
        STATE.set(schedule);
        if !due {
            resume(hwnd);
        } // Stale queued WM_TIMER after a newer keystroke.
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_quiet_period_expires_once() {
        let now = Instant::now();
        let mut s = Schedule {
            enabled: true,
            deadline: None,
        };
        s.changed(now);
        s.changed(now + WRITE_PAUSE / 2);
        assert!(!s.due(now + WRITE_PAUSE));
        assert!(s.due(now + WRITE_PAUSE * 2));
        assert!(!s.due(now + WRITE_PAUSE * 3));
    }
}
