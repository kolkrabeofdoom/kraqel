//! Single-use product window. Only the owning, authenticated reply pipe receives the result.
use super::*;
#[path = "project_autosave.rs"]
mod project_autosave;
#[path = "project_close.rs"]
mod project_close;
#[path = "project_dialog.rs"]
mod project_dialog;
#[path = "project_history.rs"]
mod project_history;
#[path = "project_menu.rs"]
mod project_menu;
#[path = "project_replace.rs"]
mod project_replace;
#[path = "project_review.rs"]
mod project_review;
#[path = "project_save_status.rs"]
mod project_save_status;
#[path = "project_search.rs"]
mod project_search;
use std::{
    fs::File,
    io::{self, Read, Write},
    sync::{Arc, mpsc},
    thread,
};
use windows_sys::Win32::UI::Controls::EM_SETREADONLY;
use zeroize::Zeroizing;

const STATUS: i32 = 204;
const COMPARE: i32 = 205;
const METRICS: i32 = 206;
const FOCUS: i32 = 210;
const METRICS_TIMER: usize = 7002;
const REQUEST: u32 = WM_APP + 40;
const SAVE_TIMER: usize = 7001;

enum Request {
    Save(Zeroizing<String>),
    SaveContinue {
        text: Arc<Zeroizing<String>>,
        next_revision: u64,
    },
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
    Continued {
        text: Arc<Zeroizing<String>>,
        revision: u64,
    },
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
    let mut continuation = None;
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
        Request::SaveContinue {
            text,
            next_revision,
        } => {
            output.write_all(&[window_receipt::SCENE_SAVE_CONTINUE])?;
            output.write_all(&(text.len() as u32).to_le_bytes())?;
            output.write_all(text.as_bytes())?;
            continuation = Some((text, next_revision));
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
        window_receipt::SAVE_ACK_COMMITTED
            if !comparing && !canceling && continuation.is_none() =>
        {
            Ok(Reply::Committed)
        }
        window_receipt::SAVE_ACK_CONTINUED => {
            let (text, expected) =
                continuation.ok_or(io::Error::other("unexpected continuation acknowledgement"))?;
            let mut bytes = [0; 8];
            input.read_exact(&mut bytes)?;
            let revision = u64::from_le_bytes(bytes);
            require(revision == expected, "invalid continuation revision")?;
            Ok(Reply::Continued { text, revision })
        }
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
        project_history::dispatch(hwnd, msg, w, l)
    }
}

fn changed_text(hwnd: HWND) -> Option<String> {
    project_metrics::draft(unsafe { GetDlgItem(hwnd, EDIT) })
}

unsafe fn status(hwnd: HWND, label: &str) {
    unsafe {
        SetDlgItemTextW(hwnd, STATUS, wide(label).as_ptr());
        SendMessageW(hwnd, WM_SIZE, 0, 0);
    }
}

unsafe fn focus_mode(hwnd: HWND) -> bool {
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) != 0 }
}

unsafe fn toggle_focus(hwnd: HWND) {
    unsafe {
        let button = GetDlgItem(hwnd, FOCUS);
        if button.is_null() {
            return;
        }
        let enabled = !focus_mode(hwnd);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, isize::from(enabled));
        SetWindowTextW(
            button,
            wide(if enabled {
                "&Fokusmodus verlassen"
            } else {
                "&Fokusmodus (F11)"
            })
            .as_ptr(),
        );
        for id in [SAVE, METRICS] {
            ShowWindow(
                GetDlgItem(hwnd, id),
                if enabled { SW_HIDE } else { SW_SHOW },
            );
        }
        if !enabled {
            project_metrics::update(hwnd, METRICS);
        }
        project_menu::refresh(hwnd);
        SendMessageW(hwnd, WM_SIZE, 0, 0);
        SetFocus(GetDlgItem(hwnd, EDIT));
    }
}

