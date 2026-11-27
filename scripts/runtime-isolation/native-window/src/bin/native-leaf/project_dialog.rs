// Shared aligned template for isolated native dialogs.
use super::*;

pub(super) type Control<'a> = (u16, u16, u16, u16, u16, u16, u32, &'a str);
pub(super) fn template(title: &str, width: u16, height: u16, controls: &[Control<'_>]) -> Vec<u32> {
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
        controls.len() as u16,
        0,
        0,
        width,
        height,
        0,
        0,
    ];
    words.extend(wide(title));
    words.push(9);
    words.extend(wide("Segoe UI"));
    for &(id, class, x, y, width, height, style, label) in controls {
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
    words
        .chunks(2)
        .map(|w| u32::from(w[0]) | (u32::from(*w.get(1).unwrap_or(&0)) << 16))
        .collect()
}
