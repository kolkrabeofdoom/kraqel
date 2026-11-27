//! Manuscript selections use catalog metadata; only approved document bodies enter rendering.
use crate::{ExportError, ExportFormat, ExportResult, ExportSourceDocument, Manuscript, render};
use kraqel_domain::{DocumentKind, DocumentNode, EntityId, Project, ProjectId};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportScope {
    Manuscript,
    Chapter(EntityId),
    Scene(EntityId),
}

impl ExportScope {
    /// Selects the displayed catalog rows, preserving their order and revision.
    /// A single scene includes the parent chapter for its heading.
    ///
    /// # Errors
    /// Returns an error if the catalog belongs to another project or the chosen
    /// document or its required chapter is absent or has the wrong kind.
    pub fn select(
        self,
        project_id: ProjectId,
        outline: &[DocumentNode],
    ) -> Result<Vec<DocumentNode>, ExportError> {
        Ok(self.selected(project_id, outline)?.cloned().collect())
    }

    fn selected(
        self,
        project_id: ProjectId,
        outline: &[DocumentNode],
    ) -> Result<impl Iterator<Item = &DocumentNode>, ExportError> {
        let invalid = || ExportError::InvalidDocument("Die Manuskriptauswahl ist ungültig.");
        if outline.iter().any(|node| node.project_id != project_id) {
            return Err(invalid());
        }
        let chapter = match self {
            Self::Manuscript => None,
            Self::Chapter(id) => Some(id),
            Self::Scene(id) => Some(
                outline
                    .iter()
                    .find(|node| node.id == id && node.kind == DocumentKind::Scene)
                    .and_then(|node| node.parent_id)
                    .ok_or_else(invalid)?,
            ),
        };
        if let Some(id) = chapter
            && !outline.iter().any(|node| {
                node.id == id && node.kind == DocumentKind::Chapter && node.parent_id.is_none()
            })
        {
            return Err(invalid());
        }
        Ok(outline.iter().filter(move |node| match self {
            Self::Manuscript => true,
            Self::Chapter(id) => node.id == id || node.parent_id == Some(id),
            Self::Scene(id) => node.id == id || Some(node.id) == chapter,
        }))
    }
}

/// Publishes one new file from a confirmed selection in the project export directory.
/// Callers obtain the document data from a database snapshot that still matches
/// the confirmed catalog rows. For a scene its chapter supplies only the heading.
///
/// # Errors
/// Rejects out-of-scope, missing or changed documents, unsupported formats or
/// structures, and publication failures.
pub fn export_excerpt(
    project_directory: &Path,
    project: &Project,
    scope: ExportScope,
    expected: &[DocumentNode],
    documents: &[ExportSourceDocument],
    format: ExportFormat,
) -> Result<ExportResult, ExportError> {
    if !scope.selected(project.id, expected)?.eq(expected)
        || !documents.iter().map(|doc| &doc.node).eq(expected)
    {
        return Err(ExportError::InvalidDocument(
            "Der Manuskriptauszug entspricht nicht der bestätigten Auswahl.",
        ));
    }
    let bytes = render_selection(project, scope, documents, format)?;
    let prefix = match scope {
        ExportScope::Manuscript => "manuskript",
        ExportScope::Chapter(_) => "kapitel",
        ExportScope::Scene(_) => "szene",
    };
    let byte_count = u64::try_from(bytes.len()).map_err(crate::internal)?;
    let file_name = crate::excerpt_file::publish(project_directory, prefix, format, &bytes)?;
    Ok(ExportResult {
        format,
        file_name,
        byte_count,
    })
}

