#![windows_subsystem = "windows"]
#[path = "native-leaf/paragraph_comparison.rs"]
mod paragraph_comparison;
use kraqel_native_scene_model::MAX_PARAGRAPH_UTF16;
use kraqel_window_probe::*;
use std::ptr::{null, null_mut};
use std::time::{Duration, Instant};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use windows_sys::Win32::{
    System::SystemServices::SS_LEFT,
    UI::Controls::{
        EM_GETFIRSTVISIBLELINE, EM_GETSEL, EM_LINESCROLL, EM_REPLACESEL, EM_SETLIMITTEXT, EM_SETSEL,
    },
};

#[used]
#[unsafe(link_section = ".kqauto")]
static AUTOSAVE_ABI: [u8; 32] = window_receipt::AUTOSAVE_ABI;

#[used]
#[unsafe(link_section = ".kqcmp")]
static COMPARE_ABI: [u8; 32] = kraqel_window_probe::window_receipt::COMPARE_ABI;

const PICK_FIRST: usize = 220;
const PICK_SECOND: usize = 221;
const PICK_BOLD: usize = 222;
const COMMIT_SELECTION: usize = 223;
const CONTINUE_SELECTION: usize = 224;
const PICK_LOCAL_CHANGES: usize = 225;
const SHOW_EDIT_BASE: usize = 226;
const PICK_DRAFT: usize = 227;
const PICK_SERVICE: usize = 228;
const UNDO_SELECTION: usize = 229;
const MERGE_FIELDS: [usize; 3] = [PICK_FIRST, PICK_SECOND, PICK_BOLD];
const MERGE_SELECTED: u32 = WM_APP + 7;
const COMPARISON: i32 = 111;
const LABEL: i32 = 100;
const EDIT: i32 = 101;
const CLOSE: i32 = 102;
const SECOND: i32 = 103;
const SAVE: i32 = 104;
const DISCARD: i32 = 105;
const WORD_COUNT: i32 = 106;
const SEARCH: i32 = 107;
const SEARCH_LABEL: i32 = 108;
const REPLACEMENT: i32 = 109;
const REPLACEMENT_LABEL: i32 = 110;
const REPLACE_ONE: usize = 208;
const REPLACE_ALL: usize = 209;
const REPLACE_DRAFT: u32 = WM_APP + 6;
const FIND: usize = 205;
const FIND_NEXT: usize = 206;
const FIND_PREVIOUS: usize = 207;

const AUTOSAVE_TIMER: usize = 0x4b51;
const WRITE_PAUSE: Duration = Duration::from_millis(700);

struct PendingSave {
    text: String,
    bold: bool,
    due: Instant,
}

const SAVE_AND_STAY: i32 = 112;
const CONTINUE_PLAIN_DRAFT: u32 = WM_APP + 8;
const OPEN_PLAIN: usize = 230;
thread_local! {
    static PLAIN_REVIEW: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static PARAGRAPH_COMPARE: std::cell::RefCell<Option<paragraph_comparison::Comparison>> = const {std::cell::RefCell::new(None)};
    static LONG_EDIT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static AUTOSAVE_TURN: std::cell::Cell<Option<(u8, u8)>> = const { std::cell::Cell::new(None) };
    static PENDING_SAVE: std::cell::RefCell<Option<PendingSave>> = const { std::cell::RefCell::new(None) };
    static SELECTION_REVIEW:std::cell::RefCell<Option<SelectionReview>>=const {std::cell::RefCell::new(None)};
    static REVISION_CONFLICT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static AWAITING_REBIND: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CLOSE_REQUESTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static ACTIVE_PARAGRAPH: std::cell::Cell<i32> = const { std::cell::Cell::new(EDIT) };
    static SECOND_WEIGHT: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    static LOADED_REVISION: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static EDIT_BASE_REVISION: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static INITIAL_BOLD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static EXPECTED: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}
const SAVE_DRAFT: u32 = WM_APP + 1;
const DISCARD_DRAFT: u32 = WM_APP + 2;
const UNDO: usize = 201;
const REDO: usize = 202;
const BOLD_SECOND: usize = 204;
const TOGGLE_BOLD: u32 = WM_APP + 5;
const FOCUS_MODE: usize = 210;
const SELECT_PARAGRAPH: usize = 203;
const HISTORY_UNDO: u32 = WM_APP + 3;
const HISTORY_REDO: u32 = WM_APP + 4;

fn in_focus_mode(hwnd: HWND) -> bool {
    unsafe {
        let state = GetMenuState(GetMenu(hwnd), FOCUS_MODE as u32, MF_BYCOMMAND);
        state != u32::MAX && state & MF_CHECKED != 0
    }
}

fn set_focus_mode(hwnd: HWND, enabled: bool) {
    unsafe {
        if in_focus_mode(hwnd) == enabled {
            return;
        }
        let checked = if enabled { MF_CHECKED } else { MF_UNCHECKED };
        if CheckMenuItem(GetMenu(hwnd), FOCUS_MODE as u32, MF_BYCOMMAND | checked) == u32::MAX {
            return;
        }
        if enabled {
            SetFocus(GetDlgItem(hwnd, ACTIVE_PARAGRAPH.get()));
        }
        for id in [
            SAVE,
            DISCARD,
            WORD_COUNT,
            SEARCH_LABEL,
            SEARCH,
            REPLACEMENT_LABEL,
            REPLACEMENT,
        ] {
            ShowWindow(
                GetDlgItem(hwnd, id),
                if enabled { SW_HIDE } else { SW_SHOWNA },
            );
        }
        layout_surface(hwnd);
    }
}

fn open_search(hwnd: HWND) {
    unsafe {
        set_focus_mode(hwnd, false);
        let field = GetDlgItem(hwnd, SEARCH);
        SetFocus(field);
        SendMessageW(field, EM_SETSEL, 0, -1);
    }
}

fn find_in_scene(hwnd: HWND, backwards: bool) {
    unsafe {
        set_focus_mode(hwnd, false);
        let Some(query) = paragraph_text(hwnd, SEARCH).filter(|q| !q.is_empty()) else {
            open_search(hwnd);
            return;
        };
        let mut hits = Vec::new();
        let mut anchor = None;
        let preview = GetDlgItem(hwnd, SAVE).is_null();
        let prefix = if preview {
            "Schreibgeschützte Textvorschau · "
        } else {
            ""
        };
        for id in [EDIT, SECOND] {
            if GetDlgItem(hwnd, id).is_null() {
                continue;
            }
            let content = if preview {
                // Win32 selection offsets count both units of each CRLF.
                Some(EXPECTED.with_borrow(|text| text.replace('\n', "\r\n")))
            } else {
                paragraph_text(hwnd, id)
            };
            let Some(text) = content else {
                continue;
            };
            let field = GetDlgItem(hwnd, id);
            if field == GetFocus() {
                let selection = SendMessageW(field, EM_GETSEL, 0, 0) as u32;
                let pos = if backwards {
                    selection & 0xffff
                } else {
                    selection >> 16
                };
                anchor = Some((id, pos as usize));
            }
            let mut byte = 0;
            let mut end = 0;
            for (offset, matched) in text.match_indices(&query) {
                let start = end + text[byte..offset].encode_utf16().count();
                end = start + matched.encode_utf16().count();
                byte = offset + matched.len();
                hits.push((id, start, end));
            }
        }
        if hits.is_empty() {
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide(format!("{prefix}Kein Treffer in Szene A")).as_ptr(),
            );
            open_search(hwnd);
            return;
        }
        let index = if backwards {
            anchor
                .and_then(|pos| hits.iter().rposition(|&(id, _, end)| (id, end) <= pos))
                .unwrap_or(hits.len() - 1)
        } else {
            anchor
                .and_then(|pos| hits.iter().position(|&(id, start, _)| (id, start) >= pos))
                .unwrap_or(0)
        };
        let (id, start, end) = hits[index];
        let field = GetDlgItem(hwnd, id);
        SetFocus(field);
        SendMessageW(field, EM_SETSEL, start, end as LPARAM);
        SendMessageW(
            field,
            windows_sys::Win32::UI::Controls::EM_SCROLLCARET,
            0,
            0,
        );
        SetDlgItemTextW(
            hwnd,
            LABEL,
            wide(format!("{prefix}Treffer {} von {}", index + 1, hits.len())).as_ptr(),
        );
    }
}

// Prepare the complete edit before touching either control. The message loop records
// the two paragraphs together in one undo state, like a normal scene edit.
fn replace_in_scene(hwnd: HWND, all: bool) -> bool {
    unsafe {
        set_focus_mode(hwnd, false);
        let Some(query) = paragraph_text(hwnd, SEARCH).filter(|q| !q.is_empty()) else {
            open_search(hwnd);
            return true;
        };
        let (Some(replacement), Some(first), Some(second)) = (
            paragraph_text(hwnd, REPLACEMENT),
            paragraph_text(hwnd, EDIT),
            paragraph_text(hwnd, SECOND),
        ) else {
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide("Bitte die Eingabefelder vor dem Ersetzen korrigieren.").as_ptr(),
            );
            return true;
        };
        let mut paragraphs = [first, second];
        let mut hits = Vec::new();
        for (index, text) in paragraphs.iter().enumerate() {
            for (byte, _) in text.match_indices(&query) {
                hits.push((index, byte));
            }
        }
        if hits.is_empty() {
            SetDlgItemTextW(hwnd, LABEL, wide("Kein Treffer in Szene A").as_ptr());
            return true;
        }
        if !all {
            let active = ACTIVE_PARAGRAPH.get();
            let index = usize::from(active == SECOND);
            let range = SendDlgItemMessageW(hwnd, active, EM_GETSEL, 0, 0) as u32;
            let start = (range & 0xffff) as usize;
            let end = (range >> 16) as usize;
            let next = hits
                .iter()
                .position(|&(i, byte)| {
                    if i != index {
                        return i > index;
                    }
                    let offset = paragraphs[i][..byte].encode_utf16().count();
                    offset >= end
                        || (offset == start && offset + query.encode_utf16().count() == end)
                })
                .unwrap_or(0);
            let hit = hits[next];
            hits.clear();
            hits.push(hit);
        }
        for &(index, byte) in hits.iter().rev() {
            paragraphs[index].replace_range(byte..byte + query.len(), &replacement);
        }
        let text = paragraphs.join("\n");
        if !paragraphs
            .iter()
            .all(|p| kraqel_native_scene_model::valid_paragraph(p))
        {
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide("Ersetzen würde einen ungültigen Absatz erzeugen.").as_ptr(),
            );
            return true;
        }
        if !restore_draft(hwnd, &text) {
            return false;
        }
        if !all {
            let (index, byte) = hits[0];
            let field = GetDlgItem(hwnd, [EDIT, SECOND][index]);
            let end = paragraphs[index][..byte].encode_utf16().count()
                + replacement.encode_utf16().count();
            SetFocus(field);
            SendMessageW(field, EM_SETSEL, end, end as LPARAM);
        }
        SetDlgItemTextW(
            hwnd,
            LABEL,
            wide(format!("{} Treffer ersetzt", hits.len())).as_ptr(),
        );
        true
    }
}

fn install_editor_menu(hwnd: HWND) -> bool {
    unsafe {
        let bar = CreateMenu();
        if bar.is_null() {
            return false;
        }
        for (title, items) in [
            (
                "&Datei",
                &[
                    (SAVE as usize, "&Speichern\tStrg+S"),
                    (CLOSE as usize, "S&chließen\tAlt+F4"),
                ][..],
            ),
            (
                "&Bearbeiten",
                &[
                    (OPEN_PLAIN, "Als längeren Klartext bearbeiten"),
                    (UNDO, "&Rückgängig\tStrg+Z"),
                    (REDO, "&Wiederholen\tStrg+Y"),
                    (BOLD_SECOND, "Zweiter Absatz &fett\tStrg+B"),
                    (FIND, "&Suchen…\tStrg+F"),
                    (FIND_NEXT, "&Nächster Treffer\tF3"),
                    (FIND_PREVIOUS, "&Vorheriger Treffer\tUmschalt+F3"),
                    (REPLACE_ONE, "Treffer &ersetzen"),
                    (REPLACE_ALL, "Alle Treffer erset&zen"),
                    (SELECT_PARAGRAPH, "&Absatz auswählen\tStrg+A"),
                ][..],
            ),
            ("&Ansicht", &[(FOCUS_MODE, "&Fokusmodus\tF11")][..]),
        ] {
            let popup = CreatePopupMenu();
            if popup.is_null()
                || AppendMenuW(bar, MF_POPUP, popup as usize, wide(title).as_ptr()) == 0
            {
                if !popup.is_null() {
                    DestroyMenu(popup);
                }
                DestroyMenu(bar);
                return false;
            }
            for &(id, label) in items {
                if AppendMenuW(popup, MF_STRING, id, wide(label).as_ptr()) == 0 {
                    DestroyMenu(bar);
                    return false;
                }
            }
        }
        if SetMenu(hwnd, bar) == 0 {
            DestroyMenu(bar);
            return false;
        }
        true
    }
}

fn enable_menu_command(hwnd: HWND, id: usize, enabled: bool) {
    unsafe {
        EnableMenuItem(
            GetMenu(hwnd),
            id as u32,
            MF_BYCOMMAND | if enabled { MF_ENABLED } else { MF_GRAYED },
        );
    }
}

fn select_paragraph(hwnd: HWND) {
    unsafe {
        let field = GetFocus();
        if field == GetDlgItem(hwnd, EDIT) || field == GetDlgItem(hwnd, SECOND) {
            SendMessageW(field, EM_SETSEL, 0, -1);
        }
    }
}
fn clipboard_command(msg: u32, key: WPARAM, ctrl: bool, shift: bool) -> bool {
    match msg {
        WM_COPY | WM_CUT | WM_PASTE | WM_CONTEXTMENU => true,
        // Translated Ctrl+C, Ctrl+V and Ctrl+X can reach EDIT as characters.
        WM_CHAR => matches!(key, 3 | 22 | 24),
        WM_KEYDOWN => {
            (ctrl && matches!(key, 0x43 | 0x56 | 0x58))
                || ((ctrl || shift) && key == usize::from(VK_INSERT))
                || (shift && key == usize::from(VK_DELETE))
        }
        _ => false,
    }
}

