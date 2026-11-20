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
fn font_description(font: HFONT) -> Option<LOGFONTW> {
    let mut desc = LOGFONTW::default();
    let size = size_of::<LOGFONTW>() as i32;
    (unsafe { GetObjectW(font, size, (&mut desc as *mut LOGFONTW).cast()) } == size).then_some(desc)
}
unsafe extern "system" fn window(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND if w & 0xffff == CLOSE as usize => {
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
                0
            }
            WM_CLOSE => {
                let mut value = [0u16; 64];
                let n = GetDlgItemTextW(hwnd, EDIT, value.as_mut_ptr(), 64);
                // Report only whether the public two-line fixture matched, never the input.
                let expected = if GetWindowLongPtrW(hwnd, GWLP_USERDATA) == 1 {
                    "Rabe\r\n43"
                } else {
                    "Rabe\r\n42"
                };
                let first = String::from_utf16_lossy(&value[..n as usize]);
                let second = GetDlgItem(hwnd, SECOND);
                let font = if second.is_null() {
                    null_mut()
                } else {
                    SendMessageW(second, WM_GETFONT, 0, 0) as HFONT
                };
                let ok = if second.is_null() {
                    first == expected
                } else {
                    let n = GetWindowTextW(second, value.as_mut_ptr(), 64);
                    let number = String::from_utf16_lossy(&value[..n as usize]);
                    format!("{first}\r\n{number}") == expected
                        && font_description(font)
                            .is_some_and(|desc| desc.lfWeight == FW_BOLD as i32)
                };
                DestroyWindow(hwnd);
                if !font.is_null() {
                    DeleteObject(font);
                }
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
) -> i32 {
    unsafe {
        let instance = GetModuleHandleW(null());
        let name = wide("KRAQEL.Synthetic.Edit");
        let class = WNDCLASSW {
            lpfnWndProc: Some(window),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            hbrBackground: 6usize as _,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
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
            (LABEL, "STATIC", "&Prüftext", SS_LEFT, 12, 22),
            (
                EDIT,
                "EDIT",
                "",
                WS_TABSTOP | WS_BORDER | ES_MULTILINE as u32 | ES_WANTRETURN as u32,
                38,
                70,
            ),
            (
                CLOSE,
                "BUTTON",
                "&Schließen",
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
        SendDlgItemMessageW(hwnd, EDIT, EM_SETLIMITTEXT, 48, 0);
        if let Some((_, text, marked)) = &surface {
            if text == "Rabe\n43" {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 1);
            }
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
                    wide("STATIC").as_ptr(),
                    wide(number).as_ptr(),
                    WS_CHILD | WS_VISIBLE | SS_LEFT,
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
                SendMessageW(edit, WM_SETFONT, normal as usize, 1);
                SendMessageW(second, WM_SETFONT, bold as usize, 1);
            }
            if SendMessageW(edit, windows_sys::Win32::UI::Controls::EM_SETREADONLY, 1, 0) == 0
                || SetWindowTextW(edit, wide(visible.replace('\n', "\r\n")).as_ptr()) == 0
            {
                return 24;
            }
        }
        ShowWindow(hwnd, SW_SHOW);
        if let Some((output, _, _)) = surface.as_mut() {
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
            // A multiline EDIT can claim Tab. This two-control form reserves it for navigation.
            if msg.message == WM_KEYDOWN
                && msg.wParam == usize::from(VK_TAB)
                && GetKeyState(i32::from(VK_CONTROL)) >= 0
                && GetKeyState(i32::from(VK_MENU)) >= 0
            {
                let next = if GetFocus() == GetDlgItem(hwnd, EDIT) {
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
                        && GetFocus() == GetDlgItem(hwnd, CLOSE)))
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
                managed_scene::exchange(input, mode)?
            } else {
                ("Rabe\n42".to_owned(), false)
            };
            Ok(show(None, Some((output, text, marked))))
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
            .map(|endpoint| show(Some(endpoint), None))
            .unwrap_or(20);
        std::process::exit(exit);
    }
    if verify_leaf().is_err() {
        std::process::exit(20);
    }
    std::process::exit(show(None, None));
}
