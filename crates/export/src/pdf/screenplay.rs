//! Explicit scene selection and bounded layout for `kraqel-screenplay-a4-v1`.
use super::{Canvas, Fonts, MAX_LINES, MAX_PACKAGE, PdfParseErrorSeverity, PdfSaveOptions};
use crate::{
    ExportError, ExportFormat, ExportResult, ExportSourceDocument, Inline, atomic_write,
    prepare_export_directory, screenplay::paragraphs,
};
use kraqel_domain::{DocumentKind, EntityId, Project, ScreenplayRole as Role};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub(super) const FONTS: [&[u8]; 4] = [
    include_bytes!("../../fonts/jetbrains-mono/JetBrainsMono-Regular.ttf"),
    include_bytes!("../../fonts/jetbrains-mono/JetBrainsMono-Bold.ttf"),
    include_bytes!("../../fonts/jetbrains-mono/JetBrainsMono-Italic.ttf"),
    include_bytes!("../../fonts/jetbrains-mono/JetBrainsMono-BoldItalic.ttf"),
];
const MM: f32 = 72.0 / 25.4;
const TOP: f32 = (297.0 - 25.0) * MM;
const BOTTOM: f32 = 20.0 * MM;
const PAGE_HEIGHT: f32 = TOP - BOTTOM;
const SIZE: f32 = 12.0;
const LEADING: f32 = 14.0;
const SPACE: f32 = 12.0;
pub(super) const MAX_STYLED_RUNS: usize = 250_000;
const MAX_TEXT: usize = 8 * 1024 * 1024;
type Result<T> = std::result::Result<T, ExportError>;
const INVALID_SELECTION: ExportError = ExportError::InvalidDocument(
    "Wähle ausschließlich gespeicherte Drehbuchszenen dieses Projekts aus.",
);
const LIMIT: ExportError =
    ExportError::InvalidDocument("Die Drehbuchausgabe überschreitet die PDF-Grenzen.");
const FONT_ERROR: ExportError = ExportError::InvalidDocument(
    "Der Text enthält Zeichen außerhalb des unterstützten Drehbuch-Schriftprofils.",
);

fn selected<'a>(
    project: &Project,
    documents: &'a [ExportSourceDocument],
    ids: &[EntityId],
) -> Result<Vec<&'a ExportSourceDocument>> {
    let mut wanted: HashSet<_> = ids.iter().copied().collect();
    if wanted.is_empty() || wanted.len() != ids.len() {
        return Err(INVALID_SELECTION);
    }
    let mut chapters = HashMap::new();
    let mut seen = HashSet::new();
    for doc in documents {
        if doc.node.project_id != project.id || !seen.insert(doc.node.id) {
            return Err(INVALID_SELECTION);
        }
        if doc.node.kind == DocumentKind::Chapter {
            if doc.node.parent_id.is_some() {
                return Err(INVALID_SELECTION);
            }
            chapters.insert(doc.node.id, doc.node.position);
        }
    }
    let mut chosen = Vec::with_capacity(wanted.len());
    for doc in documents {
        if !wanted.remove(&doc.node.id) {
            continue;
        }
        if doc.node.kind != DocumentKind::Scene {
            return Err(INVALID_SELECTION);
        }
        let parent = doc.node.parent_id.ok_or(INVALID_SELECTION)?;
        let position = *chapters.get(&parent).ok_or(INVALID_SELECTION)?;
        chosen.push((
            (
                position,
                parent.to_string(),
                doc.node.position,
                doc.node.id.to_string(),
            ),
            doc,
        ));
    }
    if !wanted.is_empty() {
        return Err(INVALID_SELECTION);
    }
    chosen.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(chosen.into_iter().map(|(_, doc)| doc).collect())
}

fn cells(count: usize) -> Result<f32> {
    u16::try_from(count).map(f32::from).map_err(|_| LIMIT)
}

struct Row {
    chars: Vec<(char, usize)>,
    left: f32,
    after: f32,
    keep_next: bool,
    prefer_next: bool,
}

pub(super) struct Layout {
    fonts: Fonts,
    rows: Vec<Row>,
    text_bytes: usize,
    cell: f32,
}

impl Layout {
    pub(super) fn new() -> Result<Self> {
        let fonts = Fonts::from_data(FONTS).map_err(|_| FONT_ERROR)?;
        let cell = fonts.advance('M', 0, SIZE).map_err(|_| FONT_ERROR)?;
        Ok(Self {
            fonts,
            cell,
            rows: Vec::new(),
            text_bytes: 0,
        })
    }

