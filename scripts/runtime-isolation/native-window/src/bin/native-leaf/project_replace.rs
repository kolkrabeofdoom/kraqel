//! Replace one freshly located match in the existing local edit buffer.
use super::*;

pub(super) const OPEN: i32 = 214;
pub(super) const APPLY: i32 = 215;

thread_local! {
    // A role owns exactly one scene window. Never retain manuscript or query text here.
    pub(super) static MATCH: std::cell::Cell<Option<(usize, usize)>> = const { std::cell::Cell::new(None) };
}

pub(super) unsafe fn show(hwnd: HWND, open: bool) {
    unsafe {
        let replacement = GetDlgItem(hwnd, REPLACEMENT);
        if replacement.is_null() {
            return;
        }
        for id in [REPLACEMENT_LABEL, REPLACEMENT, APPLY] {
            ShowWindow(GetDlgItem(hwnd, id), if open { SW_SHOWNA } else { SW_HIDE });
        }
        if open {
            let need_query = GetWindowTextLengthW(GetDlgItem(hwnd, SEARCH)) == 0;
            project_search::show(hwnd, true);
            if !need_query {
                SetFocus(replacement);
                SendMessageW(replacement, EM_SETSEL, 0, -1);
            }
        }
    }
}

pub(super) unsafe fn apply(hwnd: HWND) {
    unsafe {
        let replacement = GetDlgItem(hwnd, REPLACEMENT);
        let field = GetDlgItem(hwnd, EDIT);
        if replacement.is_null()
            || IsWindowVisible(replacement) == 0
            || GetWindowLongW(field, GWL_STYLE) & ES_READONLY != 0
        {
            return;
        }
        let selection = project_search::selection(field);
        let stale = "&Szenentext · Treffer erneut suchen";
        let message = (|| {
            let (start, end) = selection;
            if MATCH.get() != Some(selection) || start >= end {
                return stale;
            }
            let limit = project_metrics::EDIT_UNIT_LIMIT;
            let Some(scene) = project_metrics::units(field, limit) else {
                return stale;
            };
            let Some(query) = project_metrics::units(GetDlgItem(hwnd, SEARCH), limit) else {
                return stale;
            };
            if scene.get(start..end) != Some(query.as_slice()) {
                return stale;
            }
            let Some(mut value) = project_metrics::units(replacement, limit) else {
                return "&Szenentext · Ersatztext prüfen";
            };
            let new_length = scene.len() - (end - start) + value.len();
            if new_length > limit {
                return "&Szenentext · Eingabegrenze überschritten";
            }
            if char::decode_utf16(value.iter().copied())
                .any(|c| !matches!(c, Ok(ch) if !ch.is_control() || ch == '\t'))
            {
                return "&Szenentext · Ersatztext prüfen";
            }
            // units() leaves room for the terminator, avoiding another plaintext allocation.
            let caret = start + value.len();
            value.push(0);
            SetFocus(field);
            SendMessageW(field, EM_REPLACESEL, 1, value.as_ptr() as isize);
            MATCH.set(None);
            if GetWindowTextLengthW(field) as usize != new_length
                || project_search::selection(field) != (caret, caret)
            {
                return "&Szenentext · Ersetzung prüfen";
            }
            "&Szenentext · Treffer ersetzt"
        })();
        SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
    }
}
