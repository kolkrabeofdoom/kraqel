use crate::screenplay::{ROLES, presentation};
use crate::{Block, ExportError, ExportFormat, Inline, Manuscript, internal};
use kraqel_domain::ScreenplayRole as Role;
use std::{
    fmt::Write as _,
    io::{Cursor, Write},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

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

fn has_screenplay(manuscript: &Manuscript) -> bool {
    manuscript
        .chapters
        .iter()
        .flat_map(|chapter| &chapter.scenes)
        .any(|scene| matches!(scene.blocks.first(), Some(Block::Screenplay(..))))
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
pub(super) fn markdown_blocks(output: &mut String, blocks: &[Block], depth: usize, prefix: &str) {
    for block in blocks {
        match block {
            Block::Screenplay(role, inlines) => {
                let _ = write!(output, "**{}**\n\n", presentation(*role).label);
                markdown_inlines(output, inlines);
                output.push_str("\n\n");
            }
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
pub(super) fn escape_markdown(value: &str) -> String {
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
        "<!doctype html>\n<html lang=\"de\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>body{{max-width:44rem;margin:4rem auto;padding:0 1.5rem;color:#182829;background:#fffdf8;font:12pt/1.65 Georgia,serif}}h1{{font-size:2.6rem}}h2{{margin-top:3rem;border-bottom:1px solid #9aaca8}}h3{{margin-top:2rem}}blockquote{{border-left:.25rem solid #0c7772;margin-left:0;padding-left:1rem;color:#435b5c}}@media print{{body{{margin:0}}}}{screenplay_css}</style></head><body>{}</body></html>\n",
        escape_html(&manuscript.title),
        body,
        screenplay_css = if has_screenplay(manuscript) {
            include_str!("screenplay.css")
        } else {
            ""
        }
    )
}
pub(super) fn html_blocks(output: &mut String, blocks: &[Block]) {
    for block in blocks {
        match block {
            Block::Screenplay(role, inlines) => {
                let key = presentation(*role).key;
                let _ = write!(
                    output,
                    r#"<p class="screenplay screenplay-{key}" data-kraqel-role="{key}">"#
                );
                html_inlines(output, inlines);
                output.push_str("</p>\n");
            }
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
pub(super) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn docx(manuscript: &Manuscript) -> Result<Vec<u8>, ExportError> {
    let mut document = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
    );
    docx_text_paragraph(
        &mut document,
        &manuscript.title,
        Some("Title"),
        true,
        false,
        Some(44),
    );
    for chapter in &manuscript.chapters {
        docx_text_paragraph(
            &mut document,
            &chapter.title,
            Some("Heading1"),
            true,
            false,
            Some(32),
        );
        for scene in &chapter.scenes {
            docx_text_paragraph(
                &mut document,
                &scene.title,
                Some("Heading2"),
                true,
                false,
                Some(26),
            );
            docx_blocks(&mut document, &scene.blocks, 0);
        }
    }
    let screenplay = has_screenplay(manuscript);
    if screenplay {
        document.push_str(r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1417" w:right="1417" w:bottom="1134" w:left="1701" w:header="0" w:footer="0" w:gutter="0"/></w:sectPr></w:body></w:document>"#);
    } else {
        document.push_str(
            r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#,
        );
    }
    package_docx(&document, screenplay)
}
pub(super) fn docx_blocks(document: &mut String, blocks: &[Block], depth: usize) {
    for (index, block) in blocks.iter().enumerate() {
        match block {
            Block::Screenplay(role, inlines) => {
                let style = presentation(*role).style;
                let _ = write!(document, r#"<w:p><w:pPr><w:pStyle w:val="{style}"/>"#);
                if *role == Role::Dialogue
                    && matches!(
                        blocks.get(index + 1),
                        Some(Block::Screenplay(Role::Dialogue | Role::Parenthetical, _))
                    )
                {
                    document.push_str(r#"<w:spacing w:after="0"/>"#);
                }
                document.push_str("</w:pPr>");
                docx_inlines(document, inlines);
                document.push_str("</w:p>");
            }
            Block::Paragraph(inlines) => docx_paragraph(document, inlines, None, None),
            Block::Heading(level, inlines) => {
                docx_paragraph(
                    document,
                    inlines,
                    Some(format!("Heading{}", (*level).saturating_add(2).min(6))),
                    None,
                );
            }
            Block::Quote(children) => {
                docx_prefixed_blocks(document, children, depth, "› ");
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                for (index, item) in items.iter().enumerate() {
                    let marker = if matches!(block, Block::BulletList(_)) {
                        "•".to_owned()
                    } else {
                        format!("{}.", index + 1)
                    };
                    docx_list_item(document, item, depth, &marker);
                }
            }
        }
    }
}
fn docx_list_item(document: &mut String, item: &[Block], depth: usize, marker: &str) {
    let prefix = format!("{}{} ", "  ".repeat(depth), marker);
    docx_prefixed_blocks(document, item, depth, &prefix);
}
fn docx_prefixed_blocks(document: &mut String, blocks: &[Block], depth: usize, prefix: &str) {
    if let Some((Block::Paragraph(inlines), remainder)) = blocks.split_first() {
        docx_paragraph(document, inlines, None, Some(prefix));
        docx_blocks(document, remainder, depth + 1);
    } else {
        docx_text_paragraph(document, prefix, None, false, false, None);
        docx_blocks(document, blocks, depth + 1);
    }
}
fn docx_paragraph(
    document: &mut String,
    inlines: &[Inline],
    style: Option<String>,
    prefix: Option<&str>,
) {
    document.push_str("<w:p>");
    if let Some(style) = style {
        let _ = write!(document, r#"<w:pPr><w:pStyle w:val="{style}"/></w:pPr>"#);
    }
    if let Some(prefix) = prefix {
        docx_run(document, prefix, false, false, None);
    }
    docx_inlines(document, inlines);
    document.push_str("</w:p>");
}
fn docx_inlines(document: &mut String, inlines: &[Inline]) {
    for inline in inlines {
        match inline {
            Inline::HardBreak => document.push_str("<w:r><w:br/></w:r>"),
            Inline::Text { text, bold, italic } => {
                docx_run(document, text, *bold, *italic, None);
            }
        }
    }
}
pub(super) fn docx_text_paragraph(
    document: &mut String,
    text: &str,
    style: Option<&str>,
    bold: bool,
    italic: bool,
    size: Option<u16>,
) {
    document.push_str("<w:p>");
    if let Some(style) = style {
        let _ = write!(document, r#"<w:pPr><w:pStyle w:val="{style}"/></w:pPr>"#);
    }
    docx_run(document, text, bold, italic, size);
    document.push_str("</w:p>");
}
fn docx_run(document: &mut String, text: &str, bold: bool, italic: bool, size: Option<u16>) {
    document.push_str("<w:r>");
    if bold || italic || size.is_some() {
        document.push_str("<w:rPr>");
        if bold {
            document.push_str("<w:b/>");
        }
        if italic {
            document.push_str("<w:i/>");
        }
        if let Some(size) = size {
            let _ = write!(document, r#"<w:sz w:val="{size}"/>"#);
        }
        document.push_str("</w:rPr>");
    }
    document.push_str(r#"<w:t xml:space="preserve">"#);
    document.push_str(&escape_xml(text));
    document.push_str("</w:t></w:r>");
}
fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn add_screenplay_style(styles: &mut String, role: Role) {
    let presentation = presentation(role);
    // OOXML margins/indents are in twentieths of a point, rounded from the profile's millimetres.
    let (left, right) = match role {
        Role::Character => (3118, 0),
        Role::Parenthetical => (2268, 1417),
        Role::Dialogue => (1417, 1417),
        _ => (0, 0),
    };
    let alignment = if role == Role::Transition {
        "right"
    } else {
        "left"
    };
    let after = if matches!(role, Role::Character | Role::Parenthetical) {
        0
    } else {
        240
    };
    let keep_next = if matches!(
        role,
        Role::SceneHeading | Role::Character | Role::Parenthetical
    ) {
        "<w:keepNext/>"
    } else {
        ""
    };
    let _ = write!(
        styles,
        r#"<w:style w:type="paragraph" w:styleId="{}"><w:name w:val="Drehbuch – {}"/><w:basedOn w:val="Normal"/><w:pPr>{keep_next}<w:widowControl/><w:suppressAutoHyphens/><w:spacing w:before="0" w:after="{after}" w:line="280" w:lineRule="exact"/><w:ind w:left="{left}" w:right="{right}"/><w:jc w:val="{alignment}"/></w:pPr><w:rPr><w:rFonts w:ascii="JetBrains Mono" w:hAnsi="JetBrains Mono" w:eastAsia="JetBrains Mono" w:cs="JetBrains Mono"/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:style>"#,
        presentation.style, presentation.label
    );
}

fn package_docx(document: &str, screenplay: bool) -> Result<Vec<u8>, ExportError> {
    const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#;
    const PACKAGE_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/></w:style><w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/></w:style><w:style w:type="paragraph" w:styleId="Heading4"><w:name w:val="heading 4"/></w:style><w:style w:type="paragraph" w:styleId="Heading5"><w:name w:val="heading 5"/></w:style><w:style w:type="paragraph" w:styleId="Heading6"><w:name w:val="heading 6"/></w:style></w:styles>"#;

    let mut styles = STYLES.to_owned();
    if screenplay {
        styles.truncate(styles.len() - "</w:styles>".len());
        for role in ROLES {
            add_screenplay_style(&mut styles, role);
        }
        styles.push_str("</w:styles>");
    }

    let cursor = Cursor::new(Vec::new());
    let mut archive = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, content) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", PACKAGE_RELS),
        ("word/document.xml", document),
        ("word/styles.xml", styles.as_str()),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS),
    ] {
        archive.start_file(name, options).map_err(internal)?;
        archive.write_all(content.as_bytes()).map_err(internal)?;
    }
    archive.finish().map(Cursor::into_inner).map_err(internal)
}

fn pdf(manuscript: &Manuscript) -> Result<Vec<u8>, ExportError> {
    crate::pdf::text(manuscript).map_err(|_| ExportError::InvalidDocument(
        "Der PDF-Auftrag enthält nicht unterstützte Zeichen oder überschreitet die Layoutgrenzen."
    ))
}
