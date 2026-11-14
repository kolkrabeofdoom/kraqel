//! App orchestration for a fully validated, newly identified manuscript copy.
use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project, ProjectId};
use kraqel_export::{ExportSourceDocument, Manuscript, read_docx_roundtrip_file};
use kraqel_storage::{
    document_repository::{DocumentRepository, ImportedChapter},
    project_manager::{ManagedProject, ProjectError, ProjectManager},
};
use std::path::Path;

pub(super) fn import_file(
    manager: &ProjectManager,
    path: &Path,
) -> Result<ManagedProject, ProjectError> {
    let manuscript =
        read_docx_roundtrip_file(path).map_err(|_| ProjectError::InvalidImportedData)?;
    let chapters = manuscript
        .chapters
        .iter()
        .map(|chapter| {
            Ok(ImportedChapter {
                title: chapter.title.clone(),
                scenes: chapter
                    .scenes
                    .iter()
                    .map(|scene| {
                        Ok((
                            scene.title.clone(),
                            scene
                                .editor_document()
                                .map_err(|_| ProjectError::InvalidImportedData)?,
                        ))
                    })
                    .collect::<Result<_, ProjectError>>()?,
            })
        })
        .collect::<Result<Vec<_>, ProjectError>>()?;
    let project = Project {
        id: ProjectId::new(),
        name: manuscript.title.clone(),
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    manager.create_project_with_initializer(project.id.to_string(), &project, |database, _| {
        let mut repository = DocumentRepository::new(database);
        repository
            .import_manuscript(project.id, &chapters)
            .map_err(|_| ProjectError::InvalidImportedData)?;
        let documents = repository
            .list(project.id)
            .map_err(|_| ProjectError::InvalidImportedData)?
            .into_iter()
            .map(|node| {
                let stored = repository
                    .load(project.id, node.id)
                    .map_err(|_| ProjectError::InvalidImportedData)?;
                Ok(ExportSourceDocument {
                    node: stored.node,
                    content: stored.content,
                })
            })
            .collect::<Result<Vec<_>, ProjectError>>()?;
        let restored = Manuscript::from_documents(&project, &documents)
            .map_err(|_| ProjectError::InvalidImportedData)?;
        if restored != manuscript {
            return Err(ProjectError::InvalidImportedData);
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kraqel_export::{read_docx_roundtrip, write_docx_roundtrip};
    use std::{collections::BTreeSet, fs};

    const FIXTURE: &[u8] =
        include_bytes!("../../../../crates/export/tests/fixtures/roundtrip-v1/word-edited.docx");

    fn restored(opened: &mut ManagedProject) -> (Project, Vec<ExportSourceDocument>) {
        let (id, name): (String, String) = opened
            .database()
            .connection()
            .query_row("SELECT id, name FROM project", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        let project = Project {
            id: ProjectId::parse(&id).unwrap(),
            name,
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let repository = DocumentRepository::new(opened.database_mut());
        let docs = repository
            .list(project.id)
            .unwrap()
            .into_iter()
            .map(|node| {
                let stored = repository.load(project.id, node.id).unwrap();
                ExportSourceDocument {
                    node: stored.node,
                    content: stored.content,
                }
            })
            .collect();
        (project, docs)
    }

    #[test]
    fn imports_real_office_manuscript_as_independent_persistent_copies() {
        let root = tempfile::tempdir().unwrap();
        let projects = root.path().join("projects");
        fs::create_dir(&projects).unwrap();
        let manager = ProjectManager::new(&projects).unwrap();
        let source = root.path().join("edited.docx");
        fs::write(&source, FIXTURE).unwrap();
        let expected = read_docx_roundtrip(FIXTURE).unwrap();
        let mut previous_ids = BTreeSet::new();
        for _ in 0..2 {
            let mut opened = import_file(&manager, &source).unwrap();
            let (project, docs) = restored(&mut opened);
            assert!(previous_ids.insert(project.id.to_string()));
            for doc in &docs {
                assert!(previous_ids.insert(doc.node.id.to_string()));
            }
            assert_eq!(
                Manuscript::from_documents(&project, &docs).unwrap(),
                expected
            );
            let found: i64 = opened
                .database()
                .connection()
                .query_row(
                    "SELECT count(*) FROM document_fts WHERE document_fts MATCH 'EDITIERBAR'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(found, 1);
            drop(opened);
            let mut reopened = manager.open_project(project.id.to_string()).unwrap();
            let (project, docs) = restored(&mut reopened);
            assert_eq!(
                Manuscript::from_documents(&project, &docs).unwrap(),
                expected
            );
            let foreign: i64 = reopened.database().connection().query_row(
                "SELECT (SELECT count(*) FROM entity) + (SELECT count(*) FROM media_asset) + (SELECT count(*) FROM image_prompt_draft)", [], |r| r.get(0)).unwrap();
            assert_eq!(foreign, 0);
        }
        assert_eq!(fs::read(source).unwrap(), FIXTURE);
        assert_eq!(fs::read_dir(projects).unwrap().count(), 2);
    }

    #[test]
    fn invalid_package_and_late_storage_failure_publish_nothing() {
        let root = tempfile::tempdir().unwrap();
        let projects = root.path().join("projects");
        fs::create_dir(&projects).unwrap();
        let manager = ProjectManager::new(&projects).unwrap();
        let source = root.path().join("invalid.docx");
        fs::write(&source, b"not a zip").unwrap();
        assert!(import_file(&manager, &source).is_err());
        let mut manuscript = read_docx_roundtrip(FIXTURE).unwrap();
        // Valid profile; storage rejects the second chapter after earlier inserts.
        manuscript.chapters[1].title = "x".repeat(241);
        fs::write(&source, write_docx_roundtrip(&manuscript).unwrap()).unwrap();
        assert!(import_file(&manager, &source).is_err());
        assert_eq!(fs::read_dir(&projects).unwrap().count(), 0);
    }

    #[test]
    fn imported_empty_titles_remain_portable_through_snapshot_v3() {
        use kraqel_git_service::{GitService, import_snapshot_archive_file};
        let root = tempfile::tempdir().unwrap();
        let projects = root.path().join("projects");
        fs::create_dir(&projects).unwrap();
        let manager = ProjectManager::new(&projects).unwrap();
        let source = root.path().join("source.docx");
        fs::write(&source, FIXTURE).unwrap();
        let mut opened = import_file(&manager, &source).unwrap();
        let (project, _) = restored(&mut opened);
        let destination = root.path().join("copy.kraqel");
        GitService::new(opened.directory())
            .export_snapshot_archive(opened.database(), project.id, &destination)
            .unwrap();
        let (mut imported, _) = import_snapshot_archive_file(&manager, &destination).unwrap();
        let (project, docs) = restored(&mut imported);
        assert_eq!(
            Manuscript::from_documents(&project, &docs).unwrap(),
            read_docx_roundtrip(FIXTURE).unwrap()
        );
    }

    #[test]
    fn command_services_export_import_and_reject_files_without_echoing_details() {
        let root = tempfile::tempdir().unwrap();
        let state = crate::AppState {
            project_root: root.path().join("projects"),
            ai_releases: std::sync::Mutex::new(std::collections::HashMap::new()),
        };
        fs::create_dir(&state.project_root).unwrap();
        let request_id = uuid::Uuid::new_v4();
        let source = crate::create_project_impl(
            &state,
            request_id,
            crate::CreateProjectRequest {
                name: "Quelle".to_owned(),
            },
        )
        .unwrap();
        let result = crate::export_manuscript_mode_impl(
            &state,
            request_id,
            &crate::ExportProjectRequest {
                project_id: source.project.id.to_string(),
                format: kraqel_export::ExportFormat::Docx,
            },
            true,
        )
        .unwrap();
        assert_eq!(result.file_name, "manuskript-roundtrip.docx");
        let path = state
            .project_root
            .join(source.project.id.to_string())
            .join("exports")
            .join(result.file_name);
        let imported = crate::import_docx_project_file_impl(&state, request_id, &path).unwrap();
        assert_ne!(source.project.id, imported.project.id);
        assert_eq!(source.documents.len(), imported.documents.len());
        assert!(
            imported
                .documents
                .iter()
                .all(|n| source.documents.iter().all(|old| old.id != n.id))
        );
        let invalid = root.path().join("private-source.docx");
        fs::write(&invalid, b"private manuscript text").unwrap();
        let error = crate::import_docx_project_file_impl(&state, request_id, &invalid).unwrap_err();
        let json = serde_json::to_string(&error).unwrap();
        assert!(!json.contains("private"));
        assert!(!json.contains(&root.path().to_string_lossy().to_string()));
        assert_eq!(fs::read_dir(&state.project_root).unwrap().count(), 2);
    }
}
