//! Confirm a metadata-only outline selection before loading any manuscript contents.
use super::{DocumentNode, DocumentRepository, EntityId, ProjectId, ProjectManager, Result};
use kraqel_domain::{CURRENT_SCHEMA_VERSION, DocumentKind, Project};
use kraqel_export::{ExportFormat, ExportScope, ExportSourceDocument, export_excerpt};
use kraqel_native_start::{
    NativeExportChoice, NativeExportFormat, choose_export, confirm_export, notify,
    show_export_history,
};

pub(super) fn choose(
    manager: &ProjectManager,
    project: ProjectId,
    project_name: &str,
    outline: &[DocumentNode],
    selected: Option<&DocumentNode>,
    parent: Option<EntityId>,
) -> Result<()> {
    let options = scope_options(project_name, outline, selected, parent);
    let labels = options
        .iter()
        .map(|(_, label)| label.clone())
        .collect::<Vec<_>>();
    let (index, format) = loop {
        match choose_export(&labels)? {
            None => return Ok(()),
            Some(NativeExportChoice::Write(index, format)) => break (index, format),
            Some(NativeExportChoice::History) => {
                let directory = manager.root().join(project.to_string());
                match kraqel_export::list_excerpt_files(&directory) {
                    Ok(files) => {
                        let entries = files
                            .into_iter()
                            .map(|file| format!("{} · {} Byte", file.file_name, file.byte_count))
                            .collect::<Vec<_>>();
                        show_export_history(&entries, &directory.join("exports"))?;
                    }
                    Err(_) => notify("Die vorhandenen Ausgaben konnten nicht gelesen werden."),
                }
            }
        }
    };
    let (scope, summary) = options.get(index).ok_or("Ungültige Exportauswahl.")?;
    if !confirm_export(summary, format) {
        return Ok(());
    }

    let expected = scope.select(project, outline)?;
    let heading_only = match scope {
        ExportScope::Scene(id) => Some(
            expected
                .iter()
                .find(|node| node.id == *id)
                .and_then(|node| node.parent_id)
                .ok_or("Zugehöriges Kapitel fehlt.")?,
        ),
        _ => None,
    };
    let mut opened = manager.open_project(project.to_string())?;
    let directory = opened.directory().to_owned();
    let contents = DocumentRepository::new(opened.database_mut()).load_export_snapshot(
        project,
        project_name,
        &expected,
        heading_only,
        |current| scope.select(project, current).ok(),
    )?;
    drop(opened);
    let documents = expected
        .iter()
        .cloned()
        .zip(contents)
        .map(|(node, content)| ExportSourceDocument { node, content })
        .collect::<Vec<_>>();
    let metadata = Project {
        id: project,
        name: project_name.to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    let format = match format {
        NativeExportFormat::Markdown => ExportFormat::Markdown,
        NativeExportFormat::Html => ExportFormat::Html,
        NativeExportFormat::Docx => ExportFormat::Docx,
        NativeExportFormat::Pdf => ExportFormat::Pdf,
    };
    let result = export_excerpt(&directory, &metadata, *scope, &expected, &documents, format)?;
    notify(&format!(
        "Die Exportdatei wurde gespeichert.\n\n{}",
        directory.join("exports").join(result.file_name).display()
    ));
    Ok(())
}

fn scope_options(
    project_name: &str,
    outline: &[DocumentNode],
    selected: Option<&DocumentNode>,
    parent: Option<EntityId>,
) -> Vec<(ExportScope, String)> {
    let chapter = parent.or_else(|| selected.map(|node| node.id));
    let mut options = Vec::with_capacity(3);
    if let Some(scene) = selected.filter(|node| node.kind == DocumentKind::Scene) {
        options.push((
            ExportScope::Scene(scene.id),
            format!("Szene {:?}", scene.title),
        ));
    }
    if let Some(chapter) = chapter.and_then(|id| {
        outline
            .iter()
            .find(|n| n.id == id && n.kind == DocumentKind::Chapter)
    }) {
        let count = outline
            .iter()
            .filter(|node| node.parent_id == Some(chapter.id) && node.kind == DocumentKind::Scene)
            .count();
        options.push((
            ExportScope::Chapter(chapter.id),
            format!(
                "Kapitel {:?} ({} {})",
                chapter.title,
                count,
                if count == 1 { "Szene" } else { "Szenen" }
            ),
        ));
    }
    let chapters = outline
        .iter()
        .filter(|n| n.kind == DocumentKind::Chapter)
        .count();
    let scenes = outline
        .iter()
        .filter(|n| n.kind == DocumentKind::Scene)
        .count();
    options.push((
        ExportScope::Manuscript,
        format!(
            "Gesamtes Manuskript {project_name:?} ({chapters} Kapitel, {scenes} {})",
            if scenes == 1 { "Szene" } else { "Szenen" }
        ),
    ));
    options
}
