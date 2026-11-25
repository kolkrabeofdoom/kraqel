//! Explicit replacements in the existing local edit buffer.
use super::*;

pub(super) const OPEN: i32 = 214;
pub(super) const APPLY: i32 = 215;
pub(super) const ALL: i32 = 216;

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
        for id in [REPLACEMENT_LABEL, REPLACEMENT, APPLY, ALL] {
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

struct Replacement {
    whole: bool,
    scene: Zeroizing<String>,
    query: Zeroizing<String>,
    value: Zeroizing<String>,
}

impl Replacement {
    unsafe fn read(hwnd: HWND) -> Option<Self> {
        unsafe {
            Some(Self {
                whole: project_search::whole_words(),
                scene: project_search::text(GetDlgItem(hwnd, EDIT))?,
                query: project_search::text(GetDlgItem(hwnd, SEARCH))?,
                value: project_search::text(GetDlgItem(hwnd, REPLACEMENT))?,
            })
        }
    }

    unsafe fn current(&self, hwnd: HWND) -> bool {
        unsafe {
            self.whole == project_search::whole_words()
                && [
                    (EDIT, &self.scene),
                    (SEARCH, &self.query),
                    (REPLACEMENT, &self.value),
                ]
                .into_iter()
                .all(|(id, expected)| {
                    project_search::text(GetDlgItem(hwnd, id))
                        .is_some_and(|actual| actual == *expected)
                })
        }
    }

    fn size(&self, count: usize) -> Option<usize> {
        let removed = count.checked_mul(self.query.encode_utf16().count())?;
        let added = count.checked_mul(self.value.encode_utf16().count())?;
        self.scene
            .encode_utf16()
            .count()
            .checked_sub(removed)?
            .checked_add(added)
            .filter(|n| *n <= project_metrics::EDIT_UNIT_LIMIT)
    }

    fn render(&self, length: usize) -> Zeroizing<Vec<u16>> {
        let mut result = Zeroizing::new(Vec::with_capacity(length + 1));
        let mut end = 0;
        for range in project_search::hits(&self.scene, &self.query, self.whole) {
            result.extend(self.scene[end..range.start].encode_utf16());
            result.extend(self.value.encode_utf16());
            end = range.end;
        }
        result.extend(self.scene[end..].encode_utf16());
        result.push(0);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_word_replacement_preserves_partial_and_combining_matches() {
        let plan = Replacement {
            whole: true,
            scene: Zeroizing::new("xa a a, a a_ äa a, a a\u{301} 🪶a a!".into()),
            query: Zeroizing::new("a a".into()),
            value: Zeroizing::new("ß".into()),
        };
        let count = project_search::hits(&plan.scene, &plan.query, true).count();
        assert_eq!(count, 2);
        let size = plan.size(count).unwrap();
        let result = plan.render(size);
        assert_eq!(result.len(), size + 1);
        assert_eq!(
            String::from_utf16(&result[..size]).unwrap(),
            "xa ß, a a_ äa a, a a\u{301} 🪶ß!"
        );
    }

    #[test]
    fn bulk_size_checks_the_expanded_utf16_before_allocation() {
        let mut plan = Replacement {
            whole: false,
            scene: Zeroizing::new("a".repeat(4096)),
            query: Zeroizing::new("a".into()),
            value: Zeroizing::new("🪶".repeat(512)),
        };
        assert_eq!(plan.size(4096), Some(project_metrics::EDIT_UNIT_LIMIT));
        plan.value.push('a');
        assert_eq!(plan.size(4096), None);
    }
}

pub(super) unsafe fn apply_all(hwnd: HWND) {
    unsafe {
        let field = GetDlgItem(hwnd, EDIT);
        let button = GetDlgItem(hwnd, ALL);
        let allowed = || {
            !button.is_null()
                && IsWindowEnabled(hwnd) != 0
                && IsWindowVisible(button) != 0
                && IsWindowEnabled(button) != 0
                && GetWindowLongW(field, GWL_STYLE) & ES_READONLY == 0
        };
        if !allowed() {
            return;
        }
        let message = (|| {
            let Some(plan) = Replacement::read(hwnd) else {
                return "&Szenentext · Eingaben prüfen";
            };
            if plan.query.is_empty() {
                SetFocus(GetDlgItem(hwnd, SEARCH));
                return "&Szenentext · Suchbegriff eingeben";
            }
            if plan.value.chars().any(|c| c.is_control() && c != '\t') {
                return "&Szenentext · Ersatztext prüfen";
            }
            let count = project_search::hits(&plan.scene, &plan.query, plan.whole).count();
            if count == 0 {
                return "&Szenentext · Kein Treffer";
            }
            if plan.query == plan.value {
                return "&Szenentext · Ersatztext ist identisch";
            }
            let Some(length) = plan.size(count) else {
                return "&Szenentext · Eingabegrenze überschritten";
            };
            let prompt = format!("{count} Treffer im lokalen Entwurf ersetzen?");
            if MessageBoxW(
                hwnd,
                wide(prompt).as_ptr(),
                wide("Alle Treffer ersetzen?").as_ptr(),
                MB_YESNO | MB_DEFBUTTON2 | MB_ICONQUESTION,
            ) != IDYES
            {
                return "&Szenentext · Ersetzen abgebrochen";
            }
            // The modal confirmation pumps messages. Its count only authorizes the
            // exact scene, query, and replacement reviewed above.
            if !allowed() || !plan.current(hwnd) {
                return "&Szenentext · Eingaben erneut prüfen";
            }
            let result = plan.render(length);
            SetFocus(field);
            // Keep the user's selection from before the implementation selects all.
            project_history::observe(field, false, || {
                SendMessageW(field, EM_SETSEL, 0, -1);
                SendMessageW(field, EM_REPLACESEL, 0, result.as_ptr() as isize);
            });
            MATCH.set(None);
            if GetWindowTextLengthW(field) as usize != length
                || project_search::selection(field) != (length, length)
            {
                return "&Szenentext · Ersetzung prüfen";
            }
            "&Szenentext · Alle Treffer ersetzt"
        })();
        SetDlgItemTextW(hwnd, LABEL, wide(message).as_ptr());
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
            if scene.get(start..end) != Some(query.as_slice())
                || !project_search::selected_boundary(&scene, start, end)
            {
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
            SendMessageW(field, EM_REPLACESEL, 0, value.as_ptr() as isize);
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