fn align_edit_selection(hwnd: HWND, inserting: bool) -> usize {
    unsafe {
        // Scene fields are bounded, so both UTF-16 offsets fit in EM_GETSEL's packed result.
        let range = SendMessageW(hwnd, EM_GETSEL, 0, 0) as u32;
        let start = (range & 0xffff) as usize;
        let end = (range >> 16) as usize;
        let mut buf = vec![
            0u16;
            if LONG_EDIT.get() {
                2 * kraqel_native_scene_model::MAX_PREVIEW_UTF16 + 1
            } else {
                MAX_PARAGRAPH_UTF16 + 1
            }
        ];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) as usize;
        let splits_pair = |at: usize| {
            at > 0
                && at < len
                && (0xd800..=0xdbff).contains(&buf[at - 1])
                && (0xdc00..=0xdfff).contains(&buf[at])
        };
        // With an empty selection, typing at a split pair inserts after the character.
        // A deletion there, or a selection touching half a pair, covers the entire pair.
        let aligned_end = end + usize::from(splits_pair(end));
        let aligned_start = if start == end && inserting {
            aligned_end
        } else {
            start - usize::from(splits_pair(start))
        };
        if (aligned_start, aligned_end) != (start, end) {
            SendMessageW(hwnd, EM_SETSEL, aligned_start, aligned_end as LPARAM);
        }
        aligned_end.saturating_sub(aligned_start)
    }
}

unsafe extern "system" fn local_edit(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    id: usize,
    pending_high: usize,
) -> LRESULT {
    unsafe {
        // Subclass data belongs to this EDIT only. Wait for the complete scalar before
        // changing text or creating an undo entry in either paragraph.
        if msg == WM_CHAR && (0xd800..=0xdbff).contains(&w) {
            windows_sys::Win32::UI::Shell::SetWindowSubclass(hwnd, Some(local_edit), id, w);
            return 0;
        }
        if pending_high != 0
            && matches!(
                msg,
                WM_CHAR | WM_KILLFOCUS | WM_SETTEXT | EM_SETSEL | WM_LBUTTONDOWN | WM_KEYDOWN
            )
        {
            windows_sys::Win32::UI::Shell::SetWindowSubclass(hwnd, Some(local_edit), id, 0);
        }
        if msg == WM_CHAR && (0xdc00..=0xdfff).contains(&w) {
            if pending_high == 0 || GetWindowLongW(hwnd, GWL_STYLE) & ES_READONLY != 0 {
                return 0;
            }
            let selected = align_edit_selection(hwnd, true);
            let length = GetWindowTextLengthW(hwnd) as usize;
            let limit = if LONG_EDIT.get() {
                2 * kraqel_native_scene_model::MAX_PREVIEW_UTF16
            } else {
                MAX_PARAGRAPH_UTF16
            };
            if length.saturating_sub(selected) + 2 <= limit {
                let pair = [pending_high as u16, w as u16, 0];
                SendMessageW(hwnd, EM_REPLACESEL, 1, pair.as_ptr() as LPARAM);
            } else {
                let control = GetDlgCtrlID(hwnd) as usize;
                SendMessageW(
                    GetParent(hwnd),
                    WM_COMMAND,
                    control | ((EN_MAXTEXT as usize) << 16),
                    hwnd as LPARAM,
                );
            }
            return 0;
        }
        if clipboard_command(
            msg,
            w,
            GetKeyState(i32::from(VK_CONTROL)) < 0,
            GetKeyState(i32::from(VK_SHIFT)) < 0,
        ) {
            return 0;
        }
        let inserting = msg == WM_CHAR && (0x20..=0xffff).contains(&w) && w != 0x7f;
        let deleting = (msg == WM_CHAR && w == usize::from(VK_BACK))
            || (msg == WM_KEYDOWN && w == usize::from(VK_DELETE));
        if (inserting || deleting) && GetWindowLongW(hwnd, GWL_STYLE) & ES_READONLY == 0 {
            align_edit_selection(hwnd, inserting);
        }
        windows_sys::Win32::UI::Shell::DefSubclassProc(hwnd, msg, w, l)
    }
}

fn protect_editor_field(field: HWND) -> bool {
    unsafe { windows_sys::Win32::UI::Shell::SetWindowSubclass(field, Some(local_edit), 1, 0) != 0 }
}

fn font_description(font: HFONT) -> Option<LOGFONTW> {
    let mut desc = LOGFONTW::default();
    let size = size_of::<LOGFONTW>() as i32;
    (unsafe { GetObjectW(font, size, (&mut desc as *mut LOGFONTW).cast()) } == size).then_some(desc)
}
fn paragraph_text(hwnd: HWND, id: i32) -> Option<String> {
    // Read one extra UTF-16 unit so an overlong value cannot be saved as a valid prefix.
    let mut buf = [0u16; MAX_PARAGRAPH_UTF16 + 2];
    let n = unsafe { GetDlgItemTextW(hwnd, id, buf.as_mut_ptr(), buf.len() as i32) } as usize;
    if n == buf.len() - 1 {
        return None;
    }
    String::from_utf16(&buf[..n]).ok()
}

fn invalid_paragraph(hwnd: HWND) -> Option<(i32, &'static str)> {
    [
        (
            EDIT,
            "Ersten Absatz einzeilig ausfüllen (max. 48 UTF-16-Einheiten).",
        ),
        (
            SECOND,
            "Zweiten Absatz einzeilig ausfüllen (max. 48 UTF-16-Einheiten).",
        ),
    ]
    .into_iter()
    .find(|(id, _)| {
        !paragraph_text(hwnd, *id).is_some_and(|p| kraqel_native_scene_model::valid_paragraph(&p))
    })
}

fn draft_text(hwnd: HWND) -> Option<String> {
    if LONG_EDIT.get() {
        unsafe {
            let edit = GetDlgItem(hwnd, EDIT);
            let length = GetWindowTextLengthW(edit) as usize;
            if length > 2 * kraqel_native_scene_model::MAX_PREVIEW_UTF16 {
                return None;
            }
            let mut buffer = vec![0; length + 1];
            let got = GetWindowTextW(edit, buffer.as_mut_ptr(), buffer.len() as i32) as usize;
            return String::from_utf16(&buffer[..got])
                .ok()
                .map(|s| s.replace("\r\n", "\n"));
        }
    }
    Some(format!(
        "{}\n{}",
        paragraph_text(hwnd, EDIT)?,
        paragraph_text(hwnd, SECOND)?
    ))
}

fn is_bold() -> bool {
    SECOND_WEIGHT.get() == FW_BOLD as i32
}

fn set_bold(hwnd: HWND, bold: bool) -> bool {
    unsafe {
        if bold != is_bold() {
            SECOND_WEIGHT.set(if bold { FW_BOLD } else { FW_NORMAL } as i32);
            if !apply_fonts(hwnd, GetDpiForWindow(hwnd)) {
                return false;
            }
        }
        CheckMenuItem(
            GetMenu(hwnd),
            BOLD_SECOND as u32,
            MF_BYCOMMAND | if bold { MF_CHECKED } else { MF_UNCHECKED },
        );
        SetDlgItemTextW(hwnd, LABEL, wide(draft_label()).as_ptr());
        update_draft_status(hwnd);
        true
    }
}

fn draft_label() -> &'static str {
    if SECOND_WEIGHT.get() == FW_BOLD as i32 {
        "Szene A · Zwei Absätze, zweiter fett"
    } else {
        "Szene A · Zwei Absätze"
    }
}

/// Bounded to this window's message loop. A new edit removes the redo branch.
#[derive(Default)]
struct HeldEditor {
    hwnd: HWND,
    submitted: Option<(String, bool)>,
    history: Option<DraftHistory>,
    closing: bool,
    resume_after_merge: bool,
    autosave_turn: Option<(u8, u8)>,
}

struct DraftHistory {
    states: std::collections::VecDeque<(String, bool)>,
    current: usize,
}

impl DraftHistory {
    fn new(text: String) -> Self {
        Self {
            states: [(text, is_bold())].into(),
            current: 0,
        }
    }

    fn record(&mut self, text: String) {
        let state = (text, is_bold());
        if self.states[self.current] == state {
            return;
        }
        self.states.truncate(self.current + 1);
        self.states.push_back(state);
        if self.states.len() > 65 {
            self.states.pop_front();
        }
        self.current = self.states.len() - 1;
    }

    fn step(&mut self, redo: bool) -> Option<&(String, bool)> {
        let next = if redo {
            self.current + 1
        } else {
            self.current.checked_sub(1)?
        };
        let text = self.states.get(next)?;
        self.current = next;
        Some(text)
    }
}

fn restore_draft(hwnd: HWND, text: &str) -> bool {
    let Some((first, second)) = text.split_once('\n') else {
        return false;
    };
    let Some(current) = draft_text(hwnd) else {
        return false;
    };
    let Some((old_first, old_second)) = current.split_once('\n') else {
        return false;
    };
    unsafe {
        for (id, old, new) in [(EDIT, old_first, first), (SECOND, old_second, second)] {
            if old == new {
                continue;
            }
            let field = GetDlgItem(hwnd, id);
            if SetWindowTextW(field, wide(new).as_ptr()) == 0 {
                return false;
            }
            let common_suffix = old
                .chars()
                .rev()
                .zip(new.chars().rev())
                .take_while(|(a, b)| a == b)
                .map(|(_, c)| c.len_utf16())
                .sum::<usize>();
            let caret = new.encode_utf16().count() - common_suffix;
            SetFocus(field);
            SendMessageW(
                field,
                windows_sys::Win32::UI::Controls::EM_SETSEL,
                caret,
                caret as isize,
            );
        }
    }
    update_draft_status(hwnd);
    true
}

fn draft_changed(hwnd: HWND) -> bool {
    let current = draft_text(hwnd);
    is_bold() != INITIAL_BOLD.get()
        || EXPECTED.with_borrow(|initial| current.as_ref() != Some(initial))
}

fn destroy_surface(hwnd: HWND) {
    unsafe {
        let normal = SendDlgItemMessageW(hwnd, EDIT, WM_GETFONT, 0, 0) as HFONT;
        let marked = SendDlgItemMessageW(hwnd, SECOND, WM_GETFONT, 0, 0) as HFONT;
        DestroyWindow(hwnd);
        for font in [normal, marked] {
            if !font.is_null() {
                DeleteObject(font);
            }
        }
    }
}

fn scale(value: i32, dpi: u32) -> i32 {
    (value * dpi as i32 + 48) / 96
}

fn apply_fonts(hwnd: HWND, dpi: u32) -> bool {
    unsafe {
        let mut metrics = NONCLIENTMETRICSW::default();
        metrics.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
        if SystemParametersInfoForDpi(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            (&mut metrics as *mut NONCLIENTMETRICSW).cast(),
            0,
            dpi,
        ) == 0
        {
            return false;
        }
        let normal = CreateFontIndirectW(&metrics.lfMessageFont);
        if normal.is_null() {
            return false;
        }
        let second = GetDlgItem(hwnd, SECOND);
        if !second.is_null() {
            metrics.lfMessageFont.lfWeight = SECOND_WEIGHT.get();
            let marked = CreateFontIndirectW(&metrics.lfMessageFont);
            if marked.is_null() {
                DeleteObject(normal);
                return false;
            }
            let old = SendMessageW(second, WM_GETFONT, 0, 0) as HFONT;
            SendMessageW(second, WM_SETFONT, marked as usize, 1);
            if !old.is_null() {
                DeleteObject(old);
            }
        }
        let old = SendDlgItemMessageW(hwnd, EDIT, WM_GETFONT, 0, 0) as HFONT;
        for id in [
            EDIT,
            LABEL,
            SAVE,
            SAVE_AND_STAY,
            DISCARD,
            CLOSE,
            WORD_COUNT,
            COMPARISON,
            SEARCH_LABEL,
            SEARCH,
            REPLACEMENT,
            REPLACEMENT_LABEL,
        ] {
            SendDlgItemMessageW(hwnd, id, WM_SETFONT, normal as usize, 1);
        }
        if !old.is_null() {
            DeleteObject(old);
        }
        true
    }
}

fn cancel_pending_save(hwnd: HWND) {
    if PENDING_SAVE.take().is_some() {
        unsafe {
            KillTimer(hwnd, AUTOSAVE_TIMER);
        }
    }
}

fn schedule_autosave(hwnd: HWND, changed_valid_text: Option<&str>) -> bool {
    let Some(text) = changed_valid_text else {
        cancel_pending_save(hwnd);
        return true;
    };
    let bold = is_bold();
    if PENDING_SAVE.with_borrow(|p| p.as_ref().is_some_and(|p| p.text == text && p.bold == bold)) {
        return true;
    }
    cancel_pending_save(hwnd);
    if unsafe { SetTimer(hwnd, AUTOSAVE_TIMER, WRITE_PAUSE.as_millis() as u32, None) } == 0 {
        return false;
    }
    PENDING_SAVE.replace(Some(PendingSave {
        text: text.to_owned(),
        bold,
        due: Instant::now() + WRITE_PAUSE,
    }));
    true
}

