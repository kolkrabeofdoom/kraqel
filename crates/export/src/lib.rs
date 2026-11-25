//! Validierter, lokaler Manuskriptexport in portable Formate.

mod docx_scene;
mod excerpt_file;
pub use excerpt_file::{ExcerptFile, list_excerpt_files, pin_excerpt_project};
mod excerpt;
pub use excerpt::{ExportScope, export_excerpt};
pub mod illustrated;
mod manuscript;
mod pdf;
mod render;
mod roundtrip;
mod screenplay;
pub use pdf::screenplay::export_screenplay_pdf;

pub use roundtrip::{
    DocxRoundtripError, read_docx_roundtrip, read_docx_roundtrip_file, write_docx_roundtrip,
};

/// Liest den bestehenden begrenzten DOCX-Szenenimport aus einer lokalen Datei.
///
/// # Errors
/// Lehnt ungültige Pfade, Inhalte und Ressourcenüberschreitungen ohne Eingabedetails ab.
pub fn read_docx_scene_file(
    path: &std::path::Path,
) -> Result<(String, kraqel_domain::EditorDocument), DocxRoundtripError> {
    docx_scene::read_file(path).map_err(|()| DocxRoundtripError)
}

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
pub struct Manuscript {
    pub title: String,
    pub chapters: Vec<Chapter>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub title: String,
    pub scenes: Vec<Scene>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub title: String,
    pub blocks: Vec<Block>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Screenplay(kraqel_domain::ScreenplayRole, Vec<Inline>),
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    BulletList(Vec<Vec<Block>>),
    OrderedList(Vec<Vec<Block>>),
    Quote(Vec<Block>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
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
    let manuscript = Manuscript::from_export_documents(project, documents).map_err(|_| {
        ExportError::InvalidDocument(
            "Das Manuskript enthält eine ungültige oder nicht unterstützte Struktur.",
        )
    })?;
    let bytes = render::render(&manuscript, format)?;
    let directory = prepare_export_directory(project_directory)?;
    let file_name = format!("manuskript.{}", format.extension());
    atomic_write(&directory, &file_name, &bytes)?;
    Ok(ExportResult {
        format,
        file_name,
        byte_count: u64::try_from(bytes.len()).map_err(internal)?,
    })
}

/// Veröffentlicht den vollständig selbstgeprüften Roundtrip atomar unter dem festen Exportnamen.
///
/// # Errors
/// Lehnt ungültige Manuskripte und unsichere Dateiziele vor der Veröffentlichung ab.
pub fn export_project_roundtrip(
    project_directory: &Path,
    project: &Project,
    documents: &[ExportSourceDocument],
) -> Result<ExportResult, ExportError> {
    let manuscript = Manuscript::from_documents(project, documents).map_err(|_| {
        ExportError::InvalidDocument("Das Manuskript ist für diesen Austausch nicht geeignet.")
    })?;
    let bytes = write_docx_roundtrip(&manuscript).map_err(|_| {
        ExportError::InvalidDocument("Das Manuskript ist für diesen Austausch nicht geeignet.")
    })?;
    let directory = prepare_export_directory(project_directory)?;
    let file_name = "manuskript-roundtrip.docx".to_owned();
    atomic_write(&directory, &file_name, &bytes)?;
    Ok(ExportResult {
        format: ExportFormat::Docx,
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
    if content.version == kraqel_domain::SCREENPLAY_DOCUMENT_VERSION {
        return screenplay::paragraphs(content).map(|ps| {
            ps.into_iter()
                .map(|p| Block::Screenplay(p.role, p.runs))
                .collect()
        });
    }
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
    use std::io::{Cursor, Read};
    use tempfile::tempdir;
    use zip::ZipArchive;
    pub(super) fn fixture() -> (Project, Vec<ExportSourceDocument>) {
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
                        document: json!({"type":"doc","content":[ {"type":"paragraph","content":[{"type":"text","text":"Ein "},{"type":"text","text":"kluger","marks":[{"type":"bold"}]},{"type":"text","text":" Kolkrabe.","marks":[{"type":"italic"}]},{"type":"hardBreak"},{"type":"text","text":"Zweite Zeile"}]}, {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Erstens"}]}]}]}, {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Niemals eine Krähe."}]}]} ]}),
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
        let docx = fs::read(root.path().join("exports/manuskript.docx")).expect("docx");
        let mut archive = ZipArchive::new(Cursor::new(docx)).expect("valid docx archive");
        for name in [
            "[Content_Types].xml",
            "_rels/.rels",
            "word/document.xml",
            "word/styles.xml",
            "word/_rels/document.xml.rels",
        ] {
            assert!(archive.by_name(name).is_ok(), "missing DOCX part {name}");
        }
        let mut document = String::new();
        archive
            .by_name("word/document.xml")
            .expect("document part")
            .read_to_string(&mut document)
            .expect("UTF-8 document XML");
        assert!(document.contains("Der Kolkrabe &amp; das Archiv"));
        assert!(document.contains("<w:b/></w:rPr><w:t xml:space=\"preserve\">kluger</w:t>"));
        assert!(document.contains("<w:i/></w:rPr><w:t xml:space=\"preserve\"> Kolkrabe.</w:t>"));
        assert!(document.contains("<w:r><w:br/></w:r>"));
        assert!(document.contains("<w:t xml:space=\"preserve\">Zweite Zeile</w:t>"));
        assert!(document.contains("<w:t xml:space=\"preserve\">• </w:t>"));
        assert!(document.contains("<w:t xml:space=\"preserve\">› </w:t>"));
        assert!(
            fs::read(root.path().join("exports/manuskript.pdf"))
                .expect("pdf")
                .starts_with(b"%PDF")
        );
    }
    fn mixed_fixture() -> (Project, Vec<ExportSourceDocument>) {
        let (project, mut documents) = fixture();
        let mut scene = documents[1].clone();
        scene.node.id = EntityId::new();
        scene.node.title = "Die Rückseite der Karte".into();
        scene.node.position = 1;
        let mut paragraphs: Vec<_> = [
            ("sceneHeading", "INT. STADTARCHIV – MORGEN"),
            ("action", "Auf der Karte glänzt Tinte."),
            ("character", "Mira"),
            ("parenthetical", "(liest leise)"),
            ("dialogue", "<Gleis *sieben*>"),
            ("transition", "Schnitt auf:"),
        ].into_iter().map(|(role, text)| json!({"type":"screenplayBlock","attrs":{"role":role},"content":[{"type":"text","text":text}]})).collect();
        paragraphs[4]["content"][0]["marks"] = json!([{"type":"bold"},{"type":"italic"}]);
        let dialogue = paragraphs[4]["content"].as_array_mut().unwrap();
        dialogue.push(json!({"type":"hardBreak"}));
        dialogue.push(json!({"type":"text","text":"Der Zug wartet."}));
        scene.content = EditorDocument {
            version: kraqel_domain::SCREENPLAY_DOCUMENT_VERSION,
            document: json!({"type":"doc","content":paragraphs}),
        };
        documents.push(scene);
        (project, documents)
    }

    #[test]
    fn mixed_screenplay_preserves_roles_without_entering_prose_only_profiles() {
        let (project, documents) = mixed_fixture();
        let manuscript = Manuscript::from_export_documents(&project, &documents).unwrap();
        let bytes = |format| render::render(&manuscript, format).unwrap();
        let markdown = String::from_utf8(bytes(ExportFormat::Markdown)).unwrap();
        let html = String::from_utf8(bytes(ExportFormat::Html)).unwrap();
        let docx = bytes(ExportFormat::Docx);
        let mut zip = ZipArchive::new(Cursor::new(&docx)).unwrap();
        let mut part = |name| {
            let mut xml = String::new();
            zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
            xml
        };
        let document = part("word/document.xml");
        let styles = part("word/styles.xml");
        for (role, label, style) in [
            ("sceneHeading", "Szenenkopf", "SceneHeading"),
            ("action", "Handlung", "Action"),
            ("character", "Figur", "Character"),
            ("parenthetical", "Klammeranweisung", "Parenthetical"),
            ("dialogue", "Dialog", "Dialogue"),
            ("transition", "Übergang", "Transition"),
        ] {
            assert!(markdown.contains(&format!("**{label}**\n\n")));
            assert!(html.contains(&format!(
                r#"<p class="screenplay screenplay-{role}" data-kraqel-role="{role}">"#
            )));
            assert!(document.contains(&format!(r#"<w:pStyle w:val="KQScreenplay{style}"/>"#)));
            assert!(styles.contains(&format!(r#"w:styleId="KQScreenplay{style}""#)));
        }
        assert!(markdown.contains("***\\<Gleis \\*sieben\\*\\>***  \nDer Zug wartet."));
        assert!(
            html.contains("<strong><em>&lt;Gleis *sieben*&gt;</em></strong><br>Der Zug wartet.")
        );
        assert!(document.contains("Ruf im Nebel"));
        assert!(document.contains("Die Rückseite der Karte"));
        assert!(render::render(&manuscript, ExportFormat::Pdf).is_err());
        assert!(Manuscript::from_documents(&project, &documents).is_err());
        assert!(write_docx_roundtrip(&manuscript).is_err());
        assert!(read_docx_roundtrip(&docx).is_err());
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
        let directory = directory.join("screenplay");
        fs::create_dir_all(&directory).unwrap();
        let (project, documents) = mixed_fixture();
        for format in [
            ExportFormat::Markdown,
            ExportFormat::Html,
            ExportFormat::Docx,
        ] {
            export_project(&directory, &project, &documents, format).expect("screenplay QA");
        }
    }
}
