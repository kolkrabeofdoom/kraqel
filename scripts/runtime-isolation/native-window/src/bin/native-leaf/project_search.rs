//! Scene-local search. Offsets refer to the actual UTF-16 EDIT, including CRLF.
use super::*;

pub(super) const OPEN: i32 = 211;
pub(super) const PREVIOUS: i32 = 212;
pub(super) const NEXT: i32 = 213;
const BAR: [i32; 4] = [SEARCH_LABEL, SEARCH, PREVIOUS, NEXT];
const SELECTION_QUERY_LIMIT: usize = 256;

thread_local! {
    static WHOLE_WORDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn whole_words() -> bool {
    WHOLE_WORDS.get()
}

pub(super) fn set_whole_words(enabled: bool) {
    WHOLE_WORDS.set(enabled);
    project_replace::MATCH.set(None);
}

fn word_char(ch: char) -> bool {
    ch.is_alphanumeric() || unicode_ident::is_xid_continue(ch)
}

fn bounded(before: Option<char>, after: Option<char>) -> bool {
    !before.is_some_and(word_char) && !after.is_some_and(word_char)
}

// Only the two adjacent scalars are needed, including possible surrogate pairs.
pub(super) fn selected_boundary(scene: &[u16], start: usize, end: usize) -> bool {
    if !whole_words() {
        return true;
    }
    let before = &scene[..start];
    let after = &scene[end..];
    // Treating a malformed UTF-16 neighbor as a boundary would authorize a
    // replacement that the UTF-8 search path cannot produce.
    let width = if before.last().is_some_and(|c| (0xdc00..=0xdfff).contains(c)) {
        if before.len() < 2 || !(0xd800..=0xdbff).contains(&before[before.len() - 2]) {
            return false;
        }
        2
    } else {
        before.len().min(1)
    };
    let previous = char::decode_utf16(before[before.len() - width..].iter().copied())
        .next()
        .transpose();
    let next = char::decode_utf16(after.iter().copied()).next().transpose();
    matches!((previous, next), (Ok(left), Ok(right)) if bounded(left, right))
}

// Accepted matches are disjoint. A rejected partial match may overlap a later
// whole match (for example "a a" in "xa a a").
pub(super) fn hits<'a>(
    scene: &'a str,
    query: &'a str,
    whole: bool,
) -> impl Iterator<Item = std::ops::Range<usize>> + 'a {
    let mut base = 0;
    let mut candidates = scene.match_indices(query);
    std::iter::from_fn(move || {
        if query.is_empty() {
            return None;
        }
        loop {
            let (relative, matched) = candidates.next()?;
            let start = base + relative;
            let end = start + matched.len();
            if !whole
                || bounded(
                    scene[..start].chars().next_back(),
                    scene[end..].chars().next(),
                )
            {
                return Some(start..end);
            }
            // Resume at the next possible left boundary, which may lie inside the rejected range.
            let (offset, separator) = scene[start..]
                .char_indices()
                .find(|(_, ch)| !word_char(*ch))?;
            base = start + offset + separator.len_utf8();
            candidates = scene[base..].match_indices(query);
        }
    })
}

pub(super) unsafe fn visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(GetDlgItem(hwnd, SEARCH)) != 0 }
}

// Only an explicit search/replace entry may adopt selected manuscript text.
// F3 and layout/focus changes must keep the existing query.
pub(super) unsafe fn open(hwnd: HWND, replace: bool, from_button: bool) {
    unsafe {
        if replace && GetDlgItem(hwnd, REPLACEMENT).is_null() {
            return;
        }
        let field = GetDlgItem(hwnd, EDIT);
        // A real click sets focus on the opening button before BN_CLICKED.
        if from_button || GetFocus() == field {
            seed_from_selection(hwnd, field);
        }
        if replace {
            project_replace::show(hwnd, true);
        } else {
            show(hwnd, true);
        }
    }
}

unsafe fn seed_from_selection(hwnd: HWND, field: HWND) {
    unsafe {
        let range @ (start, end) = selection(field);
        if start >= end || end - start > SELECTION_QUERY_LIMIT {
            return;
        }
        let Some(mut scene) = project_metrics::units(field, project_metrics::EDIT_UNIT_LIMIT)
        else {
            return;
        };
        let Some(selected) = scene.get(start..end) else {
            return;
        };
        if char::decode_utf16(selected.iter().copied()).any(|c| {
            !matches!(c, Ok(ch) if (!ch.is_control() || ch == '\t') && !matches!(ch, '\u{2028}' | '\u{2029}'))
        }) {
            return;
        }
        // units() retains capacity for the trailing NUL, even at the end of the scene.
        // Keep the full allocation length so Zeroizing clears the whole scene on drop.
        let replaceable = selected_boundary(&scene, start, end);
        if end == scene.len() {
            scene.push(0);
        } else {
            scene[end] = 0;
        }
        let query = GetDlgItem(hwnd, SEARCH);
        if SetWindowTextW(query, scene[start..].as_ptr()) != 0
            && replaceable
            && !GetDlgItem(hwnd, project_replace::APPLY).is_null()
            && project_metrics::units(query, SELECTION_QUERY_LIMIT)
                .is_some_and(|actual| actual.as_slice() == &scene[start..end])
            && selection(field) == range
        {
            // EN_CHANGE invalidates the previous match synchronously. Only the checked
            // range can authorize an explicit single replacement, which rechecks it again.
            project_replace::MATCH.set(Some(range));
        }
    }
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
        let mut previous_byte = 0;
        let mut previous_unit = 0;
        let query_units = query.encode_utf16().count();
        for range in hits(&content, &query, whole_words()) {
            let a = previous_unit + content[previous_byte..range.start].encode_utf16().count();
            let b = a + query_units;
            previous_byte = range.end;
            previous_unit = b;
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
