//! Native project picker. No WebView, external process or network capability is needed here.
#![cfg(windows)]

mod project_name;
pub use project_name::{ask_document_title, ask_project_name};

use std::{io, ptr};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::EnableWindow,
        WindowsAndMessaging::{
            DialogBoxParamW, EndDialog, GWLP_USERDATA, GetDlgItem, GetWindowLongPtrW, IDCANCEL,
            IDOK, LB_ADDSTRING, LB_GETCURSEL, LB_SETCURSEL, LBN_DBLCLK, SendDlgItemMessageW,
            SetDlgItemTextW, SetWindowLongPtrW, WM_CLOSE, WM_COMMAND, WM_INITDIALOG,
        },
    },
};

const DIALOG: u16 = 401;
const CAPTION: i32 = 101;
const LIST: i32 = 102;
const MANAGE: i32 = 103;
const BROWSE: i32 = 104;
const EDIT_SCENE: i32 = 105;
const NEW_PROJECT: i32 = 106;
const NEW_DOCUMENT: i32 = 109;
const RENAME: i32 = 110;

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
    selected: Option<usize>,
    command: Option<(i32, Option<usize>)>,
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
    match list_result(DIALOG, names, empty, None)? {
        None => Ok(Choice::Close),
        Some((MANAGE, _)) => Ok(Choice::Management),
        Some((NEW_PROJECT, _)) => Ok(Choice::Create),
        Some((IDOK, Some(index))) => Ok(Choice::Project(index)),
        Some((BROWSE, Some(index))) => Ok(Choice::Manuscript(index)),
        _ => Err(io::Error::other("Ungültige Projektauswahl.")),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneAction {
    Read,
    Edit,
}

pub enum OutlineChoice {
    Back,
    Create(Option<usize>),
    Rename(usize),
    Open(usize),
    Edit(usize),
}

pub fn choose_chapter(names: &[String], selected: Option<usize>) -> io::Result<OutlineChoice> {
    choose_outline(
        402,
        names,
        "Dieses Projekt enthält keine Kapitel.",
        selected,
    )
}

pub fn choose_scene(names: &[String], selected: Option<usize>) -> io::Result<OutlineChoice> {
    choose_outline(403, names, "Dieses Kapitel enthält keine Szenen.", selected)
}

fn choose_outline(
    resource: u16,
    names: &[String],
    empty: &str,
    selected: Option<usize>,
) -> io::Result<OutlineChoice> {
    match list_result(resource, names, empty, selected)? {
        None => Ok(OutlineChoice::Back),
        Some((NEW_DOCUMENT, index)) => Ok(OutlineChoice::Create(index)),
        Some((RENAME, Some(index))) => Ok(OutlineChoice::Rename(index)),
        Some((IDOK, Some(index))) => Ok(OutlineChoice::Open(index)),
        Some((EDIT_SCENE, Some(index))) if resource == 403 => Ok(OutlineChoice::Edit(index)),
        _ => Err(io::Error::other("Ungültige Manuskriptauswahl.")),
    }
}

type ListCommand = Option<(i32, Option<usize>)>;

fn list_result(
    resource: u16,
    names: &[String],
    empty: &str,
    selected: Option<usize>,
) -> io::Result<ListCommand> {
    let mut entries = Entries {
        names,
        empty_label: empty,
        selected,
        command: None,
    };
    // The callback may borrow entries for the duration of this synchronous modal dialog.
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            usize::from(resource) as *const u16,
            ptr::null_mut(),
            Some(handle),
            (&raw mut entries) as LPARAM,
        )
    };
    match result {
        0 => Ok(None),
        1 if entries.command.is_some() => Ok(entries.command),
        -1 => Err(io::Error::last_os_error()),
        _ => Err(io::Error::other(
            "Die native Auswahlliste konnte nicht angezeigt werden.",
        )),
    }
}

unsafe extern "system" fn handle(hwnd: HWND, message: u32, action: WPARAM, data: LPARAM) -> isize {
    // hwnd and control handles are valid only during synchronous dialog dispatch.
    unsafe {
        match message {
            WM_INITDIALOG => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, data);
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
                    for id in [BROWSE, EDIT_SCENE, RENAME] {
                        let button = GetDlgItem(hwnd, id);
                        if !button.is_null() {
                            EnableWindow(button, 0);
                        }
                    }
                    EnableWindow(GetDlgItem(hwnd, IDOK), 0);
                } else {
                    let index = entries
                        .selected
                        .filter(|i| *i < entries.names.len())
                        .unwrap_or(0);
                    SendDlgItemMessageW(hwnd, LIST, LB_SETCURSEL, index, 0);
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
                if id == IDCANCEL {
                    EndDialog(hwnd, 0);
                    return 1;
                }
                let command = match id {
                    LIST if notification == LBN_DBLCLK => IDOK,
                    IDOK | BROWSE | EDIT_SCENE | MANAGE | NEW_PROJECT | NEW_DOCUMENT | RENAME
                        if !GetDlgItem(hwnd, id).is_null() =>
                    {
                        id
                    }
                    _ => return 0,
                };
                let entries = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Entries<'_>;
                if entries.is_null() {
                    return 0;
                }
                let entry_count = (&*entries).names.len();
                let index = usize::try_from(SendDlgItemMessageW(hwnd, LIST, LB_GETCURSEL, 0, 0))
                    .ok()
                    .filter(|i| *i < entry_count);
                if index.is_some() || matches!(command, NEW_DOCUMENT | NEW_PROJECT | MANAGE) {
                    (*entries).command = Some((command, index));
                    EndDialog(hwnd, 1);
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
