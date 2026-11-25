use super::{
    EndDialog, GWLP_USERDATA, GetModuleHandleW, GetWindowLongPtrW, HWND, IDCANCEL, IDOK,
    LB_ADDSTRING, LB_GETCURSEL, LB_SETCURSEL, LPARAM, SendDlgItemMessageW, SetDlgItemTextW,
    SetWindowLongPtrW, WM_CLOSE, WM_COMMAND, WM_INITDIALOG, WPARAM, wide,
};
use std::{io, path::Path, ptr};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DialogBoxParamW, IDYES, MB_DEFBUTTON2, MB_ICONQUESTION, MB_YESNO, MessageBoxW,
};

#[derive(Clone, Copy)]
pub enum NativeExportFormat {
    Markdown,
    Html,
    Docx,
    Pdf,
}

impl NativeExportFormat {
    const ALL: [Self; 4] = [Self::Markdown, Self::Html, Self::Docx, Self::Pdf];

    pub fn label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown (.md)",
            Self::Html => "HTML (.html)",
            Self::Docx => "Word-Dokument (.docx)",
            Self::Pdf => "PDF (.pdf)",
        }
    }
}

pub enum NativeExportChoice {
    Write(usize, NativeExportFormat),
    History,
}

struct ExportOptions<'a> {
    scopes: &'a [String],
    choice: Option<NativeExportChoice>,
}

pub fn choose_export(scopes: &[String]) -> io::Result<Option<NativeExportChoice>> {
    if scopes.is_empty() {
        return Err(io::Error::other("Es gibt keinen Exportumfang."));
    }
    let mut state = ExportOptions {
        scopes,
        choice: None,
    };
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            412 as *const u16,
            ptr::null_mut(),
            Some(handle),
            (&raw mut state) as LPARAM,
        )
    };
    match result {
        0 => Ok(None),
        1 if state.choice.is_some() => Ok(state.choice),
        _ => Err(io::Error::other(
            "Die Exportauswahl konnte nicht angezeigt werden.",
        )),
    }
}

pub fn confirm_export(scope: &str, format: NativeExportFormat) -> bool {
    let message = format!(
        "{scope}\r\nFormat: {}\r\n\r\nJetzt eine neue Datei im Unterordner „exports“ dieses Projekts speichern?",
        format.label()
    );
    unsafe {
        MessageBoxW(
            ptr::null_mut(),
            wide(&message).as_ptr(),
            wide("KRAQEL – Export bestätigen").as_ptr(),
            MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION,
        ) == IDYES
    }
}

unsafe extern "system" fn handle(hwnd: HWND, msg: u32, action: WPARAM, data: LPARAM) -> isize {
    const SCOPE: i32 = 121;
    const FORMAT: i32 = 122;
    const HISTORY: i32 = 123;
    unsafe {
        match msg {
            WM_INITDIALOG => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, data);
                let state = &*(data as *const ExportOptions<'_>);
                for (id, index, title) in state
                    .scopes
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (SCOPE, i, s.as_str()))
                    .chain(
                        NativeExportFormat::ALL
                            .iter()
                            .enumerate()
                            .map(|(i, f)| (FORMAT, i, f.label())),
                    )
                {
                    if SendDlgItemMessageW(
                        hwnd,
                        id,
                        LB_ADDSTRING,
                        0,
                        wide(title).as_ptr() as LPARAM,
                    ) != index as isize
                    {
                        EndDialog(hwnd, -2);
                        return 1;
                    }
                }
                for id in [SCOPE, FORMAT] {
                    SendDlgItemMessageW(hwnd, id, LB_SETCURSEL, 0, 0);
                }
                1
            }
            WM_CLOSE => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND if (action & 0xffff) as i32 == IDCANCEL => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND if matches!((action & 0xffff) as i32, IDOK | HISTORY) => {
                let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut ExportOptions<'_>;
                if state.is_null() {
                    return 0;
                }
                let state = &mut *state;
                if (action & 0xffff) as i32 == HISTORY {
                    state.choice = Some(NativeExportChoice::History);
                    EndDialog(hwnd, 1);
                    return 1;
                }
                let selected =
                    |id| usize::try_from(SendDlgItemMessageW(hwnd, id, LB_GETCURSEL, 0, 0)).ok();
                if let (Some(scope), Some(format)) = (
                    selected(SCOPE).filter(|&i| i < state.scopes.len()),
                    selected(FORMAT).and_then(|i| NativeExportFormat::ALL.get(i)),
                ) {
                    state.choice = Some(NativeExportChoice::Write(scope, *format));
                    EndDialog(hwnd, 1);
                }
                1
            }
            _ => 0,
        }
    }
}

struct ExportHistory<'a> {
    files: &'a [String],
    directory: &'a Path,
}

pub fn show_export_history(files: &[String], directory: &Path) -> io::Result<()> {
    let data = ExportHistory { files, directory };
    let result = unsafe {
        DialogBoxParamW(
            GetModuleHandleW(ptr::null()),
            413 as *const u16,
            ptr::null_mut(),
            Some(history_proc),
            (&raw const data) as LPARAM,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::other(
            "Die vorhandenen Ausgaben konnten nicht angezeigt werden.",
        ))
    }
}

unsafe extern "system" fn history_proc(
    hwnd: HWND,
    msg: u32,
    action: WPARAM,
    data: LPARAM,
) -> isize {
    const LIST: i32 = 124;
    const DIRECTORY: i32 = 125;
    unsafe {
        match msg {
            WM_INITDIALOG => {
                let data = &*(data as *const ExportHistory<'_>);
                if SetDlgItemTextW(
                    hwnd,
                    DIRECTORY,
                    wide(&data.directory.to_string_lossy()).as_ptr(),
                ) == 0
                {
                    EndDialog(hwnd, -2);
                    return 1;
                }
                if data.files.is_empty() {
                    SetDlgItemTextW(
                        hwnd,
                        126,
                        wide("Bisher gibt es keine Ausgaben aus der nativen Gliederung.").as_ptr(),
                    );
                }
                for (index, file) in data.files.iter().enumerate() {
                    if SendDlgItemMessageW(
                        hwnd,
                        LIST,
                        LB_ADDSTRING,
                        0,
                        wide(file).as_ptr() as LPARAM,
                    ) != index as isize
                    {
                        EndDialog(hwnd, -2);
                        return 1;
                    }
                }
                if !data.files.is_empty() {
                    SendDlgItemMessageW(hwnd, LIST, LB_SETCURSEL, 0, 0);
                }
                1
            }
            WM_CLOSE => {
                EndDialog(hwnd, 0);
                1
            }
            WM_COMMAND if matches!((action & 0xffff) as i32, IDCANCEL | IDOK) => {
                EndDialog(hwnd, 0);
                1
            }
            _ => 0,
        }
    }
}