// WM_TIMER may already be queued when the pending edit changes or the timer is killed.
fn autosave_due(hwnd: HWND) -> bool {
    let text = draft_text(hwnd);
    let due = PENDING_SAVE.with_borrow(|p| {
        p.as_ref()
            .filter(|p| Some(&p.text) == text.as_ref() && p.bold == is_bold())
            .map(|p| p.due)
    });
    if let Some(due) = due {
        let remaining = due.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return true;
        }
        // Windows timers can fire early; wait just the remainder of this edit's pause.
        let ms = remaining.as_millis() as u32 + 1;
        if unsafe { SetTimer(hwnd, AUTOSAVE_TIMER, ms, None) } != 0 {
            return false;
        }
        cancel_pending_save(hwnd);
    }
    update_draft_status(hwnd);
    false
}

struct AutosaveTurn(HWND);
impl Drop for AutosaveTurn {
    fn drop(&mut self) {
        cancel_pending_save(self.0);
        AUTOSAVE_TURN.set(None);
    }
}

fn update_draft_status(hwnd: HWND) {
    unsafe {
        let save = GetDlgItem(hwnd, SAVE);
        if LONG_EDIT.get() {
            let text = draft_text(hwnd);
            let valid = text
                .as_ref()
                .is_some_and(|s| kraqel_native_scene_model::valid_scene_edit(s, false));
            let changed = valid
                && EXPECTED.with_borrow(|base| text.as_ref() != Some(base))
                && !REVISION_CONFLICT.get()
                && !AWAITING_REBIND.get();
            let can_continue = AUTOSAVE_TURN
                .get()
                .is_some_and(|(turn, limit)| turn < limit);
            let autosave = changed && can_continue && !CLOSE_REQUESTED.get();
            let scheduled = schedule_autosave(hwnd, text.as_deref().filter(|_| autosave));
            EnableWindow(save, i32::from(changed));
            EnableWindow(
                GetDlgItem(hwnd, SAVE_AND_STAY),
                i32::from(changed && can_continue),
            );
            let label = if REVISION_CONFLICT.get() {
                if GetDlgItem(hwnd, COMPARISON).is_null() {
                    "Revisionskonflikt · Entwurf offen, Speichern gesperrt"
                } else {
                    if PLAIN_REVIEW.get() {
                        "F7 Unterschiede · F8 übernehmen · F9 Entwurf prüfen und speichern"
                    } else {
                        "Konflikt · F6 Seitenwechsel · F7 Unterschiede · Speichern gesperrt"
                    }
                }
            } else if !valid {
                "Text oder Absatzstruktur zu groß · bitte kürzen"
            } else if AUTOSAVE_TURN
                .get()
                .is_some_and(|(turn, limit)| turn == limit)
            {
                "Letzter Schreibzug · bitte ausdrücklich speichern und schließen"
            } else if autosave && !scheduled {
                "Autosave nicht verfügbar · bitte ausdrücklich speichern"
            } else if autosave {
                "Geändert · Autosave nach 700 ms Schreibpause"
            } else {
                "Klartext · bis 4.096 UTF-16-Einheiten"
            };
            SetDlgItemTextW(hwnd, LABEL, wide(label).as_ptr());
            return;
        }
        if save.is_null() || GetDlgItem(hwnd, SECOND).is_null() {
            return;
        }
        enable_menu_command(
            hwnd,
            OPEN_PLAIN,
            !REVISION_CONFLICT.get()
                && !AWAITING_REBIND.get()
                && !INITIAL_BOLD.get()
                && !is_bold()
                && LOADED_REVISION.get().is_some_and(|r| r > 0),
        );
        let current = draft_text(hwnd);
        let dirty = is_bold() != INITIAL_BOLD.get()
            || EXPECTED.with_borrow(|initial| current.as_ref() != Some(initial));
        let autosave = AUTOSAVE_TURN.get();
        let valid = current
            .as_deref()
            .is_some_and(kraqel_native_scene_model::valid_paragraphs);
        let scheduled = autosave.is_none()
            || schedule_autosave(hwnd, current.as_deref().filter(|_| dirty && valid));
        if let Some(text) = current {
            // Match countWords in packages/editor, including ECMAScript's whitespace exceptions.
            let count = text
                .split(|c: char| (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}')
                .filter(|word| !word.is_empty())
                .count();
            let label = if count == 1 {
                "1 Wort".to_owned()
            } else {
                format!("{count} Wörter")
            };
            SetDlgItemTextW(hwnd, WORD_COUNT, wide(label).as_ptr());
        }
        if REVISION_CONFLICT.get() {
            update_review(hwnd);
            cancel_pending_save(hwnd);
            EnableWindow(save, 0);
            enable_menu_command(hwnd, SAVE as usize, false);
            SetWindowTextW(
                hwnd,
                wide(if GetDlgItem(hwnd, COMPARISON).is_null() {
                    "Szene A – Revisionskonflikt · Lokaler Entwurf offen"
                } else {
                    "Szene A – Revisionskonflikt · Entwurf links · Vergleich rechts (F6)"
                })
                .as_ptr(),
            );
            return;
        }
        enable_menu_command(hwnd, SAVE as usize, dirty);
        EnableWindow(save, i32::from(dirty));
        EnableWindow(GetDlgItem(hwnd, DISCARD), i32::from(dirty));
        let status = if dirty && !valid {
            "Ungespeichert · Absätze prüfen"
        } else if dirty && autosave.is_some() && scheduled {
            "Ungespeichert · Schreibpause abwarten"
        } else if dirty {
            "Ungespeicherte Änderungen"
        } else if LOADED_REVISION.get().is_some_and(|revision| revision > 0) {
            "Gespeichert"
        } else {
            "Keine Änderungen"
        };
        if let Some(revision) = LOADED_REVISION.get() {
            let suffix = match autosave {
                Some(_) if dirty && !valid => invalid_paragraph(hwnd)
                    .map(|(_, hint)| format!(" · {hint}"))
                    .unwrap_or_default(),
                Some(_) if !scheduled => " · Bitte manuell speichern".to_owned(),
                Some((turn, limit)) if turn == limit => {
                    format!(" · Letzter Autosave {turn}/{limit}")
                }
                Some((turn, limit)) => format!(" · Autosave {turn}/{limit}"),
                None => String::new(),
            };
            SetWindowTextW(
                hwnd,
                wide(format!("Szene A · Revision {revision} – {status}{suffix}")).as_ptr(),
            );
        }
    }
}
unsafe fn install_selection_menu(hwnd: HWND) -> std::io::Result<()> {
    unsafe {
        let bar = GetMenu(hwnd);
        let menu = CreatePopupMenu();
        require(!menu.is_null(), "selection menu")?;
        for (id, label) in [
            (PICK_FIRST, "Ersten Absatz aus Entwurf"),
            (PICK_SECOND, "Zweiten Absatz aus Entwurf"),
            (PICK_BOLD, "Fettmarkierung aus Entwurf"),
            (
                PICK_LOCAL_CHANGES,
                "Eindeutig lokale Änderungen zusätzlich auswählen",
            ),
            (PICK_DRAFT, "Gesamten lokalen Entwurf auswählen"),
            (PICK_SERVICE, "Auswahl auf Dienststand zurücksetzen"),
            (UNDO_SELECTION, "Letzte Auswahl rückgängig machen"),
            (SHOW_EDIT_BASE, "Ursprüngliche Bearbeitungsbasis anzeigen"),
            (COMMIT_SELECTION, "Auswahl speichern und schließen"),
            (CONTINUE_SELECTION, "Auswahl speichern und weiterschreiben"),
        ] {
            if AppendMenuW(menu, MF_STRING, id, wide(label).as_ptr()) == 0 {
                DestroyMenu(menu);
                return Err(std::io::Error::last_os_error());
            }
        }
        if AppendMenuW(bar, MF_POPUP, menu as usize, wide("&Übernahme").as_ptr()) == 0 {
            DestroyMenu(menu);
            return Err(std::io::Error::last_os_error());
        }
        check(DrawMenuBar(hwnd))
    }
}

const MERGE_EXIT: i32 = 43;
const MERGE_CONTINUE_EXIT: i32 = 44;

fn remove_selection_menu(hwnd: HWND) -> std::io::Result<()> {
    unsafe {
        let menu = GetMenu(hwnd);
        if GetMenuState(menu, COMMIT_SELECTION as u32, MF_BYCOMMAND) != u32::MAX {
            check(DeleteMenu(
                menu,
                (GetMenuItemCount(menu) - 1) as u32,
                MF_BYPOSITION,
            ))?;
            check(DrawMenuBar(hwnd))?;
        }
        Ok(())
    }
}

fn resume_merged_editor(
    held: &mut HeldEditor,
    view: &kraqel_native_scene_model::channel::SceneView,
) -> std::io::Result<()> {
    let hwnd = held.hwnd;
    unsafe {
        remove_selection_menu(hwnd)?;
        SELECTION_REVIEW.replace(None);
        check(DestroyWindow(GetDlgItem(hwnd, COMPARISON)))?;
        REVISION_CONFLICT.set(false);
        require(
            restore_draft(hwnd, &view.text)
                && set_bold(hwnd, view.bold_second)
                && layout_surface(hwnd),
            "resume merged draft",
        )?;
        // Start undo at the accepted mixture so discarded conflict values cannot reappear.
        held.history = Some(DraftHistory::new(view.text.clone()));
        held.resume_after_merge = false;
        EnableWindow(hwnd, 1);
        SetFocus(GetDlgItem(hwnd, ACTIVE_PARAGRAPH.get()));
        Ok(())
    }
}

fn menu_checked(hwnd: HWND, id: usize) -> bool {
    let state = unsafe { GetMenuState(GetMenu(hwnd), id as u32, MF_BYCOMMAND) };
    state != u32::MAX && state & MF_CHECKED != 0
}

fn combined_draft(
    hwnd: HWND,
    view: &kraqel_native_scene_model::channel::SceneView,
) -> Option<(String, bool)> {
    let (first, second) = view.text.split_once('\n')?;
    let first = if menu_checked(hwnd, PICK_FIRST) {
        paragraph_text(hwnd, EDIT)?
    } else {
        first.to_owned()
    };
    let second = if menu_checked(hwnd, PICK_SECOND) {
        paragraph_text(hwnd, SECOND)?
    } else {
        second.to_owned()
    };
    let bold = if menu_checked(hwnd, PICK_BOLD) {
        is_bold()
    } else {
        view.bold_second
    };
    let text = format!("{first}\n{second}");
    kraqel_native_scene_model::valid_paragraphs(&text).then_some((text, bold))
}

struct SelectionReview {
    previous_picks: Option<[bool; MERGE_FIELDS.len()]>,
    basis: kraqel_native_scene_model::channel::SceneView,
    may_save: bool,
    may_resume: bool,
}

#[derive(Default)]
struct SelectionHints {
    local: Vec<usize>,
    conflicting: Vec<&'static str>,
}

fn selection_hints(
    hwnd: HWND,
    view: &kraqel_native_scene_model::channel::SceneView,
) -> SelectionHints {
    let mut hints = SelectionHints::default();
    EXPECTED.with_borrow(|base| {
        let Some((base_first, base_second)) = base.split_once('\n') else {
            return;
        };
        let Some((remote_first, remote_second)) = view.text.split_once('\n') else {
            return;
        };
        for (field, item, label, base, remote) in [
            (EDIT, PICK_FIRST, "erster Absatz", base_first, remote_first),
            (
                SECOND,
                PICK_SECOND,
                "zweiter Absatz",
                base_second,
                remote_second,
            ),
        ] {
            let Some(local) = paragraph_text(hwnd, field) else {
                continue;
            };
            if local != base && local != remote {
                if remote != base {
                    hints.conflicting.push(label);
                } else if kraqel_native_scene_model::valid_paragraph(&local) {
                    hints.local.push(item);
                }
            }
        }
    });
    if is_bold() != INITIAL_BOLD.get() && view.bold_second == INITIAL_BOLD.get() {
        hints.local.push(PICK_BOLD);
    }
    hints
}

fn change_selection(hwnd: HWND, command: usize) {
    SELECTION_REVIEW.with_borrow_mut(|review| {
        let Some(review) = review else { return };
        let before = MERGE_FIELDS.map(|field| menu_checked(hwnd, field));
        let mut after = before;
        match command {
            PICK_DRAFT => after.fill(true),
            PICK_SERVICE => after.fill(false),
            PICK_LOCAL_CHANGES => {
                let hints = selection_hints(hwnd, &review.basis);
                for (index, field) in MERGE_FIELDS.iter().enumerate() {
                    after[index] |= hints.local.contains(field);
                }
            }
            UNDO_SELECTION => {
                let Some(previous) = review.previous_picks.take() else {
                    return;
                };
                after = previous;
            }
            _ => {
                let Some(index) = MERGE_FIELDS.iter().position(|&field| field == command) else {
                    return;
                };
                after[index] = !before[index];
            }
        }
        if after != before {
            if command != UNDO_SELECTION {
                review.previous_picks = Some(before);
            }
            for (field, picked) in MERGE_FIELDS.into_iter().zip(after) {
                let mark = if picked { MF_CHECKED } else { MF_UNCHECKED };
                unsafe { CheckMenuItem(GetMenu(hwnd), field as u32, MF_BYCOMMAND | mark) };
            }
        }
    });
    update_review(hwnd);
}

fn refresh_comparison_text(field: HWND, contents: &str) {
    unsafe {
        let next = wide(contents);
        let mut previous = vec![0u16; GetWindowTextLengthW(field) as usize + 1];
        let count = GetWindowTextW(field, previous.as_mut_ptr(), previous.len() as i32) as usize;
        previous.truncate(count);
        if previous == next[..next.len() - 1] {
            return;
        }

        let first_line = SendMessageW(field, EM_GETFIRSTVISIBLELINE, 0, 0);
        let range = SendMessageW(field, EM_GETSEL, 0, 0) as u32;
        if SetWindowTextW(field, next.as_ptr()) == 0 {
            return;
        }
        // Keep a selection only while the text through its end still matches.
        let end = (range >> 16) as usize;
        if end <= previous.len() && end < next.len() && previous[..end] == next[..end] {
            SendMessageW(field, EM_SETSEL, (range & 0xffff) as usize, end as isize);
        }
        let current_line = SendMessageW(field, EM_GETFIRSTVISIBLELINE, 0, 0);
        SendMessageW(field, EM_LINESCROLL, 0, first_line - current_line);
    }
}

fn update_review(hwnd: HWND) {
    SELECTION_REVIEW.with_borrow(|review| {
        let Some(review) = review else { return };
        let view = &review.basis;
        let format_text = |text: &str, bold: bool| {
            format!(
                "{}\r\nZweiter Absatz: {}",
                text.replace('\n', "\r\n"),
                if bold { "fett" } else { "normal" }
            )
        };
        let checked = MERGE_FIELDS.iter().filter(|&&id| menu_checked(hwnd, id)).count();
        enable_menu_command(hwnd, PICK_DRAFT, checked < MERGE_FIELDS.len());
        enable_menu_command(hwnd, PICK_SERVICE, checked > 0);
        enable_menu_command(hwnd, UNDO_SELECTION, review.previous_picks.is_some());
        let selected = combined_draft(hwnd, view);
        let changed = selected
            .as_ref()
            .is_some_and(|(text, bold)| text != &view.text || *bold != view.bold_second);
        enable_menu_command(hwnd, COMMIT_SELECTION, changed && review.may_save);
        enable_menu_command(hwnd, CONTINUE_SELECTION, changed && review.may_resume);
        let preview = match selected {
            Some((text, bold)) if changed => format_text(&text, bold),
            Some(_) => "Auswahl entspricht dem Dienststand.".to_owned(),
            None => "Ausgewählte Absätze prüfen.".to_owned(),
        };
        let hints = selection_hints(hwnd, view);
        enable_menu_command(
            hwnd,
            PICK_LOCAL_CHANGES,
            hints.local.iter().any(|&item| unsafe {
                GetMenuState(GetMenu(hwnd), item as u32, MF_BYCOMMAND) & MF_CHECKED == 0
            }),
        );
        let conflict_note = if hints.conflicting.is_empty() {
            String::new()
        } else {
            format!(
                "\r\n\r\nBeidseitig unterschiedlich geändert: {}.",
                hints.conflicting.join(", ")
            )
        };
        let base_section = if menu_checked(hwnd, SHOW_EDIT_BASE) {
            EDIT_BASE_REVISION.get().map(|revision| {
                EXPECTED.with_borrow(|base| format!(
                    "Bearbeitungsbasis · Revision {revision}\r\n{}\r\n\r\n",
                    format_text(base, INITIAL_BOLD.get())
                ))
            }).unwrap_or_default()
        } else {
            String::new()
        };
        let contents = format!(
            "{base_section}Datendienst · Revision {}\r\n{}\r\n\r\nVorschau der Auswahl\r\n{}{conflict_note}",
            view.revision,
            format_text(&view.text, view.bold_second),
            preview
        );
        unsafe {
            refresh_comparison_text(GetDlgItem(hwnd, COMPARISON), &contents);
        }
    });
}

fn resolve_conflict(
    held: &mut HeldEditor,
    comparison: Option<&kraqel_native_scene_model::channel::SceneView>,
    can_continue: bool,
    room_to_resume: bool,
    review: bool,
) -> std::io::Result<Option<(String, bool)>> {
    let hwnd = held.hwnd;
    require(!hwnd.is_null(), "missing conflict editor")?;
    SELECTION_REVIEW.replace(None);
    REVISION_CONFLICT.set(true);
    AWAITING_REBIND.set(false);
    held.autosave_turn = None;
    held.resume_after_merge = false;
    let selectable = comparison.filter(|v| {
        can_continue
            && (review || LOADED_REVISION.get().and_then(|r| r.checked_add(1)) == Some(v.revision))
    });
    unsafe {
        // On refresh keep the checkmarks as a proposal; the old approval is spent.
        if !review {
            remove_selection_menu(hwnd)?;
        }
        let old = GetDlgItem(hwnd, COMPARISON);
        if !old.is_null() {
            check(DestroyWindow(old))?;
            require(layout_surface(hwnd), "scene comparison layout")?;
        }
        if let Some(view) = comparison {
            show_comparison(hwnd, view)?;
        }
        if !review && comparison.is_some() {
            install_selection_menu(hwnd)?;
        }
        let instruction = if comparison.is_none() {
            "Keine bestätigte Vergleichsbasis. Der Entwurf bleibt lokal."
        } else if selectable.is_none() {
            "Auswahl als Vorschau prüfen · Speichern in dieser Sitzung gesperrt."
        } else if review {
            "Neue Dienstrevision · Auswahl prüfen und unter Übernahme bestätigen."
        } else {
            "Unter Übernahme die gewünschten lokalen Änderungen auswählen."
        };
        SetDlgItemTextW(hwnd, LABEL, wide(instruction).as_ptr());
        let allow_resume = room_to_resume
            && selectable
                .is_some_and(|view| view.revision < window_receipt::SCENE_REVISION_LIMIT - 1);
        SELECTION_REVIEW.replace(comparison.cloned().map(|basis| SelectionReview {
            previous_picks: None,
            basis,
            may_save: selectable.is_some(),
            may_resume: allow_resume,
        }));
        EnableWindow(hwnd, 1);
        update_draft_status(hwnd);
        SetFocus(GetDlgItem(hwnd, ACTIVE_PARAGRAPH.get()));
        let mut local = (String::new(), false);
        loop {
            let code = run_editor_loop(hwnd, true, Some(&mut local), Some(held));
            if code == window_receipt::DISCARDED_EXIT as i32 {
                return Ok(None);
            }
            let resume = code == MERGE_CONTINUE_EXIT && allow_resume;
            require(code == MERGE_EXIT || resume, "conflict decision")?;
            let view = selectable.ok_or(std::io::Error::other("missing merge basis"))?;
            if let Some(chosen) = combined_draft(hwnd, view)
                .filter(|(t, b)| t != &view.text || *b != view.bold_second)
            {
                if review {
                    let question = format!(
                        "Die geprüfte Auswahl auf Basis von Revision {} speichern und {}?",
                        view.revision,
                        if resume {
                            "weiterschreiben"
                        } else {
                            "das Fenster schließen"
                        }
                    );
                    let decision = MessageBoxW(
                        hwnd,
                        wide(question).as_ptr(),
                        wide("Geprüfte Auswahl bestätigen").as_ptr(),
                        MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION,
                    );
                    if decision != IDYES {
                        continue;
                    }
                }
                held.resume_after_merge = resume;
                held.submitted = Some((view.text.clone(), view.bold_second));
                held.closing = false;
                EnableWindow(hwnd, 0);
                return Ok(Some(chosen));
            }
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide("Gültige lokale Änderung unter Übernahme auswählen.").as_ptr(),
            );
        }
    }
}

