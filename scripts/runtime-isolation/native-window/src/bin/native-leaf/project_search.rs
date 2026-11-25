//! Scene-local search. Offsets refer to the actual UTF-16 EDIT, including CRLF.
use super::*;

pub(super) const OPEN: i32 = 211;
pub(super) const PREVIOUS: i32 = 212;
pub(super) const NEXT: i32 = 213;
const BAR: [i32; 4] = [SEARCH_LABEL, SEARCH, PREVIOUS, NEXT];

pub(super) unsafe fn visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(GetDlgItem(hwnd, SEARCH)) != 0 }
}

pub(super) unsafe fn show(hwnd: HWND, open: bool) {
    unsafe {
        if !open {
            project_replace::MATCH.set(None);
            project_replace::show(hwnd, false);
        }
        for id in BAR {
            ShowWindow(GetDlgItem(hwnd, id), if open { SW_SHOWNA } else { SW_HIDE });
        }
        SetDlgItemTextW(
            hwnd,
            OPEN,
            wide(if open {
                "Suche schließen"
            } else {
                "S&uchen (Strg+F)"
            })
            .as_ptr(),
        );
        SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext").as_ptr());
        SendMessageW(hwnd, WM_SIZE, 0, 0);
        let field = GetDlgItem(hwnd, if open { SEARCH } else { EDIT });
        SetFocus(field);
        if open {
            SendMessageW(field, EM_SETSEL, 0, -1);
        }
    }
}

// Reserve once so conversion never frees a partial plaintext buffer.
pub(super) fn text(field: HWND) -> Option<Zeroizing<String>> {
    let units = project_metrics::units(field, project_metrics::EDIT_UNIT_LIMIT)?;
    let mut result = Zeroizing::new(String::with_capacity(units.len() * 3));
    for ch in char::decode_utf16(units.iter().copied()) {
        result.push(ch.ok()?);
    }
    Some(result)
}

pub(super) unsafe fn selection(field: HWND) -> (usize, usize) {
    let (mut start, mut end) = (0u32, 0u32);
    unsafe {
        SendMessageW(
            field,
            EM_GETSEL,
            (&mut start as *mut u32) as usize,
            (&mut end as *mut u32) as isize,
        );
    }
    (start as usize, end as usize)
}

pub(super) unsafe fn find(hwnd: HWND, backwards: bool) {
    unsafe {
        project_replace::MATCH.set(None);
        if !visible(hwnd) {
            show(hwnd, true);
        }
        let field = GetDlgItem(hwnd, EDIT);
        let query_field = GetDlgItem(hwnd, SEARCH);
        let Some(query) = text(query_field) else {
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide("&Szenentext · Suchbegriff prüfen").as_ptr(),
            );
            return;
        };
        if query.is_empty() {
            SetFocus(query_field);
            return;
        }
        let Some(content) = text(field) else {
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide("&Szenentext · Suche nicht möglich").as_ptr(),
            );
            return;
        };
        let (start, end) = selection(field);
        let mut count = 0;
        let mut selected = None;
        let mut wrapped = None;
        for (a, b) in utf16_hits(&content, &query) {
            count += 1;
            let hit = (count, a, b);
            if backwards {
                wrapped = Some(hit);
                if b <= start {
                    selected = Some(hit);
                }
            } else {
                wrapped.get_or_insert(hit);
                if a >= end {
                    selected.get_or_insert(hit);
                }
            }
        }
        if let Some((number, a, b)) = selected.or(wrapped) {
            SetFocus(field);
            SendMessageW(field, EM_SETSEL, a, b as isize);
            if !GetDlgItem(hwnd, project_replace::APPLY).is_null() {
                project_replace::MATCH.set(Some((a, b)));
            }
            SendMessageW(
                field,
                windows_sys::Win32::UI::Controls::EM_SCROLLCARET,
                0,
                0,
            );
            SetDlgItemTextW(
                hwnd,
                LABEL,
                wide(format!("&Szenentext · Treffer {number} von {count}")).as_ptr(),
            );
        } else {
            SetDlgItemTextW(hwnd, LABEL, wide("&Szenentext · Kein Treffer").as_ptr());
            SetFocus(query_field);
        }
    }
}

pub(super) unsafe fn layout(
    hwnd: HWND,
    dpi: u32,
    width: i32,
    margin: i32,
    gap: i32,
    height: i32,
) -> i32 {
    unsafe {
        let button = scale(124, dpi);
        MoveWindow(
            GetDlgItem(hwnd, LABEL),
            margin,
            margin,
            width - button - gap,
            height,
            1,
        );
        MoveWindow(
            GetDlgItem(hwnd, OPEN),
            margin + width - button,
            margin,
            button,
            height,
            1,
        );
        let mut top = margin + height + gap;
        if visible(hwnd) {
            let label = scale(76, dpi);
            let nav = scale(62, dpi);
            let input = width - label - 2 * nav - 3 * gap;
            let mut x = margin;
            for (id, w) in [
                (SEARCH_LABEL, label),
                (SEARCH, input),
                (PREVIOUS, nav),
                (NEXT, nav),
            ] {
                MoveWindow(GetDlgItem(hwnd, id), x, top, w, height, 1);
                x += w + gap;
            }
            top += height + gap;
            if IsWindowVisible(GetDlgItem(hwnd, REPLACEMENT)) != 0 {
                let button = 2 * nav + gap;
                let wrap = width < scale(520, dpi);
                let edit_width = if wrap {
                    width - label - gap
                } else {
                    width - label - 2 * button - 3 * gap
                };
                let mut x = margin;
                for (id, w) in [
                    (REPLACEMENT_LABEL, label),
                    (REPLACEMENT, edit_width),
                    (project_replace::APPLY, button),
                    (project_replace::ALL, button),
                ] {
                    if id == project_replace::APPLY && wrap {
                        x = margin + width - 2 * button - gap;
                        top += height + gap;
                    }
                    MoveWindow(GetDlgItem(hwnd, id), x, top, w, height, 1);
                    x += w + gap;
                }
                top += height + gap;
            }
        }
        top
    }
}
