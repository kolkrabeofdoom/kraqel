use super::*;
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::SetFocus, WindowsAndMessaging::GetWindowTextLengthW,
};

pub fn confirm_removal(details: &str) -> io::Result<bool> {
    let text = wide(details);
    // The stable slice is borrowed only during synchronous initialization of this modal dialog.
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            408usize as *const u16,
            ptr::null_mut(),
            Some(handle),
            (&raw const text) as LPARAM,
        )
    };
    match result {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(io::Error::other(
            "Die Löschvorschau konnte nicht vollständig angezeigt werden.",
        )),
    }
}

unsafe extern "system" fn handle(hwnd: HWND, message: u32, action: WPARAM, data: LPARAM) -> isize {
    unsafe {
        match message {
            WM_INITDIALOG => {
                let text = &*(data as *const Vec<u16>);
                let field = GetDlgItem(hwnd, 115);
                if field.is_null()
                    || SetDlgItemTextW(hwnd, 115, text.as_ptr()) == 0
                    || usize::try_from(GetWindowTextLengthW(field)).ok() != Some(text.len() - 1)
                {
                    EndDialog(hwnd, -2);
                    return 0;
                }
                SetFocus(GetDlgItem(hwnd, IDCANCEL));
                0
            }
            WM_CLOSE => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND => {
                let id = (action & 0xffff) as i32;
                if id == 116 && data as HWND == GetDlgItem(hwnd, 116) && action >> 16 == 0 {
                    EndDialog(hwnd, 1);
                } else if id == IDCANCEL || id == IDOK {
                    EndDialog(hwnd, 0);
                } else {
                    return 0;
                }
                1
            }
            _ => 0,
        }
    }
}