fn render_selection(
    project: &Project,
    scope: ExportScope,
    documents: &[ExportSourceDocument],
    format: ExportFormat,
) -> Result<Vec<u8>, ExportError> {
    if let (ExportFormat::Pdf, ExportScope::Scene(id)) = (format, scope)
        && documents.iter().any(|doc| {
            doc.node.id == id && doc.content.version == kraqel_domain::SCREENPLAY_DOCUMENT_VERSION
        })
    {
        return crate::pdf::screenplay::render_selected(project, documents, &[id]);
    }
    let manuscript = Manuscript::from_export_documents(project, documents).map_err(|_| {
        ExportError::InvalidDocument(
            "Das Manuskript enthält eine ungültige oder nicht unterstützte Struktur.",
        )
    })?;
    render::render(&manuscript, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_export_requires_exact_selection_and_publishes_new_files() {
        let (project, mut docs) = crate::tests::fixture();
        let chapter = docs[0].node.id;
        docs[0].node.position = 1;
        docs[1].node.position = 1;
        let scope = ExportScope::Scene(docs[1].node.id);
        let mut sibling = docs[1].clone();
        sibling.node.id = EntityId::new();
        sibling.node.position = 0;
        sibling.node.title = "Andere Szene".into();
        sibling.content.document = serde_json::json!({"type":"doc","content":[{
            "type":"paragraph","content":[{"type":"text","text":"Geschwistertext"}]
        }]});
        let mut other = docs[0].clone();
        other.node.id = EntityId::new();
        other.node.position = 0;
        other.node.title = "Anderes Kapitel".into();
        let mut elsewhere = sibling.clone();
        elsewhere.node.id = EntityId::new();
        elsewhere.node.parent_id = Some(other.node.id);
        elsewhere.node.title = "Fremder Szenentitel".into();
        let mut outline = vec![
            other.node.clone(),
            docs[0].node.clone(),
            sibling.node.clone(),
            docs[1].node.clone(),
            elsewhere.node.clone(),
        ];
        let expected = scope.select(project.id, &outline).unwrap();
        assert_eq!(expected, [docs[0].node.clone(), docs[1].node.clone()]);
        let chapter_scope = ExportScope::Chapter(chapter);
        let chapter_nodes = chapter_scope.select(project.id, &outline).unwrap();
        assert_eq!(chapter_nodes, outline[1..4]);
        assert_eq!(
            ExportScope::Manuscript
                .select(project.id, &outline)
                .unwrap(),
            outline
        );
        assert!(
            ExportScope::Scene(chapter)
                .select(project.id, &outline)
                .is_err()
        );
        outline[4].project_id = ProjectId::new();
        assert!(scope.select(project.id, &outline).is_err());
        outline[4].project_id = project.id;
        let root = tempfile::tempdir().unwrap();
        let export = |bound, loaded: &[ExportSourceDocument]| {
            export_excerpt(
                root.path(),
                &project,
                scope,
                bound,
                loaded,
                ExportFormat::Markdown,
            )
        };
        assert!(export(&outline, &docs).is_err());
        let mut unrelated = docs.clone();
        unrelated.push(sibling.clone());
        assert!(export(&expected, &unrelated).is_err());
        let mut stale = expected.clone();
        stale[1].revision += 1;
        assert!(export(&stale, &docs[..2]).is_err());
        assert!(!root.path().join("exports").exists());

        let first = export(&expected, &docs[..2]).unwrap();
        let second = export(&expected, &docs[..2]).unwrap();
        assert_ne!(first.file_name, second.file_name);
        for name in [first.file_name, second.file_name] {
            let markdown = std::fs::read_to_string(root.path().join("exports").join(name)).unwrap();
            assert!(markdown.contains("Ruf im Nebel"));
            assert!(markdown.contains("**kluger**"));
            assert!(!markdown.contains("Geschwistertext"));
            assert!(!markdown.contains("Anderes Kapitel"));
            assert!(!markdown.contains("Fremder Szenentitel"));
        }
        docs.insert(1, sibling);
        let result = export_excerpt(
            root.path(),
            &project,
            chapter_scope,
            &chapter_nodes,
            &docs,
            ExportFormat::Html,
        )
        .unwrap();
        let html =
            std::fs::read_to_string(root.path().join("exports").join(result.file_name)).unwrap();
        assert!(html.find("Geschwistertext").unwrap() < html.find("Ruf im Nebel").unwrap());
        assert!(!html.contains("Fremder Szenentitel"));
    }
    #[test]
    fn screenplay_excerpt_selects_structured_or_roles_only_pdf() {
        let (project, mut docs) = crate::tests::fixture();
        let prose = docs[1].clone();
        let scene = docs[1].node.id;
        docs[1].content = kraqel_domain::EditorDocument {
            version: kraqel_domain::SCREENPLAY_DOCUMENT_VERSION,
            document: serde_json::json!({"type":"doc","content":[
                {"type":"screenplayBlock","attrs":{"role":"sceneHeading"},
                 "content":[{"type":"text","text":"INT. ARCHIV - TAG"}]},
                {"type":"screenplayBlock","attrs":{"role":"action"},
                 "content":[{"type":"text","text":"Ein Rabe landet."}]}
            ]}),
        };
        let root = tempfile::tempdir().unwrap();
        let nodes: Vec<_> = docs.iter().map(|doc| doc.node.clone()).collect();
        let result = export_excerpt(
            root.path(),
            &project,
            ExportScope::Chapter(nodes[0].id),
            &nodes,
            &docs,
            ExportFormat::Pdf,
        )
        .unwrap();
        let extracted = |file: String| {
            let pdf = lopdf::Document::load(root.path().join("exports").join(file)).unwrap();
            pdf.extract_text(&pdf.get_pages().into_keys().collect::<Vec<_>>())
                .unwrap()
        };
        let chapter = extracted(result.file_name);
        assert!(
            chapter.contains("Ankunft")
                && chapter.contains("Ruf im Nebel")
                && chapter.contains("ARCHIV")
        );
        let mut sibling = prose;
        sibling.node.id = EntityId::new();
        sibling.node.position = 1;
        let mut mixed = docs.clone();
        mixed.push(sibling);
        let full: Vec<_> = mixed.iter().map(|doc| doc.node.clone()).collect();
        let full_pdf = export_excerpt(
            root.path(),
            &project,
            ExportScope::Manuscript,
            &full,
            &mixed,
            ExportFormat::Pdf,
        )
        .unwrap();
        let full_text = extracted(full_pdf.file_name);
        assert!(full_text.contains("ARCHIV") && full_text.contains("Niemals eine Krähe."));
        assert!(!chapter.contains("Niemals eine Krähe."));
        // Both drafts can be saved; neither may publish a partial manuscript.
        for roles in [["dialogue", "action"], ["sceneHeading", "sceneHeading"]] {
            for (index, role) in roles.into_iter().enumerate() {
                mixed[1].content.document["content"][index]["attrs"]["role"] =
                    serde_json::json!(role);
            }
            assert!(mixed[1].content.is_valid());
            assert!(
                export_excerpt(
                    root.path(),
                    &project,
                    ExportScope::Manuscript,
                    &full,
                    &mixed,
                    ExportFormat::Pdf
                )
                .is_err()
            );
        }
        assert_eq!(
            std::fs::read_dir(root.path().join("exports"))
                .unwrap()
                .count(),
            2
        );

        let pdf = export_excerpt(
            root.path(),
            &project,
            ExportScope::Scene(scene),
            &nodes,
            &docs,
            ExportFormat::Pdf,
        )
        .unwrap();
        let path = root.path().join("exports").join(pdf.file_name);
        let parsed = lopdf::Document::load(path).unwrap();
        assert_eq!(parsed.get_pages().len(), 1);
        let roles = parsed.extract_text(&[1]).unwrap();
        assert!(roles.contains("ARCHIV"));
        assert!(!roles.contains("Ankunft") && !roles.contains("Ruf im Nebel"));
    }
}