fn navigate_paragraph_difference(hwnd: HWND, backwards: bool) {
    let Some(draft) = draft_text(hwnd) else {
        return;
    };
    let result = PARAGRAPH_COMPARE.with_borrow_mut(|state| {
        state
            .as_mut()
            .map(|comparison| comparison.step(&draft, backwards))
    });
    let Some(selection) = result else { return };
    unsafe {
        let message = if let Some((index, count, local, remote)) = selection {
            let pane = GetDlgItem(hwnd, COMPARISON);
            SetFocus(pane);
            for (control, range) in [(GetDlgItem(hwnd, EDIT), local), (pane, remote)] {
                SendMessageW(control, EM_SETSEL, range.start, range.end as isize);
                SendMessageW(
                    control,
                    windows_sys::Win32::UI::Controls::EM_SCROLLCARET,
                    0,
                    0,
                );
            }
            format!(
                "Unterschied {index} von {count} · F7 weiter · Umschalt+F7 zurück · F8 in Entwurf übernehmen"
            )
        } else {
            String::from("Beide Texte stimmen überein · Speichern gesperrt")
        };
        SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
    }
}

fn take_service_paragraphs(hwnd: HWND) {
    let Some(draft) = draft_text(hwnd) else {
        return;
    };
    let merged =
        PARAGRAPH_COMPARE.with_borrow(|state| state.as_ref()?.selected_service_block(&draft));
    unsafe {
        let message = match merged {
            None => "Zuerst mit F7 einen Unterschied auswählen",
            Some(text) if !kraqel_native_scene_model::valid_scene_edit(&text, false) => {
                "Übernahme überschreitet die Textgrenzen · Entwurf unverändert"
            }
            Some(text) => {
                let edit = GetDlgItem(hwnd, EDIT);
                SetFocus(edit);
                let replacement = wide(text.replace('\n', "\r\n"));
                SendMessageW(edit, EM_SETSEL, 0, -1);
                SendMessageW(edit, EM_REPLACESEL, 1, replacement.as_ptr() as LPARAM);
                if PLAIN_REVIEW.get() {
                    "Dienstabsätze übernommen · F7 weiter · F9 Entwurf prüfen und speichern"
                } else {
                    "Dienstabsätze im lokalen Entwurf übernommen · F7 weiter · Speichern gesperrt"
                }
            }
        };
        SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
    }
}

fn inspect_conflict(
    held: &mut HeldEditor,
    view: Option<(u32, &str)>,
    allow_review: bool,
) -> std::io::Result<Option<(String, bool)>> {
    require(!held.hwnd.is_null(), "plain conflict editor")?;
    AWAITING_REBIND.set(false);
    REVISION_CONFLICT.set(true);
    held.autosave_turn = None;
    unsafe {
        PARAGRAPH_COMPARE.replace(None);
        if !LONG_EDIT.get() {
            SELECTION_REVIEW.replace(None);
            remove_selection_menu(held.hwnd)?;
        }
        let old = GetDlgItem(held.hwnd, COMPARISON);
        if !old.is_null() {
            check(DestroyWindow(old))?;
        }
        if let Some((revision, text)) = view {
            let header = format!("Datendienst · Revision {revision}\r\n\r\n");
            create_comparison_field(
                held.hwnd,
                &format!("{header}{}", text.replace('\n', "\r\n")),
            )?;
            if LONG_EDIT.get() {
                PARAGRAPH_COMPARE
                    .replace(Some(paragraph_comparison::Comparison::new(text, &header)));
            }
        }
        PLAIN_REVIEW.set(allow_review);
        EnableWindow(held.hwnd, 1);
        update_draft_status(held.hwnd);
        if !LONG_EDIT.get() {
            SetDlgItemTextW(
                held.hwnd,
                LABEL,
                wide("Entwurf links, Textvorschau rechts (F6) · Speichern gesperrt").as_ptr(),
            );
        } else {
            SetWindowTextW(
                held.hwnd,
                wide("Szene A · Klartext · Revisionskonflikt").as_ptr(),
            );
        }
        SetFocus(GetDlgItem(held.hwnd, EDIT));
        let mut draft = (String::new(), false);
        loop {
            let code = run_editor_loop(held.hwnd, true, Some(&mut draft), Some(held));
            if code == window_receipt::DISCARDED_EXIT as i32 {
                PLAIN_REVIEW.set(false);
                return Ok(None);
            }
            require(
                allow_review && code == MERGE_EXIT,
                "plain conflict decision",
            )?;
            let (revision, base) = view.ok_or(std::io::Error::other("missing plain comparison"))?;
            let Some(text) = draft_text(held.hwnd)
                .filter(|t| t != base && kraqel_native_scene_model::valid_scene_edit(t, false))
            else {
                SetDlgItemTextW(
                    held.hwnd,
                    LABEL,
                    wide("Zum Speichern ist ein gültiger, abweichender Entwurf erforderlich")
                        .as_ptr(),
                );
                continue;
            };
            if MessageBoxW(held.hwnd, wide(format!("Den gesamten lokalen Entwurf auf Basis von Revision {revision} speichern und schließen?")).as_ptr(),
                wide("Entwurf speichern").as_ptr(), MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION) != IDYES {
                continue;
            }
            PLAIN_REVIEW.set(false);
            held.submitted = Some((base.to_owned(), false));
            held.closing = false;
            held.resume_after_merge = false;
            EnableWindow(held.hwnd, 0);
            return Ok(Some((text, false)));
        }
    }
}

fn show_comparison(
    hwnd: HWND,
    view: &kraqel_native_scene_model::channel::SceneView,
) -> std::io::Result<()> {
    let weight = if view.bold_second { "fett" } else { "normal" };
    let content = format!(
        "Datendienst · Revision {}\r\nZweiter Absatz: {weight}\r\n\r\n{}",
        view.revision,
        view.text.replace('\n', "\r\n")
    );
    create_comparison_field(hwnd, &content)
}

fn create_comparison_field(hwnd: HWND, content: &str) -> std::io::Result<()> {
    unsafe {
        let field = CreateWindowExW(
            0,
            wide("EDIT").as_ptr(),
            wide(content).as_ptr(),
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | WS_BORDER
                | WS_VSCROLL
                | (ES_MULTILINE | ES_READONLY) as u32,
            0,
            0,
            0,
            0,
            hwnd,
            COMPARISON as _,
            GetModuleHandleW(null()),
            null(),
        );
        require(
            !field.is_null() && protect_editor_field(field),
            "scene comparison field",
        )?;
        SendMessageW(
            field,
            WM_SETFONT,
            SendDlgItemMessageW(hwnd, EDIT, WM_GETFONT, 0, 0) as usize,
            1,
        );
        require(layout_surface(hwnd), "scene comparison layout")
    }
}