    fn row(&mut self, chars: Vec<(char, usize)>, role: Role) -> Result<()> {
        if self.rows.len() >= MAX_LINES {
            return Err(LIMIT);
        }
        let indent = match role {
            Role::Character => 55.0 * MM,
            Role::Parenthetical => 40.0 * MM,
            Role::Dialogue => 25.0 * MM,
            Role::Transition => 155.0 * MM - cells(chars.len())? * self.cell,
            Role::SceneHeading | Role::Action => 0.0,
        };
        self.rows.push(Row {
            chars,
            left: 30.0 * MM + indent,
            after: 0.0,
            keep_next: false,
            prefer_next: false,
        });
        Ok(())
    }

    fn paragraph(&mut self, role: Role, runs: &[Inline]) -> Result<()> {
        let width = match role {
            Role::Character => 100.0,
            Role::Parenthetical => 90.0,
            Role::Dialogue => 105.0,
            _ => 155.0,
        } * MM;
        let mut line: Vec<(char, usize)> = Vec::new();
        for run in runs {
            let (text, style) = match run {
                Inline::HardBreak => ("\n", 0),
                Inline::Text { text, bold, italic } => {
                    (text.as_str(), usize::from(*bold) + 2 * usize::from(*italic))
                }
            };
            self.text_bytes = self
                .text_bytes
                .checked_add(text.len())
                .filter(|&n| n <= MAX_TEXT)
                .ok_or(LIMIT)?;
            let normalized = self.fonts.normalized(text, style).map_err(|_| FONT_ERROR)?;
            for c in normalized.chars() {
                if c == '\n' {
                    self.row(std::mem::take(&mut line), role)?;
                    continue;
                }
                if (self.fonts.advance(c, style, SIZE).map_err(|_| FONT_ERROR)? - self.cell).abs()
                    > f32::EPSILON
                {
                    return Err(FONT_ERROR);
                }
                if cells(line.len() + 1)? * self.cell > width {
                    let cut = line
                        .iter()
                        .rposition(|(ch, _)| *ch == ' ')
                        .map_or(line.len(), |i| i + 1);
                    let rest = line.split_off(cut);
                    self.row(std::mem::replace(&mut line, rest), role)?;
                }
                line.push((c, style));
            }
        }
        self.row(line, role)
    }

    pub(super) fn text_bytes(&self) -> usize {
        self.text_bytes
    }

    pub(super) fn scene(&mut self, blocks: &[(Role, &[Inline])]) -> Result<std::ops::Range<usize>> {
        let first_row = self.rows.len();
        let mut spans = Vec::with_capacity(blocks.len());
        for (idx, &(role, runs)) in blocks.iter().enumerate() {
            let start = self.rows.len();
            self.paragraph(role, runs)?;
            let end = self.rows.len();
            if end - start >= 2 {
                self.rows[end - 2].prefer_next = true;
            }
            let next = blocks.get(idx + 1).map(|p| p.0);
            let in_dialogue =
                matches!(role, Role::Character | Role::Parenthetical | Role::Dialogue)
                    && matches!(next, Some(Role::Parenthetical | Role::Dialogue));
            if !in_dialogue {
                self.rows[end - 1].after = SPACE;
            }
            spans.push(start..end);
        }
        for (idx, &(role, _)) in blocks.iter().enumerate() {
            let span = &spans[idx];
            let keep_until = match role {
                Role::SceneHeading => {
                    let mut until = span.end;
                    let mut following = 0;
                    for (offset, row) in self.rows[span.end..].iter().enumerate() {
                        if row.chars.iter().any(|(ch, _)| !ch.is_whitespace()) {
                            until = span.end + offset + 1;
                            following += 1;
                            if following == 2 {
                                break;
                            }
                        }
                    }
                    until
                }
                Role::Character | Role::Parenthetical => {
                    let dialogue = if blocks
                        .get(idx + 1)
                        .is_some_and(|b| b.0 == Role::Parenthetical)
                    {
                        idx + 2
                    } else {
                        idx + 1
                    };
                    if let Some(next) = spans.get(dialogue) {
                        // Two dialogue lines can belong to separate consecutive paragraphs.
                        let end = if blocks
                            .get(dialogue + 1)
                            .is_some_and(|b| b.0 == Role::Dialogue)
                        {
                            spans[dialogue + 1].end
                        } else {
                            next.end
                        };
                        (next.start + 2).min(end)
                    } else {
                        span.end
                    }
                }
                _ => span.start,
            };
            for row in &mut self.rows[span.start..keep_until.saturating_sub(1).max(span.start)] {
                row.keep_next = true;
            }
        }
        Ok(first_row..self.rows.len())
    }

