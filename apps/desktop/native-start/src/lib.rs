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
const BROWSE: i32 = 104;
const SELECTED: isize = 10;

pub enum Choice {
    Close,
    Management,
    Project(usize),
    Manuscript(usize),
}

struct Entries<'a> {
    names: &'a [String],
    empty_label: &'a str,
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
    let empty = if names.is_none() {
        "Die lokalen Projekte konnten nicht gelesen werden."
    } else {
        "Auf diesem Rechner sind noch keine lokalen Projekte angelegt."
    };
    choose_list(DIALOG, names.unwrap_or_default(), empty)
}

pub enum ManuscriptLevel {
    Chapters,
    Scenes,
}

pub fn choose_document(level: ManuscriptLevel, names: &[String]) -> io::Result<Option<usize>> {
    let (resource, empty) = match level {
        ManuscriptLevel::Chapters => (402, "Dieses Projekt enthält keine Kapitel."),
        ManuscriptLevel::Scenes => (403, "Dieses Kapitel enthält keine Szenen."),
    };
    match choose_list(resource, names, empty)? {
        Choice::Project(index) => Ok(Some(index)),
        Choice::Close => Ok(None),
        _ => Err(io::Error::other("Ungültige Dokumentauswahl.")),
    }
}

fn choose_list(resource: u16, names: &[String], empty: &str) -> io::Result<Choice> {
    let entries = Entries {
        names,
        empty_label: empty,
    };
    // The current executable owns the dialog resource. DialogBoxParamW is synchronous.
    // Its WM_INITDIALOG handler reads entries only before this stack frame returns.
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            usize::from(resource) as *const u16,
            ptr::null_mut(),
            Some(handle),
            (&raw const entries) as LPARAM,
        )
    };
    match result {
        0 => Ok(Choice::Close),
        1 => Ok(Choice::Management),
        n if n >= SELECTED => {
            let selection = (n - SELECTED) as usize;
            let index = selection / 2;
            if index >= entries.names.len() {
                return Err(io::Error::other("Ungültige Auswahl."));
            }
            Ok(if selection.is_multiple_of(2) {
                Choice::Project(index)
            } else {
                Choice::Manuscript(index)
            })
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
                    SetDlgItemTextW(hwnd, CAPTION, wide(entries.empty_label).as_ptr());
                    let browse = GetDlgItem(hwnd, BROWSE);
                    if !browse.is_null() {
                        EnableWindow(browse, 0);
                    }
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
                if id == IDOK || id == BROWSE || (id == LIST && notification == LBN_DBLCLK) {
                    let index = SendDlgItemMessageW(hwnd, LIST, LB_GETCURSEL, 0, 0);
                    if index >= 0 {
                        EndDialog(hwnd, SELECTED + index * 2 + isize::from(id == BROWSE));
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

pub fn confirm_edit() -> io::Result<bool> {
    unsafe extern "system" fn action(hwnd: HWND, message: u32, value: WPARAM, _: LPARAM) -> isize {
        let id = (value & 0xffff) as i32;
        if message == WM_CLOSE || (message == WM_COMMAND && (id == IDCANCEL || id == 105)) {
            unsafe { EndDialog(hwnd, isize::from(message == WM_COMMAND && id == 105)) };
            return 1;
        }
        0
    }
    // Only the explicitly pressed edit button yields 1; close and Escape return 0.
    match unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            404usize as *const u16,
            ptr::null_mut(),
            Some(action),
            0,
        )
    } {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(io::Error::last_os_error()),
    }
}

pub fn notify(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
    // Both UTF-16 buffers live until the modal call returns.
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            wide(text).as_ptr(),
            wide("KRAQEL – Manuskript").as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
