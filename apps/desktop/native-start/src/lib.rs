//! Native project picker. No WebView, external process or network capability is needed here.
#![cfg(windows)]

use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::EnableWindow,
        WindowsAndMessaging::{
            DialogBoxParamW, EndDialog, GetDlgItem, IDCANCEL, IDOK, LB_ADDSTRING, LB_GETCURSEL,
            LB_SETCURSEL, LBN_DBLCLK, SendDlgItemMessageW, SetDlgItemTextW, WM_CLOSE, WM_COMMAND,
            WM_INITDIALOG,
        },
    },
};

const DIALOG: u16 = 401;
const CAPTION: i32 = 101;
const LIST: i32 = 102;
const MANAGE: i32 = 103;
const SELECTED: isize = 10;

pub enum Choice {
    Close,
    Management,
    Project(usize),
}

struct Entries<'a> {
    names: &'a [String],
    catalog_failed: bool,
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16()
        .map(|c| if c == 0 { 0xfffd } else { c })
        .chain([0])
        .collect()
}

/// Shows the dialog embedded in the desktop executable and waits for an explicit action.
/// A missing catalog is shown separately from an empty list.
pub fn choose(names: Option<&[String]>) -> io::Result<Choice> {
    let entries = Entries {
        names: names.unwrap_or_default(),
        catalog_failed: names.is_none(),
    };
    // The current executable owns the dialog resource. DialogBoxParamW is synchronous.
    // Its WM_INITDIALOG handler reads entries only before this stack frame returns.
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            usize::from(DIALOG) as *const u16,
            ptr::null_mut(),
            Some(handle),
            (&raw const entries) as LPARAM,
        )
    };
    match result {
        0 => Ok(Choice::Close),
        1 => Ok(Choice::Management),
        n if n >= SELECTED && ((n - SELECTED) as usize) < entries.names.len() => {
            Ok(Choice::Project((n - SELECTED) as usize))
        }
        -1 => Err(io::Error::last_os_error()),
        _ => Err(io::Error::other(
            "Die native Projektliste konnte nicht angezeigt werden.",
        )),
    }
}

unsafe extern "system" fn handle(hwnd: HWND, message: u32, action: WPARAM, data: LPARAM) -> isize {
    // hwnd and control handles are valid only during synchronous dialog dispatch.
    unsafe {
        match message {
            WM_INITDIALOG => {
                let entries = &*(data as *const Entries<'_>);
                for (i, name) in entries.names.iter().enumerate() {
                    let name = wide(name);
                    if SendDlgItemMessageW(hwnd, LIST, LB_ADDSTRING, 0, name.as_ptr() as LPARAM)
                        != i as isize
                    {
                        EndDialog(hwnd, -2);
                        return 1;
                    }
                }
                if entries.names.is_empty() {
                    let status = if entries.catalog_failed {
                        "Die lokalen Projekte konnten nicht gelesen werden."
                    } else {
                        "Auf diesem Rechner sind noch keine lokalen Projekte angelegt."
                    };
                    SetDlgItemTextW(hwnd, CAPTION, wide(status).as_ptr());
                    EnableWindow(GetDlgItem(hwnd, IDOK), 0);
                } else {
                    SendDlgItemMessageW(hwnd, LIST, LB_SETCURSEL, 0, 0);
                }
                1
            }
            WM_CLOSE => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND => {
                let id = (action & 0xffff) as i32;
                let notification = (action >> 16) as u32;
                if id == IDOK || (id == LIST && notification == LBN_DBLCLK) {
                    let index = SendDlgItemMessageW(hwnd, LIST, LB_GETCURSEL, 0, 0);
                    if index >= 0 {
                        EndDialog(hwnd, SELECTED + index);
                    }
                } else if id == MANAGE {
                    EndDialog(hwnd, 1);
                } else if id == IDCANCEL {
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