fn enter_plain_editor(hwnd: HWND, text: &str) -> bool {
    unsafe {
        cancel_pending_save(hwnd);
        let menu = GetMenu(hwnd);
        if !menu.is_null() {
            if SetMenu(hwnd, null_mut()) == 0 {
                return false;
            }
            DestroyMenu(menu);
            DrawMenuBar(hwnd);
        }
        let stay = CreateWindowExW(
            0,
            wide("BUTTON").as_ptr(),
            wide("Speichern und weiterschreiben").as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
            0,
            0,
            0,
            0,
            hwnd,
            SAVE_AND_STAY as _,
            GetModuleHandleW(null()),
            null(),
        );
        if stay.is_null() {
            return false;
        }
        SendMessageW(
            stay,
            WM_SETFONT,
            SendDlgItemMessageW(hwnd, SAVE, WM_GETFONT, 0, 0) as usize,
            1,
        );
        LONG_EDIT.set(true);
        for id in [
            SECOND,
            DISCARD,
            SEARCH,
            SEARCH_LABEL,
            WORD_COUNT,
            REPLACEMENT,
            REPLACEMENT_LABEL,
        ] {
            ShowWindow(GetDlgItem(hwnd, id), SW_HIDE);
        }
        let field = GetDlgItem(hwnd, EDIT);
        SendMessageW(
            field,
            EM_SETLIMITTEXT,
            2 * kraqel_native_scene_model::MAX_PREVIEW_UTF16,
            0,
        );
        SetDlgItemTextW(
            hwnd,
            SAVE,
            wide("Speichern und schließen (Strg+S)").as_ptr(),
        );
        SetWindowTextW(
            hwnd,
            wide(format!(
                "Szene A · Klartext · Revision {}",
                LOADED_REVISION.get().unwrap_or(0)
            ))
            .as_ptr(),
        );
        if SetWindowTextW(field, wide(text.replace('\n', "\r\n")).as_ptr()) == 0
            || !layout_surface(hwnd)
        {
            return false;
        }
        update_draft_status(hwnd);
        SetFocus(field);
        true
    }
}

fn layout_surface(hwnd: HWND) -> bool {
    unsafe {
        let mut area = RECT::default();
        if GetClientRect(hwnd, &mut area) == 0 {
            return false;
        }
        let px = |value| scale(value, GetDpiForWindow(hwnd));
        let margin = px(15);
        let gap = px(8);
        let width = (area.right - 2 * margin).max(1);
        let editing = !GetDlgItem(hwnd, SAVE).is_null();
        let replace_y = area.bottom - px(38);
        let search_y = replace_y - px(32);
        let count_y = search_y - px(26);
        let buttons_y = if editing {
            count_y - px(36)
        } else {
            area.bottom - px(44)
        };
        let text_top = px(38);
        let focused = editing && in_focus_mode(hwnd);
        let searchable_preview = !editing && !GetDlgItem(hwnd, SEARCH).is_null();
        let text_bottom = if searchable_preview {
            buttons_y - px(32) - gap
        } else if focused {
            area.bottom - margin
        } else {
            buttons_y - gap
        };
        let text_height = (text_bottom - text_top).max(1);
        let place = |id, x, y, w, h| MoveWindow(GetDlgItem(hwnd, id), x, y, w, h, 1) != 0;
        if LONG_EDIT.get() {
            let y = area.bottom - px(44);
            let compare = !GetDlgItem(hwnd, COMPARISON).is_null();
            let left = if compare { (width - gap) / 2 } else { width };
            let height = (y - gap - text_top).max(1);
            return place(LABEL, margin, px(10), width, px(24))
                && place(EDIT, margin, text_top, left, height)
                && (!compare
                    || place(
                        COMPARISON,
                        margin + left + gap,
                        text_top,
                        width - left - gap,
                        height,
                    ))
                && place(SAVE_AND_STAY, margin, y, (width - gap) / 2, px(30))
                && place(
                    SAVE,
                    margin + (width + gap) / 2,
                    y,
                    width - (width + gap) / 2,
                    px(30),
                );
        }
        if !place(LABEL, margin, px(10), width, px(24)) {
            return false;
        }
        if GetDlgItem(hwnd, SECOND).is_null() {
            if !place(EDIT, margin, text_top, width, text_height) {
                return false;
            }
        } else {
            let width = if !GetDlgItem(hwnd, COMPARISON).is_null() {
                let left = (width - gap) / 2;
                if !place(
                    COMPARISON,
                    margin + left + gap,
                    text_top,
                    width - left - gap,
                    text_height,
                ) {
                    return false;
                }
                left
            } else {
                width
            };
            let first_height = (text_height - gap) / 2;
            if !place(EDIT, margin, text_top, width, first_height)
                || !place(
                    SECOND,
                    margin,
                    text_top + first_height + gap,
                    width,
                    text_height - first_height - gap,
                )
            {
                return false;
            }
        }
        if focused {
            return true;
        }
        if editing {
            let half = (width - gap) / 2;
            place(SAVE, margin, buttons_y, half, px(30))
                && place(
                    DISCARD,
                    margin + half + gap,
                    buttons_y,
                    width - half - gap,
                    px(30),
                )
                && place(WORD_COUNT, margin, count_y, width, px(22))
                && place(SEARCH_LABEL, margin, search_y + px(3), px(120), px(24))
                && place(SEARCH, margin + px(120), search_y, width - px(120), px(26))
                && place(
                    REPLACEMENT_LABEL,
                    margin,
                    replace_y + px(3),
                    px(120),
                    px(24),
                )
                && place(
                    REPLACEMENT,
                    margin + px(120),
                    replace_y,
                    width - px(120),
                    px(26),
                )
        } else {
            place(CLOSE, margin, buttons_y, width, px(30))
                && (!searchable_preview
                    || (place(SEARCH_LABEL, margin, buttons_y - px(29), px(120), px(24))
                        && place(
                            SEARCH,
                            margin + px(120),
                            buttons_y - px(32),
                            width - px(120),
                            px(26),
                        )))
        }
    }
}

