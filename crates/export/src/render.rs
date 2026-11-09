use crate::{Block, ExportError, ExportFormat, Inline, Manuscript, internal};
use docx_rs::{BreakType, Docx, Paragraph, Run};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use std::{
    fmt::Write as _,
    io::{BufWriter, Cursor, Write},
};

pub(crate) fn render(
    manuscript: &Manuscript,
    format: ExportFormat,
) -> Result<Vec<u8>, ExportError> {
    match format {
        ExportFormat::Markdown => Ok(markdown(manuscript).into_bytes()),
        ExportFormat::Html => Ok(html(manuscript).into_bytes()),
        ExportFormat::Docx => docx(manuscript),
        ExportFormat::Pdf => pdf(manuscript),
    }
}

fn markdown(manuscript: &Manuscript) -> String {
    let mut output = format!("# {}\n\n", escape_markdown(&manuscript.title));
    for chapter in &manuscript.chapters {
        let _ = write!(output, "## {}\n\n", escape_markdown(&chapter.title));
        for scene in &chapter.scenes {
            let _ = write!(output, "### {}\n\n", escape_markdown(&scene.title));
            markdown_blocks(&mut output, &scene.blocks, 0, "");
        }
    }
    output
}
fn markdown_blocks(output: &mut String, blocks: &[Block], depth: usize, prefix: &str) {
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => {
                output.push_str(prefix);
                markdown_inlines(output, inlines);
                output.push_str("\n\n");
            }
            Block::Heading(level, inlines) => {
                output.push_str(&"#".repeat(usize::from((*level).saturating_add(3).min(6))));
                output.push(' ');
                markdown_inlines(output, inlines);
                output.push_str("\n\n");
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                for item in items {
                    output.push_str(&"  ".repeat(depth));
                    output.push_str(if matches!(block, Block::BulletList(_)) {
                        "- "
                    } else {
                        "1. "
                    });
                    markdown_blocks(output, item, depth + 1, "");
                }
            }
            Block::Quote(children) => markdown_blocks(output, children, depth, "> "),
        }
    }
}
fn markdown_inlines(output: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::HardBreak => output.push_str("  \n"),
            Inline::Text { text, bold, italic } => {
                let marker = match (*bold, *italic) {
                    (true, true) => "***",
                    (true, false) => "**",
                    (false, true) => "_",
                    _ => "",
                };
                output.push_str(marker);
                output.push_str(&escape_markdown(text));
                output.push_str(marker);
            }
        }
    }
}
fn escape_markdown(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if "\\`*_{}[]<>#".contains(character) {
                vec!['\\', character]
            } else {
                vec![character]
            }
        })
        .collect()
}