    pub(super) fn draw(
        &self,
        canvas: &mut Canvas,
        rows: std::ops::Range<usize>,
        font_offset: usize,
    ) -> Result<()> {
        let mut y = canvas.y.min(TOP);
        let mut start = rows.start;
        while start < rows.end {
            let mut last = start;
            while self.rows[last].keep_next {
                last += 1;
            }
            let group = &self.rows[start..=last];
            let height =
                group.iter().map(|r| LEADING + r.after).sum::<f32>() - group[group.len() - 1].after;
            if height > PAGE_HEIGHT {
                return Err(ExportError::InvalidDocument(
                    "Eine zusammengehörige Drehbuchgruppe ist höher als die Druckseite.",
                ));
            }
            let preferred_height = if self.rows[last].prefer_next && height + LEADING <= PAGE_HEIGHT
            {
                height + LEADING
            } else {
                height
            };
            if y - preferred_height < BOTTOM {
                if canvas.document.pages.len() >= 2047 {
                    return Err(LIMIT);
                }
                canvas.finish_page();
                y = TOP;
            }
            for row in group {
                let mut x = row.left;
                for run in row.chars.chunk_by(|a, b| a.1 == b.1) {
                    let text = run.iter().map(|(c, _)| *c).collect();
                    canvas
                        .write_run(
                            text,
                            SIZE,
                            x,
                            y - self.fonts.ascent(SIZE),
                            font_offset + run[0].1,
                        )
                        .map_err(|_| LIMIT)?;
                    x += cells(run.len())? * self.cell;
                }
                y -= LEADING + row.after;
            }
            start = last + 1;
        }
        canvas.y = y;
        Ok(())
    }

    fn render(&self) -> Result<Vec<u8>> {
        let mut canvas = Canvas::with_fonts("KRAQEL – Drehbuch", FONTS).map_err(|_| FONT_ERROR)?;
        canvas.remaining_styled_runs = Some(MAX_STYLED_RUNS);
        self.draw(&mut canvas, 0..self.rows.len(), 0)?;
        canvas.finish_page();
        let mut warnings = Vec::new();
        let bytes = canvas
            .document
            .save(&PdfSaveOptions::default(), &mut warnings);
        if bytes.len() > MAX_PACKAGE
            || warnings
                .iter()
                .any(|w| w.severity == PdfParseErrorSeverity::Error)
        {
            return Err(LIMIT);
        }
        Ok(bytes)
    }
}

/// Writes only the explicitly selected screenplay scenes, in manuscript order.
///
/// # Errors
/// Rejects stale selections, incomplete dialogue, unsupported glyphs and unsafe destinations.
pub fn export_screenplay_pdf(
    directory: &Path,
    project: &Project,
    documents: &[ExportSourceDocument],
    ids: &[EntityId],
) -> Result<ExportResult> {
    let bytes = render_selected(project, documents, ids)?;
    let output = prepare_export_directory(directory)?;
    atomic_write(&output, "drehbuch.pdf", &bytes)?;
    Ok(ExportResult {
        format: ExportFormat::Pdf,
        file_name: "drehbuch.pdf".to_owned(),
        byte_count: bytes.len() as u64,
    })
}

