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
        self.choices.resize(rows, 0);
        self.position = self.position.min(rows - 1);
        let mut dialog = Dialog {
            local,
            remote,
            revision,
            refreshed,
            choices: &mut self.choices,
            pos: self.position,
        };
        let button = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
        let label = WS_CHILD | WS_VISIBLE;
        let read = button | WS_BORDER | WS_VSCROLL | ES_MULTILINE as u32 | ES_READONLY as u32;
        let controls = [
            (INFO as u16, 0x82, 8, 6, 388, 31, label, ""),
            (0xffff, 0x82, 8, 39, 185, 12, label, "Lokaler &Entwurf"),
            (LOCAL as u16, 0x81, 8, 52, 188, 52, read, ""),
            (
                0xffff,
                0x82,
                203,
                39,
                188,
                12,
                label,
                "Gespeicherter &Absatz",
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
            (PREV, 0x80, 8, 129, 85, 17, button, "&Vorheriger Absatz"),
            (NEXT, 0x80, 99, 129, 85, 17, button, "&Nächster Absatz"),
            (
                0xffff,
                0x82,
                8,
                151,
                380,
                12,
                label,
                "Vollständige &Auswahl zum Speichern",
            ),
            (MERGED as u16, 0x81, 8, 164, 383, 58, read, ""),
            (
                BACK,
                0x80,
                124,
                228,
                126,
                18,
                button | BS_DEFPUSHBUTTON as u32,
                "&Zurück zum Entwurf",
            ),
            (
                COMMIT,
                0x80,
                257,
                228,
                134,
                18,
                button,
                "Auswahl &speichern",
            ),
        ];
        let template =
            project_dialog::template("KRAQEL – Fassungen vergleichen", 399, 253, &controls);
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
    revision: u64,
    refreshed: bool,
    choices: &'a mut [u8],
    pos: usize,
}

impl Dialog<'_> {
    fn render(&self, hwnd: HWND) -> bool {
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
        let status = format!(
            "{hint}Revision {} · Absatzposition {} von {} · Auswahl: {source}. Entwurf: {}, gespeichert: {}.",
            self.revision,
            self.pos + 1,
            self.choices.len(),
            side(&self.local),
            side(&self.remote)
        );
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
        unsafe {
            for (id, text) in [
                (INFO, status.as_str()),
                (LOCAL, self.local.get(self.pos).copied().unwrap_or("")),
                (REMOTE, self.remote.get(self.pos).copied().unwrap_or("")),
                (MERGED, preview.as_str()),
            ] {
                let display = Zeroizing::new(wide(text.replace('\n', "\r\n")));
                if SetDlgItemTextW(hwnd, id, display.as_ptr()) == 0
                    || usize::try_from(GetWindowTextLengthW(GetDlgItem(hwnd, id))).ok()
                        != Some(display.len() - 1)
                {
                    return false;
                }
            }
            EnableWindow(GetDlgItem(hwnd, PREV.into()), i32::from(self.pos > 0));
            EnableWindow(
                GetDlgItem(hwnd, NEXT.into()),
                i32::from(self.pos + 1 < self.choices.len()),
            );
            EnableWindow(
                GetDlgItem(hwnd, COMMIT.into()),
                i32::from(preview.len() <= PROJECT_EDIT_BYTES),
            );
        }
        true
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
            if !(&*(l as *const Dialog<'_>)).render(hwnd) {
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
            USE_LOCAL => dialog.choices[dialog.pos] = 0,
            USE_REMOTE => dialog.choices[dialog.pos] = 1,
            COMMIT => {
                EndDialog(hwnd, 1);
                return 1;
            }
            _ => return 0,
        }
        if !dialog.render(hwnd) {
            EndDialog(hwnd, 0);
        }
        1
    }
}