unsafe extern "system" fn procedure(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND if w == project_autosave::TOGGLE && l == 0 => {
                let save = GetDlgItem(hwnd, SAVE);
                if !save.is_null()
                    && IsWindowEnabled(save) != 0
                    && GetWindowLongW(GetDlgItem(hwnd, EDIT), GWL_STYLE) & ES_READONLY == 0
                {
                    project_autosave::toggle(hwnd);
                }
                0
            }
            WM_COMMAND if l == 0 && matches!(w, UNDO | REDO) => {
                project_history::step(GetDlgItem(hwnd, EDIT), w == REDO);
                0
            }
            WM_INITMENU if w == GetMenu(hwnd) as usize => {
                project_menu::refresh(hwnd);
                0
            }
            WM_COMMAND if l == 0 && w == SELECT_PARAGRAPH => {
                let field = GetDlgItem(hwnd, EDIT);
                SetFocus(field);
                SendMessageW(field, EM_SETSEL, 0, -1);
                0
            }
            WM_COMMAND if l == 0 && w == project_menu::WHOLE_WORDS => {
                project_search::set_whole_words(!project_search::whole_words());
                SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext").as_ptr());
                project_menu::refresh(hwnd);
                0
            }
            WM_COMMAND if l == 0 && w == project_menu::SEARCH_MENU => {
                project_search::open(hwnd, false, false);
                0
            }
            WM_CLOSE => {
                PostMessageW(hwnd, REQUEST, CLOSE as usize, 0);
                0
            }
            WM_COMMAND if matches!(w, id if id == CLOSE as usize || id == IDCANCEL as usize || id == SAVE as usize || id == project_menu::CONTINUE || id == COMPARE as usize) =>
            {
                PostMessageW(hwnd, REQUEST, w, 0);
                0
            }
            WM_COMMAND if w == FOCUS as usize => {
                toggle_focus(hwnd);
                0
            }
            WM_COMMAND if w == project_search::OPEN as usize => {
                if project_search::visible(hwnd) {
                    project_search::show(hwnd, false);
                } else {
                    project_search::open(hwnd, false, true);
                }
                0
            }
            WM_COMMAND if w == project_replace::OPEN as usize => {
                if l != 0 || GetWindowLongW(GetDlgItem(hwnd, EDIT), GWL_STYLE) & ES_READONLY == 0 {
                    project_search::open(hwnd, true, l != 0);
                }
                0
            }
            WM_COMMAND if w == project_replace::APPLY as usize => {
                project_replace::apply(hwnd);
                0
            }
            WM_COMMAND if w == project_replace::ALL as usize => {
                project_replace::apply_all(hwnd);
                0
            }
            WM_COMMAND
                if w == project_search::NEXT as usize || w == project_search::PREVIOUS as usize =>
            {
                project_search::find(hwnd, w == project_search::PREVIOUS as usize);
                0
            }
            WM_COMMAND if (w & 0xffff) == SEARCH as usize && (w >> 16) == EN_CHANGE as usize => {
                project_replace::MATCH.set(None);
                SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext").as_ptr());
                0
            }
            WM_COMMAND if (w & 0xffff) == EDIT as usize && (w >> 16) == EN_CHANGE as usize => {
                project_history::changed();
                project_replace::MATCH.set(None);
                if project_search::visible(hwnd) {
                    SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext").as_ptr());
                }
                if !GetDlgItem(hwnd, METRICS).is_null() && !focus_mode(hwnd) {
                    if SetTimer(hwnd, METRICS_TIMER, 120, None) == 0 {
                        project_metrics::update(hwnd, METRICS);
                    }
                }
                0
            }
            WM_TIMER if w == METRICS_TIMER => {
                KillTimer(hwnd, METRICS_TIMER);
                if !focus_mode(hwnd) {
                    project_metrics::update(hwnd, METRICS);
                }
                0
            }
            WM_GETMINMAXINFO => {
                let min = &mut *(l as *mut MINMAXINFO);
                let dpi = GetDpiForWindow(hwnd);
                min.ptMinTrackSize.x = scale(365, dpi);
                min.ptMinTrackSize.y = scale(420, dpi);
                0
            }
            WM_SIZE => {
                let dpi = GetDpiForWindow(hwnd);
                let mut rect = RECT::default();
                GetClientRect(hwnd, &mut rect);
                let margin = scale(12, dpi);
                let gap = scale(6, dpi);
                let height = scale(28, dpi);
                let width = rect.right - 2 * margin;
                let editor = !GetDlgItem(hwnd, FOCUS).is_null();
                let focused = focus_mode(hwnd);
                let compare = GetDlgItem(hwnd, COMPARE);
                let comparing = !compare.is_null() && IsWindowVisible(compare) != 0;
                let mut bottom = rect.bottom - margin - height;
                let mut right = rect.right - margin;
                for (id, logical_width, shown) in [
                    (CLOSE, 92, true),
                    (SAVE, 96, editor && !focused),
                    (FOCUS, 158, editor),
                    (project_replace::OPEN, 140, editor),
                    (COMPARE, 176, comparing),
                ] {
                    if !shown {
                        continue;
                    }
                    let w = scale(logical_width, dpi);
                    if right - w < margin {
                        bottom -= height + gap;
                        right = rect.right - margin;
                    }
                    right -= w;
                    MoveWindow(GetDlgItem(hwnd, id), right, bottom, w, height, 1);
                    right -= gap;
                }
                if editor {
                    let status_height = if GetWindowTextLengthW(GetDlgItem(hwnd, STATUS)) > 0 {
                        2 * height
                    } else {
                        0
                    };
                    bottom -= status_height;
                    MoveWindow(
                        GetDlgItem(hwnd, STATUS),
                        margin,
                        bottom,
                        width,
                        status_height,
                        1,
                    );
                    if !focused {
                        let metrics_height = if width < scale(500, dpi) {
                            2 * height
                        } else {
                            height
                        };
                        bottom -= metrics_height;
                        MoveWindow(
                            GetDlgItem(hwnd, METRICS),
                            margin,
                            bottom,
                            width,
                            metrics_height,
                            1,
                        );
                    }
                }
                let top = project_search::layout(hwnd, dpi, width, margin, gap, height);
                MoveWindow(
                    GetDlgItem(hwnd, EDIT),
                    margin,
                    top,
                    width,
                    (bottom - gap - top).max(1),
                    1,
                );
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
    revision: u64,
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
        let window_title = if edit_mode {
            format!("Szene · Revision {revision} · Gespeichert – Bearbeiten")
        } else {
            format!("{title} · Revision {revision} – Schreibgeschützt")
        };
        let hwnd = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            class.as_ptr(),
            wide(window_title).as_ptr(),
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
                project_search::OPEN,
                "BUTTON",
                "S&uchen (Strg+F)",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (SEARCH_LABEL, "STATIC", "Such&begriff", SS_LEFT),
            (
                SEARCH,
                "EDIT",
                "",
                WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
            ),
            (
                project_search::PREVIOUS,
                "BUTTON",
                "&Zurück",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (
                project_search::NEXT,
                "BUTTON",
                "&Weiter",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (
                project_replace::OPEN,
                "BUTTON",
                "Ersetzen (Strg+H)",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (REPLACEMENT_LABEL, "STATIC", "&Ersatztext", SS_LEFT),
            (
                REPLACEMENT,
                "EDIT",
                "",
                WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
            ),
            (
                project_replace::APPLY,
                "BUTTON",
                "E&rsetzen (Enter)",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (
                project_replace::ALL,
                "BUTTON",
                "Alle (Strg+Enter) …",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (METRICS, "STATIC", "", SS_LEFT),
            (STATUS, "STATIC", "", SS_LEFT),
            (
                COMPARE,
                "BUTTON",
                "Fassungen &vergleichen",
                WS_TABSTOP | BS_PUSHBUTTON as u32,
            ),
            (
                FOCUS,
                "BUTTON",
                "&Fokusmodus (F11)",
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
            if matches!(
                id,
                SAVE | STATUS
                    | COMPARE
                    | METRICS
                    | FOCUS
                    | REPLACEMENT
                    | REPLACEMENT_LABEL
                    | project_replace::OPEN
                    | project_replace::APPLY
                    | project_replace::ALL
            ) && !edit_mode
            {
                continue;
            }
            let child = CreateWindowExW(
                0,
                wide(kind).as_ptr(),
                wide(label).as_ptr(),
                WS_CHILD
                    | (if matches!(
                        id,
                        COMPARE
                            | SEARCH
                            | SEARCH_LABEL
                            | REPLACEMENT
                            | REPLACEMENT_LABEL
                            | project_replace::APPLY
                            | project_replace::ALL
                            | project_search::PREVIOUS
                            | project_search::NEXT
                    ) {
                        0
                    } else {
                        WS_VISIBLE
                    })
                    | control_style,
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
        // Lay out the wrapping EDIT before loading prose. At its initial one-pixel width,
        // wrapping a long paragraph can exceed the authenticated surface deadline.
        require(project_menu::install(hwnd, edit_mode), "install scene menu")?;
        project_autosave::stop(hwnd);
        project_autosave::retry(hwnd, false);
        project_menu::refresh(hwnd);
        SendMessageW(hwnd, WM_SIZE, 0, 0);
        let field = GetDlgItem(hwnd, EDIT);
        // Apply the UTF-8 storage limit at submission. During editing allow a longer
        // draft to be shortened explicitly, without submitting a truncated prefix.
        for id in [EDIT, SEARCH, REPLACEMENT] {
            let control = GetDlgItem(hwnd, id);
            if control.is_null() {
                continue;
            }
            require(
                windows_sys::Win32::UI::Shell::SetWindowSubclass(control, Some(scene_input), 1, 0)
                    != 0,
                "protect project text",
            )?;
            SendMessageW(
                control,
                EM_SETLIMITTEXT,
                project_metrics::EDIT_UNIT_LIMIT,
                0,
            );
        }
        require(
            SetWindowTextW(field, wide(text.replace('\n', "\r\n")).as_ptr()) != 0
                && matches_preview_text(hwnd, text),
            "project scene readback",
        )?;
        if edit_mode {
            project_metrics::update(hwnd, METRICS);
        }
        let _history = edit_mode.then(|| project_history::start(field));
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
        edit_loop(hwnd, field, revision, text, channel)
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
        let repeated = msg.lParam & (1 << 30) != 0;
        let own_control = msg.hwnd == hwnd || IsChild(hwnd, msg.hwnd) != 0;
        // The dialog manager also handles child mnemonics (e.g. Such&begriff).
        // Give the native menu its top-level access keys first.
        if own_control
            && msg.message == WM_SYSKEYDOWN
            && msg.lParam & (1 << 29) != 0
            && [VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN]
                .iter()
                .all(|key| GetKeyState(i32::from(*key)) >= 0)
            && (msg.wParam == usize::from(VK_D)
                || msg.wParam == usize::from(VK_B)
                || (msg.wParam == usize::from(VK_A) && !GetDlgItem(hwnd, FOCUS).is_null()))
        {
            if !repeated {
                SendMessageW(
                    hwnd,
                    WM_SYSCOMMAND,
                    SC_KEYMENU as usize,
                    (msg.wParam as u8).to_ascii_lowercase() as isize,
                );
            }
            return;
        }
        if msg.message == WM_KEYDOWN && own_control {
            let field = GetDlgItem(hwnd, EDIT);
            let save = GetDlgItem(hwnd, SAVE);
            let key = msg.wParam;
            let ctrl = GetKeyState(i32::from(VK_CONTROL)) < 0;
            let shift = GetKeyState(i32::from(VK_SHIFT)) < 0;
            let plain = [VK_MENU, VK_LWIN, VK_RWIN]
                .iter()
                .all(|modifier| GetKeyState(i32::from(*modifier)) >= 0);
            let ctrl_only = ctrl && !shift && plain;
            if ctrl_only
                && key == usize::from(VK_RETURN)
                && msg.hwnd == field
                && GetFocus() == field
            {
                if !repeated && !save.is_null() && IsWindowEnabled(save) != 0 {
                    PostMessageW(hwnd, REQUEST, project_menu::CONTINUE, 0);
                }
                return;
            }
            if ctrl_only
                && matches!(key, k if k == usize::from(VK_Z) || k == usize::from(VK_Y))
                && msg.hwnd == field
                && GetFocus() == field
            {
                if !repeated && !save.is_null() {
                    project_history::step(field, key == usize::from(VK_Y));
                }
                return;
            }
            if ctrl_only && key == usize::from(VK_H) {
                if !repeated {
                    project_search::open(hwnd, true, false);
                }
                return;
            }
            if ctrl_only && key == usize::from(VK_F) {
                if !repeated {
                    project_search::open(hwnd, false, false);
                }
                return;
            }
            let replacement = GetDlgItem(hwnd, REPLACEMENT);
            if key == usize::from(VK_RETURN) && !replacement.is_null() && msg.hwnd == replacement {
                if !repeated && plain && !shift && GetFocus() == replacement {
                    if ctrl {
                        project_replace::apply_all(hwnd);
                    } else {
                        project_replace::apply(hwnd);
                    }
                    // Repeats from a held Enter stay in this field instead of editing prose.
                    SetFocus(replacement);
                }
                return;
            }
            if plain
                && !ctrl
                && (key == usize::from(VK_F3)
                    || (key == usize::from(VK_RETURN) && msg.hwnd == GetDlgItem(hwnd, SEARCH)))
            {
                if !repeated {
                    project_search::find(hwnd, shift);
                }
                return;
            }
            if key == usize::from(VK_ESCAPE) && (project_search::visible(hwnd) || repeated) {
                if !repeated {
                    project_search::show(hwnd, false);
                }
                return;
            }
            if ctrl_only && key == usize::from(VK_A) && msg.hwnd == field && GetFocus() == field {
                if !repeated {
                    SendMessageW(hwnd, WM_COMMAND, SELECT_PARAGRAPH, 0);
                }
                return;
            }
            if ctrl_only && key == usize::from(VK_S) {
                if !repeated && !save.is_null() && IsWindowEnabled(save) != 0 {
                    PostMessageW(hwnd, REQUEST, SAVE as usize, 0);
                }
                return;
            }
            if !save.is_null()
                && (key == usize::from(VK_F11)
                    || (key == usize::from(VK_ESCAPE) && (focus_mode(hwnd) || repeated)))
            {
                if !repeated {
                    toggle_focus(hwnd);
                }
                return;
            }
        }
        if IsDialogMessageW(hwnd, msg) == 0 {
            TranslateMessage(msg);
            DispatchMessageW(msg);
        }
    }
}

unsafe fn edit_loop(
    hwnd: HWND,
    field: HWND,
    mut revision: u64,
    text: &str,
    channel: Channel,
) -> io::Result<()> {
    unsafe {
        let mut saved: Option<Arc<Zeroizing<String>>> = None;
        let mut submitted: Option<Arc<Zeroizing<String>>> = None;
        let mut previous_title = None;
        let mut update_title = true;
        let mut pending = false;
        let mut pending_continue = false;
        let mut background_pending = false;
        let mut deferred = None;
        let mut available = true;
        let mut compare_available = false;
        let mut review: Option<project_review::Review> = None;
        let mut confirming = false;
        let committed = loop {
            if project_save_status::take_changed() || update_title {
                use project_save_status::Phase;
                let phase = if !available {
                    if compare_available {
                        Phase::Conflict
                    } else {
                        Phase::Unknown
                    }
                } else if pending {
                    Phase::Pending(submitted.as_ref().map(|s| s.as_str()))
                } else if project_autosave::retry_required() {
                    Phase::Failed
                } else {
                    Phase::Idle(saved.as_ref().map_or(text, |s| s.as_str()))
                };
                project_save_status::render(hwnd, revision, phase, &mut previous_title);
                update_title = false;
            }
            // Give an explicit close/save priority over a due follow-up autosave.
            let message = if !pending && let Some(action) = deferred.take() {
                MSG {
                    hwnd,
                    message: REQUEST,
                    wParam: action,
                    ..Default::default()
                }
            } else {
                next_message()?
            };
            if message.hwnd == hwnd && message.message == project_save_status::REFRESH {
                continue;
            }
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
                submitted = None;
                update_title = true;
                let background = std::mem::take(&mut background_pending);
                if matches!(&outcome, Err(_) | Ok(Reply::Failed | Reply::Conflict)) {
                    project_autosave::retry(hwnd, true);
                    // A queued close still asks about the draft. A failed write must
                    // not be retried before the user has seen its result.
                    if deferred != Some(CLOSE as usize) {
                        deferred = None;
                    }
                }
                if matches!(outcome, Ok(Reply::Committed)) {
                    break true;
                }
                pending = false;
                if pending_continue {
                    EnableWindow(hwnd, 1);
                    pending_continue = false;
                }
                let status = match outcome {
                    Ok(Reply::Continued {
                        text,
                        revision: confirmed,
                    }) => {
                        saved = Some(text);
                        revision = confirmed;
                        project_autosave::retry(hwnd, false);
                        if background {
                            "Automatischer Zwischenstand bestätigt."
                        } else {
                            "Zwischenstand bestätigt."
                        }
                    }
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
                                false,
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
                                false,
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
                if !background {
                    SendMessageW(field, EM_SETREADONLY, 0, 0);
                    EnableWindow(GetDlgItem(hwnd, project_replace::APPLY), 1);
                    EnableWindow(GetDlgItem(hwnd, project_replace::ALL), 1);
                    SetFocus(field);
                }
                project_menu::refresh(hwnd);
                SetWindowTextW(GetDlgItem(hwnd, STATUS), wide(status).as_ptr());
                SendMessageW(hwnd, WM_SIZE, 0, 0);
                if deferred.is_none() {
                    project_autosave::resume(hwnd);
                }
                continue;
            }
            let automatic = message.hwnd == hwnd
                && message.message == WM_TIMER
                && message.wParam == project_autosave::TIMER;
            if automatic && !project_autosave::due(hwnd, pending) {
                continue;
            }
            if automatic || (message.hwnd == hwnd && message.message == REQUEST) {
                if pending {
                    if background_pending {
                        let id = message.wParam;
                        let close = id == CLOSE as usize || id == IDCANCEL as usize;
                        if close || (id == SAVE as usize && deferred != Some(CLOSE as usize)) {
                            deferred = Some(if close { CLOSE as usize } else { id });
                        } else if id == project_menu::CONTINUE && deferred.is_none() {
                            deferred = Some(id);
                        }
                    }
                    continue;
                }
                let close = message.wParam == CLOSE as usize || message.wParam == IDCANCEL as usize;
                let continuing = automatic || message.wParam == project_menu::CONTINUE;
                let baseline = saved.as_ref().map_or(text, |s| s.as_str());
                let draft = changed_text(hwnd).map(Zeroizing::new);
                if message.wParam == SAVE as usize
                    && saved.is_some()
                    && available
                    && draft.as_ref().is_some_and(|s| s.as_str() == baseline)
                {
                    // The latest write is already durable, including a queued Strg+S.
                    break false;
                }
                if close {
                    if draft.as_deref().map(String::as_str) == Some(baseline) {
                        break false;
                    }
                    match project_close::choose(hwnd, available) {
                        project_close::Choice::Discard => break false,
                        project_close::Choice::Continue => {
                            SetFocus(field);
                            project_autosave::changed(hwnd);
                            continue;
                        }
                        project_close::Choice::Save => {}
                    }
                }
                let compare = message.wParam == COMPARE as usize && compare_available;
                if !compare
                    && (!available || (!close && !continuing && message.wParam != SAVE as usize))
                {
                    continue;
                }
                let Some(draft) = draft else {
                    if automatic {
                        status(
                            hwnd,
                            "Automatisches Speichern wartet auf einen gültigen Entwurf.",
                        );
                        continue;
                    }
                    MessageBoxW(hwnd, wide("Der Entwurf enthält ungültigen Text oder überschreitet die Grenze von 64 KiB UTF-8.").as_ptr(), wide("Speichern").as_ptr(), MB_OK | MB_ICONWARNING);
                    SetFocus(field);
                    continue;
                };
                let (command, status) = if compare {
                    review = Some(project_review::Review::new(draft.clone()));
                    (Request::Compare(draft), "Vergleich wird geladen …")
                } else if continuing {
                    if draft.as_str() == baseline {
                        if !project_autosave::retry_required() {
                            status(hwnd, "Keine Änderungen zu speichern.");
                        }
                        continue;
                    }
                    let Some(next_revision) = revision.checked_add(1) else {
                        available = false;
                        project_autosave::stop(hwnd);
                        EnableWindow(GetDlgItem(hwnd, SAVE), 0);
                        project_menu::refresh(hwnd);
                        SetWindowTextW(
                            GetDlgItem(hwnd, STATUS),
                            wide("Das Revisionslimit ist erreicht. Der Entwurf bleibt offen.")
                                .as_ptr(),
                        );
                        SendMessageW(hwnd, WM_SIZE, 0, 0);
                        continue;
                    };
                    let text = Arc::new(draft);
                    submitted = Some(Arc::clone(&text));
                    (
                        Request::SaveContinue {
                            text,
                            next_revision,
                        },
                        if automatic {
                            "Automatisches Speichern läuft …"
                        } else {
                            "Zwischenstand wird gespeichert …"
                        },
                    )
                } else {
                    (Request::Save(draft), "Wird gespeichert …")
                };
                pending = start_request(hwnd, field, &channel, command, status, automatic);
                update_title = true;
                background_pending = pending && automatic;
                if continuing && pending && !automatic {
                    pending_continue = true;
                    EnableWindow(hwnd, 0);
                }
                if !pending {
                    submitted = None;
                    project_autosave::retry(hwnd, true);
                    review = None;
                    SetWindowTextW(
                        GetDlgItem(hwnd, STATUS),
                        wide("Die Anfrage ist fehlgeschlagen. Der Entwurf bleibt offen.").as_ptr(),
                    );
                    SendMessageW(hwnd, WM_SIZE, 0, 0);
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
    background: bool,
) -> bool {
    unsafe {
        if SetTimer(hwnd, SAVE_TIMER, 100, None) == 0 {
            return false;
        }
        if channel.request.send(command).is_err() {
            KillTimer(hwnd, SAVE_TIMER);
            return false;
        }
        if !background {
            SendMessageW(field, EM_SETREADONLY, 1, 0);
            EnableWindow(GetDlgItem(hwnd, project_replace::APPLY), 0);
            EnableWindow(GetDlgItem(hwnd, project_replace::ALL), 0);
        }
        project_menu::refresh(hwnd);
        SetWindowTextW(GetDlgItem(hwnd, STATUS), wide(status).as_ptr());
        SendMessageW(hwnd, WM_SIZE, 0, 0);
        true
    }
}
