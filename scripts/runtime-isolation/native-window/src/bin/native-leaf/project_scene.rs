//! Single-use product window. Only the owning, authenticated reply pipe receives the result.
use super::*;
use std::{
    fs::File,
    io::{self, Write},
};

unsafe extern "system" fn scene_input(
    hwnd: HWND,
    msg: u32,
    w: WPARAM,
    l: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    unsafe {
        if clipboard_command(
            msg,
            w,
            GetKeyState(VK_CONTROL as i32) < 0,
            GetKeyState(VK_SHIFT as i32) < 0,
        ) {
            return 0;
        }
        windows_sys::Win32::UI::Shell::DefSubclassProc(hwnd, msg, w, l)
    }
}

fn changed_text(hwnd: HWND) -> Option<String> {
    unsafe {
        let edit = GetDlgItem(hwnd, EDIT);
        let units = usize::try_from(GetWindowTextLengthW(edit)).ok()?;
        if units > 2 * PROJECT_EDIT_BYTES {
            return None;
        }
        let mut buf = vec![0u16; units + 1];
        SetLastError(0);
        let read = GetWindowTextW(edit, buf.as_mut_ptr(), buf.len() as i32) as usize;
        if read != units || (read == 0 && GetLastError() != 0) {
            return None;
        }
        let value = String::from_utf16(&buf[..read]).ok()?.replace("\r\n", "\n");
        (value.len() <= PROJECT_EDIT_BYTES && !value.contains(['\0', '\r'])).then_some(value)
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CLOSE => {
                PostQuitMessage(0);
                0
            }
            WM_COMMAND if w == CLOSE as usize || w == IDCANCEL as usize => {
                PostQuitMessage(0);
                0
            }
            WM_COMMAND if w == SAVE as usize && !GetDlgItem(hwnd, SAVE).is_null() => {
                if changed_text(hwnd).is_some() {
                    PostQuitMessage(1);
                } else {
                    MessageBoxW(hwnd, wide("Der Entwurf enthält ungültigen Text oder überschreitet die Grenze von 64 KiB UTF-8.").as_ptr(),wide("Speichern").as_ptr(), MB_OK | MB_ICONWARNING);
                }
                0
            }
            WM_SIZE => {
                let dpi = GetDpiForWindow(hwnd);
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect);
                let margin = scale(12, dpi);
                let button = scale(105, dpi);
                let height = scale(26, dpi);
                let bottom = rect.bottom - margin - height;
                for (id, x, y, width, h) in [
                    (LABEL, margin, margin, rect.right - margin * 2, height),
                    (
                        EDIT,
                        margin,
                        margin + height,
                        rect.right - margin * 2,
                        bottom - margin * 2 - height,
                    ),
                    (
                        SAVE,
                        rect.right - margin * 2 - 2 * button,
                        bottom,
                        button,
                        height,
                    ),
                    (CLOSE, rect.right - margin - button, bottom, button, height),
                ] {
                    let child = GetDlgItem(hwnd, id);
                    if !child.is_null() {
                        MoveWindow(child, x.max(0), y.max(0), width.max(1), h.max(1), 1);
                    }
                }
                0
            }
            _ => DefWindowProcW(hwnd, msg, w, l),
        }
    }
}

pub(super) fn display(mut reply: File, title: &str, text: &str, edit_mode: bool) -> io::Result<()> {
    unsafe {
        let instance = GetModuleHandleW(null());
        let class = wide("KRAQEL.Synthetic.Edit");
        let desc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            hbrBackground: 6usize as _,
            ..Default::default()
        };
        require(RegisterClassW(&desc) != 0, "register project window")?;
        let title = format!(
            "{title} – {}",
            if edit_mode {
                "Bearbeiten"
            } else {
                "Schreibgeschützt"
            }
        );
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class.as_ptr(),
            wide(title).as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            scale(680, GetDpiForSystem()),
            scale(420, GetDpiForSystem()),
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        require(!hwnd.is_null(), "create project window")?;
        let mut style = WS_TABSTOP
            | WS_BORDER
            | WS_VSCROLL
            | ES_MULTILINE as u32
            | ES_AUTOVSCROLL as u32
            | ES_WANTRETURN as u32;
        if !edit_mode {
            style |= ES_READONLY as u32;
        }
        for (id, kind, label, control_style) in [
            (LABEL, "STATIC", "&Szenentext", SS_LEFT),
            (EDIT, "EDIT", "", style),
            (
                SAVE,
                "BUTTON",
                "&Speichern",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (
                CLOSE,
                "BUTTON",
                if edit_mode {
                    "&Abbrechen"
                } else {
                    "S&chließen"
                },
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
        ] {
            if id == SAVE && !edit_mode {
                continue;
            }
            let child = CreateWindowExW(
                0,
                wide(kind).as_ptr(),
                wide(label).as_ptr(),
                WS_CHILD | WS_VISIBLE | control_style,
                0,
                0,
                1,
                1,
                hwnd,
                id as _,
                instance,
                null(),
            );
            require(!child.is_null(), "create project control")?;
            SendMessageW(
                child,
                WM_SETFONT,
                GetStockObject(DEFAULT_GUI_FONT) as usize,
                0,
            );
        }
        let field = GetDlgItem(hwnd, EDIT);
        require(
            windows_sys::Win32::UI::Shell::SetWindowSubclass(field, Some(scene_input), 1, 0) != 0,
            "protect project text",
        )?;
        // Validate UTF-8 size at submission. Allow editing beyond the save limit so
        // a long draft can be shortened explicitly, without saving a truncated prefix.
        SendMessageW(field, EM_SETLIMITTEXT, 4 * 1024 * 1024, 0);
        require(
            SetWindowTextW(field, wide(text.replace('\n', "\r\n")).as_ptr()) != 0
                && matches_preview_text(hwnd, text),
            "project scene readback",
        )?;
        SendMessageW(hwnd, WM_SIZE, 0, 0);
        ShowWindow(hwnd, SW_SHOW);
        SetFocus(field);
        reply.write_all(&(hwnd as usize as u64).to_le_bytes())?;
        let mut message = MSG::default();
        loop {
            let result = GetMessageW(&mut message, null_mut(), 0, 0);
            require(result >= 0, "project window loop")?;
            if result == 0 {
                break;
            }
            if IsDialogMessageW(hwnd, &message) == 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        let accepted = match message.wParam {
            0 => None,
            1 if edit_mode => {
                Some(changed_text(hwnd).ok_or(io::Error::other("invalid scene draft"))?)
            }
            _ => return Err(io::Error::other("unexpected scene result")),
        };
        require(DestroyWindow(hwnd) != 0, "close project window")?;
        reply.write_all(&[u8::from(accepted.is_some())])?;
        if let Some(content) = accepted {
            reply.write_all(&(content.len() as u32).to_le_bytes())?;
            reply.write_all(content.as_bytes())?;
        }
        Ok(())
    }
}
