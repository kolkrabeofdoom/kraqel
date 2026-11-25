//! A position in one plain-text version corresponds to the same position in the other.
//! Missing sides omit the row; a present empty paragraph contributes an empty line.
use super::*;
use windows_sys::Win32::UI::WindowsAndMessaging::DM_SETDEFID;

const LOCAL: i32 = 211;
const REMOTE: i32 = 212;
const INFO: i32 = 213;
const PREV: u16 = 214;
const NEXT: u16 = 215;
const USE_LOCAL: u16 = 216;
const USE_REMOTE: u16 = 217;
const COMMIT: u16 = 218;
const BACK: u16 = 219;
const MERGED: i32 = 220;
const DIFF_PREV: u16 = 221;
const DIFF_NEXT: u16 = 222;
const ALL_LOCAL: u16 = 223;
const ALL_REMOTE: u16 = 224;
const UNDO_CHOICE: u16 = 225;
const REDO_CHOICE: u16 = 226;

// A dialog owns its selection history. A new revision only carries over the
// currently selected sources; no action from the old revision can be replayed.
#[derive(Default)]
struct ChoiceHistory {
    undo: std::collections::VecDeque<Vec<u8>>,
    redo: Vec<Vec<u8>>,
}

impl ChoiceHistory {
    fn choose(&mut self, choices: &mut [u8], at: std::ops::Range<usize>, side: u8) -> bool {
        if choices[at.clone()].iter().all(|selected| *selected == side) {
            return false;
        }
        self.redo.clear();
        if self.undo.len() == 16 {
            self.undo.pop_front();
        }
        self.undo.push_back(choices.to_vec());
        choices[at].fill(side);
        true
    }

