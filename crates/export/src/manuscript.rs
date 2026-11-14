//! Format-neutral manuscript validation and editor conversion.
use crate::{Block, Chapter, DocxRoundtripError, ExportSourceDocument, Inline, Manuscript, Scene};
use kraqel_domain::{DocumentKind, EDITOR_DOCUMENT_VERSION, EditorDocument, Project};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Default)]
struct Budget {
    nodes: usize,
    text: usize,
    inlines: usize,
    ordinary_export: bool,
}
impl Budget {
    fn node(&mut self, depth: usize) -> Result<(), DocxRoundtripError> {
        self.nodes += 1;
        if (!self.ordinary_export && self.nodes > 4096) || depth > 32 {
            return Err(DocxRoundtripError);
        }
        Ok(())
    }
    fn text(&mut self, text: &str) -> Result<(), DocxRoundtripError> {
        self.text = self
            .text
            .checked_add(text.len())
            .ok_or(DocxRoundtripError)?;
        if (!self.ordinary_export && self.text > 2 * 1024 * 1024)
            || !crate::docx_scene::valid_text(text)
        {
            return Err(DocxRoundtripError);
        }
        Ok(())
    }
}

impl Manuscript {
    /// Validiert Grenzen und bildet benachbarte gleich formatierte Textläufe kanonisch ab.
    ///
    /// # Errors
    /// Lehnt ungültige oder übermäßig große/tiefe Manuskripte ab.
    pub fn canonicalized(&self) -> Result<Self, DocxRoundtripError> {
        self.canonicalized_for(false)
    }
    fn canonicalized_for(&self, ordinary_export: bool) -> Result<Self, DocxRoundtripError> {
        let mut budget = Budget {
            ordinary_export,
            ..Budget::default()
        };
        budget.node(1)?;
        budget.node(2)?;
        budget.text(&self.title)?;
        let mut chapters = Vec::new();
        for chapter in &self.chapters {
            budget.node(2)?;
            budget.node(3)?;
            budget.text(&chapter.title)?;
            let mut scenes = Vec::new();
            for scene in &chapter.scenes {
                budget.node(3)?;
                budget.node(4)?;
                budget.text(&scene.title)?;
                scenes.push(Scene {
                    title: scene.title.clone(),
                    blocks: blocks(&scene.blocks, 4, &mut budget)?,
                });
            }
            chapters.push(Chapter {
                title: chapter.title.clone(),
                scenes,
            });
        }
        Ok(Self {
            title: self.title.clone(),
            chapters,
        })
    }

    /// Erstellt den Roundtrip-Umfang aus einem projektgebundenen Editorbaum.
    ///
    /// # Errors
    /// Lehnt fremde/doppelte Knoten und nicht verlustfrei darstellbare Editorattribute ab.
    pub fn from_documents(
        project: &Project,
        documents: &[ExportSourceDocument],
    ) -> Result<Self, DocxRoundtripError> {
        Self::from_documents_for(project, documents, false)
    }
    pub(crate) fn from_export_documents(
        project: &Project,
        documents: &[ExportSourceDocument],
    ) -> Result<Self, DocxRoundtripError> {
        Self::from_documents_for(project, documents, true)
    }
    fn from_documents_for(
        project: &Project,
        documents: &[ExportSourceDocument],
        ordinary_export: bool,
    ) -> Result<Self, DocxRoundtripError> {
        if !ordinary_export && documents.len() > 4096 {
            return Err(DocxRoundtripError);
        }
        let mut preflight = Budget {
            ordinary_export,
            ..Budget::default()
        };
        let mut editor_nodes = 0;
        preflight.text(&project.name)?;
        let mut ids = BTreeSet::new();
        for document in documents {
            if !ids.insert(document.node.id) {
                return Err(DocxRoundtripError);
            }
            preflight.text(&document.node.title)?;
            editor_shape(
                &document.content.document,
                &mut editor_nodes,
                &mut preflight,
            )?;
            if document.node.kind == DocumentKind::Chapter {
                let content = crate::parse_editor_document(&document.content)
                    .map_err(|_| DocxRoundtripError)?;
                if content
                    .iter()
                    .any(|b| !matches!(b, Block::Paragraph(runs) if runs.is_empty()))
                {
                    return Err(DocxRoundtripError);
                }
            }
        }
        crate::build_manuscript(project, documents)
            .map_err(|_| DocxRoundtripError)?
            .canonicalized_for(ordinary_export)
    }
}

fn blocks(
    source: &[Block],
    depth: usize,
    budget: &mut Budget,
) -> Result<Vec<Block>, DocxRoundtripError> {
    source
        .iter()
        .map(|block| {
            budget.node(depth)?;
            Ok(match block {
                Block::Paragraph(runs) => Block::Paragraph(inlines(runs, budget)?),
                Block::Heading(level, runs) if (1..=6).contains(level) => {
                    Block::Heading(*level, inlines(runs, budget)?)
                }
                Block::Quote(children) if !children.is_empty() => {
                    Block::Quote(blocks(children, depth + 1, budget)?)
                }
                Block::Heading(_, _) | Block::Quote(_) => return Err(DocxRoundtripError),
                Block::BulletList(items) | Block::OrderedList(items) => {
                    if items.is_empty() {
                        return Err(DocxRoundtripError);
                    }
                    let mut result = Vec::new();
                    for item in items {
                        budget.node(depth + 1)?;
                        if !matches!(item.first(), Some(Block::Paragraph(_))) {
                            return Err(DocxRoundtripError);
                        }
                        result.push(blocks(item, depth + 2, budget)?);
                    }
                    if matches!(block, Block::BulletList(_)) {
                        Block::BulletList(result)
                    } else {
                        Block::OrderedList(result)
                    }
                }
            })
        })
        .collect()
}

