//! Native modal close decision for a dirty project scene. No external dialog resource is loaded.
use super::*;
use windows_sys::Win32::UI::WindowsAndMessaging::DM_SETDEFID;

const STORE: u16 = 201;
const DROP_DRAFT: u16 = 202;
const KEEP: u16 = 203;

pub(super) enum Choice {
    Save,
    Discard,
    Continue,
}

unsafe extern "system" fn handle(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> isize {
    unsafe {
        match msg {
            WM_INITDIALOG => {
                EnableWindow(GetDlgItem(hwnd, i32::from(STORE)), i32::from(l != 0));
                SendMessageW(hwnd, DM_SETDEFID, usize::from(KEEP), 0);
                SetFocus(GetDlgItem(hwnd, i32::from(KEEP)));
                0
            }
            WM_CLOSE => {
                EndDialog(hwnd, KEEP as isize);
                1
            }
            WM_COMMAND => {
                let id = (w & 0xffff) as u16;
                if matches!(id, STORE | DROP_DRAFT)
                    && w >> 16 == 0
                    && l as HWND == GetDlgItem(hwnd, i32::from(id))
                {
                    EndDialog(hwnd, id as isize);
                } else if id == KEEP || id == IDCANCEL as u16 || id == IDOK as u16 {
                    EndDialog(hwnd, KEEP as isize);
                } else {
                    return 0;
                }
                1
            }
            _ => 0,
        }
    }
}

pub(super) fn choose(owner: HWND, available: bool) -> Choice {
    // DialogBoxIndirectParamW requires a DWORD-aligned, UTF-16 DLGTEMPLATE.
    let style = WS_POPUP
        | WS_CAPTION
        | WS_SYSMENU
        | DS_MODALFRAME as u32
        | DS_SETFONT as u32
        | DS_CENTER as u32;
    let mut words = vec![
        style as u16,
        (style >> 16) as u16,
        0,
        0,
        4,
        0,
        0,
        300,
        86,
        0,
        0,
    ];
    words.extend(wide("KRAQEL – Ungespeicherter Entwurf"));
    words.push(9);
    words.extend(wide("Segoe UI"));
    let button = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
    for (id, class, x, y, width, height, style, label) in [
        (
            0xffff,
            0x82,
            10,
            10,
            280,
            30,
            WS_CHILD | WS_VISIBLE,
            "Der Entwurf enthält ungespeicherte Änderungen.",
        ),
        (
            KEEP,
            0x80,
            10,
            55,
            100,
            19,
            button | BS_DEFPUSHBUTTON as u32,
            "&Weiterschreiben",
        ),
        (STORE, 0x80, 121, 55, 80, 19, button, "&Speichern"),
        (DROP_DRAFT, 0x80, 210, 55, 80, 19, button, "&Verwerfen"),
    ] {
        if !words.len().is_multiple_of(2) {
            words.push(0);
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
        words.extend(wide(label));
        words.push(0);
    }
    let template: Vec<u32> = words
        .chunks(2)
        .map(|w| u32::from(w[0]) | (u32::from(*w.get(1).unwrap_or(&0)) << 16))
        .collect();
    let choice = unsafe {
        DialogBoxIndirectParamW(
            GetModuleHandleW(null()),
            template.as_ptr().cast(),
            owner,
            Some(handle),
            isize::from(available),
        )
    };
    match choice {
        n if n == isize::from(STORE as i16) && available => Choice::Save,
        n if n == isize::from(DROP_DRAFT as i16) => Choice::Discard,
        _ => Choice::Continue,
    }
}
