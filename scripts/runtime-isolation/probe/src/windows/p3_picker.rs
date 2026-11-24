//! Local P3 research picker. The dialog receives no paths and has three fixed outcomes.
use super::{Result, Value, fail, json};
use std::ffi::OsStr;
use std::ptr::null_mut;
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

fn invalid(stage: &'static str) -> super::Failure {
    super::Failure { stage, code: -1 }
}

const NORTH: u16 = 101;
const SOUTH: u16 = 102;

unsafe extern "system" fn on_dialog(hwnd: HWND, msg: u32, w: usize, _: isize) -> isize {
    match msg {
        WM_INITDIALOG => {
            unsafe { SendMessageW(hwnd, WM_NEXTDLGCTL, GetDlgItem(hwnd, IDCANCEL) as usize, 1) };
            0
        }
        WM_CLOSE => unsafe { EndDialog(hwnd, IDCANCEL as isize) as isize },
        WM_COMMAND if matches!(w as u16, NORTH | SOUTH | 2) && w >> 16 == 0 => unsafe {
            EndDialog(hwnd, (w as u16) as isize) as isize
        },
        _ => 0,
    }
}

fn string(words: &mut Vec<u16>, text: &str) {
    words.extend(text.encode_utf16().chain(Some(0)));
}

fn directory_identity(argument: &OsStr) -> Result<(u64, String)> {
    let raw: usize = argument
        .to_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| invalid("p3-selection-handle"))?;
    let handle = raw as windows_sys::Win32::Foundation::HANDLE;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    let mut id = FILE_ID_INFO::default();
    if unsafe { GetFileType(handle) } != FILE_TYPE_DISK
        || unsafe { GetFileInformationByHandle(handle, &mut info) } == 0
        || info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
            != FILE_ATTRIBUTE_DIRECTORY
        || unsafe {
            GetFileInformationByHandleEx(
                handle,
                FileIdInfo,
                (&raw mut id).cast(),
                size_of::<FILE_ID_INFO>() as u32,
            )
        } == 0
        || id.VolumeSerialNumber == 0
        || id.FileId.Identifier == [0; 16]
    {
        return Err(invalid("p3-selection-directory"));
    }
    Ok((
        id.VolumeSerialNumber,
        format!("{:032x}", u128::from_le_bytes(id.FileId.Identifier)),
    ))
}

pub(super) fn choose(north: &OsStr, south: &OsStr) -> Result<Value> {
    let north = directory_identity(north)?;
    let south = directory_identity(south)?;
    if north == south {
        return Err(invalid("p3-selection-duplicate"));
    }

    // Standard DLGTEMPLATE and DLGITEMTEMPLATE structures use little-endian WORDs.
    let style = WS_POPUP | WS_CAPTION | WS_SYSMENU | DS_MODALFRAME as u32 | DS_SETFONT as u32;
    let mut words = vec![
        style as u16,
        (style >> 16) as u16,
        0,
        0,
        4,
        0,
        0,
        220,
        73,
        0,
        0,
    ];
    string(&mut words, "KRAQEL - P3-Quellauswahl");
    words.push(9);
    string(&mut words, "Segoe UI");
    for (id, class, x, y, width, height, label) in [
        (
            0xffff,
            0x82,
            8,
            8,
            204,
            24,
            "Welchen synthetischen Projektordner als Testkopie erfassen?",
        ),
        (NORTH, 0x80, 8, 45, 62, 17, "Projekt Nord"),
        (SOUTH, 0x80, 78, 45, 62, 17, "Projekt Sued"),
        (IDCANCEL as u16, 0x80, 148, 45, 64, 17, "Abbrechen"),
    ] {
        if words.len() % 2 != 0 {
            words.push(0);
        }
        let mut style = WS_CHILD | WS_VISIBLE;
        if class == 0x80 {
            style |= WS_TABSTOP;
        }
        if id == IDCANCEL as u16 {
            style |= BS_DEFPUSHBUTTON as u32;
        }
        words.extend([
            style as u16,
            (style >> 16) as u16,
            0,
            0,
            x,
            y,
            width,
            height,
            id,
            0xffff,
            class,
        ]);
        string(&mut words, label);
        words.push(0);
    }
    let template: Vec<u32> = words
        .chunks(2)
        .map(|w| u32::from(w[0]) | (u32::from(*w.get(1).unwrap_or(&0)) << 16))
        .collect();
    let result = unsafe {
        DialogBoxIndirectParamW(
            null_mut(),
            template.as_ptr().cast(),
            null_mut(),
            Some(on_dialog),
            0,
        )
    };
    let selection = match result {
        2 => return Ok(json!({"selection":"cancel"})),
        n if n == NORTH as isize => ("north", north),
        n if n == SOUTH as isize => ("south", south),
        _ => return Err(fail("p3-source-dialog")),
    };
    Ok(json!({"selection": selection.0, "directory": selection.1}))
}
