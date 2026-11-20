#![windows_subsystem = "windows"]
use kraqel_native_scene_model::MAX_PARAGRAPH_UTF16;
use kraqel_window_probe::*;
use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use windows_sys::Win32::{
    System::SystemServices::SS_LEFT,
    UI::Controls::{EM_GETSEL, EM_REPLACESEL, EM_SETLIMITTEXT, EM_SETSEL},
};

const LABEL: i32 = 100;
const EDIT: i32 = 101;
const CLOSE: i32 = 102;
const SECOND: i32 = 103;
const SAVE: i32 = 104;
const DISCARD: i32 = 105;
const WORD_COUNT: i32 = 106;

thread_local! {
    static SECOND_WEIGHT: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    static EXPECTED: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}
const SAVE_DRAFT: u32 = WM_APP + 1;
const DISCARD_DRAFT: u32 = WM_APP + 2;
const UNDO: usize = 201;
const REDO: usize = 202;
const SELECT_PARAGRAPH: usize = 203;
const HISTORY_UNDO: u32 = WM_APP + 3;
const HISTORY_REDO: u32 = WM_APP + 4;

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
                    (UNDO, "&Rückgängig\tStrg+Z"),
                    (REDO, "&Wiederholen\tStrg+Y"),
                    (SELECT_PARAGRAPH, "&Absatz auswählen\tStrg+A"),
                ][..],
            ),
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
        let mut buf = [0u16; MAX_PARAGRAPH_UTF16 + 1];
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
            if length.saturating_sub(selected) + 2 <= MAX_PARAGRAPH_UTF16 {
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

fn draft_text(hwnd: HWND) -> Option<String> {
    Some(format!(
        "{}\n{}",
        paragraph_text(hwnd, EDIT)?,
        paragraph_text(hwnd, SECOND)?
    ))
}

fn draft_label() -> &'static str {
    if SECOND_WEIGHT.get() == FW_BOLD as i32 {
        "Szene A · Zwei Absätze, zweiter fett"
    } else {
        "Szene A · Zwei Absätze"
    }
}

/// Bounded to this window's message loop. A new edit removes the redo branch.
struct DraftHistory {
    states: std::collections::VecDeque<String>,
    current: usize,
}

impl DraftHistory {
    fn new(text: String) -> Self {
        Self {
            states: [text].into(),
            current: 0,
        }
    }

    fn record(&mut self, text: String) {
        if self.states[self.current] == text {
            return;
        }
        self.states.truncate(self.current + 1);
        self.states.push_back(text);
        if self.states.len() > 65 {
            self.states.pop_front();
        }
        self.current = self.states.len() - 1;
    }

    fn step(&mut self, redo: bool) -> Option<&str> {
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
    EXPECTED.with_borrow(|initial| current.as_ref() != Some(initial))
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
        for id in [EDIT, LABEL, SAVE, DISCARD, CLOSE, WORD_COUNT] {
            SendDlgItemMessageW(hwnd, id, WM_SETFONT, normal as usize, 1);
        }
        if !old.is_null() {
            DeleteObject(old);
        }
        true
    }
}

