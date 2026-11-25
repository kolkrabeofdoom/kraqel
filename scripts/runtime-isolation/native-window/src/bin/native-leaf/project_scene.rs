//! Single-use product window. Only the owning, authenticated reply pipe receives the result.
use super::*;
#[path = "project_close.rs"]
mod project_close;
#[path = "project_dialog.rs"]
mod project_dialog;
#[path = "project_review.rs"]
mod project_review;
use std::{
    fs::File,
    io::{self, Read, Write},
    sync::mpsc,
    thread,
};
use windows_sys::Win32::UI::Controls::EM_SETREADONLY;
use zeroize::Zeroizing;

const STATUS: i32 = 204;
const COMPARE: i32 = 205;
const REQUEST: u32 = WM_APP + 40;
const SAVE_TIMER: usize = 7001;

enum Request {
    Save(Zeroizing<String>),
    Compare(Zeroizing<String>),
    CancelReview(u64),
    Confirm {
        generation: u64,
        revision: u64,
        choices: Vec<u8>,
    },
    Close,
}

enum Reply {
    Canceled,
    Committed,
    Conflict,
    Failed,
    Review {
        generation: u64,
        revision: u64,
        text: Zeroizing<String>,
    },
}

fn exchange(input: &mut File, output: &mut File, request: Request) -> io::Result<Reply> {
    let reviewing = matches!(request, Request::Compare(_) | Request::Confirm { .. });
    let comparing = matches!(request, Request::Compare(_));
    let canceling = matches!(request, Request::CancelReview(_));
    match request {
        Request::Save(text) | Request::Compare(text) => {
            output.write_all(&[if comparing {
                window_receipt::SCENE_COMPARE
            } else {
                1
            }])?;
            output.write_all(&(text.len() as u32).to_le_bytes())?;
            output.write_all(text.as_bytes())?;
        }
        Request::Confirm {
            generation,
            revision,
            choices,
        } => {
            output.write_all(&[window_receipt::SCENE_CONFIRM])?;
            output.write_all(&generation.to_le_bytes())?;
            output.write_all(&revision.to_le_bytes())?;
            output.write_all(&(choices.len() as u32).to_le_bytes())?;
            output.write_all(&choices)?;
        }
        Request::CancelReview(generation) => {
            output.write_all(&[window_receipt::SCENE_CANCEL_REVIEW])?;
            output.write_all(&generation.to_le_bytes())?;
        }
        Request::Close => return Err(io::Error::other("unexpected close")),
    }
    let mut ack = [0];
    input.read_exact(&mut ack)?;
    match ack[0] {
        window_receipt::SAVE_ACK_COMMITTED if !comparing && !canceling => Ok(Reply::Committed),
        window_receipt::SAVE_ACK_CONFLICT if !reviewing && !canceling => Ok(Reply::Conflict),
        window_receipt::SAVE_ACK_CANCELED if canceling => Ok(Reply::Canceled),
        window_receipt::SAVE_ACK_FAILED => Ok(Reply::Failed),
        window_receipt::SAVE_ACK_REVIEW if reviewing => {
            let mut generation = [0; 8];
            let mut revision = [0; 8];
            let mut length = [0; 4];
            input.read_exact(&mut generation)?;
            input.read_exact(&mut revision)?;
            input.read_exact(&mut length)?;
            let length = u32::from_le_bytes(length) as usize;
            require(length <= PROJECT_EDIT_BYTES, "invalid review length")?;
            let mut bytes = vec![0; length];
            input.read_exact(&mut bytes)?;
            let text = Zeroizing::new(
                String::from_utf8(bytes).map_err(|_| io::Error::other("invalid review text"))?,
            );
            require(!text.contains(['\r', '\0']), "invalid review text")?;
            Ok(Reply::Review {
                generation: u64::from_le_bytes(generation),
                revision: u64::from_le_bytes(revision),
                text,
            })
        }
        _ => Err(io::Error::other("invalid scene acknowledgement")),
    }
}

struct Channel {
    request: mpsc::Sender<Request>,
    acknowledgement: mpsc::Receiver<io::Result<Reply>>,
    worker: thread::JoinHandle<io::Result<()>>,
}

impl Channel {
    fn new(mut input: File, mut output: File) -> io::Result<Self> {
        let (request, requests) = mpsc::channel();
        let (acks, acknowledgement) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("scene-save".into())
            .spawn(move || {
                while let Ok(action) = requests.recv() {
                    match action {
                        Request::Close => return output.write_all(&[0]),
                        request => {
                            let result = exchange(&mut input, &mut output, request);
                            let finished = matches!(result, Ok(Reply::Committed) | Err(_));
                            if acks.send(result).is_err() || finished {
                                break;
                            }
                        }
                    }
                }
                Ok(())
            })?;
        Ok(Self {
            request,
            acknowledgement,
            worker,
        })
    }

