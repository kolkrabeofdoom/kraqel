//! Local, single-use scene grants issued from the selected catalog row.
use crate::document_repository::{DocumentError, DocumentRepository, SaveGuard, StoredDocument};
use kraqel_domain::{
    DocumentKind, DocumentNode, EDITOR_DOCUMENT_VERSION, EditorDocument, EntityId, ProjectId,
};
use serde_json::{Value, json};

/// Maximum UTF-8 text size for the native scene channel.
pub const NATIVE_SCENE_TEXT_BYTES: usize = 64 * 1024;

/// A non-cloneable grant, issued from a scene row in the selected project catalog.
pub struct SceneReadGrant {
    project: ProjectId,
    scene: EntityId,
    revision: u64,
}

/// A display snapshot. An edit grant exists only for a losslessly reversible document.
pub struct NativeScene {
    pub text: String,
    pub edit: Option<SceneEditGrant>,
}

/// Bound to the exact document version that was shown to the person editing.
pub struct SceneEditGrant {
    binding: SceneReadGrant,
    original: EditorDocument,
}

impl SceneReadGrant {
    /// # Errors
    /// Chapters cannot receive a scene grant.
    pub fn from_node(node: &DocumentNode) -> Result<Self, DocumentError> {
        if node.kind != DocumentKind::Scene {
            return Err(DocumentError::InvalidInput);
        }
        Ok(Self {
            project: node.project_id,
            scene: node.id,
            revision: node.revision,
        })
    }

    /// # Errors
    /// Rejects stale, missing or unrepresentable data. Consumes the grant on every attempt.
    pub fn read(self, repository: &DocumentRepository<'_>) -> Result<NativeScene, DocumentError> {
        let source = repository.load(self.project, self.scene)?;
        if source.node.kind != DocumentKind::Scene || source.node.revision != self.revision {
            return Err(DocumentError::RevisionConflict);
        }
        self.project_source(source)
    }

    /// Loads only the current version of the explicitly requested scene from the selected project.
    ///
    /// # Errors
    /// Rejects missing scenes, wrong document kinds and documents outside the native text limit.
    pub fn latest_prose(
        project: ProjectId,
        scene: EntityId,
        repository: &DocumentRepository<'_>,
    ) -> Result<(u64, String), DocumentError> {
        let source = repository.load(project, scene)?;
        let grant = Self::from_node(&source.node)?;
        let (text, editable) = project_text(&source.content)?;
        if !editable {
            return Err(DocumentError::InvalidInput);
        }
        Ok((grant.revision, text))
    }

    fn project_source(self, source: StoredDocument) -> Result<NativeScene, DocumentError> {
        let (text, editable) = project_text(&source.content)?;
        let edit = editable.then_some(SceneEditGrant {
            binding: self,
            original: source.content,
        });
        Ok(NativeScene { text, edit })
    }
}

impl SceneEditGrant {
    /// Issues a fresh one-shot grant only for the exact canonical prose and revision reviewed.
    ///
    /// # Errors
    /// A later revision or changed source blocks the write, including changes during commit.
    pub fn save_reviewed(
        repository: &mut DocumentRepository<'_>,
        project: ProjectId,
        scene: EntityId,
        revision: u64,
        reviewed: &str,
        merged: &str,
    ) -> Result<u64, DocumentError> {
        let fresh = SceneReadGrant {
            project,
            scene,
            revision,
        }
        .read(repository)?;
        if fresh.text != reviewed {
            return Err(DocumentError::RevisionConflict);
        }
        fresh
            .edit
            .ok_or(DocumentError::RevisionConflict)?
            .save(repository, merged)
    }

    /// # Errors
    /// Rejects unsupported text or changes to the exact bound scene, revision or original structure.
    pub fn save(
        self,
        repository: &mut DocumentRepository<'_>,
        text: &str,
    ) -> Result<u64, DocumentError> {
        if !displayable(text) {
            return Err(DocumentError::InvalidInput);
        }
        repository.save_content(
            self.binding.project,
            self.binding.scene,
            self.binding.revision,
            &text_document(text),
            SaveGuard::ExactProse(&self.original),
        )
    }
}

fn project_text(content: &EditorDocument) -> Result<(String, bool), DocumentError> {
    let mut text = String::new();
    append_view(&content.document, &mut text)?;
    if !displayable(&text) {
        return Err(DocumentError::InvalidInput);
    }
    let editable = content.version == EDITOR_DOCUMENT_VERSION && text_document(&text) == *content;
    Ok((text, editable))
}

fn displayable(text: &str) -> bool {
    text.len() <= NATIVE_SCENE_TEXT_BYTES
        && !text
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
}

fn text_document(text: &str) -> EditorDocument {
    let paragraphs = text
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                json!({"type":"paragraph"})
            } else {
                json!({"type":"paragraph","content":[{"type":"text","text":line}]})
            }
        })
        .collect::<Vec<_>>();
    EditorDocument {
        version: EDITOR_DOCUMENT_VERSION,
        document: json!({"type":"doc","content":paragraphs}),
    }
}

fn append_view(node: &Value, output: &mut String) -> Result<(), DocumentError> {
    match node["type"].as_str() {
        Some("text") => output.push_str(
            node["text"]
                .as_str()
                .ok_or(DocumentError::InvalidStoredData)?,
        ),
        Some("hardBreak") => output.push('\n'),
        Some(kind) => {
            let inline = matches!(kind, "heading" | "paragraph" | "screenplayBlock");
            if let Some(children) = node.get("content") {
                for (i, child) in children
                    .as_array()
                    .ok_or(DocumentError::InvalidStoredData)?
                    .iter()
                    .enumerate()
                {
                    if i != 0 && !inline {
                        output.push('\n');
                    }
                    append_view(child, output)?;
                }
            }
        }
        None => return Err(DocumentError::InvalidStoredData),
    }
    if output.len() > NATIVE_SCENE_TEXT_BYTES {
        return Err(DocumentError::InvalidInput);
    }
    Ok(())
}
