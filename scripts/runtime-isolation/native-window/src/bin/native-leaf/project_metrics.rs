//! Count the same normalized plain text that the product window submits.
pub(super) fn word_separator(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

#[cfg(feature = "manuscript-window")]
pub(super) use product::{EDIT_UNIT_LIMIT, draft, units, update};

#[cfg(feature = "manuscript-window")]
mod product {
    use super::super::*;
    use zeroize::Zeroizing;

    pub(in super::super) const EDIT_UNIT_LIMIT: usize = 4 * 1024 * 1024;

    struct Metrics {
        words: usize,
        bytes: usize,
        valid_controls: bool,
    }

    fn measure(units: &[u16], mut accept: impl FnMut(char)) -> Option<Metrics> {
        let mut metrics = Metrics {
            words: 0,
            bytes: 0,
            valid_controls: true,
        };
        let mut in_word = false;
        let mut chars = char::decode_utf16(units.iter().copied()).peekable();
        while let Some(decoded) = chars.next() {
            let mut c = decoded.ok()?;
            if c == '\r' && matches!(chars.peek(), Some(Ok('\n'))) {
                chars.next();
                c = '\n';
            }
            let word = !super::word_separator(c);
            metrics.words += usize::from(word && !in_word);
            in_word = word;
            metrics.bytes += c.len_utf8();
            metrics.valid_controls &= !c.is_control() || matches!(c, '\n' | '\t');
            accept(c);
        }
        Some(metrics)
    }

    pub(in super::super) fn units(field: HWND, limit: usize) -> Option<Zeroizing<Vec<u16>>> {
        unsafe {
            if field.is_null() {
                return None;
            }
            SetLastError(0);
            let length = usize::try_from(GetWindowTextLengthW(field)).ok()?;
            if length > limit || (length == 0 && GetLastError() != 0) {
                return None;
            }
            let mut buf = Zeroizing::new(vec![0; length + 1]);
            SetLastError(0);
            let copied = GetWindowTextW(field, buf.as_mut_ptr(), buf.len() as i32);
            if copied as usize != length || (copied == 0 && GetLastError() != 0) {
                return None;
            }
            buf.truncate(length);
            Some(buf)
        }
    }

    pub(in super::super) fn draft(field: HWND) -> Option<String> {
        let input = units(field, PROJECT_EDIT_BYTES * 2)?;
        // One UTF-16 code unit accounts for at most three UTF-8 bytes.
        // Reserve once so growing an edit never frees a partial plaintext buffer.
        let capacity = (input.len() * 3).min(PROJECT_EDIT_BYTES);
        let mut text = Zeroizing::new(String::with_capacity(capacity));
        let size = measure(&input, |c| {
            if text.len() + c.len_utf8() <= capacity {
                text.push(c);
            }
        })?;
        (size.valid_controls && size.bytes <= PROJECT_EDIT_BYTES)
            .then(|| std::mem::take(&mut *text))
    }

    pub(in super::super) fn update(hwnd: HWND, label_id: i32) {
        unsafe {
            let size = units(GetDlgItem(hwnd, EDIT), EDIT_UNIT_LIMIT)
                .and_then(|input| measure(&input, |_| {}));
            let label = match size {
                Some(m) => {
                    let noun = if m.words == 1 { "Wort" } else { "Wörter" };
                    let status = if !m.valid_controls {
                        " · Ungültige Steuerzeichen"
                    } else if m.bytes > PROJECT_EDIT_BYTES {
                        " · Speichergrenze überschritten"
                    } else {
                        ""
                    };
                    format!(
                        "{} {} · {} / {} UTF-8-Bytes{}",
                        m.words, noun, m.bytes, PROJECT_EDIT_BYTES, status
                    )
                }
                None => "Textumfang nicht bestimmbar · Eingabe prüfen".to_owned(),
            };
            SetWindowTextW(GetDlgItem(hwnd, label_id), wide(label).as_ptr());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn metrics_follow_stored_text_and_m1_word_boundaries() {
            for (edit, normalized, words, valid) in [
                ("", "", 0, true),
                ("Ä🦉\r\nx\u{feff}y\t", "Ä🦉\nx\u{feff}y\t", 3, true),
                ("a\u{85}b\rc", "a\u{85}b\rc", 2, false),
            ] {
                let mut text = String::new();
                let m =
                    measure(&edit.encode_utf16().collect::<Vec<_>>(), |c| text.push(c)).unwrap();
                assert_eq!(
                    (text.as_str(), m.words, m.bytes, m.valid_controls),
                    (normalized, words, normalized.len(), valid)
                );
            }
            assert!(measure(&[0xd800], |_| {}).is_none());
        }
    }
}
