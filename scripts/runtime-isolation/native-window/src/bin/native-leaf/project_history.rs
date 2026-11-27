//! Plaintext history for one live product edit control; never shared with its channel.
use super::*;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use windows_sys::Win32::UI::Controls::{EM_CANUNDO, EM_EMPTYUNDOBUFFER, EM_SCROLLCARET, EM_UNDO};

type Selection = (usize, usize);
const MAX_STATES: usize = 65;
const UNIT_BUDGET: usize = 2 * 1024 * 1024;

struct State {
    // NUL-terminated so restoring cannot reallocate a plaintext copy.
    text: Zeroizing<Vec<u16>>,
    after_edit: Selection,
    before_next_edit: Selection,
}

#[derive(Default)]
struct History {
    states: VecDeque<State>,
    index: usize,
    incomplete_utf16: Option<Selection>,
}

impl History {
    fn record(&mut self, text: Option<Zeroizing<Vec<u16>>>, before: Selection, after: Selection) {
        let Some(mut text) = text else {
            // No traversal across a state that could not be captured within budget.
            *self = Self::default();
            return;
        };
        if char::decode_utf16(text.iter().copied()).any(|c| c.is_err()) {
            self.states.truncate(self.index + 1);
            // WM_CHAR can deliver an emoji in two messages. The intermediate half is
            // never a restorable version; keep the selection preceding the first half.
            self.incomplete_utf16.get_or_insert(before);
            return;
        }
        let before = self.incomplete_utf16.take().unwrap_or(before);
        text.push(0);
        if let Some(current) = self.states.get_mut(self.index) {
            if current.text == text {
                return;
            }
            current.before_next_edit = before;
        }
        self.states.truncate(self.index + 1);
        self.states.push_back(State {
            text,
            after_edit: after,
            before_next_edit: after,
        });
        let mut cost: usize = self.states.iter().map(|s| s.text.capacity()).sum();
        while self.states.len() > MAX_STATES || cost > UNIT_BUDGET {
            if let Some(old) = self.states.pop_front() {
                cost -= old.text.capacity();
            }
        }
        self.index = self.states.len().saturating_sub(1);
    }

