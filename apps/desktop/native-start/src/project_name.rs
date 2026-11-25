use super::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWLP_USERDATA, GetDlgItemTextW, GetWindowLongPtrW, GetWindowTextLengthW, SetWindowLongPtrW,
};

struct ProjectName {
    validate: fn(&str) -> bool,
    result: Option<String>,
}

pub fn ask_project_name(validate: fn(&str) -> bool) -> io::Result<Option<String>> {
    let mut input = ProjectName {
        validate,
        result: None,
    };
    // The modal dialog owns this borrow until it returns on the calling thread.
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            405usize as *const u16,
            ptr::null_mut(),
            Some(handle_name),
            (&raw mut input) as LPARAM,
        )
    };
    match result {
        0 => Ok(None),
        1 if input.result.is_some() => Ok(input.result),
        _ => Err(io::Error::other(
            "Die Projekteingabe konnte nicht geöffnet werden.",
        )),
    }
}

unsafe extern "system" fn handle_name(
    hwnd: HWND,
    message: u32,
    action: WPARAM,
    data: LPARAM,
) -> isize {
    const NAME: i32 = 107;
    const HINT: i32 = 108;
    unsafe {
        match message {
            WM_INITDIALOG => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, data);
                1
            }
            WM_CLOSE => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND => match (action & 0xffff) as i32 {
                IDCANCEL => {
                    EndDialog(hwnd, 0);
                    1
                }
                IDOK => {
                    let input = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ProjectName;
                    if input.is_null() {
                        EndDialog(hwnd, -2);
                        return 1;
                    }
                    let len = GetWindowTextLengthW(GetDlgItem(hwnd, NAME));
                    let mut value = vec![0; len as usize + 1];
                    let count = GetDlgItemTextW(hwnd, NAME, value.as_mut_ptr(), len + 1);
                    let name = String::from_utf16(&value[..count as usize]).ok();
                    if let Some(name) = name.filter(|s| ((*input).validate)(s)) {
                        (*input).result = Some(name.trim().to_owned());
                        EndDialog(hwnd, 1);
                    } else {
                        SetDlgItemTextW(
                            hwnd,
                            HINT,
                            wide("Bitte einen Projektnamen mit 1 bis 120 Zeichen eingeben.")
                                .as_ptr(),
                        );
                    }
                    1
                }
                _ => 0,
            },
            _ => 0,
        }
    }
}
