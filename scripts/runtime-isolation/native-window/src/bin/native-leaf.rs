#![windows_subsystem = "windows"]
use kraqel_window_probe::*;
use std::ptr::{null, null_mut};
use windows_sys::Win32::{
    Foundation::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
use windows_sys::Win32::{System::SystemServices::SS_LEFT, UI::Controls::EM_SETLIMITTEXT};

const LABEL: i32 = 100;
const EDIT: i32 = 101;
const CLOSE: i32 = 102;
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
                let ok = String::from_utf16_lossy(&value[..n as usize]) == "Rabe\r\n42";
                DestroyWindow(hwnd);
                PostQuitMessage(if ok { 40 } else { 41 });
                0
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}
fn show(endpoint: Option<bootstrap::Endpoint>) -> i32 {
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
        ShowWindow(hwnd, SW_SHOW);
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
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
fn main() {
    if bootstrap::Role::Window.receipt().exists() {
        let exit = bootstrap::Endpoint::open(bootstrap::Role::Window)
            .map(|endpoint| show(Some(endpoint)))
            .unwrap_or(20);
        std::process::exit(exit);
    }
    if verify_leaf().is_err() {
        std::process::exit(20);
    }
    std::process::exit(show(None));
}
