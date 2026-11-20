#![windows_subsystem = "windows"]
use kraqel_window_probe::*;
use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use windows_sys::Win32::{System::SystemServices::SS_LEFT, UI::Controls::EM_SETLIMITTEXT};

const LABEL: i32 = 100;
const EDIT: i32 = 101;
const CLOSE: i32 = 102;
const SECOND: i32 = 103;
const SAVE: i32 = 104;
const DISCARD: i32 = 105;
const DRAFT_LABEL: &str = "Szene A · Zwei Absätze, zweiter fett";
thread_local! {
    static EXPECTED: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}
const SAVE_DRAFT: u32 = WM_APP + 1;
const DISCARD_DRAFT: u32 = WM_APP + 2;
unsafe extern "system" fn local_edit(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_COPY | WM_CUT | WM_PASTE | WM_CONTEXTMENU => 0,
        _ => unsafe { windows_sys::Win32::UI::Shell::DefSubclassProc(hwnd, msg, w, l) },
    }
}
fn font_description(font: HFONT) -> Option<LOGFONTW> {
    let mut desc = LOGFONTW::default();
    let size = size_of::<LOGFONTW>() as i32;
    (unsafe { GetObjectW(font, size, (&mut desc as *mut LOGFONTW).cast()) } == size).then_some(desc)
}
fn draft_text(hwnd: HWND) -> Option<String> {
    let paragraph = |id| {
        let mut buf = [0u16; 49];
        let n = unsafe { GetDlgItemTextW(hwnd, id, buf.as_mut_ptr(), buf.len() as i32) };
        String::from_utf16(&buf[..n as usize]).ok()
    };
    Some(format!("{}\n{}", paragraph(EDIT)?, paragraph(SECOND)?))
}

fn draft_changed(hwnd: HWND) -> bool {
    let current = draft_text(hwnd);
    EXPECTED.with_borrow(|initial| current.as_ref() != Some(initial))
}

fn destroy_surface(hwnd: HWND) {
    unsafe {
        let second = GetDlgItem(hwnd, SECOND);
        let font = if second.is_null() {
            null_mut()
        } else {
            SendMessageW(second, WM_GETFONT, 0, 0) as HFONT
        };
        DestroyWindow(hwnd);
        if !font.is_null() {
            DeleteObject(font);
        }
    }
}

fn update_draft_status(hwnd: HWND) {
    unsafe {
        let save = GetDlgItem(hwnd, SAVE);
        if save.is_null() || GetDlgItem(hwnd, SECOND).is_null() {
            return;
        }
        let dirty = draft_changed(hwnd);
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
unsafe extern "system" fn window(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND
                if (w >> 16) as u32 == EN_CHANGE
                    && matches!((w & 0xffff) as i32, EDIT | SECOND) =>
            {
                update_draft_status(hwnd);
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
                        wide("Änderungen an Szene A verwerfen und das Fenster schließen?").as_ptr(),
                        wide("Ungespeicherte Änderungen").as_ptr(),
                        MB_YESNO | MB_DEFBUTTON2 | MB_ICONWARNING,
                    );
                    if answer != IDYES {
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
                            .is_some_and(|desc| desc.lfWeight == FW_BOLD as i32)
                };
                destroy_surface(hwnd);
                PostQuitMessage(if ok { 40 } else { 41 });
                0
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
fn show(
    endpoint: Option<bootstrap::Endpoint>,
    mut surface: Option<(std::fs::File, String, bool)>,
    mut draft: Option<&mut String>,
) -> i32 {
    unsafe {
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
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            460,
            220,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return 23;
        }
        for (id, kind, title, style, y, height) in [
            (
                LABEL,
                "STATIC",
                if draft.is_some() {
                    DRAFT_LABEL
                } else {
                    "&Prüftext"
                },
                SS_LEFT,
                12,
                22,
            ),
            (
                EDIT,
                "EDIT",
                "",
                WS_TABSTOP | WS_BORDER | ES_MULTILINE as u32 | ES_WANTRETURN as u32,
                38,
                70,
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
                120,
                30,
            ),
        ] {
            if CreateWindowExW(
                0,
                wide(kind).as_ptr(),
                wide(title).as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                15,
                y,
                410,
                height,
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
            if MoveWindow(GetDlgItem(hwnd, SAVE), 15, 120, 195, 30, 0) == 0
                || CreateWindowExW(
                    0,
                    wide("BUTTON").as_ptr(),
                    wide("Änderungen &verwerfen").as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_PUSHBUTTON as u32,
                    219,
                    120,
                    206,
                    30,
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
        SendDlgItemMessageW(hwnd, EDIT, EM_SETLIMITTEXT, 48, 0);
        if let Some((_, text, marked)) = &surface {
            let edit = GetDlgItem(hwnd, EDIT);
            let mut visible = text.as_str();
            if *marked {
                let Some((first, number)) = text.split_once('\n') else {
                    return 24;
                };
                visible = first;
                let normal = GetStockObject(DEFAULT_GUI_FONT);
                let Some(mut desc) = font_description(normal) else {
                    return 24;
                };
                desc.lfWeight = FW_BOLD as i32;
                let bold = CreateFontIndirectW(&desc);
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
                    18,
                    68,
                    400,
                    32,
                    hwnd,
                    SECOND as _,
                    instance,
                    null(),
                );
                if bold.is_null() || second.is_null() || MoveWindow(edit, 15, 38, 410, 26, 1) == 0 {
                    return 24;
                }
                if draft.is_some() {
                    for field in [edit, second] {
                        SendMessageW(field, EM_SETLIMITTEXT, 48, 0);
                        if windows_sys::Win32::UI::Shell::SetWindowSubclass(
                            field,
                            Some(local_edit),
                            1,
                            0,
                        ) == 0
                        {
                            return 24;
                        }
                    }
                }
                SendMessageW(edit, WM_SETFONT, normal as usize, 1);
                SendMessageW(second, WM_SETFONT, bold as usize, 1);
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
                DestroyWindow(hwnd);
            }
            return exit;
        }
        SetFocus(GetDlgItem(hwnd, EDIT));
        let mut msg = MSG::default();
        loop {
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                -1 => return 25,
                0 => return msg.wParam as i32,
                _ => {}
            }
            if let Some(result) = draft.as_deref_mut() {
                let key = msg.wParam as u16;
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
                    for (id, text) in [(EDIT, first), (SECOND, second), (LABEL, DRAFT_LABEL)] {
                        if SetWindowTextW(GetDlgItem(hwnd, id), wide(text).as_ptr()) == 0 {
                            return 24;
                        }
                    }
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
                    SetWindowTextW(
                        GetDlgItem(hwnd, LABEL),
                        wide("Bitte beide Absätze einzeilig ausfüllen.").as_ptr(),
                    );
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
            let (input, output) =
                managed_scene::confirm_bootstrap(Some(&profile), bootstrap::Role::Window)?;
            let (text, marked) = if let Some(mode) = scene {
                managed_scene::exchange(input, mode, |initial| {
                    let mut draft = String::new();
                    let surface = (output.try_clone().ok()?, initial.to_owned(), true);
                    let code = show(None, Some(surface), Some(&mut draft));
                    (code == 40 && !draft.is_empty()).then_some(draft)
                })?
            } else {
                ("Rabe\n42".to_owned(), false)
            };
            Ok(show(None, Some((output, text, marked)), None))
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