    fn step(&mut self, choices: &mut Vec<u8>, redo: bool) -> bool {
        if redo {
            let Some(next) = self.redo.pop() else {
                return false;
            };
            self.undo.push_back(std::mem::replace(choices, next));
        } else {
            let Some(previous) = self.undo.pop_back() else {
                return false;
            };
            self.redo.push(std::mem::replace(choices, previous));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::ChoiceHistory;

    #[test]
    fn bulk_undo_restores_mixed_sources_and_a_noop_keeps_redo() {
        let mut h = ChoiceHistory::default();
        let mut sides = vec![0, 1, 0];
        assert!(h.choose(&mut sides, 0..3, 1));
        assert!(h.step(&mut sides, false));
        assert_eq!(sides, [0, 1, 0]);
        assert!(!h.choose(&mut sides, 1..2, 1));
        assert!(h.step(&mut sides, true));
        assert_eq!(sides, [1, 1, 1]);
        assert!(h.step(&mut sides, false));
        assert!(h.choose(&mut sides, 0..1, 1));
        assert!(!h.step(&mut sides, true));
        assert_eq!(sides, [1, 1, 0]);
    }
}

pub(super) struct Review {
    pub draft: Zeroizing<String>,
    pub choices: Vec<u8>,
    position: usize,
}

impl Review {
    pub fn new(draft: Zeroizing<String>) -> Self {
        Self {
            draft,
            choices: Vec::new(),
            position: 0,
        }
    }

    pub fn choose(&mut self, owner: HWND, remote: &str, revision: u64, refreshed: bool) -> bool {
        let local: Vec<_> = self.draft.split('\n').collect();
        let remote: Vec<_> = remote.split('\n').collect();
        let rows = local.len().max(remote.len());
        let differences: Vec<_> = (0..rows)
            .filter(|&i| local.get(i) != remote.get(i))
            .collect();
        if !refreshed {
            self.position = differences.first().copied().unwrap_or(0);
        }
        self.choices.resize(rows, 0);
        self.position = self.position.min(rows - 1);
        let mut dialog = Dialog {
            local,
            remote,
            differences,
            revision,
            refreshed,
            choices: &mut self.choices,
            pos: self.position,
            history: ChoiceHistory::default(),
        };
        let button = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
        let label = WS_CHILD | WS_VISIBLE;
        let read = button | WS_BORDER | WS_VSCROLL | ES_MULTILINE as u32 | ES_READONLY as u32;
        let controls = [
            (INFO as u16, 0x82, 8, 6, 388, 32, label, ""),
            (0xffff, 0x82, 8, 39, 185, 12, label, "Lokaler &Entwurf (F6)"),
            (LOCAL as u16, 0x81, 8, 52, 188, 52, read, ""),
            (
                0xffff,
                0x82,
                203,
                39,
                188,
                12,
                label,
                "Gespeicherter &Absatz (F6)",
            ),
            (REMOTE as u16, 0x81, 203, 52, 188, 52, read, ""),
            (USE_LOCAL, 0x80, 8, 107, 150, 17, button, "&Entwurf wählen"),
            (
                USE_REMOTE,
                0x80,
                203,
                107,
                188,
                17,
                button,
                "&Gespeicherten Absatz wählen",
            ),
            (ALL_LOCAL, 0x80, 8, 129, 188, 17, button, "Alle aus Entwurf"),
            (
                ALL_REMOTE,
                0x80,
                203,
                129,
                188,
                17,
                button,
                "Alle aus gespeicherter Fassung",
            ),
            (PREV, 0x80, 8, 151, 85, 17, button, "&Vorheriger Absatz"),
            (NEXT, 0x80, 99, 151, 85, 17, button, "&Nächster Absatz"),
            (
                UNDO_CHOICE,
                0x80,
                203,
                151,
                91,
                17,
                button,
                "Rückgängig (Strg+Z)",
            ),
            (
                REDO_CHOICE,
                0x80,
                299,
                151,
                92,
                17,
                button,
                "Wiederholen (Strg+Y)",
            ),
            (
                DIFF_PREV,
                0x80,
                8,
                173,
                188,
                17,
                button,
                "Voriger Unterschied (Umschalt+F7)",
            ),
            (
                DIFF_NEXT,
                0x80,
                203,
                173,
                188,
                17,
                button,
                "Nächster Unterschied (F7)",
            ),
            (
                0xffff,
                0x82,
                8,
                196,
                380,
                12,
                label,
                "Vollständige &Auswahl zum Speichern",
            ),
            (MERGED as u16, 0x81, 8, 209, 383, 58, read, ""),
            (
                BACK,
                0x80,
                124,
                273,
                126,
                18,
                button | BS_DEFPUSHBUTTON as u32,
                "&Zurück zum Entwurf",
            ),
            (
                COMMIT,
                0x80,
                257,
                273,
                134,
                18,
                button,
                "Auswahl &speichern",
            ),
        ];
        let template =
            project_dialog::template("KRAQEL – Fassungen vergleichen", 399, 298, &controls);
        let result = unsafe {
            DialogBoxIndirectParamW(
                GetModuleHandleW(null()),
                template.as_ptr().cast(),
                owner,
                Some(handle),
                (&raw mut dialog) as LPARAM,
            )
        };
        self.position = dialog.pos;
        result == 1
    }
}

struct Dialog<'a> {
    local: Vec<&'a str>,
    remote: Vec<&'a str>,
    differences: Vec<usize>,
    revision: u64,
    refreshed: bool,
    choices: &'a mut Vec<u8>,
    history: ChoiceHistory,
    pos: usize,
}

impl Dialog<'_> {
    fn render(&self, hwnd: HWND, update_preview: bool) -> bool {
        let side = |lines: &[&str]| match lines.get(self.pos) {
            None => "fehlt",
            Some(&"") => "leer",
            Some(_) => "vorhanden",
        };
        let source = if self.choices[self.pos] == 0 {
            "Entwurf"
        } else {
            "gespeichert"
        };
        let hint = if self.refreshed {
            "Neue Vergleichsbasis. Auswahl je Position erneut prüfen. "
        } else {
            ""
        };
        let difference = match self.differences.binary_search(&self.pos) {
            Ok(index) => format!("Unterschied {} von {}", index + 1, self.differences.len()),
            Err(_) => format!(
                "Absätze gleich · Unterschiede insgesamt: {}",
                self.differences.len()
            ),
        };
        let status = format!(
            "{hint}Revision {} · Absatzposition {} von {} · {difference} · Auswahl: {source}. Entwurf: {}, gespeichert: {}.",
            self.revision,
            self.pos + 1,
            self.choices.len(),
            side(&self.local),
            side(&self.remote)
        );
        unsafe {
            let focus = GetFocus();
            let undo = GetDlgItem(hwnd, UNDO_CHOICE.into());
            let redo = GetDlgItem(hwnd, REDO_CHOICE.into());
            EnableWindow(undo, i32::from(!self.history.undo.is_empty()));
            EnableWindow(redo, i32::from(!self.history.redo.is_empty()));
            if focus == undo && self.history.undo.is_empty() && !self.history.redo.is_empty() {
                SetFocus(redo);
            }
            if focus == redo && self.history.redo.is_empty() && !self.history.undo.is_empty() {
                SetFocus(undo);
            }
            for (id, text) in [
                (INFO, status.as_str()),
                (LOCAL, self.local.get(self.pos).copied().unwrap_or("")),
                (REMOTE, self.remote.get(self.pos).copied().unwrap_or("")),
            ] {
                if !set_text(hwnd, id, text) {
                    return false;
                }
            }
            EnableWindow(GetDlgItem(hwnd, PREV.into()), i32::from(self.pos > 0));
            EnableWindow(
                GetDlgItem(hwnd, NEXT.into()),
                i32::from(self.pos + 1 < self.choices.len()),
            );
            for id in [DIFF_PREV, DIFF_NEXT] {
                EnableWindow(
                    GetDlgItem(hwnd, id.into()),
                    i32::from(!self.differences.is_empty()),
                );
            }
            if update_preview {
                let preview = Zeroizing::new(
                    self.choices
                        .iter()
                        .enumerate()
                        .filter_map(|(i, choice)| {
                            (if *choice == 0 {
                                &self.local
                            } else {
                                &self.remote
                            })
                            .get(i)
                            .copied()
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
                if !set_text(hwnd, MERGED, &preview) {
                    return false;
                }
                EnableWindow(
                    GetDlgItem(hwnd, COMMIT.into()),
                    i32::from(preview.len() <= PROJECT_EDIT_BYTES),
                );
            }
        }
        true
    }
}

unsafe fn set_text(hwnd: HWND, id: i32, text: &str) -> bool {
    let length = text.encode_utf16().count() + text.bytes().filter(|b| *b == b'\n').count();
    let mut display = Zeroizing::new(Vec::with_capacity(length + 1));
    for unit in text.encode_utf16() {
        if unit == 10 {
            display.push(13);
        }
        display.push(unit);
    }
    display.push(0);
    unsafe {
        SetDlgItemTextW(hwnd, id, display.as_ptr()) != 0
            && usize::try_from(GetWindowTextLengthW(GetDlgItem(hwnd, id))).ok() == Some(length)
    }
}

unsafe extern "system" fn keys(
    control: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    unsafe {
        let native = || windows_sys::Win32::UI::Shell::DefSubclassProc(control, msg, w, l);
        let ctrl = GetKeyState(i32::from(VK_CONTROL)) < 0;
        let shift = GetKeyState(i32::from(VK_SHIFT)) < 0;
        let history = ctrl && !shift && (w == usize::from(VK_Z) || w == usize::from(VK_Y));
        let navigation = !ctrl && (w == usize::from(VK_F6) || w == usize::from(VK_F7));
        if !(history || navigation)
            || [VK_MENU, VK_LWIN, VK_RWIN]
                .iter()
                .any(|key| GetKeyState(i32::from(*key)) < 0)
        {
            return native();
        }
        if msg == WM_GETDLGCODE && l != 0 {
            return native() | DLGC_WANTMESSAGE as isize;
        }
        if msg != WM_KEYDOWN {
            return native();
        }
        if l & (1 << 30) == 0 {
            let hwnd = GetParent(control);
            if w == usize::from(VK_F6) {
                SetFocus(GetDlgItem(
                    hwnd,
                    if GetFocus() == GetDlgItem(hwnd, LOCAL) {
                        REMOTE
                    } else {
                        LOCAL
                    },
                ));
            } else {
                let id = if history {
                    if w == usize::from(VK_Z) {
                        UNDO_CHOICE
                    } else {
                        REDO_CHOICE
                    }
                } else if shift {
                    DIFF_PREV
                } else {
                    DIFF_NEXT
                };
                let button = GetDlgItem(hwnd, id.into());
                if IsWindowEnabled(button) != 0 {
                    SendMessageW(hwnd, WM_COMMAND, id.into(), button as isize);
                }
            }
        }
        0
    }
}

unsafe extern "system" fn handle(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> isize {
    unsafe {
        if msg == WM_INITDIALOG {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, l);
            for id in [LOCAL, REMOTE, MERGED] {
                let h = GetDlgItem(hwnd, id);
                if windows_sys::Win32::UI::Shell::SetWindowSubclass(h, Some(scene_input), 1, 0) == 0
                {
                    EndDialog(hwnd, 0);
                    return 0;
                }
                SendMessageW(h, EM_SETLIMITTEXT, 4 * PROJECT_EDIT_BYTES, 0);
            }
            for id in [
                LOCAL,
                REMOTE,
                MERGED,
                PREV.into(),
                NEXT.into(),
                USE_LOCAL.into(),
                USE_REMOTE.into(),
                ALL_LOCAL.into(),
                ALL_REMOTE.into(),
                UNDO_CHOICE.into(),
                REDO_CHOICE.into(),
                DIFF_PREV.into(),
                DIFF_NEXT.into(),
                BACK.into(),
                COMMIT.into(),
            ] {
                if windows_sys::Win32::UI::Shell::SetWindowSubclass(
                    GetDlgItem(hwnd, id),
                    Some(keys),
                    2,
                    0,
                ) == 0
                {
                    EndDialog(hwnd, 0);
                    return 0;
                }
            }
            if !(&*(l as *const Dialog<'_>)).render(hwnd, true) {
                EndDialog(hwnd, 0);
                return 0;
            }
            SendMessageW(hwnd, DM_SETDEFID, BACK.into(), 0);
            SetFocus(GetDlgItem(hwnd, BACK.into()));
            return 0;
        }
        if msg == WM_CLOSE {
            EndDialog(hwnd, 0);
            return 1;
        }
        if msg != WM_COMMAND {
            return 0;
        }
        let id = (w & 0xffff) as u16;
        if id == IDCANCEL as u16 || id == IDOK as u16 || id == BACK {
            EndDialog(hwnd, 0);
            return 1;
        }
        if w >> 16 != 0 || l as HWND != GetDlgItem(hwnd, id.into()) || l == 0 {
            return 0;
        }
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Dialog<'_>;
        if ptr.is_null() {
            return 0;
        }
        let dialog = &mut *ptr;
        match id {
            PREV => dialog.pos = dialog.pos.saturating_sub(1),
            NEXT => dialog.pos = (dialog.pos + 1).min(dialog.choices.len() - 1),
            DIFF_PREV | DIFF_NEXT => {
                let ds = &dialog.differences;
                if ds.is_empty() {
                    return 1;
                }
                let index = if id == DIFF_PREV {
                    ds.partition_point(|p| *p < dialog.pos)
                        .checked_sub(1)
                        .unwrap_or(ds.len() - 1)
                } else {
                    ds.partition_point(|p| *p <= dialog.pos) % ds.len()
                };
                dialog.pos = ds[index];
            }
            USE_LOCAL | USE_REMOTE | ALL_LOCAL | ALL_REMOTE => {
                let range = if matches!(id, ALL_LOCAL | ALL_REMOTE) {
                    0..dialog.choices.len()
                } else {
                    dialog.pos..dialog.pos + 1
                };
                if !dialog.history.choose(
                    dialog.choices,
                    range,
                    u8::from(matches!(id, USE_REMOTE | ALL_REMOTE)),
                ) {
                    return 1;
                }
            }
            UNDO_CHOICE | REDO_CHOICE => {
                if !dialog.history.step(dialog.choices, id == REDO_CHOICE) {
                    return 1;
                }
            }
            COMMIT => {
                EndDialog(hwnd, 1);
                return 1;
            }
            _ => return 0,
        }
        if !dialog.render(
            hwnd,
            matches!(
                id,
                USE_LOCAL | USE_REMOTE | ALL_LOCAL | ALL_REMOTE | UNDO_CHOICE | REDO_CHOICE
            ),
        ) {
            EndDialog(hwnd, 0);
        }
        1
    }
}