    fn finish(self, committed: bool) -> io::Result<()> {
        if !committed {
            let _ = self.request.send(Request::Close);
        }
        drop(self.request);
        self.worker
            .join()
            .map_err(|_| io::Error::other("scene channel failed"))?
    }
}

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
                PostMessageW(hwnd, REQUEST, CLOSE as usize, 0);
                0
            }
            WM_COMMAND if matches!(w, id if id == CLOSE as usize || id == IDCANCEL as usize || id == SAVE as usize || id == COMPARE as usize) =>
            {
                PostMessageW(hwnd, REQUEST, w, 0);
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
                let status_height = if GetDlgItem(hwnd, STATUS).is_null() {
                    0
                } else {
                    height * 2
                };
                for (id, x, y, width, h) in [
                    (LABEL, margin, margin, rect.right - margin * 2, height),
                    (
                        EDIT,
                        margin,
                        margin + height,
                        rect.right - margin * 2,
                        bottom - margin * 2 - height - status_height,
                    ),
                    (
                        STATUS,
                        margin,
                        bottom - status_height,
                        rect.right - 2 * margin,
                        status_height,
                    ),
                    (
                        SAVE,
                        rect.right - margin * 2 - 2 * button,
                        bottom,
                        button,
                        height,
                    ),
                    (COMPARE, margin, bottom, scale(184, dpi), height),
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

pub(super) fn display(
    input: File,
    mut reply: File,
    title: &str,
    text: &str,
    edit_mode: bool,
) -> io::Result<()> {
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
            (STATUS, "STATIC", "", SS_LEFT),
            (
                COMPARE,
                "BUTTON",
                "Fassungen &vergleichen",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
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
            if matches!(id, SAVE | STATUS | COMPARE) && !edit_mode {
                continue;
            }
            let child = CreateWindowExW(
                0,
                wide(kind).as_ptr(),
                wide(label).as_ptr(),
                WS_CHILD | (if id == COMPARE { 0 } else { WS_VISIBLE }) | control_style,
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
        if !edit_mode {
            loop {
                let message = next_message()?;
                if message.hwnd == hwnd
                    && message.message == REQUEST
                    && matches!(message.wParam as i32, CLOSE | IDCANCEL)
                {
                    break;
                }
                dispatch(hwnd, &message);
            }
            require(DestroyWindow(hwnd) != 0, "close project reader")?;
            return reply.write_all(&[0]);
        }
        let channel = Channel::new(input, reply)?;
        edit_loop(hwnd, field, text, channel)
    }
}

fn next_message() -> io::Result<MSG> {
    let mut msg = MSG::default();
    require(
        unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) } > 0,
        "project window loop",
    )?;
    Ok(msg)
}

unsafe fn dispatch(hwnd: HWND, msg: &MSG) {
    unsafe {
        if IsDialogMessageW(hwnd, msg) == 0 {
            TranslateMessage(msg);
            DispatchMessageW(msg);
        }
    }
}

unsafe fn edit_loop(hwnd: HWND, field: HWND, text: &str, channel: Channel) -> io::Result<()> {
    unsafe {
        let mut pending = false;
        let mut available = true;
        let mut compare_available = false;
        let mut review: Option<project_review::Review> = None;
        let mut confirming = false;
        let committed = loop {
            let message = next_message()?;
            if message.hwnd == hwnd
                && message.message == WM_TIMER
                && message.wParam == SAVE_TIMER
                && pending
            {
                let outcome = match channel.acknowledgement.try_recv() {
                    Ok(outcome) => outcome,
                    Err(mpsc::TryRecvError::Empty) => continue,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Err(io::Error::other("scene channel ended"))
                    }
                };
                KillTimer(hwnd, SAVE_TIMER);
                if matches!(outcome, Ok(Reply::Committed)) {
                    break true;
                }
                pending = false;
                let status = match outcome {
                    Ok(Reply::Conflict) => {
                        available = false;
                        compare_available = true;
                        ShowWindow(GetDlgItem(hwnd, COMPARE), SW_SHOW);
                        "Die Szene wurde zwischenzeitlich geändert. Du kannst die Fassungen vergleichen."
                    }
                    Ok(Reply::Review {
                        generation,
                        revision,
                        text: remote,
                    }) => {
                        let selection = review
                            .as_mut()
                            .ok_or(io::Error::other("unexpected comparison"))?;
                        if selection.choose(hwnd, &remote, revision, confirming) {
                            let command = Request::Confirm {
                                generation,
                                revision,
                                choices: selection.choices.clone(),
                            };
                            if start_request(
                                hwnd,
                                field,
                                &channel,
                                command,
                                "Auswahl wird gespeichert …",
                            ) {
                                pending = true;
                                confirming = true;
                                continue;
                            }
                            "Die Auswahl konnte nicht gesendet werden. Der Entwurf bleibt offen."
                        } else {
                            if start_request(
                                hwnd,
                                field,
                                &channel,
                                Request::CancelReview(generation),
                                "Vergleich wird geschlossen …",
                            ) {
                                pending = true;
                                continue;
                            }
                            "Der Vergleich wurde geschlossen. Der Entwurf bleibt offen."
                        }
                    }
                    Ok(Reply::Canceled) => {
                        "Vergleich geschlossen. Der lokale Entwurf bleibt erhalten."
                    }
                    Ok(Reply::Failed) if review.is_some() => {
                        "Der Vergleich konnte nicht abgeschlossen werden. Bitte die Fassungen erneut vergleichen."
                    }
                    Ok(Reply::Failed) => {
                        "Das Speichern ist fehlgeschlagen. Der Entwurf bleibt offen."
                    }
                    _ => {
                        available = false;
                        compare_available = false;
                        EnableWindow(GetDlgItem(hwnd, COMPARE), 0);
                        "Die Speicherbestätigung fehlt. Der Entwurf bleibt offen."
                    }
                };
                review = None;
                confirming = false;
                if !available {
                    EnableWindow(GetDlgItem(hwnd, SAVE), 0);
                }
                SendMessageW(field, EM_SETREADONLY, 0, 0);
                SetWindowTextW(GetDlgItem(hwnd, STATUS), wide(status).as_ptr());
                SetFocus(field);
                continue;
            }
            if message.hwnd == hwnd && message.message == REQUEST {
                if pending {
                    continue;
                }
                let close = message.wParam == CLOSE as usize || message.wParam == IDCANCEL as usize;
                let draft = changed_text(hwnd).map(Zeroizing::new);
                if close {
                    if draft.as_deref().map(String::as_str) == Some(text) {
                        break false;
                    }
                    match project_close::choose(hwnd, available) {
                        project_close::Choice::Discard => break false,
                        project_close::Choice::Continue => {
                            SetFocus(field);
                            continue;
                        }
                        project_close::Choice::Save => {}
                    }
                }
                let compare = message.wParam == COMPARE as usize && compare_available;
                if !compare && (!available || (!close && message.wParam != SAVE as usize)) {
                    continue;
                }
                let Some(draft) = draft else {
                    MessageBoxW(hwnd, wide("Der Entwurf enthält ungültigen Text oder überschreitet die Grenze von 64 KiB UTF-8.").as_ptr(), wide("Speichern").as_ptr(), MB_OK | MB_ICONWARNING);
                    SetFocus(field);
                    continue;
                };
                let (command, status) = if compare {
                    review = Some(project_review::Review::new(draft.clone()));
                    (Request::Compare(draft), "Vergleich wird geladen …")
                } else {
                    (Request::Save(draft), "Wird gespeichert …")
                };
                pending = start_request(hwnd, field, &channel, command, status);
                if !pending {
                    review = None;
                    SetWindowTextW(
                        GetDlgItem(hwnd, STATUS),
                        wide("Die Anfrage ist fehlgeschlagen. Der Entwurf bleibt offen.").as_ptr(),
                    );
                }
                continue;
            }
            dispatch(hwnd, &message);
        };
        require(DestroyWindow(hwnd) != 0, "close project window")?;
        channel.finish(committed)
    }
}

unsafe fn start_request(
    hwnd: HWND,
    field: HWND,
    channel: &Channel,
    command: Request,
    status: &str,
) -> bool {
    unsafe {
        if SetTimer(hwnd, SAVE_TIMER, 100, None) == 0 {
            return false;
        }
        if channel.request.send(command).is_err() {
            KillTimer(hwnd, SAVE_TIMER);
            return false;
        }
        SendMessageW(field, EM_SETREADONLY, 1, 0);
        SetWindowTextW(GetDlgItem(hwnd, STATUS), wide(status).as_ptr());
        true
    }
}
