//! Validierter, lokaler Manuskriptexport in portable Formate.

mod render;

use kraqel_domain::{
    DocumentKind, DocumentNode, EDITOR_DOCUMENT_VERSION, EditorDocument, ImagePromptDraft, Project,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    error::Error,
    fmt, fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportFormat {
    Markdown,
    Html,
    Docx,
    Pdf,
}

impl ExportFormat {
    const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Docx => "docx",
            Self::Pdf => "pdf",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSourceDocument {
    pub node: DocumentNode,
    pub content: EditorDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub format: ExportFormat,
    pub file_name: String,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptExportResult {
    pub file_name: String,
    pub byte_count: u64,
}

#[derive(Debug)]
pub enum ExportError {
    InvalidDocument(&'static str),
    UnsafeDestination,
    Internal(Box<dyn Error + Send + Sync>),
}
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDocument(message) => f.write_str(message),
            Self::UnsafeDestination => f.write_str("Das Exportziel ist nicht sicher."),
            Self::Internal(_) => f.write_str("Der Export konnte nicht erstellt werden."),
        }
    }
}
impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::Internal(error) = self {
            Some(error.as_ref())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Manuscript {
    pub(crate) title: String,
    pub(crate) chapters: Vec<Chapter>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Chapter {
    pub(crate) title: String,
    pub(crate) scenes: Vec<Scene>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scene {
    pub(crate) title: String,
    pub(crate) blocks: Vec<Block>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Block {
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    BulletList(Vec<Vec<Block>>),
    OrderedList(Vec<Vec<Block>>),
    Quote(Vec<Block>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Inline {
    Text {
        text: String,
        bold: bool,
        italic: bool,
    },
    HardBreak,
}

/// Exportiert ein Projekt ausschließlich in dessen lokalem Unterordner `exports`.
///
/// # Errors
/// Liefert einen Fehler bei ungültigen Dokumenten, unsicheren Zielen oder fehlgeschlagenem Rendern.
pub fn export_project(
    project_directory: &Path,
    project: &Project,
    documents: &[ExportSourceDocument],
    format: ExportFormat,
) -> Result<ExportResult, ExportError> {
    let bytes = render::render(&build_manuscript(project, documents)?, format)?;
    let directory = prepare_export_directory(project_directory)?;
    let file_name = format!("manuskript.{}", format.extension());
    atomic_write(&directory, &file_name, &bytes)?;
    Ok(ExportResult {
        format,
        file_name,
        byte_count: u64::try_from(bytes.len()).map_err(internal)?,
    })
}

/// Exportiert genau einen gespeicherten Prompt als UTF-8/LF-Text unter `exports/prompts`.
///
/// # Errors
/// Liefert einen Fehler bei projektfremden Daten oder einem unsicheren Exportziel.
pub fn export_prompt_draft(
    project_directory: &Path,
    project: &Project,
    draft: &ImagePromptDraft,
) -> Result<PromptExportResult, ExportError> {
    if draft.project_id != project.id || draft.prompt_text.trim().is_empty() {
        return Err(ExportError::InvalidDocument(
            "Der Bildprompt-Entwurf ist ungültig.",
        ));
    }
    let root = prepare_export_directory(project_directory)?;
    let directory = prepare_child_directory(&root, "prompts")?;
    let file_name = format!("bildprompt-{}.txt", draft.id);
    let mut normalized = draft.prompt_text.replace("\r\n", "\n").replace('\r', "\n");
    if !normalized.ends_with('\n') {
        normalized.push('\n');
    }
    let bytes = normalized.as_bytes();
    atomic_write(&directory, &file_name, bytes)?;
    Ok(PromptExportResult {
        file_name,
        byte_count: u64::try_from(bytes.len()).map_err(internal)?,
    })
}

fn build_manuscript(
    project: &Project,
    documents: &[ExportSourceDocument],
) -> Result<Manuscript, ExportError> {
    if documents
        .iter()
        .any(|document| document.node.project_id != project.id)
    {
        return Err(ExportError::InvalidDocument(
            "Der Dokumentbaum enthält projektfremde Einträge.",
        ));
    }
    let mut chapters: Vec<_> = documents
        .iter()
        .filter(|document| document.node.kind == DocumentKind::Chapter)
        .collect();
    chapters.sort_by_key(|document| (document.node.position, document.node.id.to_string()));
    let mut claimed_scenes = 0;
    let mut result = Vec::with_capacity(chapters.len());
    for chapter in chapters {
        if chapter.node.parent_id.is_some() {
            return Err(ExportError::InvalidDocument(
                "Die Kapitelstruktur ist ungültig.",
            ));
        }
        let mut scenes: Vec<_> = documents
            .iter()
            .filter(|document| document.node.parent_id == Some(chapter.node.id))
            .collect();
        scenes.sort_by_key(|document| (document.node.position, document.node.id.to_string()));
        claimed_scenes += scenes.len();
        let scenes = scenes
            .into_iter()
            .map(|scene| {
                if scene.node.kind != DocumentKind::Scene {
                    return Err(ExportError::InvalidDocument(
                        "Die Szenenstruktur ist ungültig.",
                    ));
                }
                Ok(Scene {
                    title: scene.node.title.clone(),
                    blocks: parse_editor_document(&scene.content)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        result.push(Chapter {
            title: chapter.node.title.clone(),
            scenes,
        });
    }
    if claimed_scenes
        != documents
            .iter()
            .filter(|document| document.node.kind == DocumentKind::Scene)
            .count()
    {
        return Err(ExportError::InvalidDocument(
            "Der Dokumentbaum enthält verwaiste Szenen.",
        ));
    }
    Ok(Manuscript {
        title: project.name.clone(),
        chapters: result,
    })
}

fn object(value: &Value) -> Result<&Map<String, Value>, ExportError> {
    value.as_object().ok_or(ExportError::InvalidDocument(
        "Ein Editorknoten ist ungültig.",
    ))
}
fn array<'value>(
    value: Option<&'value Value>,
    message: &'static str,
) -> Result<&'value [Value], ExportError> {
    match value {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(ExportError::InvalidDocument(message)),
    }
}
fn parse_editor_document(content: &EditorDocument) -> Result<Vec<Block>, ExportError> {
    if content.version != EDITOR_DOCUMENT_VERSION {
        return Err(ExportError::InvalidDocument(
            "Die Editor-Dokumentversion wird nicht unterstützt.",
        ));
    }
    let root = object(&content.document)?;
    if root.get("type").and_then(Value::as_str) != Some("doc") {
        return Err(ExportError::InvalidDocument(
            "Der Editorinhalt besitzt keine Dokumentwurzel.",
        ));
    }
    parse_blocks(root.get("content"))
}
fn parse_blocks(value: Option<&Value>) -> Result<Vec<Block>, ExportError> {
    array(value, "Der Blockinhalt ist ungültig.")?
        .iter()
        .map(parse_block)
        .collect()
}
fn parse_block(value: &Value) -> Result<Block, ExportError> {
    let node = object(value)?;
    match node.get("type").and_then(Value::as_str) {
        Some("paragraph") => Ok(Block::Paragraph(parse_inlines(node.get("content"))?)),
        Some("heading") => {
            let level = node
                .get("attrs")
                .and_then(|attrs| attrs.get("level"))
                .and_then(Value::as_u64)
                .and_then(|level| u8::try_from(level).ok())
                .filter(|level| (1..=6).contains(level))
                .ok_or(ExportError::InvalidDocument(
                    "Die Überschriftenebene ist ungültig.",
                ))?;
            Ok(Block::Heading(level, parse_inlines(node.get("content"))?))
        }
        Some("bulletList") => Ok(Block::BulletList(parse_list(node.get("content"))?)),
        Some("orderedList") => Ok(Block::OrderedList(parse_list(node.get("content"))?)),
        Some("blockquote") => Ok(Block::Quote(parse_blocks(node.get("content"))?)),
        _ => Err(ExportError::InvalidDocument(
            "Der Editorinhalt enthält eine noch nicht unterstützte Formatierung.",
        )),
    }
}
fn parse_list(value: Option<&Value>) -> Result<Vec<Vec<Block>>, ExportError> {
    array(value, "Die Liste ist ungültig.")?
        .iter()
        .map(|item| {
            let item = object(item)?;
            if item.get("type").and_then(Value::as_str) != Some("listItem") {
                return Err(ExportError::InvalidDocument(
                    "Ein Listeneintrag ist ungültig.",
                ));
            }
            parse_blocks(item.get("content"))
        })
        .collect()
}
fn parse_inlines(value: Option<&Value>) -> Result<Vec<Inline>, ExportError> {
    array(value, "Der Textinhalt ist ungültig.")?
        .iter()
        .map(|value| {
            let node = object(value)?;
            match node.get("type").and_then(Value::as_str) {
                Some("hardBreak") => Ok(Inline::HardBreak),
                Some("text") => {
                    let text = node
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or(ExportError::InvalidDocument(
                            "Ein Textknoten enthält keinen Text.",
                        ))?
                        .to_owned();
                    let mut bold = false;
                    let mut italic = false;
                    for mark in array(node.get("marks"), "Eine Textmarkierung ist ungültig.")? {
                        match object(mark)?.get("type").and_then(Value::as_str) {
                            Some("bold") if !bold => bold = true,
                            Some("italic") if !italic => italic = true,
                            _ => {
                                return Err(ExportError::InvalidDocument(
                                    "Der Text enthält eine noch nicht unterstützte Markierung.",
                                ));
                            }
                        }
                    }
                    Ok(Inline::Text { text, bold, italic })
                }
                _ => Err(ExportError::InvalidDocument(
                    "Der Text enthält einen noch nicht unterstützten Knoten.",
                )),
            }
        })
        .collect()
}

fn prepare_export_directory(project_directory: &Path) -> Result<PathBuf, ExportError> {
    let metadata = fs::symlink_metadata(project_directory).map_err(internal)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExportError::UnsafeDestination);
    }
    let directory = project_directory.join("exports");
    match fs::symlink_metadata(&directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(ExportError::UnsafeDestination);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&directory).map_err(internal)?;
        }
        Err(error) => return Err(internal(error)),
    }
    Ok(directory)
}

fn prepare_child_directory(parent: &Path, name: &str) -> Result<PathBuf, ExportError> {
    let directory = parent.join(name);
    match fs::symlink_metadata(&directory) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(ExportError::UnsafeDestination)
        }
        Ok(_) => Ok(directory),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&directory).map_err(internal)?;
            Ok(directory)
        }
        Err(error) => Err(internal(error)),
    }
}
fn atomic_write(directory: &Path, file_name: &str, bytes: &[u8]) -> Result<(), ExportError> {
    let temporary = directory.join(format!(".{file_name}.tmp"));
    let previous = directory.join(format!(".{file_name}.previous"));
    let destination = directory.join(file_name);
    if fs::symlink_metadata(&temporary).is_ok()
        || fs::symlink_metadata(&previous).is_ok()
        || fs::symlink_metadata(&destination)
            .is_ok_and(|metadata| metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(ExportError::UnsafeDestination);
    }
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(internal)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(internal)?;
    drop(file);
    let had_destination = destination.exists();
    if had_destination {
        fs::rename(&destination, &previous).map_err(internal)?;
    }
    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ = fs::remove_file(&temporary);
        if had_destination {
            let _ = fs::rename(&previous, &destination);
        }
        return Err(internal(error));
    }
    if had_destination {
        fs::remove_file(&previous).map_err(internal)?;
    }
    Ok(())
}
pub(crate) fn internal(error: impl Error + Send + Sync + 'static) -> ExportError {
    ExportError::Internal(Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId, ProjectId};
    use serde_json::json;
    use tempfile::tempdir;
    fn fixture() -> (Project, Vec<ExportSourceDocument>) {
        let project_id = ProjectId::new();
        let chapter_id = EntityId::new();
        let project = Project {
            id: project_id,
            name: "Der Kolkrabe & das Archiv".to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let node = |id, parent_id, kind, title: &str, position| DocumentNode {
            id,
            project_id,
            parent_id,
            kind,
            title: title.to_owned(),
            position,
            revision: 1,
        };
        (
            project,
            vec![
                ExportSourceDocument {
                    node: node(chapter_id, None, DocumentKind::Chapter, "Ankunft", 0),
                    content: EditorDocument {
                        version: 1,
                        document: json!({"type":"doc","content":[]}),
                    },
                },
                ExportSourceDocument {
                    node: node(
                        EntityId::new(),
                        Some(chapter_id),
                        DocumentKind::Scene,
                        "Ruf im Nebel",
                        0,
                    ),
                    content: EditorDocument {
                        version: 1,
                        document: json!({"type":"doc","content":[ {"type":"paragraph","content":[{"type":"text","text":"Ein "},{"type":"text","text":"kluger","marks":[{"type":"bold"}]},{"type":"text","text":" Kolkrabe.","marks":[{"type":"italic"}]}]}, {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Erstens"}]}]}]}, {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Niemals eine Krähe."}]}]} ]}),
                    },
                },
            ],
        )
    }
    #[test]
    fn all_formats_are_written_under_exports() {
        let root = tempdir().expect("root");
        let (project, documents) = fixture();
        for format in [
            ExportFormat::Markdown,
            ExportFormat::Html,
            ExportFormat::Docx,
            ExportFormat::Pdf,
        ] {
            let result = export_project(root.path(), &project, &documents, format).expect("export");
            let bytes = fs::read(root.path().join("exports").join(result.file_name)).expect("read");
            assert_ne!(bytes.len(), 0);
        }
        assert!(
            fs::read_to_string(root.path().join("exports/manuskript.md"))
                .expect("markdown")
                .contains("**kluger**")
        );
        assert!(
            fs::read_to_string(root.path().join("exports/manuskript.html"))
                .expect("html")
                .contains("<strong>kluger</strong>")
        );
        assert!(
            fs::read(root.path().join("exports/manuskript.docx"))
                .expect("docx")
                .starts_with(b"PK")
        );
        assert!(
            fs::read(root.path().join("exports/manuskript.pdf"))
                .expect("pdf")
                .starts_with(b"%PDF")
        );
    }
    #[test]
    fn unsupported_nodes_are_rejected_instead_of_dropped() {
        let (project, mut documents) = fixture();
        documents[1].content.document = json!({"type":"doc","content":[{"type":"codeBlock"}]});
        assert!(matches!(
            build_manuscript(&project, &documents),
            Err(ExportError::InvalidDocument(_))
        ));
    }

    #[test]
    fn prompt_draft_is_exported_as_normalized_utf8_text() {
        let root = tempdir().expect("root");
        let (project, _) = fixture();
        let draft = ImagePromptDraft {
            id: EntityId::new(),
            project_id: project.id,
            name: "Abendlicht".to_owned(),
            source_document_id: EntityId::new(),
            source_document_revision: 3,
            entity_ids: vec![EntityId::new()],
            style_preset_id: None,
            prompt_text: "Kolkrabe im Abendlicht\r\nüber dem Turm".to_owned(),
        };
        let result = export_prompt_draft(root.path(), &project, &draft).expect("prompt export");
        let bytes = fs::read(root.path().join("exports/prompts").join(&result.file_name))
            .expect("prompt file");
        assert_eq!(bytes, "Kolkrabe im Abendlicht\nüber dem Turm\n".as_bytes());
        assert_eq!(result.byte_count, bytes.len() as u64);
    }

    #[test]
    #[ignore = "erzeugt ausschließlich visuelle QA-Artefakte unter target/export-qa"]
    fn generate_visual_qa_documents() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/export-qa")
            .join("project");
        fs::create_dir_all(&directory).expect("QA directory");
        let (project, documents) = fixture();
        export_project(&directory, &project, &documents, ExportFormat::Docx).expect("DOCX QA");
        export_project(&directory, &project, &documents, ExportFormat::Pdf).expect("PDF QA");
    }
}