unsafe extern "system" fn window(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DPICHANGED => {
                let rect = &*(l as *const RECT);
                SetWindowPos(
                    hwnd,
                    null_mut(),
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                if !GetDlgItem(hwnd, EDIT).is_null() {
                    if !apply_fonts(hwnd, (w & 0xffff) as u32) || !layout_surface(hwnd) {
                        PostQuitMessage(24);
                    }
                }
                0
            }
            WM_GETMINMAXINFO => {
                let bounds = &mut *(l as *mut MINMAXINFO);
                bounds.ptMinTrackSize.x = bounds
                    .ptMinTrackSize
                    .x
                    .max(scale(460, GetDpiForWindow(hwnd)));
                bounds.ptMinTrackSize.y = bounds.ptMinTrackSize.y.max(scale(
                    if GetMenu(hwnd).is_null() { 250 } else { 322 },
                    GetDpiForWindow(hwnd),
                ));
                0
            }
            WM_SIZE if w != SIZE_MINIMIZED as usize && !GetDlgItem(hwnd, EDIT).is_null() => {
                layout_surface(hwnd);
                0
            }
            WM_COMMAND
                if matches!((w >> 16) as u32, EN_CHANGE | EN_MAXTEXT)
                    && matches!((w & 0xffff) as i32, EDIT | SECOND) =>
            {
                if !GetDlgItem(hwnd, SAVE).is_null() {
                    let id = (w & 0xffff) as i32;
                    if LONG_EDIT.get() && id == EDIT && (w >> 16) as u32 == EN_CHANGE {
                        PARAGRAPH_COMPARE.with_borrow_mut(|state| {
                            if let Some(state) = state {
                                state.reset();
                            }
                        });
                    }
                    let full =
                        GetWindowTextLengthW(GetDlgItem(hwnd, id)) == MAX_PARAGRAPH_UTF16 as i32;
                    let label = if full || (w >> 16) as u32 == EN_MAXTEXT {
                        if id == EDIT {
                            "Erster Absatz · Eingabegrenze erreicht"
                        } else {
                            "Zweiter Absatz · Eingabegrenze erreicht"
                        }
                    } else {
                        draft_label()
                    };
                    SetDlgItemTextW(hwnd, LABEL, wide(label).as_ptr());
                }
                update_draft_status(hwnd);
                0
            }
            WM_COMMAND
                if (w >> 16) as u32 == EN_SETFOCUS
                    && matches!((w & 0xffff) as i32, EDIT | SECOND) =>
            {
                ACTIVE_PARAGRAPH.set((w & 0xffff) as i32);
                0
            }
            WM_COMMAND
                if REVISION_CONFLICT.get()
                    && (MERGE_FIELDS.contains(&w)
                        || matches!(
                            w,
                            PICK_DRAFT | PICK_SERVICE | PICK_LOCAL_CHANGES | UNDO_SELECTION
                        )) =>
            {
                change_selection(hwnd, w);
                0
            }
            WM_COMMAND
                if REVISION_CONFLICT.get()
                    && matches!(w, COMMIT_SELECTION | CONTINUE_SELECTION | SHOW_EDIT_BASE) =>
            {
                let menu = GetMenu(hwnd);
                let state = GetMenuState(menu, w as u32, MF_BYCOMMAND);
                if state != u32::MAX && state & (MF_GRAYED | MF_DISABLED) == 0 {
                    if matches!(w, COMMIT_SELECTION | CONTINUE_SELECTION) {
                        PostMessageW(hwnd, MERGE_SELECTED, w, 0);
                    } else {
                        CheckMenuItem(
                            menu,
                            w as u32,
                            MF_BYCOMMAND
                                | if state & MF_CHECKED == 0 {
                                    MF_CHECKED
                                } else {
                                    MF_UNCHECKED
                                },
                        );
                        update_review(hwnd);
                    }
                }
                0
            }
            WM_COMMAND if w == REPLACE_ONE || w == REPLACE_ALL => {
                PostMessageW(hwnd, REPLACE_DRAFT, w, 0);
                0
            }
            WM_COMMAND if w == FOCUS_MODE && !GetMenu(hwnd).is_null() => {
                set_focus_mode(hwnd, !in_focus_mode(hwnd));
                0
            }
            WM_COMMAND if w == OPEN_PLAIN => {
                if !LONG_EDIT.get()
                    && !REVISION_CONFLICT.get()
                    && !AWAITING_REBIND.get()
                    && !INITIAL_BOLD.get()
                    && !is_bold()
                    && LOADED_REVISION.get().is_some_and(|r| r > 0)
                    && !GetDlgItem(hwnd, SAVE).is_null()
                {
                    if let Some(text) = draft_text(hwnd) {
                        if !enter_plain_editor(hwnd, &text) {
                            PostQuitMessage(24);
                        }
                    }
                }
                0
            }
            WM_COMMAND if w == FIND => {
                open_search(hwnd);
                0
            }
            WM_COMMAND if w == FIND_NEXT || w == FIND_PREVIOUS => {
                find_in_scene(hwnd, w == FIND_PREVIOUS);
                0
            }
            WM_COMMAND if (w >> 16) as u32 == EN_CHANGE && (w & 0xffff) as i32 == SEARCH => {
                SetDlgItemTextW(
                    hwnd,
                    LABEL,
                    wide(if GetDlgItem(hwnd, SAVE).is_null() {
                        "Schreibgeschützte Textvorschau · Formatierungen ausgeblendet"
                    } else {
                        draft_label()
                    })
                    .as_ptr(),
                );
                0
            }
            WM_COMMAND if w == UNDO || w == REDO => {
                PostMessageW(
                    hwnd,
                    if w == UNDO {
                        HISTORY_UNDO
                    } else {
                        HISTORY_REDO
                    },
                    0,
                    0,
                );
                0
            }
            WM_INITMENU if !GetMenu(hwnd).is_null() => {
                let focus = GetFocus();
                enable_menu_command(
                    hwnd,
                    SELECT_PARAGRAPH,
                    focus == GetDlgItem(hwnd, EDIT) || focus == GetDlgItem(hwnd, SECOND),
                );
                0
            }
            WM_COMMAND if w == BOLD_SECOND => {
                PostMessageW(hwnd, TOGGLE_BOLD, 0, 0);
                0
            }
            WM_COMMAND if w == SELECT_PARAGRAPH => {
                select_paragraph(hwnd);
                0
            }
            WM_COMMAND if w & 0xffff == DISCARD as usize => {
                PostMessageW(hwnd, DISCARD_DRAFT, 0, 0);
                0
            }
            WM_COMMAND if w == SAVE_AND_STAY as usize && LONG_EDIT.get() => {
                PostMessageW(hwnd, CONTINUE_PLAIN_DRAFT, 0, 0);
                0
            }
            WM_COMMAND if w & 0xffff == SAVE as usize => {
                PostMessageW(hwnd, SAVE_DRAFT, 0, 0);
                0
            }
            WM_COMMAND if w & 0xffff == CLOSE as usize => {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
                0
            }
            WM_CLOSE
                if REVISION_CONFLICT.get()
                    || AWAITING_REBIND.get()
                    || AUTOSAVE_TURN.get().is_some() =>
            {
                if IsWindowEnabled(hwnd) != 0 && !CLOSE_REQUESTED.replace(true) {
                    cancel_pending_save(hwnd);
                    PostMessageW(hwnd, WM_NULL, 0, 0);
                }
                0
            }
            WM_CLOSE => finish_close(hwnd),
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
unsafe fn finish_close(hwnd: HWND) -> isize {
    unsafe {
        if REVISION_CONFLICT.get() {
            let previous_focus = GetFocus();
            let answer = MessageBoxW(
                hwnd,
                wide("Lokalen Entwurf verwerfen und das Konfliktfenster schließen?").as_ptr(),
                wide("Revisionskonflikt").as_ptr(),
                MB_YESNO | MB_DEFBUTTON2 | MB_ICONWARNING,
            );
            if answer != IDYES {
                if IsChild(hwnd, previous_focus) != 0 {
                    SetFocus(previous_focus);
                }
                return 0;
            }
        } else if !GetDlgItem(hwnd, SAVE).is_null() && draft_changed(hwnd) {
            if IsWindowEnabled(hwnd) == 0 {
                return 0;
            }
            cancel_pending_save(hwnd);
            let previous_focus = GetFocus();
            let answer = MessageBoxW(
                        hwnd,
                        wide("Änderungen an Szene A speichern?\n\nJa: Speichern.\nNein: Änderungen verwerfen.\nAbbrechen: Weiterarbeiten.").as_ptr(),
                        wide("Ungespeicherte Änderungen").as_ptr(),
                        MB_YESNOCANCEL | MB_DEFBUTTON3 | MB_ICONWARNING,
                    );
            if answer == IDYES {
                PostMessageW(hwnd, SAVE_DRAFT, 1, 0);
                return 0;
            }
            if answer != IDNO {
                update_draft_status(hwnd);
                if IsChild(hwnd, previous_focus) != 0 {
                    SetFocus(previous_focus);
                }
                return 0;
            }
        }
        let expected = EXPECTED.with_borrow(Clone::clone);
        // A Win32 multiline EDIT exposes CRLF for each stored LF.
        let mut value =
            vec![0u16; expected.encode_utf16().count() + expected.matches('\n').count() + 2];
        let n = GetDlgItemTextW(hwnd, EDIT, value.as_mut_ptr(), value.len() as i32);
        let first = String::from_utf16_lossy(&value[..n as usize]);
        let second = GetDlgItem(hwnd, SECOND);
        let font = if second.is_null() {
            null_mut()
        } else {
            SendMessageW(second, WM_GETFONT, 0, 0) as HFONT
        };
        let ok = if second.is_null() {
            first.replace("\r\n", "\n") == expected
        } else {
            let n = GetWindowTextW(second, value.as_mut_ptr(), value.len() as i32);
            let number = String::from_utf16_lossy(&value[..n as usize]);
            format!("{first}\n{number}") == expected
                && font_description(font).is_some_and(|desc| desc.lfWeight == SECOND_WEIGHT.get())
        };
        let editing = !GetDlgItem(hwnd, SAVE).is_null();
        destroy_surface(hwnd);
        PostQuitMessage(if editing {
            window_receipt::DISCARDED_EXIT as i32
        } else if ok {
            40
        } else {
            41
        });
        0
    }
}

struct SceneSurface {
    output: Option<std::fs::File>,
    text: String,
    second_bold: Option<bool>,
    text_preview: bool,
}

fn show(
    endpoint: Option<bootstrap::Endpoint>,
    mut surface: Option<SceneSurface>,
    draft: Option<&mut (String, bool)>,
    keep_editor: Option<&mut HeldEditor>,
) -> i32 {
    unsafe {
        if SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_null() {
            return 24;
        }
        LONG_EDIT.set(false);
        let long = draft.is_some()
            && surface
                .as_ref()
                .is_some_and(|s| !kraqel_native_scene_model::valid_paragraphs(&s.text));
        let bold = surface.as_ref().and_then(|s| s.second_bold) == Some(true);
        EDIT_BASE_REVISION.set(LOADED_REVISION.get());
        INITIAL_BOLD.set(bold);
        SECOND_WEIGHT.set(if bold { FW_BOLD } else { FW_NORMAL } as i32);
        EXPECTED.with_borrow_mut(|value| {
            *value = surface
                .as_ref()
                .map(|s| s.text.clone())
                .unwrap_or_else(|| "Rabe\n42".into())
        });
        let preview = surface.as_ref().is_some_and(|s| s.text_preview);
        let action = if draft.is_some() { SAVE } else { CLOSE };
        let instance = GetModuleHandleW(null());
        let name = wide("KRAQEL.Synthetic.Edit");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            hbrBackground: 6usize as _,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
            return 22;
        }
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            name.as_ptr(),
            wide("KRAQEL – Offline-Fensterprobe").as_ptr(),
            WS_OVERLAPPED
                | WS_CAPTION
                | WS_SYSMENU
                | WS_THICKFRAME
                | WS_MAXIMIZEBOX
                | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            scale(640, GetDpiForSystem()),
            scale(if draft.is_some() { 352 } else { 280 }, GetDpiForSystem()),
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return 23;
        }
        if draft.is_some() && !install_editor_menu(hwnd) {
            return 24;
        }
        let dpi = GetDpiForWindow(hwnd);
        if dpi != GetDpiForSystem()
            && SetWindowPos(
                hwnd,
                null_mut(),
                0,
                0,
                scale(640, dpi),
                scale(if draft.is_some() { 352 } else { 280 }, dpi),
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            ) == 0
        {
            return 24;
        }
        for (id, kind, title, style) in [
            (
                LABEL,
                "STATIC",
                if draft.is_some() {
                    draft_label()
                } else if preview {
                    "Schreibgeschützte Textvorschau · Formatierungen ausgeblendet"
                } else {
                    "&Prüftext"
                },
                SS_LEFT,
            ),
            (
                EDIT,
                "EDIT",
                "",
                WS_TABSTOP
                    | WS_BORDER
                    | ES_NOHIDESEL as u32
                    | ES_MULTILINE as u32
                    | ES_WANTRETURN as u32
                    | if preview || draft.is_some() {
                        WS_VSCROLL
                    } else {
                        0
                    },
            ),
            (
                action,
                "BUTTON",
                if draft.is_some() {
                    "&Speichern (Strg+S)"
                } else {
                    "&Schließen"
                },
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (WORD_COUNT, "STATIC", "", SS_LEFT),
            (REPLACEMENT_LABEL, "STATIC", "Ersatz (Strg+H)", SS_LEFT),
            (
                REPLACEMENT,
                "EDIT",
                "",
                WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
            ),
            (SEARCH_LABEL, "STATIC", "&Suche (Enter)", SS_LEFT),
            (
                SEARCH,
                "EDIT",
                "",
                WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32,
            ),
        ] {
            if matches!(
                id,
                WORD_COUNT | SEARCH_LABEL | SEARCH | REPLACEMENT | REPLACEMENT_LABEL
            ) && draft.is_none()
                && !(preview && matches!(id, SEARCH | SEARCH_LABEL))
            {
                continue;
            }
            if CreateWindowExW(
                0,
                wide(kind).as_ptr(),
                wide(title).as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                0,
                0,
                0,
                0,
                hwnd,
                id as _,
                instance,
                null(),
            )
            .is_null()
            {
                return 24;
            }
        }
        if draft.is_some() {
            if CreateWindowExW(
                0,
                wide("BUTTON").as_ptr(),
                wide("Änderungen &verwerfen").as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
                0,
                0,
                0,
                0,
                hwnd,
                DISCARD as _,
                instance,
                null(),
            )
            .is_null()
            {
                return 24;
            }
        }
        {
            for id in [SEARCH, REPLACEMENT] {
                let query = GetDlgItem(hwnd, id);
                if query.is_null() {
                    continue;
                }
                if !protect_editor_field(query) {
                    return 24;
                }
                SendMessageW(query, EM_SETLIMITTEXT, MAX_PARAGRAPH_UTF16, 0);
            }
        }
        if !protect_editor_field(GetDlgItem(hwnd, EDIT)) {
            return 24;
        }
        let limit = if preview {
            2 * kraqel_native_scene_model::MAX_PREVIEW_UTF16
        } else {
            MAX_PARAGRAPH_UTF16
        };
        SendDlgItemMessageW(hwnd, EDIT, EM_SETLIMITTEXT, limit, 0);
        if let Some(SceneSurface {
            text,
            second_bold: marked,
            ..
        }) = &surface
        {
            let edit = GetDlgItem(hwnd, EDIT);
            let mut visible = text.as_str();
            if marked.is_some() && !preview && !long {
                let Some((first, number)) = text.split_once('\n') else {
                    return 24;
                };
                visible = first;
                let second = CreateWindowExW(
                    0,
                    wide(if draft.is_some() { "EDIT" } else { "STATIC" }).as_ptr(),
                    wide(number).as_ptr(),
                    WS_CHILD
                        | WS_VISIBLE
                        | if draft.is_some() {
                            WS_BORDER | WS_TABSTOP | ES_AUTOHSCROLL as u32
                        } else {
                            SS_LEFT | windows_sys::Win32::System::SystemServices::SS_NOPREFIX
                        },
                    0,
                    0,
                    0,
                    0,
                    hwnd,
                    SECOND as _,
                    instance,
                    null(),
                );
                if second.is_null() {
                    return 24;
                }
                if draft.is_some() {
                    SendMessageW(second, EM_SETLIMITTEXT, MAX_PARAGRAPH_UTF16, 0);
                    if !protect_editor_field(second) {
                        return 24;
                    }
                }
            }
            if SendMessageW(
                edit,
                windows_sys::Win32::UI::Controls::EM_SETREADONLY,
                usize::from(draft.is_none()),
                0,
            ) == 0
                || SetWindowTextW(edit, wide(visible.replace('\n', "\r\n")).as_ptr()) == 0
            {
                return 24;
            }
        }
        if draft.is_some() {
            if !set_bold(hwnd, bold) {
                return 24;
            }
        }
        if !apply_fonts(hwnd, GetDpiForWindow(hwnd)) || !layout_surface(hwnd) {
            return 24;
        }
        if draft.is_none() {
            if let Some(revision) = LOADED_REVISION.get() {
                SetWindowTextW(
                    hwnd,
                    wide(format!("Szene A · Revision {revision} – Schreibgeschützt")).as_ptr(),
                );
            }
        }
        if long && !enter_plain_editor(hwnd, &surface.as_ref().unwrap().text) {
            return 24;
        }
        ShowWindow(hwnd, SW_SHOW);
        if let Some(output) = surface
            .as_mut()
            .filter(|_| draft.is_none())
            .and_then(|s| s.output.as_mut())
        {
            use std::io::Write;
            if output
                .write_all(&(hwnd as usize as u64).to_le_bytes())
                .is_err()
            {
                return 20;
            }
        }
        if let Some(endpoint) = endpoint {
            let exit = endpoint
                .window(
                    |text| {
                        let edit = GetDlgItem(hwnd, EDIT);
                        require(
                            SendMessageW(
                                edit,
                                windows_sys::Win32::UI::Controls::EM_SETREADONLY,
                                1,
                                0,
                            ) != 0,
                            "readonly",
                        )?;
                        let shown = wide(text.replace('\n', "\r\n"));
                        check(SetWindowTextW(edit, shown.as_ptr()))?;
                        let mut actual = [0u16; 64];
                        let n = GetWindowTextW(edit, actual.as_mut_ptr(), 64);
                        require(actual[..n as usize] == shown[..shown.len() - 1], "display")
                    },
                    |title| {
                        check(SetWindowTextW(hwnd, wide(title).as_ptr()))?;
                        let mut msg = MSG::default();
                        loop {
                            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                                -1 => return Err(std::io::Error::last_os_error()),
                                0 => {
                                    return require(
                                        msg.wParam == 40 && IsWindow(hwnd) == 0,
                                        "window close",
                                    );
                                }
                                _ => {
                                    TranslateMessage(&msg);
                                    DispatchMessageW(&msg);
                                }
                            }
                        }
                    },
                )
                .unwrap_or(64);
            if IsWindow(hwnd) != 0 {
                destroy_surface(hwnd);
            }
            return exit;
        }
        SetFocus(GetDlgItem(hwnd, EDIT));
        run_editor_loop(hwnd, surface.is_some(), draft, keep_editor)
    }
}