fn inlines(source: &[Inline], budget: &mut Budget) -> Result<Vec<Inline>, DocxRoundtripError> {
    let mut output = Vec::new();
    for inline in source {
        budget.inlines += 1;
        if !budget.ordinary_export && budget.inlines > 100_000 {
            return Err(DocxRoundtripError);
        }
        match inline {
            Inline::HardBreak => {
                budget.text("\n")?;
                output.push(Inline::HardBreak);
            }
            Inline::Text { text, bold, italic } => {
                budget.text(text)?;
                if text.is_empty() {
                    continue;
                }
                if let Some(Inline::Text {
                    text: previous,
                    bold: b,
                    italic: i,
                }) = output.last_mut()
                    && b == bold
                    && i == italic
                {
                    previous.push_str(text);
                } else {
                    output.push(inline.clone());
                }
            }
        }
    }
    Ok(output)
}

impl Scene {
    /// Wandelt die validierte Manuskriptszene zurück in das bestehende Editorformat.
    ///
    /// # Errors
    /// Lehnt ungültige, zu große oder zu tiefe Szenen ab.
    pub fn editor_document(&self) -> Result<EditorDocument, DocxRoundtripError> {
        let blocks = blocks(&self.blocks, 4, &mut Budget::default())?;
        Ok(EditorDocument {
            version: EDITOR_DOCUMENT_VERSION,
            document: json!({"type":"doc", "content":editor_blocks(&blocks)}),
        })
    }
}
fn editor_blocks(blocks: &[Block]) -> Vec<Value> {
    blocks.iter().map(|block| match block {
        Block::Paragraph(runs) => json!({"type":"paragraph", "content":editor_inlines(runs)}),
        Block::Heading(level,runs) => json!({"type":"heading", "attrs":{"level":level}, "content":editor_inlines(runs)}),
        Block::Quote(children) => json!({"type":"blockquote", "content":editor_blocks(children)}),
        Block::BulletList(items) | Block::OrderedList(items) => json!({"type": if matches!(block,Block::BulletList(_)) { "bulletList" } else { "orderedList" }, "content":items.iter().map(|item| json!({"type":"listItem","content":editor_blocks(item)})).collect::<Vec<_>>() }),
    }).collect()
}
fn editor_inlines(runs: &[Inline]) -> Vec<Value> {
    runs.iter()
        .map(|run| match run {
            Inline::HardBreak => json!({"type":"hardBreak"}),
            Inline::Text { text, bold, italic } => {
                let mut value = json!({"type":"text","text":text});
                let mut marks = Vec::new();
                if *bold {
                    marks.push(json!({"type":"bold"}));
                }
                if *italic {
                    marks.push(json!({"type":"italic"}));
                }
                if !marks.is_empty() {
                    value["marks"] = json!(marks);
                }
                value
            }
        })
        .collect()
}

fn editor_shape(
    root: &Value,
    count: &mut usize,
    budget: &mut Budget,
) -> Result<(), DocxRoundtripError> {
    let mut pending = vec![(root, 1)];
    while let Some((value, depth)) = pending.pop() {
        *count += 1;
        if depth > 32 || (!budget.ordinary_export && *count > 100_000) {
            return Err(DocxRoundtripError);
        }
        let node = value.as_object().ok_or(DocxRoundtripError)?;
        let kind = node
            .get("type")
            .and_then(Value::as_str)
            .ok_or(DocxRoundtripError)?;
        if kind == "text" {
            budget.text(
                node.get("text")
                    .and_then(Value::as_str)
                    .ok_or(DocxRoundtripError)?,
            )?;
        }
        if kind == "hardBreak" {
            budget.text("\n")?;
        }
        let allowed: &[&str] = match kind {
            "doc" => &["type", "content"],
            "text" => &["type", "text", "marks", "attrs"],
            "hardBreak" => &["type", "attrs"],
            "paragraph" | "heading" | "bulletList" | "orderedList" | "listItem" | "blockquote" => {
                &["type", "content", "attrs"]
            }
            _ => return Err(DocxRoundtripError),
        };
        if node.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err(DocxRoundtripError);
        }
        if let Some(attrs) = node.get("attrs") {
            let attrs = attrs.as_object().ok_or(DocxRoundtripError)?;
            for (key, value) in attrs {
                match (kind, key.as_str()) {
                    ("heading", "level")
                        if value.as_u64().is_some_and(|v| (1..=6).contains(&v)) => {}
                    ("orderedList", "start") if value.as_u64() == Some(1) => {}
                    ("orderedList", "type") if value.is_null() => {}
                    _ => return Err(DocxRoundtripError),
                }
            }
        }
        if let Some(marks) = node.get("marks") {
            for mark in marks.as_array().ok_or(DocxRoundtripError)? {
                let mark = mark.as_object().ok_or(DocxRoundtripError)?;
                if mark.len() != 1
                    || !matches!(
                        mark.get("type").and_then(Value::as_str),
                        Some("bold" | "italic")
                    )
                {
                    return Err(DocxRoundtripError);
                }
            }
        }
        if let Some(children) = node.get("content") {
            pending.extend(
                children
                    .as_array()
                    .ok_or(DocxRoundtripError)?
                    .iter()
                    .map(|child| (child, depth + 1)),
            );
        }
    }
    Ok(())
}