fn html(manuscript: &Manuscript) -> String {
    let mut body = format!("<h1>{}</h1>\n", escape_html(&manuscript.title));
    for chapter in &manuscript.chapters {
        let _ = writeln!(body, "<section><h2>{}</h2>", escape_html(&chapter.title));
        for scene in &chapter.scenes {
            let _ = writeln!(body, "<article><h3>{}</h3>", escape_html(&scene.title));
            html_blocks(&mut body, &scene.blocks);
            body.push_str("</article>\n");
        }
        body.push_str("</section>\n");
    }
    format!(
        "<!doctype html>\n<html lang=\"de\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>body{{max-width:44rem;margin:4rem auto;padding:0 1.5rem;color:#182829;background:#fffdf8;font:12pt/1.65 Georgia,serif}}h1{{font-size:2.6rem}}h2{{margin-top:3rem;border-bottom:1px solid #9aaca8}}h3{{margin-top:2rem}}blockquote{{border-left:.25rem solid #0c7772;margin-left:0;padding-left:1rem;color:#435b5c}}@media print{{body{{margin:0}}}}</style></head><body>{}</body></html>\n",
        escape_html(&manuscript.title),
        body
    )
}
fn html_blocks(output: &mut String, blocks: &[Block]) {
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => {
                output.push_str("<p>");
                html_inlines(output, inlines);
                output.push_str("</p>\n");
            }
            Block::Heading(level, inlines) => {
                let level = (*level).saturating_add(3).min(6);
                let _ = write!(output, "<h{level}>");
                html_inlines(output, inlines);
                let _ = writeln!(output, "</h{level}>");
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                let tag = if matches!(block, Block::BulletList(_)) {
                    "ul"
                } else {
                    "ol"
                };
                let _ = writeln!(output, "<{tag}>");
                for item in items {
                    output.push_str("<li>");
                    html_blocks(output, item);
                    output.push_str("</li>\n");
                }
                let _ = writeln!(output, "</{tag}>");
            }
            Block::Quote(children) => {
                output.push_str("<blockquote>");
                html_blocks(output, children);
                output.push_str("</blockquote>\n");
            }
        }
    }
}
fn html_inlines(output: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::HardBreak => output.push_str("<br>"),
            Inline::Text { text, bold, italic } => {
                if *bold {
                    output.push_str("<strong>");
                }
                if *italic {
                    output.push_str("<em>");
                }
                output.push_str(&escape_html(text));
                if *italic {
                    output.push_str("</em>");
                }
                if *bold {
                    output.push_str("</strong>");
                }
            }
        }
    }
}
fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn docx(manuscript: &Manuscript) -> Result<Vec<u8>, ExportError> {
    let mut document = Docx::new().page_size(12_240, 15_840).add_paragraph(
        Paragraph::new()
            .style("Title")
            .add_run(Run::new().add_text(&manuscript.title).bold().size(44)),
    );
    for chapter in &manuscript.chapters {
        document = document.add_paragraph(
            Paragraph::new()
                .style("Heading1")
                .add_run(Run::new().add_text(&chapter.title).bold().size(32)),
        );
        for scene in &chapter.scenes {
            document = document.add_paragraph(
                Paragraph::new()
                    .style("Heading2")
                    .add_run(Run::new().add_text(&scene.title).bold().size(26)),
            );
            document = docx_blocks(document, &scene.blocks, 0);
        }
    }
    let mut cursor = Cursor::new(Vec::new());
    document.build().pack(&mut cursor).map_err(internal)?;
    Ok(cursor.into_inner())
}
fn docx_blocks(mut document: Docx, blocks: &[Block], depth: usize) -> Docx {
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => {
                document = document.add_paragraph(docx_paragraph(inlines, None));
            }
            Block::Heading(level, inlines) => {
                document = document.add_paragraph(docx_paragraph(
                    inlines,
                    Some(format!("Heading{}", (*level).saturating_add(2).min(6))),
                ));
            }
            Block::Quote(children) => {
                document = docx_prefixed_blocks(document, children, depth, "› ");
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                for (index, item) in items.iter().enumerate() {
                    let marker = if matches!(block, Block::BulletList(_)) {
                        "•".to_owned()
                    } else {
                        format!("{}.", index + 1)
                    };
                    document = docx_list_item(document, item, depth, &marker);
                }
            }
        }
    }
    document
}
fn docx_list_item(document: Docx, item: &[Block], depth: usize, marker: &str) -> Docx {
    let prefix = format!("{}{} ", "  ".repeat(depth), marker);
    docx_prefixed_blocks(document, item, depth, &prefix)
}
fn docx_prefixed_blocks(mut document: Docx, blocks: &[Block], depth: usize, prefix: &str) -> Docx {
    if let Some((Block::Paragraph(inlines), remainder)) = blocks.split_first() {
        let paragraph = Paragraph::new().add_run(Run::new().add_text(prefix.to_owned()));
        document = document.add_paragraph(docx_inlines(paragraph, inlines));
        docx_blocks(document, remainder, depth + 1)
    } else {
        document = document
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text(prefix.to_owned())));
        docx_blocks(document, blocks, depth + 1)
    }
}
fn docx_paragraph(inlines: &[Inline], style: Option<String>) -> Paragraph {
    let mut paragraph = Paragraph::new();
    if let Some(style) = style {
        paragraph = paragraph.style(&style);
    }
    docx_inlines(paragraph, inlines)
}
fn docx_inlines(mut paragraph: Paragraph, inlines: &[Inline]) -> Paragraph {
    for inline in inlines {
        paragraph = match inline {
            Inline::HardBreak => paragraph.add_run(Run::new().add_break(BreakType::TextWrapping)),
            Inline::Text { text, bold, italic } => {
                let mut run = Run::new().add_text(text);
                if *bold {
                    run = run.bold();
                }
                if *italic {
                    run = run.italic();
                }
                paragraph.add_run(run)
            }
        };
    }
    paragraph
}

