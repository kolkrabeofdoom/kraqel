//! Local, single-use scene grants issued from the selected catalog row.
use crate::document_repository::{DocumentError, DocumentRepository, SaveGuard};
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
        let mut text = String::new();
        append_view(&source.content.document, &mut text)?;
        if !displayable(&text) {
            return Err(DocumentError::InvalidInput);
        }
        let editable = source.content.version == EDITOR_DOCUMENT_VERSION
            && text_document(&text) == source.content;
        let edit = editable.then_some(SceneEditGrant {
            binding: self,
            original: source.content,
        });
        Ok(NativeScene { text, edit })
    }
}

impl SceneEditGrant {
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