unsafe fn run_editor_loop(
    hwnd: HWND,
    has_surface: bool,
    mut draft: Option<&mut (String, bool)>,
    mut keep_editor: Option<&mut HeldEditor>,
) -> i32 {
    unsafe {
        let action = if draft.is_some() { SAVE } else { CLOSE };
        let _autosave = keep_editor
            .as_ref()
            .and_then(|held| held.autosave_turn)
            .map(|turn| {
                AUTOSAVE_TURN.set(Some(turn));
                update_draft_status(hwnd);
                AutosaveTurn(hwnd)
            });
        let mut history = keep_editor
            .as_deref_mut()
            .and_then(|held| held.history.take())
            .or_else(|| {
                draft
                    .as_ref()
                    .and_then(|_| draft_text(hwnd))
                    .map(DraftHistory::new)
            });
        let mut msg = MSG::default();
        loop {
            // TranslateMessage can enqueue a character behind WM_CLOSE. Account for
            // these edits against the confirmed basis before deciding whether to prompt.
            if CLOSE_REQUESTED.get() && PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE) == 0 {
                CLOSE_REQUESTED.set(false);
                finish_close(hwnd);
            }
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                -1 => return 25,
                0 => return msg.wParam as i32,
                _ => {}
            }
            if msg.message == MERGE_SELECTED
                && REVISION_CONFLICT.get()
                && matches!(msg.wParam, COMMIT_SELECTION | CONTINUE_SELECTION)
                && GetMenuState(GetMenu(hwnd), msg.wParam as u32, MF_BYCOMMAND)
                    & (MF_DISABLED | MF_GRAYED)
                    == 0
            {
                if let Some(held) = keep_editor.as_deref_mut() {
                    held.history = history.take();
                }
                return if msg.wParam == CONTINUE_SELECTION {
                    MERGE_CONTINUE_EXIT
                } else {
                    MERGE_EXIT
                };
            }
            let key = msg.wParam as u16;
            if LONG_EDIT.get() {
                let field = GetDlgItem(hwnd, EDIT);
                let save = GetDlgItem(hwnd, SAVE);
                let ctrl = msg.message == WM_KEYDOWN && GetKeyState(i32::from(VK_CONTROL)) < 0;
                let stay = GetDlgItem(hwnd, SAVE_AND_STAY);
                let enter = msg.message == WM_KEYDOWN && key == VK_RETURN;
                if msg.message == WM_KEYDOWN
                    && key == VK_F7
                    && LONG_EDIT.get()
                    && REVISION_CONFLICT.get()
                {
                    navigate_paragraph_difference(hwnd, GetKeyState(i32::from(VK_SHIFT)) < 0);
                    continue;
                }
                if msg.message == WM_KEYDOWN
                    && key == VK_F9
                    && PLAIN_REVIEW.get()
                    && REVISION_CONFLICT.get()
                    && msg.lParam & (1 << 30) == 0
                {
                    return MERGE_EXIT;
                }
                if msg.message == WM_KEYDOWN && key == VK_F8 && REVISION_CONFLICT.get() {
                    if GetFocus() == GetDlgItem(hwnd, COMPARISON)
                        && !GetFocus().is_null()
                        && msg.lParam & (1 << 30) == 0
                    {
                        take_service_paragraphs(hwnd);
                    }
                    continue;
                }
                let timer =
                    msg.message == WM_TIMER && msg.hwnd == hwnd && msg.wParam == AUTOSAVE_TIMER;
                if timer
                    && (CLOSE_REQUESTED.get()
                        || REVISION_CONFLICT.get()
                        || AWAITING_REBIND.get()
                        || IsWindowEnabled(hwnd) == 0
                        || IsWindowEnabled(stay) == 0
                        || !autosave_due(hwnd))
                {
                    continue;
                }
                let continuing = timer
                    || msg.message == CONTINUE_PLAIN_DRAFT
                    || (enter && (GetFocus() == stay || ctrl));
                if continuing
                    || msg.message == SAVE_DRAFT
                    || (ctrl && key == 0x53)
                    || (msg.message == WM_KEYDOWN && key == VK_RETURN && GetFocus() == save)
                {
                    if !CLOSE_REQUESTED.get()
                        && !REVISION_CONFLICT.get()
                        && IsWindowEnabled(save) != 0
                        && (!continuing
                            || (IsWindowEnabled(stay) != 0
                                && AUTOSAVE_TURN
                                    .get()
                                    .is_some_and(|(turn, limit)| turn < limit)))
                    {
                        if let (Some(value), Some(held), Some(result)) = (
                            draft_text(hwnd),
                            keep_editor.as_deref_mut(),
                            draft.as_deref_mut(),
                        ) {
                            if kraqel_native_scene_model::valid_scene_edit(&value, false)
                                && draft_changed(hwnd)
                            {
                                *result = (value, false);
                                held.submitted = Some(result.clone());
                                held.hwnd = hwnd;
                                held.closing = !continuing;
                                AWAITING_REBIND.set(continuing);
                                if continuing {
                                    SetFocus(field);
                                    SetWindowTextW(
                                        hwnd,
                                        wide("Szene A · Klartext · Bestätigung ausstehend")
                                            .as_ptr(),
                                    );
                                } else {
                                    EnableWindow(hwnd, 0);
                                }

                                return 40;
                            }
                        }
                    }
                } else if msg.message == WM_KEYDOWN
                    && key == VK_F6
                    && !GetDlgItem(hwnd, COMPARISON).is_null()
                {
                    let comparison = GetDlgItem(hwnd, COMPARISON);
                    SetFocus(if GetFocus() == comparison {
                        field
                    } else {
                        comparison
                    });
                } else if msg.message == WM_KEYDOWN && key == VK_TAB {
                    let order = [field, GetDlgItem(hwnd, COMPARISON), stay, save];
                    let index = order
                        .iter()
                        .position(|&item| item == GetFocus())
                        .unwrap_or(0);
                    let step = if GetKeyState(i32::from(VK_SHIFT)) < 0 {
                        order.len() - 1
                    } else {
                        1
                    };
                    for n in 1..=order.len() {
                        let next = order[(index + n * step) % order.len()];
                        if !next.is_null() && IsWindowEnabled(next) != 0 {
                            SetFocus(next);
                            break;
                        }
                    }
                } else if msg.message == WM_KEYDOWN && key == VK_ESCAPE {
                    PostMessageW(hwnd, WM_CLOSE, 0, 0);
                } else {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                continue;
            }
            let search = GetDlgItem(hwnd, SEARCH);
            let ctrl = msg.message == WM_KEYDOWN
                && GetKeyState(i32::from(VK_CONTROL)) < 0
                && GetKeyState(i32::from(VK_MENU)) >= 0;
            let in_search = GetFocus() == search;
            if !search.is_null() && GetFocus() != GetDlgItem(hwnd, COMPARISON) {
                if (ctrl && key == 0x46) || (msg.message == WM_CHAR && key == 6) {
                    open_search(hwnd);
                    continue;
                }
                if msg.message == WM_KEYDOWN && (key == VK_F3 || (in_search && key == VK_RETURN)) {
                    find_in_scene(hwnd, GetKeyState(i32::from(VK_SHIFT)) < 0);
                    continue;
                }
            }
            if let Some(result) = draft.as_deref_mut() {
                let comparison = GetDlgItem(hwnd, COMPARISON);
                if !comparison.is_null() && msg.message == WM_KEYDOWN && key == VK_F6 {
                    SetFocus(if GetFocus() == comparison {
                        GetDlgItem(hwnd, ACTIVE_PARAGRAPH.get())
                    } else {
                        comparison
                    });
                    continue;
                }
                if !comparison.is_null()
                    && GetFocus() == comparison
                    && matches!(msg.message, WM_KEYDOWN | WM_CHAR)
                    && key != VK_TAB
                    && !(msg.message == WM_KEYDOWN && key == VK_ESCAPE)
                {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    continue;
                }
                let Some(current_history) = history.as_mut() else {
                    return 24;
                };
                if let Some(text) = draft_text(hwnd) {
                    current_history.record(text);
                }
                enable_menu_command(hwnd, UNDO, current_history.current > 0);
                enable_menu_command(
                    hwnd,
                    REDO,
                    current_history.current + 1 < current_history.states.len(),
                );
                let in_replacement = GetFocus() == GetDlgItem(hwnd, REPLACEMENT);
                if msg.message == REPLACE_DRAFT
                    || (in_replacement && msg.message == WM_KEYDOWN && key == VK_RETURN)
                {
                    if !replace_in_scene(
                        hwnd,
                        msg.message == REPLACE_DRAFT && msg.wParam == REPLACE_ALL,
                    ) {
                        return 24;
                    }
                    continue;
                }
                if msg.message == WM_KEYDOWN
                    && (key == VK_F11 || (key == VK_ESCAPE && in_focus_mode(hwnd)))
                {
                    set_focus_mode(hwnd, key == VK_F11 && !in_focus_mode(hwnd));
                    continue;
                }
                if ctrl && key == 0x48 {
                    set_focus_mode(hwnd, false);
                    SetFocus(GetDlgItem(hwnd, REPLACEMENT));
                    SendDlgItemMessageW(hwnd, REPLACEMENT, EM_SETSEL, 0, -1);
                    continue;
                }
                if msg.message == TOGGLE_BOLD
                    || (ctrl && key == 0x42)
                    || (msg.message == WM_CHAR && key == 2)
                {
                    if !set_bold(hwnd, !is_bold()) {
                        return 24;
                    }
                    continue;
                }
                if (ctrl && key == 0x41) || (msg.message == WM_CHAR && key == 1) {
                    if in_search || in_replacement {
                        SendMessageW(GetFocus(), EM_SETSEL, 0, -1);
                    } else {
                        select_paragraph(hwnd);
                    }
                    continue;
                }
                let undo = msg.message == HISTORY_UNDO
                    || (ctrl && key == 0x5a)
                    || (msg.message == WM_CHAR && key == 26);
                let redo = msg.message == HISTORY_REDO
                    || (ctrl && key == 0x59)
                    || (msg.message == WM_CHAR && key == 25);
                if (in_search || in_replacement)
                    && (undo || redo)
                    && !matches!(msg.message, HISTORY_UNDO | HISTORY_REDO)
                {
                    if undo {
                        SendMessageW(GetFocus(), WM_UNDO, 0, 0);
                    }
                    continue;
                }
                if undo || redo {
                    let forward = redo || (ctrl && GetKeyState(i32::from(VK_SHIFT)) < 0);
                    if let Some((text, bold)) = current_history.step(forward) {
                        if !set_bold(hwnd, *bold) || !restore_draft(hwnd, text) {
                            return 24;
                        }
                    }
                    continue;
                }
                if msg.message == DISCARD_DRAFT
                    || (msg.message == WM_KEYDOWN
                        && key == VK_RETURN
                        && GetFocus() == GetDlgItem(hwnd, DISCARD))
                {
                    if REVISION_CONFLICT.get() {
                        PostMessageW(hwnd, WM_CLOSE, 0, 0);
                        continue;
                    }
                    let initial = EXPECTED.with_borrow(Clone::clone);
                    let Some((first, second)) = initial.split_once('\n') else {
                        return 24;
                    };
                    if !set_bold(hwnd, INITIAL_BOLD.get()) {
                        return 24;
                    }
                    for (id, text) in [(EDIT, first), (SECOND, second), (LABEL, draft_label())] {
                        if SetWindowTextW(GetDlgItem(hwnd, id), wide(text).as_ptr()) == 0 {
                            return 24;
                        }
                    }
                    *current_history = DraftHistory::new(initial.clone());
                    update_draft_status(hwnd);
                    SetFocus(GetDlgItem(hwnd, EDIT));
                    continue;
                }
                let ctrl_save = msg.message == WM_KEYDOWN
                    && key == 0x53
                    && GetKeyState(i32::from(VK_CONTROL)) < 0;
                let enter_save = msg.message == WM_KEYDOWN
                    && key == VK_RETURN
                    && GetFocus() == GetDlgItem(hwnd, action);
                let timer =
                    msg.message == WM_TIMER && msg.hwnd == hwnd && msg.wParam == AUTOSAVE_TIMER;
                if timer
                    && (AUTOSAVE_TURN.get().is_none()
                        || IsWindowEnabled(hwnd) == 0
                        || !autosave_due(hwnd))
                {
                    continue;
                }
                if timer || msg.message == SAVE_DRAFT || ctrl_save || enter_save {
                    if CLOSE_REQUESTED.get() || REVISION_CONFLICT.get() {
                        cancel_pending_save(hwnd);
                        continue;
                    }
                    if let Some(value) = draft_text(hwnd) {
                        if is_bold() == INITIAL_BOLD.get()
                            && EXPECTED.with_borrow(|initial| initial == &value)
                        {
                            continue;
                        }
                        if kraqel_native_scene_model::valid_paragraphs(&value) {
                            *result = (value, is_bold());
                            if let Some(held) = keep_editor.as_deref_mut() {
                                held.hwnd = hwnd;
                                held.closing = msg.message == SAVE_DRAFT && msg.wParam == 1;
                                held.history = history.take();
                                held.submitted = Some(result.clone());
                                // Keep keyboard focus and the thread input queue across publication.
                                // Dispatch resumes only after the next authenticated read.
                                let continuing = !held.closing
                                    && held.autosave_turn.is_some_and(|(turn, limit)| turn < limit);
                                AWAITING_REBIND.set(continuing);
                                if !continuing {
                                    EnableWindow(hwnd, 0);
                                }
                                SetWindowTextW(
                                    hwnd,
                                    wide(
                                        "Szene A – Speichert · Bestätigung des Starters ausstehend",
                                    )
                                    .as_ptr(),
                                );
                            } else {
                                destroy_surface(hwnd);
                            }
                            return 40;
                        }
                    }
                    if let Some((id, message)) = invalid_paragraph(hwnd) {
                        SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
                        SetFocus(GetDlgItem(hwnd, id));
                    }
                    continue;
                }
                if msg.message == WM_KEYDOWN
                    && key == VK_RETURN
                    && GetFocus() != GetDlgItem(hwnd, action)
                {
                    continue;
                }
            }
            // Multiline EDIT controls can claim Tab. Use the visible, enabled controls
            // in reading order, including the comparison pane when a conflict is open.
            if msg.message == WM_KEYDOWN
                && msg.wParam == usize::from(VK_TAB)
                && GetKeyState(i32::from(VK_CONTROL)) >= 0
                && GetKeyState(i32::from(VK_MENU)) >= 0
            {
                let order: &[i32] = if draft.is_some() {
                    &[
                        EDIT,
                        SECOND,
                        COMPARISON,
                        action,
                        DISCARD,
                        SEARCH,
                        REPLACEMENT,
                    ]
                } else {
                    &[EDIT, SEARCH, CLOSE]
                };
                let backwards = GetKeyState(i32::from(VK_SHIFT)) < 0;
                let focus = GetFocus();
                let mut index = order
                    .iter()
                    .position(|&id| !focus.is_null() && GetDlgItem(hwnd, id) == focus)
                    .unwrap_or(if backwards { 0 } else { order.len() - 1 });
                let step = if backwards { order.len() - 1 } else { 1 };
                for _ in order {
                    index = (index + step) % order.len();
                    let target = GetDlgItem(hwnd, order[index]);
                    if !target.is_null()
                        && IsWindowVisible(target) != 0
                        && IsWindowEnabled(target) != 0
                    {
                        SetFocus(target);
                        break;
                    }
                }
                continue;
            }
            if has_surface
                && msg.message == WM_KEYDOWN
                && (msg.wParam == usize::from(VK_ESCAPE)
                    || (msg.wParam == usize::from(VK_RETURN)
                        && GetFocus() == GetDlgItem(hwnd, action)))
            {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
                continue;
            }
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
#[used]
#[unsafe(link_section = ".kqmerge")]
static MERGE_ABI: [u8; 32] = kraqel_window_probe::window_receipt::MERGE_ABI;
#[used]
#[unsafe(link_section = ".kqfresh")]
static REFRESH_ABI: [u8; 32] = kraqel_window_probe::window_receipt::REFRESH_ABI;
#[used]
#[unsafe(link_section = ".kqgap")]
static GAP_ABI: [u8; 32] = kraqel_window_probe::window_receipt::GAP_ABI;
#[used]
#[unsafe(link_section = ".kqagain")]
static REPEAT_ABI: [u8; 32] = kraqel_window_probe::window_receipt::REPEAT_ABI;
fn main() {
    std::hint::black_box(&REPEAT_ABI);
    std::hint::black_box(&GAP_ABI);
    std::hint::black_box(&REFRESH_ABI);
    std::hint::black_box(&MERGE_ABI);
    std::hint::black_box(&COMPARE_ABI);
    let mut arg = std::env::args_os().nth(1);
    let repeat_arg = arg
        .as_deref()
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_suffix("-repeat"))
        .map(std::ffi::OsString::from);
    let repeat = repeat_arg.is_some();
    if repeat {
        arg = repeat_arg;
    }
    let gap_arg = arg
        .as_deref()
        .and_then(|a| a.to_str())
        .and_then(|a| a.strip_suffix("-gap"))
        .map(std::ffi::OsString::from);
    let gap = gap_arg.is_some();
    if gap {
        arg = gap_arg;
    }

    let refreshed = arg
        .as_deref()
        .and_then(|v| v.to_str())
        .and_then(|v| v.strip_suffix("-refresh"))
        .map(std::ffi::OsString::from);
    let refresh = refreshed.is_some();
    if refresh {
        arg = refreshed;
    }
    let merged = arg
        .as_deref()
        .and_then(|v| v.to_str())
        .and_then(|v| v.strip_suffix("-merge"))
        .map(std::ffi::OsString::from);
    let merge = merged.is_some();
    if merge {
        arg = merged;
    }
    let base = arg
        .as_ref()
        .and_then(|v| v.to_str())
        .and_then(|v| v.strip_suffix("-compare"))
        .map(std::ffi::OsString::from);
    let compare = base.is_some();
    if compare {
        arg = base;
    }
    use window_receipt::SceneOperation;
    let checkpoint = [SceneOperation::KeyboardEdit, SceneOperation::Revise]
        .into_iter()
        .find(|mode| {
            arg.as_deref()
                == Some(std::ffi::OsStr::new(window_receipt::checkpoint_argument(
                    *mode,
                )))
        });
    let continued = [SceneOperation::KeyboardEdit, SceneOperation::Revise]
        .into_iter()
        .find(|mode| {
            arg.as_deref()
                == Some(std::ffi::OsStr::new(window_receipt::continue_argument(
                    *mode,
                )))
        });
    std::hint::black_box(&AUTOSAVE_ABI);
    let budgeted = [SceneOperation::KeyboardEdit, SceneOperation::Revise]
        .into_iter()
        .find(|mode| {
            arg.as_deref()
                == Some(std::ffi::OsStr::new(window_receipt::budgeted_argument(
                    *mode,
                )))
        });
    let continued = continued.or(budgeted);
    let checkpoint = checkpoint.or(continued);
    let scene = checkpoint.or_else(|| SceneOperation::from_argument(arg.as_deref()));
    if scene.is_some() || arg.as_deref() == Some(std::ffi::OsStr::new(window_receipt::UI_ARG)) {
        let result = (|| -> std::io::Result<i32> {
            let profile = std::env::args()
                .nth(2)
                .ok_or(std::io::Error::other("missing profile"))?;
            let (input, mut output) =
                managed_scene::confirm_bootstrap(Some(&profile), bootstrap::Role::Window)?;
            let (text, marked, revision, text_preview) = if let Some(mode) = scene {
                let mut held = HeldEditor::default();
                let mut expected_next = None;
                let mut turn_mode = mode;
                let mut control = input;
                let limit = if budgeted.is_some() {
                    use std::io::Read;
                    let mut budget = [0u8];
                    control.read_exact(&mut budget)?;
                    require(budget[0] > 0, "scene autosave budget")?;
                    budget[0]
                } else if continued.is_some() {
                    2
                } else {
                    1
                };
                use kraqel_native_scene_model::channel::ReviewedWrite;
                let reviewed_write = if gap && repeat {
                    ReviewedWrite::Repeat
                } else if gap {
                    ReviewedWrite::Once
                } else {
                    ReviewedWrite::Unavailable
                };
                let mut turn = 1u8;
                let mut selection = None;
                loop {
                    let remaining = window_receipt::SCENE_REVISION_LIMIT
                        .saturating_sub(expected_next.unwrap_or(0));
                    let display_limit = limit.min(turn.saturating_add(
                        u8::try_from(remaining.saturating_sub(1)).unwrap_or(u8::MAX),
                    ));
                    held.autosave_turn = continued.map(|_| (turn, display_limit));
                    let mut dialog_output = Some(output.try_clone()?);
                    let view = managed_scene::exchange(
                        control.try_clone()?,
                        turn_mode,
                        &mut output,
                        compare,
                        refresh,
                        reviewed_write,
                        |event| {
                            use kraqel_native_scene_model::channel::EditorEvent;
                            let loaded = match event {
                                EditorEvent::Loaded(view) => view,
                                EditorEvent::Preview(view) => {
                                    require(held.hwnd.is_null(), "preview requires fresh session")?;
                                    LOADED_REVISION.set(Some(view.revision));
                                    let code = show(
                                        None,
                                        Some(SceneSurface {
                                            output: None,
                                            text: view.preview_text.clone(),
                                            second_bold: Some(false),
                                            text_preview: true,
                                        }),
                                        None,
                                        None,
                                    );
                                    require(code == 40, "preview close")?;
                                    return Ok(None);
                                }
                                EditorEvent::PlainConflict(view) => {
                                    let allowed = LONG_EDIT.get()
                                        && merge
                                        && turn < limit
                                        && view.revision
                                            == LOADED_REVISION.get().unwrap_or(0).saturating_add(1)
                                        && view.revision < window_receipt::SCENE_REVISION_LIMIT;
                                    selection = inspect_conflict(
                                        &mut held,
                                        Some((view.revision, &view.text)),
                                        allowed,
                                    )?;
                                    return Ok(selection.clone());
                                }
                                EditorEvent::RefreshedComparison(view) if LONG_EDIT.get() => {
                                    inspect_conflict(
                                        &mut held,
                                        Some((view.revision, &view.text)),
                                        false,
                                    )?;
                                    return Ok(None);
                                }
                                EditorEvent::RefreshedComparison(view) => {
                                    selection = resolve_conflict(
                                        &mut held,
                                        Some(view),
                                        gap && turn < limit
                                            && view.revision < window_receipt::SCENE_REVISION_LIMIT,
                                        limit - turn >= 2,
                                        true,
                                    )?;
                                    return Ok(selection.clone());
                                }
                                EditorEvent::TextConflict(view) => {
                                    inspect_conflict(
                                        &mut held,
                                        Some((view.revision, view.preview_text.as_str())),
                                        false,
                                    )?;
                                    return Ok(None);
                                }
                                EditorEvent::RevisionConflict(view) if LONG_EDIT.get() => {
                                    inspect_conflict(
                                        &mut held,
                                        view.map(|v| (v.revision, v.text.as_str())),
                                        false,
                                    )?;
                                    return Ok(None);
                                }
                                EditorEvent::RevisionConflict(comparison) => {
                                    selection = resolve_conflict(
                                        &mut held,
                                        comparison,
                                        merge && turn < limit,
                                        limit - turn >= 2,
                                        false,
                                    )?;
                                    return Ok(selection.clone());
                                }
                            };
                            LOADED_REVISION.set(Some(loaded.revision));
                            let mut draft = (String::new(), loaded.bold_second);
                            let code =
                                if held.hwnd.is_null() {
                                    show(
                                        None,
                                        Some(SceneSurface {
                                            output: Some(dialog_output.take().ok_or(
                                                std::io::Error::other("duplicate editor"),
                                            )?),
                                            text: loaded.text.clone(),
                                            second_bold: Some(loaded.bold_second),
                                            text_preview: false,
                                        }),
                                        Some(&mut draft),
                                        Some(&mut held),
                                    )
                                } else {
                                    require(
                                        expected_next == Some(loaded.revision)
                                            && held.submitted.take()
                                                == Some((loaded.text.clone(), loaded.bold_second)),
                                        "scene continuation basis",
                                    )?;
                                    if let Some(chosen) = selection.take() {
                                        held.submitted = Some(chosen.clone());
                                        held.closing = !held.resume_after_merge;
                                        return Ok(Some(chosen));
                                    }
                                    EXPECTED.with_borrow_mut(|v| v.clone_from(&loaded.text));
                                    EDIT_BASE_REVISION.set(Some(loaded.revision));
                                    INITIAL_BOLD.set(loaded.bold_second);
                                    if held.resume_after_merge {
                                        resume_merged_editor(&mut held, loaded)?;
                                    }
                                    unsafe {
                                        AWAITING_REBIND.set(false);
                                        if LONG_EDIT.get() {
                                            SetWindowTextW(
                                                held.hwnd,
                                                wide(format!(
                                                    "Szene A · Klartext · Revision {} gespeichert",
                                                    loaded.revision
                                                ))
                                                .as_ptr(),
                                            );
                                        }
                                        update_draft_status(held.hwnd);
                                        run_editor_loop(
                                            held.hwnd,
                                            true,
                                            Some(&mut draft),
                                            Some(&mut held),
                                        )
                                    }
                                };
                            match code {
                                40 if LONG_EDIT.get() || !draft.0.is_empty() => Ok(Some(draft)),
                                code if code == window_receipt::DISCARDED_EXIT as i32 => Ok(None),
                                _ => Err(std::io::Error::other("editor ended unexpectedly")),
                            }
                        },
                    )?;
                    let Some(view) = view else {
                        return Ok(window_receipt::DISCARDED_EXIT as i32);
                    };
                    if checkpoint.is_some() {
                        use std::io::{Read, Write};
                        let (text, bold, revision, _) = view;
                        require(
                            !held.hwnd.is_null()
                                && held.submitted.as_ref().is_some_and(|(submitted, marked)| {
                                    submitted == &text && *marked == bold
                                }),
                            "scene publication view",
                        )?;
                        output.write_all(&(held.hwnd as usize as u64).to_le_bytes())?;
                        let mut frame = [0u8; 36];
                        control.read_exact(&mut frame)?;
                        require(
                            Some(u32::from_le_bytes(frame[..4].try_into().unwrap())) == revision
                                && frame[4..] != [0u8; 32],
                            "scene publication revision",
                        )?;
                        unsafe {
                            check(SetWindowTextW(
                                held.hwnd,
                                wide(format!(
                                    "Szene A · Revision {} – Vom Starter bestätigt",
                                    u32::from_le_bytes(frame[..4].try_into().unwrap())
                                ))
                                .as_ptr(),
                            ))?;
                        }
                        if turn < limit
                            && !held.closing
                            && revision != Some(window_receipt::SCENE_REVISION_LIMIT)
                        {
                            output.write_all(&window_receipt::continue_receipt(&frame))?;
                            turn += 1;
                            expected_next = revision;
                            turn_mode = SceneOperation::Revise;
                            continue;
                        }
                        output.write_all(&window_receipt::publication_receipt(&frame))?;
                        if turn < limit {
                            let mut end = [1u8];
                            control.read_exact(&mut end)?;
                            require(end == [0], "scene close confirmation")?;
                        }
                        destroy_surface(held.hwnd);
                        return Ok(40);
                    }
                    if !held.hwnd.is_null() {
                        destroy_surface(held.hwnd);
                    }
                    break view;
                }
            } else {
                ("Rabe\n42".to_owned(), false, None, false)
            };
            LOADED_REVISION.set(revision);
            let split = marked
                || matches!(
                    scene,
                    Some(
                        window_receipt::SceneOperation::ReadPersistent
                            | window_receipt::SceneOperation::ReadRevision
                            | window_receipt::SceneOperation::Revise
                            | window_receipt::SceneOperation::KeyboardEdit
                            | window_receipt::SceneOperation::SavePersistent
                    )
                );
            Ok(show(
                None,
                Some(SceneSurface {
                    output: Some(output),
                    text,
                    second_bold: split.then_some(marked),
                    text_preview,
                }),
                None,
                None,
            ))
        })();
        std::process::exit(result.unwrap_or(20));
    }
    if arg.as_deref() == Some(std::ffi::OsStr::new(window_receipt::ARG)) {
        std::process::exit(
            if managed_scene::confirm_bootstrap(None, bootstrap::Role::Window).is_ok() {
                0
            } else {
                20
            },
        );
    }
    if bootstrap::Role::Window.receipt().exists() {
        let exit = bootstrap::Endpoint::open(bootstrap::Role::Window)
            .map(|endpoint| show(Some(endpoint), None, None, None))
            .unwrap_or(20);
        std::process::exit(exit);
    }
    if verify_leaf().is_err() {
        std::process::exit(20);
    }
    std::process::exit(show(None, None, None, None));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipboard_shortcuts_and_direct_commands_are_consumed_but_typing_is_allowed() {
        for msg in [WM_COPY, WM_CUT, WM_PASTE, WM_CONTEXTMENU] {
            assert!(clipboard_command(msg, 0, false, false));
        }
        for (letter, ch) in [(0x43, 3), (0x56, 22), (0x58, 24)] {
            assert!(clipboard_command(WM_KEYDOWN, letter, true, false));
            assert!(clipboard_command(WM_CHAR, ch, false, false));
            assert!(!clipboard_command(WM_CHAR, letter, false, false));
        }
        assert!(clipboard_command(WM_KEYDOWN, VK_INSERT.into(), true, false));
        assert!(clipboard_command(WM_KEYDOWN, VK_INSERT.into(), false, true));
        assert!(clipboard_command(WM_KEYDOWN, VK_DELETE.into(), false, true));
        assert!(!clipboard_command(
            WM_KEYDOWN,
            VK_DELETE.into(),
            false,
            false
        ));
        assert!(!clipboard_command(WM_KEYDOWN, 0x5a, true, false));
    }
}