fn pdf(manuscript: &Manuscript) -> Result<Vec<u8>, ExportError> {
    let (document, mut page, mut layer) =
        PdfDocument::new(&manuscript.title, Mm(210.0), Mm(297.0), "Manuskript");
    let regular = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(internal)?;
    let bold = document
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(internal)?;
    let italic = document
        .add_builtin_font(BuiltinFont::HelveticaOblique)
        .map_err(internal)?;
    let bold_italic = document
        .add_builtin_font(BuiltinFont::HelveticaBoldOblique)
        .map_err(internal)?;
    let mut y = 275.0_f32;
    let mut lines = vec![PdfLine::plain(&manuscript.title, 24.0, 1)];
    for chapter in &manuscript.chapters {
        lines.push(PdfLine::plain(&chapter.title, 18.0, 1));
        for scene in &chapter.scenes {
            lines.push(PdfLine::plain(&scene.title, 14.0, 1));
            collect_pdf_lines(&mut lines, &scene.blocks, 0);
        }
    }
    for line in lines {
        if y < 22.0 {
            (page, layer) = document.add_page(Mm(210.0), Mm(297.0), "Manuskript");
            y = 275.0;
        }
        let page_layer = document.get_page(page).get_layer(layer);
        let mut x = 22.0_f32;
        for (text, style) in line.chunks {
            let font = match style {
                1 => &bold,
                2 => &italic,
                3 => &bold_italic,
                _ => &regular,
            };
            page_layer.use_text(&text, line.size, Mm(x), Mm(y), font);
            let character_count = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
            x += f32::from(character_count) * line.size * 0.18;
        }
        y -= line.size * 0.48;
        y -= 3.0;
    }
    let mut bytes = BufWriter::new(Vec::new());
    document.save(&mut bytes).map_err(internal)?;
    bytes.flush().map_err(internal)?;
    bytes.into_inner().map_err(internal)
}

struct PdfLine {
    chunks: Vec<(String, u8)>,
    size: f32,
}

impl PdfLine {
    fn plain(text: &str, size: f32, style: u8) -> Self {
        Self {
            chunks: vec![(text.to_owned(), style)],
            size,
        }
    }
}

fn collect_pdf_lines(lines: &mut Vec<PdfLine>, blocks: &[Block], depth: usize) {
    for block in blocks {
        match block {
            Block::Paragraph(inlines) => lines.extend(styled_lines(inlines, 82, 11.0, 0)),
            Block::Heading(_, inlines) => lines.extend(styled_lines(inlines, 72, 12.0, 1)),
            Block::Quote(children) => {
                let first_line = lines.len();
                collect_pdf_lines(lines, children, depth + 1);
                if let Some(line) = lines.get_mut(first_line) {
                    line.chunks.insert(0, ("› ".to_owned(), 2));
                }
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                for (index, item) in items.iter().enumerate() {
                    let marker = if matches!(block, Block::BulletList(_)) {
                        "•".to_owned()
                    } else {
                        format!("{}.", index + 1)
                    };
                    let first_line = lines.len();
                    collect_pdf_lines(lines, item, depth + 1);
                    if let Some(line) = lines.get_mut(first_line) {
                        line.chunks
                            .insert(0, (format!("{}{} ", "  ".repeat(depth), marker), 0));
                    } else {
                        lines.push(PdfLine::plain(
                            &format!("{}{}", "  ".repeat(depth), marker),
                            11.0,
                            0,
                        ));
                    }
                }
            }
        }
    }
}

fn styled_lines(inlines: &[Inline], width: usize, size: f32, default_style: u8) -> Vec<PdfLine> {
    let mut result = Vec::new();
    let mut chunks: Vec<(String, u8)> = Vec::new();
    let mut length = 0;
    for inline in inlines {
        match inline {
            Inline::HardBreak => {
                result.push(PdfLine {
                    chunks: std::mem::take(&mut chunks),
                    size,
                });
                length = 0;
            }
            Inline::Text { text, bold, italic } => {
                let style = match (*bold, *italic) {
                    (true, true) => 3,
                    (true, false) => 1,
                    (false, true) => 2,
                    _ => default_style,
                };
                for word in text.split_whitespace() {
                    let word_length = word.chars().count();
                    if length > 0 && length + word_length + 1 > width {
                        result.push(PdfLine {
                            chunks: std::mem::take(&mut chunks),
                            size,
                        });
                        length = 0;
                    }
                    let value = if length == 0 {
                        word.to_owned()
                    } else {
                        format!(" {word}")
                    };
                    length += value.chars().count();
                    chunks.push((value, style));
                }
            }
        }
    }
    if !chunks.is_empty() || result.is_empty() {
        result.push(PdfLine { chunks, size });
    }
    result
}