fn update_draft_status(hwnd: HWND) {
    unsafe {
        let save = GetDlgItem(hwnd, SAVE);
        if save.is_null() || GetDlgItem(hwnd, SECOND).is_null() {
            return;
        }
        let current = draft_text(hwnd);
        let dirty = EXPECTED.with_borrow(|initial| current.as_ref() != Some(initial));
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
        enable_menu_command(hwnd, SAVE as usize, dirty);
        EnableWindow(save, i32::from(dirty));
        EnableWindow(GetDlgItem(hwnd, DISCARD), i32::from(dirty));
        let title = if dirty {
            "Szene A – Ungespeicherte Änderungen"
        } else {
            "Szene A – Keine Änderungen"
        };
        SetWindowTextW(hwnd, wide(title).as_ptr());
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
        let count_y = area.bottom - px(30);
        let buttons_y = if editing {
            count_y - px(36)
        } else {
            area.bottom - px(44)
        };
        let text_top = px(38);
        let text_height = (buttons_y - gap - text_top).max(1);
        let place = |id, x, y, w, h| MoveWindow(GetDlgItem(hwnd, id), x, y, w, h, 1) != 0;
        if !place(LABEL, margin, px(10), width, px(24)) {
            return false;
        }
        if GetDlgItem(hwnd, SECOND).is_null() {
            if !place(EDIT, margin, text_top, width, text_height) {
                return false;
            }
        } else {
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
        } else {
            place(CLOSE, margin, buttons_y, width, px(30))
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
                bounds.ptMinTrackSize.y = bounds
                    .ptMinTrackSize
                    .y
                    .max(scale(250, GetDpiForWindow(hwnd)));
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
            WM_COMMAND if w == SELECT_PARAGRAPH => {
                select_paragraph(hwnd);
                0
            }
            WM_COMMAND if w & 0xffff == DISCARD as usize => {
                PostMessageW(hwnd, DISCARD_DRAFT, 0, 0);
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
            WM_CLOSE => {
                if !GetDlgItem(hwnd, SAVE).is_null() && draft_changed(hwnd) {
                    if IsWindowEnabled(hwnd) == 0 {
                        return 0;
                    }
                    let previous_focus = GetFocus();
                    let answer = MessageBoxW(
                        hwnd,
                        wide("Änderungen an Szene A speichern?\n\nJa: Speichern.\nNein: Änderungen verwerfen.\nAbbrechen: Weiterarbeiten.").as_ptr(),
                        wide("Ungespeicherte Änderungen").as_ptr(),
                        MB_YESNOCANCEL | MB_DEFBUTTON3 | MB_ICONWARNING,
                    );
                    if answer == IDYES {
                        PostMessageW(hwnd, SAVE_DRAFT, 0, 0);
                        return 0;
                    }
                    if answer != IDNO {
                        if IsChild(hwnd, previous_focus) != 0 {
                            SetFocus(previous_focus);
                        }
                        return 0;
                    }
                }
                let mut value = [0u16; 64];
                let n = GetDlgItemTextW(hwnd, EDIT, value.as_mut_ptr(), 64);
                let expected = EXPECTED.with_borrow(Clone::clone);
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
                    let n = GetWindowTextW(second, value.as_mut_ptr(), 64);
                    let number = String::from_utf16_lossy(&value[..n as usize]);
                    format!("{first}\n{number}") == expected
                        && font_description(font)
                            .is_some_and(|desc| desc.lfWeight == SECOND_WEIGHT.get())
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
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
fn show(
    endpoint: Option<bootstrap::Endpoint>,
    mut surface: Option<(std::fs::File, String, Option<bool>)>,
    mut draft: Option<&mut String>,
) -> i32 {
    unsafe {
        if SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_null() {
            return 24;
        }
        let bold = surface.as_ref().and_then(|(_, _, style)| *style) == Some(true);
        SECOND_WEIGHT.set(if bold { FW_BOLD } else { FW_NORMAL } as i32);
        EXPECTED.with_borrow_mut(|value| {
            *value = surface
                .as_ref()
                .map(|(_, text, _)| text.clone())
                .unwrap_or_else(|| "Rabe\n42".into())
        });
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
            scale(280, GetDpiForSystem()),
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
                scale(280, dpi),
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
                } else {
                    "&Prüftext"
                },
                SS_LEFT,
            ),
            (
                EDIT,
                "EDIT",
                "",
                WS_TABSTOP | WS_BORDER | ES_MULTILINE as u32 | ES_WANTRETURN as u32,
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
        ] {
            if id == WORD_COUNT && draft.is_none() {
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
        if !protect_editor_field(GetDlgItem(hwnd, EDIT)) {
            return 24;
        }
        SendDlgItemMessageW(hwnd, EDIT, EM_SETLIMITTEXT, MAX_PARAGRAPH_UTF16, 0);
        if let Some((_, text, marked)) = &surface {
            let edit = GetDlgItem(hwnd, EDIT);
            let mut visible = text.as_str();
            if marked.is_some() {
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
            update_draft_status(hwnd);
        }
        if !apply_fonts(hwnd, GetDpiForWindow(hwnd)) || !layout_surface(hwnd) {
            return 24;
        }
        ShowWindow(hwnd, SW_SHOW);
        if let Some((output, _, _)) = surface.as_mut().filter(|_| draft.is_none()) {
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
        let mut history = draft
            .as_ref()
            .and_then(|_| draft_text(hwnd))
            .map(DraftHistory::new);
        let mut msg = MSG::default();
        loop {
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                -1 => return 25,
                0 => return msg.wParam as i32,
                _ => {}
            }
            if let Some(result) = draft.as_deref_mut() {
                let key = msg.wParam as u16;
                let Some(history) = history.as_mut() else {
                    return 24;
                };
                if let Some(text) = draft_text(hwnd) {
                    history.record(text);
                }
                enable_menu_command(hwnd, UNDO, history.current > 0);
                enable_menu_command(hwnd, REDO, history.current + 1 < history.states.len());
                let ctrl = msg.message == WM_KEYDOWN
                    && GetKeyState(i32::from(VK_CONTROL)) < 0
                    && GetKeyState(i32::from(VK_MENU)) >= 0;
                if (ctrl && key == 0x41) || (msg.message == WM_CHAR && key == 1) {
                    select_paragraph(hwnd);
                    continue;
                }
                let undo = msg.message == HISTORY_UNDO
                    || (ctrl && key == 0x5a)
                    || (msg.message == WM_CHAR && key == 26);
                let redo = msg.message == HISTORY_REDO
                    || (ctrl && key == 0x59)
                    || (msg.message == WM_CHAR && key == 25);
                if undo || redo {
                    let forward = redo || (ctrl && GetKeyState(i32::from(VK_SHIFT)) < 0);
                    if let Some(text) = history.step(forward) {
                        if !restore_draft(hwnd, text) {
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
                    let Some((_, initial, _)) = surface.as_ref() else {
                        return 24;
                    };
                    let Some((first, second)) = initial.split_once('\n') else {
                        return 24;
                    };
                    for (id, text) in [(EDIT, first), (SECOND, second), (LABEL, draft_label())] {
                        if SetWindowTextW(GetDlgItem(hwnd, id), wide(text).as_ptr()) == 0 {
                            return 24;
                        }
                    }
                    *history = DraftHistory::new(initial.clone());
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
                if msg.message == SAVE_DRAFT || ctrl_save || enter_save {
                    if let Some(value) = draft_text(hwnd) {
                        if EXPECTED.with_borrow(|initial| initial == &value) {
                            continue;
                        }
                        if kraqel_native_scene_model::valid_paragraphs(&value) {
                            *result = value;
                            destroy_surface(hwnd);
                            return 40;
                        }
                    }
                    for (id, message) in [
                        (
                            EDIT,
                            "Ersten Absatz einzeilig ausfüllen (max. 48 UTF-16-Einheiten).",
                        ),
                        (
                            SECOND,
                            "Zweiten Absatz einzeilig ausfüllen (max. 48 UTF-16-Einheiten).",
                        ),
                    ] {
                        if !paragraph_text(hwnd, id)
                            .is_some_and(|p| kraqel_native_scene_model::valid_paragraph(&p))
                        {
                            SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
                            SetFocus(GetDlgItem(hwnd, id));
                            break;
                        }
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
            // A multiline EDIT can claim Tab. This two-control form reserves it for navigation.
            if msg.message == WM_KEYDOWN
                && msg.wParam == usize::from(VK_TAB)
                && GetKeyState(i32::from(VK_CONTROL)) >= 0
                && GetKeyState(i32::from(VK_MENU)) >= 0
            {
                let next = if draft.is_some() {
                    let ids: &[i32] = if IsWindowEnabled(GetDlgItem(hwnd, action)) != 0 {
                        &[EDIT, SECOND, action, DISCARD]
                    } else {
                        &[EDIT, SECOND]
                    };
                    let index = ids
                        .iter()
                        .position(|id| GetDlgItem(hwnd, *id) == GetFocus())
                        .unwrap_or(0);
                    ids[(index
                        + if GetKeyState(i32::from(VK_SHIFT)) < 0 {
                            ids.len() - 1
                        } else {
                            1
                        })
                        % ids.len()]
                } else if GetFocus() == GetDlgItem(hwnd, EDIT) {
                    CLOSE
                } else {
                    EDIT
                };
                SetFocus(GetDlgItem(hwnd, next));
                continue;
            }
            if surface.is_some()
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
fn main() {
    let arg = std::env::args_os().nth(1);
    let scene = window_receipt::SceneOperation::from_argument(arg.as_deref());
    if scene.is_some() || arg.as_deref() == Some(std::ffi::OsStr::new(window_receipt::UI_ARG)) {
        let result = (|| -> std::io::Result<i32> {
            let profile = std::env::args()
                .nth(2)
                .ok_or(std::io::Error::other("missing profile"))?;
            let (input, mut output) =
                managed_scene::confirm_bootstrap(Some(&profile), bootstrap::Role::Window)?;
            let (text, marked) = if let Some(mode) = scene {
                let dialog_output = output.try_clone()?;
                let view = managed_scene::exchange(input, mode, &mut output, |initial, bold| {
                    let mut draft = String::new();
                    match show(
                        None,
                        Some((dialog_output, initial.to_owned(), Some(bold))),
                        Some(&mut draft),
                    ) {
                        40 if !draft.is_empty() => Ok(Some(draft)),
                        code if code == window_receipt::DISCARDED_EXIT as i32 => Ok(None),
                        _ => Err(std::io::Error::other("editor ended unexpectedly")),
                    }
                })?;
                let Some(view) = view else {
                    return Ok(window_receipt::DISCARDED_EXIT as i32);
                };
                view
            } else {
                ("Rabe\n42".to_owned(), false)
            };
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
                Some((output, text, split.then_some(marked))),
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
            .map(|endpoint| show(Some(endpoint), None, None))
            .unwrap_or(20);
        std::process::exit(exit);
    }
    if verify_leaf().is_err() {
        std::process::exit(20);
    }
    std::process::exit(show(None, None, None));
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
