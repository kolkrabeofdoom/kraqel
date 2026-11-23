//! Bounded PDF layout with embedded fonts, explicit glyph coverage and measured advances.
mod fonts;
use crate::{
    Block, Inline, Manuscript,
    illustrated::{IllustratedError, MAX_PACKAGE, Manifest},
};
use fonts::Fonts;
use printpdf::{
    FontId, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfParseErrorSeverity,
    PdfSaveOptions, Point, Pt, RawImage, RawImageData, RawImageFormat, TextItem, XObjectTransform,
};
use std::collections::BTreeMap;

const LEFT: f32 = 56.7;
const TOP: f32 = 785.0;
const BOTTOM: f32 = 56.7;
const WIDTH: f32 = 481.8;
const MAX_LINES: usize = 100_000;
type Result<T> = std::result::Result<T, IllustratedError>;

#[derive(Clone)]
struct Line {
    chars: Vec<(char, usize)>,
    size: f32,
    indent: f32,
}
enum Item {
    Line(Line),
    Gap(f32),
    FigureEnd,
    Image {
        key: String,
        width: f32,
        height: f32,
    },
}

struct Layout {
    fonts: Fonts,
    items: Vec<Item>,
    lines: usize,
    text_bytes: usize,
    image_bytes: u64,
}
impl Layout {
    fn new() -> Result<Self> {
        Ok(Self {
            fonts: Fonts::new()?,
            items: Vec::new(),
            lines: 0,
            text_bytes: 0,
            image_bytes: 0,
        })
    }
    fn push_line(&mut self, chars: Vec<(char, usize)>, size: f32, indent: f32) -> Result<()> {
        self.lines += 1;
        if self.lines > MAX_LINES {
            return Err(IllustratedError::Invalid);
        }
        self.items.push(Item::Line(Line {
            chars,
            size,
            indent,
        }));
        Ok(())
    }
    fn paragraph(
        &mut self,
        runs: &[Inline],
        size: f32,
        indent: f32,
        prefix: &str,
        default_style: usize,
    ) -> Result<()> {
        self.text_bytes = self
            .text_bytes
            .checked_add(prefix.len())
            .ok_or(IllustratedError::Invalid)?;
        for run in runs {
            if let Inline::Text { text, .. } = run {
                self.text_bytes = self
                    .text_bytes
                    .checked_add(text.len())
                    .ok_or(IllustratedError::Invalid)?;
            }
        }
        if self.text_bytes > 8 * 1024 * 1024 {
            return Err(IllustratedError::Invalid);
        }
        let mut chars: Vec<_> = self
            .fonts
            .normalized(prefix, default_style)?
            .chars()
            .map(|c| (c, default_style))
            .collect();
        for run in runs {
            match run {
                Inline::HardBreak => chars.push(('\n', default_style)),
                Inline::Text { text, bold, italic } => {
                    let style = if *bold || *italic {
                        usize::from(*bold) + 2 * usize::from(*italic)
                    } else {
                        default_style
                    };
                    chars.extend(
                        self.fonts
                            .normalized(text, style)?
                            .chars()
                            .map(|c| (c, style)),
                    );
                }
            }
        }
        let mut line = Vec::new();
        let mut width = 0.0;
        for (c, style) in chars {
            if c == '\n' {
                self.push_line(std::mem::take(&mut line), size, indent)?;
                width = 0.0;
                continue;
            }
            let advance = self.fonts.advance(c, style, size)?;
            if width + advance > WIDTH - indent && !line.is_empty() {
                let tail = if let Some(index) = line.iter().rposition(|(c, _)| *c == ' ') {
                    line.split_off(index + 1)
                } else {
                    Vec::new()
                };
                self.push_line(std::mem::take(&mut line), size, indent)?;
                line = tail;
                width = line
                    .iter()
                    .map(|(c, s)| self.fonts.advance(*c, *s, size))
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .sum();
            }
            if width + advance > WIDTH - indent {
                self.push_line(std::mem::take(&mut line), size, indent)?;
                width = 0.0;
                if advance > WIDTH - indent {
                    return Err(IllustratedError::Invalid);
                }
            }
            width += advance;
            line.push((c, style));
        }
        self.push_line(line, size, indent)?;
        self.items.push(Item::Gap(5.0));
        Ok(())
    }
    fn plain(&mut self, text: &str, size: f32, style: usize) -> Result<()> {
        self.paragraph(
            &[Inline::Text {
                text: text.to_owned(),
                bold: false,
                italic: false,
            }],
            size,
            0.0,
            "",
            style,
        )
    }
    fn blocks(&mut self, blocks: &[Block], depth: usize, prefix: &str) -> Result<()> {
        if depth > 32 {
            return Err(IllustratedError::Invalid);
        }
        let indent =
            f32::from(u16::try_from(depth.min(12)).map_err(|_| IllustratedError::Invalid)?) * 12.0;
        if blocks.is_empty() && !prefix.is_empty() {
            return self.paragraph(&[], 11.0, indent, prefix, 0);
        }
        for (index, block) in blocks.iter().enumerate() {
            let lead = if index == 0 { prefix } else { "" };
            match block {
                Block::Paragraph(runs) => self.paragraph(runs, 11.0, indent, lead, 0)?,
                Block::Heading(_, runs) => self.paragraph(runs, 13.0, indent, lead, 1)?,
                Block::Quote(children) => self.blocks(children, depth + 1, &format!("{lead}› "))?,
                Block::BulletList(items) | Block::OrderedList(items) => {
                    for (n, item) in items.iter().enumerate() {
                        let marker = if matches!(block, Block::BulletList(_)) {
                            "• ".to_owned()
                        } else {
                            format!("{}. ", n + 1)
                        };
                        self.blocks(item, depth + 1, &marker)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn figures(&mut self, manifest: &Manifest, c: usize, s: usize) -> Result<()> {
        for placement in manifest
            .placements
            .iter()
            .filter(|p| p.chapter_position == c && p.scene_position == s)
        {
            self.fonts.normalized(&placement.title, 0)?;
            let asset = manifest
                .assets
                .iter()
                .find(|a| a.id == placement.asset_id)
                .ok_or(IllustratedError::Invalid)?;
            self.image_bytes = self
                .image_bytes
                .checked_add(u64::from(asset.export_width) * u64::from(asset.export_height) * 3)
                .ok_or(IllustratedError::Invalid)?;
            if self.image_bytes > MAX_PACKAGE as u64 {
                return Err(IllustratedError::Invalid);
            }
            let start = self.items.len();
            self.plain(
                &format!("Bildbeschreibung: {}", placement.alt_text),
                10.0,
                0,
            )?;
            if let Some(caption) = &placement.caption {
                self.plain(caption, 10.0, 0)?;
            }
            let mut texts = self.items.split_off(start);
            let text_height: text_height::Height = text_height::Height::of(&texts);
            let maximum = (TOP - BOTTOM - text_height.0 - 16.0).min(540.0);
            if maximum <= 0.0 {
                return Err(IllustratedError::Invalid);
            }
            let natural_w = f32::from(
                u16::try_from(asset.export_width).map_err(|_| IllustratedError::Invalid)?,
            ) * 0.75;
            let natural_h = f32::from(
                u16::try_from(asset.export_height).map_err(|_| IllustratedError::Invalid)?,
            ) * 0.75;
            let factor = 1.0_f32.min(WIDTH / natural_w).min(maximum / natural_h);
            self.items.push(Item::Image {
                key: asset.location.clone(),
                width: natural_w * factor,
                height: natural_h * factor,
            });
            self.items.append(&mut texts);
            self.items.push(Item::FigureEnd);
        }
        Ok(())
    }
}

// All figure lines and their image are planned as one page-bounded group.
mod text_height {
    use super::Item;
    pub(super) struct Height(pub f32);
    impl Height {
        pub(super) fn of(items: &[Item]) -> Self {
            Self(
                items
                    .iter()
                    .map(|i| match i {
                        Item::Line(l) => l.size * 1.4,
                        Item::Gap(g) => *g,
                        Item::Image { height, .. } => *height + 8.0,
                        Item::FigureEnd => 10.0,
                    })
                    .sum(),
            )
        }
    }
}

struct Canvas {
    document: PdfDocument,
    ops: Vec<Op>,
    fonts: Vec<FontId>,
    y: f32,
}
impl Canvas {
    fn new(title: &str) -> Result<Self> {
        let mut document = PdfDocument::new(title);
        let mut fonts = Vec::new();
        for data in fonts::DATA {
            let font = ParsedFont::from_bytes(data, 0, &mut Vec::new())
                .ok_or(IllustratedError::Invalid)?;
            fonts.push(document.add_font(&font));
        }
        Ok(Self {
            document,
            ops: Vec::new(),
            fonts,
            y: TOP,
        })
    }

    fn finish_page(&mut self) {
        self.document.pages.push(PdfPage::new(
            Mm(210.0),
            Mm(297.0),
            std::mem::take(&mut self.ops),
        ));
    }

    fn write_run(&mut self, text: String, size: f32, x: f32, y: f32, style: usize) {
        self.ops.extend([
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point { x: Pt(x), y: Pt(y) },
            },
            Op::SetFont {
                font: PdfFontHandle::External(self.fonts[style].clone()),
                size: Pt(size),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text)],
            },
            Op::EndTextSection,
        ]);
    }
    fn room(&mut self, height: f32) -> Result<()> {
        if height > TOP - BOTTOM {
            return Err(IllustratedError::Invalid);
        }
        if self.y - height < BOTTOM {
            if self.document.pages.len() >= 2047 {
                return Err(IllustratedError::Invalid);
            }
            self.finish_page();
            self.y = TOP;
        }
        Ok(())
    }
    fn line(&mut self, line: &Line, metrics: &Fonts) -> Result<()> {
        self.room(line.size * 1.4)?;
        let baseline = self.y - metrics.ascent(line.size);
        let mut x = LEFT + line.indent;
        let mut current = String::new();
        let mut style = 0;
        for (c, s) in &line.chars {
            if *s != style && !current.is_empty() {
                let advance = current
                    .chars()
                    .map(|c| metrics.advance(c, style, line.size))
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .sum::<f32>();
                self.write_run(std::mem::take(&mut current), line.size, x, baseline, style);
                x += advance;
            }
            style = *s;
            current.push(*c);
        }
        if !current.is_empty() {
            self.write_run(current, line.size, x, baseline, style);
        }
        self.y -= line.size * 1.4;
        Ok(())
    }
    fn image(&mut self, png: &[u8], width: f32, height: f32) -> Result<()> {
        let pixels = image::load_from_memory(png)
            .map_err(|_| IllustratedError::Invalid)?
            .to_rgb8();
        let px_width = u16::try_from(pixels.width()).map_err(|_| IllustratedError::Invalid)?;
        let px_height = u16::try_from(pixels.height()).map_err(|_| IllustratedError::Invalid)?;
        let id = self.document.add_image(&RawImage {
            width: usize::from(px_width),
            height: usize::from(px_height),
            pixels: RawImageData::U8(pixels.into_raw()),
            data_format: RawImageFormat::RGB8,
            tag: Vec::new(),
        });
        self.y -= height;
        self.ops.push(Op::UseXobject {
            id,
            transform: XObjectTransform {
                translate_x: Some(Pt(LEFT)),
                translate_y: Some(Pt(self.y)),
                scale_x: Some(width / f32::from(px_width)),
                scale_y: Some(height / f32::from(px_height)),
                dpi: Some(72.0),
                ..Default::default()
            },
        });
        self.y -= 8.0;
        Ok(())
    }
}

fn plan(manuscript: &Manuscript, manifest: Option<&Manifest>) -> Result<Layout> {
    let mut layout = Layout::new()?;
    layout.plain(&manuscript.title, 22.0, 1)?;
    for (c, chapter) in manuscript.chapters.iter().enumerate() {
        layout.plain(&chapter.title, 17.0, 1)?;
        for (s, scene) in chapter.scenes.iter().enumerate() {
            layout.plain(&scene.title, 14.0, 1)?;
            layout.blocks(&scene.blocks, 0, "")?;
            if let Some(manifest) = manifest {
                layout.figures(manifest, c, s)?;
            }
        }
    }
    Ok(layout)
}

fn render(
    manuscript: &Manuscript,
    manifest: Option<&Manifest>,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>> {
    let layout = plan(manuscript, manifest)?;
    let mut canvas = Canvas::new(&manuscript.title)?;
    for (n, item) in layout.items.iter().enumerate() {
        match item {
            Item::Line(line) => canvas.line(line, &layout.fonts)?,
            Item::Gap(gap) => canvas.y -= gap,
            Item::FigureEnd => canvas.y -= 10.0,
            Item::Image { key, width, height } => {
                let following: Vec<_> = layout.items[n + 1..]
                    .iter()
                    .take_while(|i| !matches!(i, Item::Image { .. }))
                    .collect();
                // Figure descriptions are followed by a 10 pt separator; stop at that separator.
                let mut total = *height + 8.0;
                for next in following {
                    match next {
                        Item::Line(l) => total += l.size * 1.4,
                        Item::FigureEnd => break,
                        Item::Gap(g) => total += g,
                        Item::Image { .. } => {}
                    }
                }
                canvas.room(total)?;
                canvas.image(
                    images.get(key).ok_or(IllustratedError::Invalid)?,
                    *width,
                    *height,
                )?;
            }
        }
    }
    canvas.finish_page();
    let options = PdfSaveOptions {
        image_optimization: None,
        ..Default::default()
    };
    let mut warnings = Vec::new();
    let bytes = canvas.document.save(&options, &mut warnings);
    if bytes.len() > MAX_PACKAGE
        || warnings
            .iter()
            .any(|w| w.severity == PdfParseErrorSeverity::Error)
    {
        return Err(IllustratedError::Invalid);
    }
    verify_images(&bytes, manifest, images)
}

pub(crate) fn text(manuscript: &Manuscript) -> Result<Vec<u8>> {
    render(manuscript, None, &BTreeMap::new())
}

fn verify_images(
    bytes: &[u8],
    manifest: Option<&Manifest>,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeSet;
    let mut document = lopdf::Document::load_mem(bytes).map_err(|_| IllustratedError::Invalid)?;
    let mut expected = BTreeMap::new();
    if let Some(manifest) = manifest {
        for asset in &manifest.assets {
            let png = images
                .get(&asset.location)
                .ok_or(IllustratedError::Invalid)?;
            let pixels = image::load_from_memory(png)
                .map_err(|_| IllustratedError::Invalid)?
                .to_rgb8();
            expected.insert(
                (
                    asset.export_width,
                    asset.export_height,
                    format!("{:x}", Sha256::digest(pixels.as_raw())),
                ),
                asset
                    .pdf_image_id
                    .clone()
                    .ok_or(IllustratedError::Invalid)?,
            );
        }
    }
    let mut found = BTreeSet::new();
    for object in document.objects.values_mut() {
        if let Ok(stream) = object.as_stream_mut() {
            if stream
                .dict
                .get(b"Subtype")
                .and_then(lopdf::Object::as_name)
                .ok()
                != Some(b"Image".as_slice())
            {
                continue;
            }
            let raw = if stream.dict.has(b"Filter") {
                stream
                    .decompressed_content()
                    .map_err(|_| IllustratedError::Invalid)?
            } else {
                stream.content.clone()
            };
            let dimension = |key: &[u8]| -> Result<u32> {
                u32::try_from(
                    stream
                        .dict
                        .get(key)
                        .and_then(lopdf::Object::as_i64)
                        .map_err(|_| IllustratedError::Invalid)?,
                )
                .map_err(|_| IllustratedError::Invalid)
            };
            let key = (
                dimension(b"Width")?,
                dimension(b"Height")?,
                format!("{:x}", Sha256::digest(&raw)),
            );
            let id = expected.get(&key).ok_or(IllustratedError::Invalid)?;
            if stream
                .dict
                .get(b"SMask")
                .is_ok_and(|value| !matches!(value, lopdf::Object::Null))
                || stream
                    .dict
                    .get(b"BitsPerComponent")
                    .and_then(lopdf::Object::as_i64)
                    .ok()
                    != Some(8)
                || stream
                    .dict
                    .get(b"ColorSpace")
                    .and_then(lopdf::Object::as_name)
                    .ok()
                    != Some(b"DeviceRGB".as_slice())
            {
                return Err(IllustratedError::Invalid);
            }
            stream.dict.remove(b"SMask");
            stream
                .dict
                .set("KRAQELImageId", lopdf::Object::Name(id.as_bytes().to_vec()));
            found.insert(key);
        }
    }
    if found.len() != expected.len() {
        return Err(IllustratedError::Invalid);
    }
    let mut result = Vec::new();
    document
        .save_to(&mut result)
        .map_err(|_| IllustratedError::Invalid)?;
    if result.len() > MAX_PACKAGE {
        return Err(IllustratedError::Invalid);
    }
    Ok(result)
}
pub(crate) fn illustrated(
    manuscript: &Manuscript,
    manifest: &Manifest,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>> {
    render(manuscript, Some(manifest), images)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn measured_lines_and_embedded_fonts_preserve_unicode_without_fallback() {
        let mut layout = Layout::new().unwrap();
        layout.plain(&"W".repeat(300), 11.0, 0).unwrap();
        assert!(layout.lines > 1);
        for item in &layout.items {
            if let Item::Line(line) = item {
                let width: f32 = line
                    .chars
                    .iter()
                    .map(|(c, s)| layout.fonts.advance(*c, *s, line.size).unwrap())
                    .sum();
                assert!(width <= WIDTH);
            }
        }
        let manuscript = Manuscript {
            title: "ÄÖÜ Ελληνικά Кириллица".to_owned(),
            chapters: vec![crate::Chapter {
                title: "Kapitel".into(),
                scenes: vec![crate::Scene {
                    title: "Szene".into(),
                    blocks: vec![Block::Paragraph(
                        [
                            (false, false, "Normal"),
                            (true, false, "Fett"),
                            (false, true, "Kursiv"),
                            (true, true, "Beides"),
                        ]
                        .into_iter()
                        .map(|(bold, italic, text)| Inline::Text {
                            bold,
                            italic,
                            text: text.into(),
                        })
                        .collect(),
                    )],
                }],
            }],
        };
        let bytes = text(&manuscript).unwrap();
        let document = lopdf::Document::load_mem(&bytes).unwrap();
        let descriptors = document
            .objects
            .values()
            .filter_map(|o| o.as_dict().ok())
            .filter(|d| d.has(b"FontFile2"))
            .count();
        let maps = document
            .objects
            .values()
            .filter_map(|o| o.as_dict().ok())
            .filter(|d| d.has(b"ToUnicode"))
            .count();
        assert_eq!(descriptors, 4);
        assert_eq!(maps, 4);
        let extracted = document.extract_text(&[1]).unwrap();
        for expected in [
            manuscript.title.as_str(),
            "Normal",
            "Fett",
            "Kursiv",
            "Beides",
        ] {
            assert!(
                extracted.contains(expected),
                "missing {expected:?} in {extracted:?}"
            );
        }
        let bad = Manuscript {
            title: "PRIVATE 🙂".to_owned(),
            ..manuscript
        };
        assert!(text(&bad).is_err());
    }
}