    fn target(&self, redo: bool) -> Option<usize> {
        if self.incomplete_utf16.is_some() {
            return None;
        }
        if redo {
            (self.index + 1 < self.states.len()).then_some(self.index + 1)
        } else {
            self.index.checked_sub(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(text: &[u16]) -> Option<Zeroizing<Vec<u16>>> {
        let mut copy = Vec::with_capacity(text.len() + 1);
        copy.extend_from_slice(text);
        Some(Zeroizing::new(copy))
    }

    #[test]
    fn incomplete_and_oversized_versions_never_become_restore_targets() {
        let mut history = History::default();
        history.record(units(&[65]), (0, 0), (0, 0));
        history.record(units(&[0xd83e]), (0, 1), (1, 1));
        assert!(history.target(false).is_none());
        history.record(units(&[0xd83e, 0xdeb6]), (1, 1), (2, 2));
        assert_eq!(history.target(false), Some(0));
        assert_eq!(history.states[0].before_next_edit, (0, 1));
        assert_eq!(history.states[1].after_edit, (2, 2));

        history.record(None, (0, 2), (0, 0));
        history.record(units(&[66]), (0, 0), (1, 1));
        assert!(history.target(false).is_none());
        let mut large = Zeroizing::new(vec![65; UNIT_BUDGET]);
        large.pop(); // retain exactly the whole budget in this allocation
        history.record(Some(large), (0, 1), (1, 1));
        assert_eq!(history.states.len(), 1);
        assert!(history.target(false).is_none());
    }
}

thread_local! {
    static FIELD: Cell<HWND> = const { Cell::new(null_mut()) };
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static CHANGED: Cell<bool> = const { Cell::new(false) };
    static HISTORY: RefCell<History> = RefCell::new(History::default());
}

pub(super) struct Session;
impl Drop for Session {
    fn drop(&mut self) {
        FIELD.set(null_mut());
        HISTORY.take();
    }
}

pub(super) unsafe fn start(field: HWND) -> Session {
    unsafe {
        let text = project_metrics::units(field, UNIT_BUDGET - 1);
        let selection = project_search::selection(field);
        HISTORY.with_borrow_mut(|h| h.record(text, selection, selection));
        FIELD.set(field);
        Session
    }
}

pub(super) fn available(redo: bool) -> bool {
    HISTORY.with_borrow(|h| h.target(redo).is_some())
}

pub(super) fn changed() {
    if ACTIVE.get() {
        CHANGED.set(true);
    }
}

// EN_CHANGE is synchronous. Read a fresh text snapshot only when native editing
// actually reports a change (multiline WM_SETTEXT is the documented exception).
pub(super) unsafe fn observe<T>(field: HWND, forced: bool, operation: impl FnOnce() -> T) -> T {
    unsafe {
        if FIELD.get() != field || ACTIVE.replace(true) {
            return operation();
        }
        CHANGED.set(false);
        let before = project_search::selection(field);
        let result = operation();
        if forced || CHANGED.get() {
            let text = project_metrics::units(field, UNIT_BUDGET - 1);
            let after = project_search::selection(field);
            HISTORY.with_borrow_mut(|h| h.record(text, before, after));
            project_autosave::changed(GetParent(field));
            project_save_status::changed(GetParent(field));
            SendMessageW(field, EM_EMPTYUNDOBUFFER, 0, 0);
            project_menu::refresh(GetParent(field));
        }
        ACTIVE.set(false);
        result
    }
}

pub(super) unsafe fn step(field: HWND, redo: bool) -> bool {
    unsafe {
        if FIELD.get() != field
            || GetWindowLongW(field, GWL_STYLE) & ES_READONLY != 0
            || ACTIVE.replace(true)
        {
            return false;
        }
        // No RefCell borrow across Win32 callbacks.
        let mut history = HISTORY.take();
        let restored = if let Some(index) = history.target(redo) {
            let target = &history.states[index];
            let (start, end) = if redo {
                target.after_edit
            } else {
                target.before_next_edit
            };
            let ok = SetWindowTextW(field, target.text.as_ptr()) != 0
                && project_metrics::units(field, UNIT_BUDGET - 1)
                    .is_some_and(|s| s.as_slice() == &target.text[..target.text.len() - 1]);
            if ok {
                history.index = index;
                SetFocus(field);
                SendMessageW(field, EM_SETSEL, start, end as isize);
                SendMessageW(field, EM_SCROLLCARET, 0, 0);
                project_replace::MATCH.set(None);
                let hwnd = GetParent(field);
                project_autosave::changed(hwnd);
                project_save_status::changed(hwnd);
                SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext").as_ptr());
                if !focus_mode(hwnd) {
                    project_metrics::update(hwnd, METRICS);
                }
            } else {
                history = History::default();
                let sel = project_search::selection(field);
                history.record(project_metrics::units(field, UNIT_BUDGET - 1), sel, sel);
            }
            SendMessageW(field, EM_EMPTYUNDOBUFFER, 0, 0);
            ok
        } else {
            false
        };
        HISTORY.replace(history);
        ACTIVE.set(false);
        project_menu::refresh(GetParent(field));
        restored
    }
}

pub(super) unsafe fn dispatch(field: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        let native = || windows_sys::Win32::UI::Shell::DefSubclassProc(field, msg, w, l);
        if FIELD.get() != field {
            return native();
        }
        match msg {
            WM_UNDO | EM_UNDO => LRESULT::from(step(field, false)),
            EM_CANUNDO => LRESULT::from(
                available(false) && GetWindowLongW(field, GWL_STYLE) & ES_READONLY == 0,
            ),
            WM_CHAR if w == 26 || w == 25 => {
                if GetKeyState(i32::from(VK_CONTROL)) < 0
                    && [VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
                        .iter()
                        .all(|key| GetKeyState(i32::from(*key)) >= 0)
                    && l & (1 << 30) == 0
                {
                    step(field, w == 25);
                }
                0
            }
            EM_GETSEL | WM_GETTEXT | WM_GETTEXTLENGTH | WM_NCDESTROY => native(),
            _ => observe(field, msg == WM_SETTEXT, native),
        }
    }
}
