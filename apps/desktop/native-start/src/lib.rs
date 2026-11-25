//! Native project picker. No WebView, external process or network capability is needed here.
#![cfg(windows)]

mod project_name;
pub use project_name::ask_project_name;

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
const EDIT_SCENE: i32 = 105;
const SELECTED: isize = 10;

pub enum Choice {
    Close,
    Management,
    Create,
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
    let names = names.unwrap_or_default();
    let result = list_result(DIALOG, names, empty)?;
    match result {
        0 => Ok(Choice::Close),
        1 => Ok(Choice::Management),
        2 => Ok(Choice::Create),
        n if n >= SELECTED => {
            let (index, secondary) = selected_index(n, names.len())?;
            Ok(if !secondary {
                Choice::Project(index)
            } else {
                Choice::Manuscript(index)
            })
        }
        _ => Err(io::Error::other(
            "Die native Projektliste konnte nicht angezeigt werden.",
        )),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneAction {
    Read,
    Edit,
}

pub fn choose_chapter(names: &[String]) -> io::Result<Option<usize>> {
    match list_result(402, names, "Dieses Projekt enthält keine Kapitel.")? {
        0 => Ok(None),
        value => match selected_index(value, names.len())? {
            (index, false) => Ok(Some(index)),
            _ => Err(io::Error::other("Ungültige Kapitelauswahl.")),
        },
    }
}

pub fn choose_scene(names: &[String]) -> io::Result<Option<(usize, SceneAction)>> {
    match list_result(403, names, "Dieses Kapitel enthält keine Szenen.")? {
        0 => Ok(None),
        value => {
            let (index, edit) = selected_index(value, names.len())?;
            Ok(Some((
                index,
                if edit {
                    SceneAction::Edit
                } else {
                    SceneAction::Read
                },
            )))
        }
    }
}

fn selected_index(value: isize, count: usize) -> io::Result<(usize, bool)> {
    let selection = value
        .checked_sub(SELECTED)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| io::Error::other("Ungültige Auswahl."))?;
    let index = selection / 2;
    if index >= count {
        return Err(io::Error::other("Ungültige Auswahl."));
    }
    Ok((index, !selection.is_multiple_of(2)))
}

fn list_result(resource: u16, names: &[String], empty: &str) -> io::Result<isize> {
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
    if result == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result)
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
                    for id in [BROWSE, EDIT_SCENE] {
                        let button = GetDlgItem(hwnd, id);
                        if !button.is_null() {
                            EnableWindow(button, 0);
                        }
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
                let secondary =
                    matches!(id, BROWSE | EDIT_SCENE) && !GetDlgItem(hwnd, id).is_null();
                if id == IDOK || secondary || (id == LIST && notification == LBN_DBLCLK) {
                    let index = SendDlgItemMessageW(hwnd, LIST, LB_GETCURSEL, 0, 0);
                    if index >= 0 {
                        EndDialog(hwnd, SELECTED + index * 2 + isize::from(secondary));
                    }
                } else if id == 106 {
                    EndDialog(hwnd, 2);
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