pub(crate) fn render_selected(
    project: &Project,
    documents: &[ExportSourceDocument],
    ids: &[EntityId],
) -> Result<Vec<u8>> {
    let mut layout = Layout::new()?;
    for scene in selected(project, documents, ids)? {
        let paragraphs = paragraphs(&scene.content)?;
        let blocks: Vec<_> = paragraphs
            .iter()
            .map(|p| (p.role, p.runs.as_slice()))
            .collect();
        layout.scene(&blocks)?;
    }
    layout.render()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn figure_keeps_two_short_dialogue_paragraphs_at_the_page_boundary() {
        let (project, mut docs) = crate::tests::fixture();
        let block = |role: &str, text: &str| {
            json!({
                "type":"screenplayBlock", "attrs":{"role":role},
                "content":[{"type":"text","text":text}]
            })
        };
        let action = vec!["Schritte."; 46].join("\n");
        docs[1].content = kraqel_domain::EditorDocument {
            version: kraqel_domain::SCREENPLAY_DOCUMENT_VERSION,
            document: json!({"type":"doc","content":[
                block("sceneHeading", "INT. ARCHIV"),
                block("action", &action),
                block("character", "MIRA"),
                block("dialogue", "Erste Zeile."),
                block("dialogue", "Zweite Zeile.")
            ]}),
        };
        let bytes = render_selected(&project, &docs, &[docs[1].node.id]).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.get_pages().len(), 2);
        let second = pdf.extract_text(&[2]).unwrap();
        for expected in ["MIRA", "Erste Zeile.", "Zweite Zeile."] {
            assert!(second.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn selected_screenplay_preserves_unicode_styles_and_dialogue_across_pages() {
        let root = tempfile::tempdir().unwrap();
        let (project, mut docs) = crate::tests::fixture();
        let block = |role: &str, text: &str| json!({"type":"screenplayBlock","attrs":{"role":role},"content":[{"type":"text","text":text}]});
        let mut blocks = vec![
            block("sceneHeading", "INT. STADTARCHIV – MORGEN"),
            block(
                "action",
                "Das Licht fällt auf eine Karte. Jede Linie führt zum selben Bahnhof.",
            ),
            block("character", "MIRA"),
            block("parenthetical", "(liest die Rückseite)"),
            json!({"type":"screenplayBlock","attrs":{"role":"dialogue"},"content":[
                {"type":"text","text":"Drei Namen. "},
                {"type":"text","text":"Αθήνα","marks":[{"type":"bold"}]},
                {"type":"text","text":", "},
                {"type":"text","text":"Київ","marks":[{"type":"italic"}]},
                {"type":"text","text":" und "},
                {"type":"text","text":"Zürich","marks":[{"type":"bold"},{"type":"italic"}]},
                {"type":"text","text":". Wer hat sie eingetragen?"}
            ]}),
        ];
        // Fill to the page boundary to exercise the shared character/parenthetical/dialogue group.
        let stops = (1..=39)
            .map(|n| format!("Gleis {n:02} – die Anzeigetafel blinkt."))
            .collect::<Vec<_>>()
            .join("\n");
        blocks.extend([
            block("action", &stops),
            block("character", "JONAS"),
            block("parenthetical", "(dreht die Karte um)"),
            block(
                "dialogue",
                "Sieh auf das Wasserzeichen.\nEs ist unser Bahnhof.",
            ),
            block("transition", "SCHNITT AUF:"),
        ]);
        docs[1].content.version = kraqel_domain::SCREENPLAY_DOCUMENT_VERSION;
        docs[1].content.document = json!({"type":"doc","content":blocks});
        let ids = [docs[1].node.id];
        let result = export_screenplay_pdf(root.path(), &project, &docs, &ids).unwrap();
        let path = root.path().join("exports").join(result.file_name);
        let bytes = std::fs::read(&path).unwrap();
        let pdf = lopdf::Document::load_mem(&bytes).unwrap();
        assert_eq!(pdf.get_pages().len(), 2);
        let first = pdf.extract_text(&[1]).unwrap();
        for expected in ["STADTARCHIV", "Αθήνα", "Київ", "Zürich"] {
            assert!(first.contains(expected), "missing {expected}");
        }
        let second = pdf.extract_text(&[2]).unwrap();
        for expected in [
            "JONAS",
            "(dreht die Karte um)",
            "Wasserzeichen.",
            "SCHNITT AUF:",
        ] {
            assert!(second.contains(expected), "missing {expected}");
        }
        for (i, data) in FONTS.iter().enumerate() {
            let face = ttf_parser::Face::parse(data, 0).unwrap();
            assert!(face.is_monospaced());
            for c in "ÄßéΑθήναКиїв".chars() {
                let glyph = face.glyph_index(c).expect("profile sample glyph");
                assert_eq!(face.glyph_hor_advance(glyph), Some(600));
            }
            assert_eq!(face.weight().to_number() >= 700, i % 2 == 1);
            assert_eq!(face.is_italic(), i >= 2);
            assert_eq!(
                face.permissions(),
                Some(ttf_parser::Permissions::Installable)
            );
        }
        assert!(export_screenplay_pdf(root.path(), &project, &docs, &[docs[0].node.id]).is_err());
        docs[1].content.document["content"][0]["content"][0]["text"] = json!("Archiv 🪶");
        assert!(export_screenplay_pdf(root.path(), &project, &docs, &ids).is_err());
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}
