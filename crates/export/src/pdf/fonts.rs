use crate::illustrated::IllustratedError;
use unicode_normalization::UnicodeNormalization;

pub(super) const DATA: [&[u8]; 4] = [
    include_bytes!("../../fonts/NotoSans-Regular.ttf"),
    include_bytes!("../../fonts/NotoSans-Bold.ttf"),
    include_bytes!("../../fonts/NotoSans-Italic.ttf"),
    include_bytes!("../../fonts/NotoSans-BoldItalic.ttf"),
];
pub(super) struct Fonts {
    faces: Vec<ttf_parser::Face<'static>>,
}
impl Fonts {
    pub(super) fn ascent(&self, size: f32) -> f32 {
        self.faces
            .iter()
            .map(|f| f32::from(f.ascender()) * size / f32::from(f.units_per_em()))
            .fold(0.0, f32::max)
    }
    pub(super) fn new() -> Result<Self, IllustratedError> {
        let faces = DATA
            .iter()
            .map(|data| ttf_parser::Face::parse(data, 0).map_err(|_| IllustratedError::Invalid))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { faces })
    }
    pub(super) fn normalized(&self, text: &str, style: usize) -> Result<String, IllustratedError> {
        let text: String = text
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\t', "    ")
            .nfc()
            .collect();
        for c in text.chars() {
            if c == '\n' {
                continue;
            }
            // This renderer does not implement bidi, combining-mark placement or complex shaping.
            let supported = matches!(u32::from(c),0x20..=0x2ff|0x370..=0x52f|0x1e00..=0x1fff|0x2000..=0x206f|0x20a0..=0x20cf|0x2100..=0x214f|0x2190..=0x21ff|0x2200..=0x22ff|0x25a0..=0x25ff);
            if !supported
                || matches!(u32::from(c),0x200b..=0x200f|0x2028..=0x202e|0x2060..=0x206f)
                || c == '\u{ad}'
                || self.faces[style].glyph_index(c).is_none()
            {
                return Err(IllustratedError::Invalid);
            }
        }
        Ok(text)
    }
    pub(super) fn advance(
        &self,
        c: char,
        style: usize,
        size: f32,
    ) -> Result<f32, IllustratedError> {
        let face = &self.faces[style];
        let glyph = face.glyph_index(c).ok_or(IllustratedError::Invalid)?;
        Ok(f32::from(
            face.glyph_hor_advance(glyph)
                .ok_or(IllustratedError::Invalid)?,
        ) * size
            / f32::from(face.units_per_em()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn font_coverage_is_explicit_and_faces_allow_embedding() {
        let fonts = Fonts::new().unwrap();
        for style in 0..4 {
            assert_eq!(
                fonts.faces[style].permissions(),
                Some(ttf_parser::Permissions::Installable)
            );
            assert_eq!(
                fonts
                    .normalized("A\u{308}ÖÜ ß é Ελληνικά Кириллица € •", style)
                    .unwrap(),
                "ÄÖÜ ß é Ελληνικά Кириллица € •"
            );
            for bad in [
                "→",
                "漢字",
                "🙂",
                "العربية",
                "x\u{301}",
                "a\u{200d}b",
                "a\u{202e}b",
            ] {
                assert!(fonts.normalized(bad, style).is_err());
            }
            assert!(
                fonts.advance('W', style, 11.0).unwrap() > fonts.advance('i', style, 11.0).unwrap()
            );
        }
    }
}
