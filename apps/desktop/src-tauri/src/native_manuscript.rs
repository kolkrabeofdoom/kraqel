//! Native metadata navigation. The separate authenticated roles own the manuscript control.
use super::{DocumentKind, DocumentNode, DocumentRepository, ProjectId, ProjectManager};
use kraqel_domain::EntityId;
use kraqel_isolation_transport::{SceneOutcome, SceneRequest, SceneSaveStatus};
use kraqel_native_start::{
    OutlineChoice, SceneAction, ask_copy_title, ask_document_title, choose_chapter,
    choose_copy_destination, choose_destination, choose_position, choose_scene, confirm_removal,
    notify,
};
use kraqel_storage::document_repository::{
    DeletePreview, DocumentError, OutlineMove, is_valid_document_title,
};
use kraqel_storage::native_scene::{NativeScene, SceneEditGrant, SceneReadGrant};
use std::{error::Error, path::Path};
use zeroize::Zeroizing;

mod export;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn titles(documents: &[&DocumentNode]) -> Vec<String> {
    documents.iter().map(|doc| doc.title.clone()).collect()
}

fn read(manager: &ProjectManager, scene: &DocumentNode) -> Result<NativeScene> {
    let mut opened = manager.open_project(scene.project_id.to_string())?;
    Ok(SceneReadGrant::from_node(scene)?.read(&DocumentRepository::new(opened.database_mut()))?)
}

fn isolated(
    home: &Path,
    scene: &DocumentNode,
    text: &str,
    save: Option<&mut dyn FnMut(SceneRequest<'_>) -> SceneSaveStatus>,
) -> Result<SceneOutcome> {
    let images: [&[u8]; 2] = [
        include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/pipe-service.exe")),
        include_bytes!(concat!(env!("KRAQEL_REVIEW_IMAGES"), "/native-leaf.exe")),
    ];
    Ok(kraqel_isolation_transport::open_project_scene(
        home,
        scene.project_id,
        scene.id,
        scene.revision,
        text,
        save,
        images,
    )?)
}

fn show(
    home: &Path,
    manager: &ProjectManager,
    scene: &DocumentNode,
    action: SceneAction,
) -> Result<()> {
    // Read only after explicit selection, rechecking its project, scene ID and catalog revision.
    let NativeScene { text, edit } = read(manager, scene)?;
    let text = Zeroizing::new(text);
    if action == SceneAction::Read {
        if isolated(home, scene, &text, None)? != SceneOutcome::Closed {
            return Err("Ungültige Antwort auf die Lesefreigabe.".into());
        }
        return Ok(());
    }
    let Some(grant) = edit else {
        notify(
            "Diese Szene enthält Formatierungen oder eine Dokumentstruktur, die sich im nativen Texteditor nur lesen lässt. Wähle „Lesen“ für die Textansicht.",
        );
        return Ok(());
    };
    let mut session = grant.into_session();
    let mut save = |request: SceneRequest<'_>| {
        let Ok(mut opened) = manager.open_project(scene.project_id.to_string()) else {
            return SceneSaveStatus::Failed;
        };
        let mut repository = DocumentRepository::new(opened.database_mut());
        let draft = match request {
            SceneRequest::Save(draft) => draft,
            SceneRequest::ReadLatest => return latest(&repository, scene),
            SceneRequest::Confirm {
                revision,
                reviewed,
                merged,
            } => {
                return match SceneEditGrant::save_reviewed(
                    &mut repository,
                    scene.project_id,
                    scene.id,
                    revision,
                    reviewed,
                    merged,
                ) {
                    Ok(revision) => SceneSaveStatus::Committed(revision),
                    Err(DocumentError::RevisionConflict) => latest(&repository, scene),
                    Err(_) => SceneSaveStatus::Failed,
                };
            }
        };
        match session.save(&mut repository, draft) {
            Ok(revision) => SceneSaveStatus::Committed(revision),
            Err(DocumentError::RevisionConflict | DocumentError::NotFound) => {
                SceneSaveStatus::Conflict
            }
            Err(_) => SceneSaveStatus::Failed,
        }
    };
    match isolated(home, scene, &text, Some(&mut save))? {
        SceneOutcome::SavedAndClosed => notify("Die Szene wurde gespeichert."),
        SceneOutcome::ClosedAfterCheckpoint => {
            notify("Der zuletzt bestätigte Zwischenstand wurde gespeichert.");
        }
        SceneOutcome::Closed => {}
    }
    Ok(())
}

fn latest(repository: &DocumentRepository<'_>, scene: &DocumentNode) -> SceneSaveStatus {
    match SceneReadGrant::latest_prose(scene.project_id, scene.id, repository) {
        Ok((revision, text)) => SceneSaveStatus::Review {
            revision,
            text: Zeroizing::new(text),
        },
        Err(_) => SceneSaveStatus::Failed,
    }
}

pub(super) fn browse(home: &Path, manager: &ProjectManager, project: ProjectId) -> Result<()> {
    // Keep the catalog's directory identities bound from navigation through export publication.
    let _root_guard = kraqel_export::pin_excerpt_project(manager.root())?;
    let _project_guard =
        kraqel_export::pin_excerpt_project(&manager.root().join(project.to_string()))?;
    let mut parent: Option<EntityId> = None;
    let mut selected = None;
    loop {
        let (project_name, outline) = list_outline(manager, project)?;
        let chapters: Vec<_> = outline
            .iter()
            .filter(|n| n.kind == DocumentKind::Chapter && n.parent_id.is_none())
            .collect();
        let chapter_index = chapters.iter().position(|n| Some(n.id) == parent);
        if parent.is_some() && chapter_index.is_none() {
            parent = None;
            selected = None;
        }
        let (kind, label, nodes) = outline_entries(&outline, &chapters, parent);
        let names = titles(&nodes);
        let index = nodes.iter().position(|node| Some(node.id) == selected);
        let choice = match kind {
            DocumentKind::Chapter => choose_chapter(&names, index)?,
            DocumentKind::Scene => choose_scene(&names, index, chapters.len() > 1)?,
        };
        let node = choice
            .selected_index()
            .map(|i| nodes.get(i).copied().ok_or("Ungültige Dokumentauswahl."))
            .transpose()?;
        selected = node.map(|n| n.id);
        match choice {
            OutlineChoice::Export(_) => {
                if let Err(error) =
                    export::choose(manager, project, &project_name, &outline, node, parent)
                {
                    report_export_error(error.as_ref());
                }
            }
            OutlineChoice::Copy(_) => {
                let source = node.ok_or("Dokumentauswahl fehlt.")?;
                match copy_document(manager, source, &outline, &chapters, chapter_index, label) {
                    Ok(Some(copy)) => {
                        parent = copy.parent_id;
                        selected = Some(copy.id);
                    }
                    Ok(None) => {}
                    Err(_) => notify(
                        "Die Kopie konnte nicht angelegt werden. Bitte prüfe die aktualisierte Gliederung und wähle Quelle und Ziel erneut.",
                    ),
                }
            }
            OutlineChoice::Remove(_) => {
                if remove_document(manager, node.ok_or("Dokumentauswahl fehlt.")?).is_err() {
                    notify(
                        "Der Vorgang konnte nicht bestätigt werden. Bitte prüfe die aktualisierte Gliederung und öffne bei Bedarf eine neue Löschvorschau.",
                    );
                }
            }
            OutlineChoice::Create(_) | OutlineChoice::Rename(_) => {
                let rename = matches!(choice, OutlineChoice::Rename(_));
                let current = node.filter(|_| rename);
                if let Some(changed) = change_title(manager, project, parent, kind, label, current)?
                {
                    selected = Some(changed.id);
                }
            }
            OutlineChoice::Open(_) | OutlineChoice::Edit(_) => {
                let node = node.ok_or("Dokumentauswahl fehlt.")?;
                if node.kind == DocumentKind::Chapter {
                    parent = Some(node.id);
                    selected = None;
                } else {
                    open_scene(home, manager, node, &choice);
                }
            }
            OutlineChoice::Up(_)
            | OutlineChoice::Down(_)
            | OutlineChoice::Move(_)
            | OutlineChoice::Position(_) => {
                let id = node.ok_or("Dokumentauswahl fehlt.")?.id;
                if let Some(direction) = choose_move(&choice, &names, &chapters, chapter_index)?
                    && let Some(target) = apply_move(manager, project, &outline, id, direction)?
                {
                    parent = Some(target);
                }
            }
            OutlineChoice::Back => {
                if parent.is_none() {
                    return Ok(());
                }
                selected = parent.take();
            }
        }
    }
}

fn outline_entries<'a>(
    outline: &'a [DocumentNode],
    chapters: &[&'a DocumentNode],
    parent: Option<EntityId>,
) -> (DocumentKind, &'static str, Vec<&'a DocumentNode>) {
    match parent {
        Some(chapter) => (
            DocumentKind::Scene,
            "Szene",
            outline
                .iter()
                .filter(|n| n.kind == DocumentKind::Scene && n.parent_id == Some(chapter))
                .collect(),
        ),
        None => (DocumentKind::Chapter, "Kapitel", chapters.to_vec()),
    }
}

fn open_scene(home: &Path, manager: &ProjectManager, node: &DocumentNode, choice: &OutlineChoice) {
    let action = if matches!(choice, OutlineChoice::Edit(_)) {
        SceneAction::Edit
    } else {
        SceneAction::Read
    };
    if show(home, manager, node, action).is_err() {
        notify(
            "Die gebundene Szenensitzung konnte nicht abgeschlossen werden. Bitte prüfe die aktuelle Auswahl.",
        );
    }
}

fn report_export_error(error: &(dyn Error + 'static)) {
    if matches!(
        error.downcast_ref::<DocumentError>(),
        Some(DocumentError::RevisionConflict)
    ) {
        notify(
            "Der bestätigte Manuskriptstand hat sich geändert. Bitte den Export aus der aktuellen Gliederung erneut wählen.",
        );
    } else if let Some(kraqel_export::ExportError::InvalidDocument(message)) =
        error.downcast_ref::<kraqel_export::ExportError>()
    {
        notify(message);
    } else {
        notify(
            "Der Export konnte nicht erstellt werden. Bitte prüfe die Auswahl und den lokalen Exportordner.",
        );
    }
}

fn apply_move(
    manager: &ProjectManager,
    project: ProjectId,
    outline: &[DocumentNode],
    id: EntityId,
    direction: OutlineMove,
) -> Result<Option<EntityId>> {
    let mut opened = manager.open_project(project.to_string())?;
    if DocumentRepository::new(opened.database_mut())
        .move_in_outline(project, outline, id, direction)
        .is_err()
    {
        notify("Der Eintrag wurde nicht verschoben. Bitte prüfe die aktualisierte Gliederung.");
    } else if let OutlineMove::AppendToChapter(target) = direction {
        return Ok(Some(target));
    }
    Ok(None)
}

fn copy_document(
    manager: &ProjectManager,
    source: &DocumentNode,
    outline: &[DocumentNode],
    chapters: &[&DocumentNode],
    chapter_index: Option<usize>,
    label: &'static str,
) -> Result<Option<DocumentNode>> {
    let (children, destination) = match source.kind {
        DocumentKind::Chapter => (
            outline
                .iter()
                .filter(|n| n.parent_id == Some(source.id))
                .collect::<Vec<_>>(),
            None,
        ),
        DocumentKind::Scene => {
            let Some(index) =
                choose_copy_destination(&titles(chapters), chapter_index.ok_or("Kapitel fehlt.")?)?
            else {
                return Ok(None);
            };
            (
                vec![],
                Some(*chapters.get(index).ok_or("Zielkapitel fehlt.")?),
            )
        }
    };
    let suggested = format!("Kopie von {:.230}", source.title.trim());
    let scope = match destination {
        Some(target) => format!(
            "Zielkapitel {}: {:?}",
            u64::from(target.position) + 1,
            target.title
        ),
        None if children.len() == 1 => "Kapitel mit 1 Szene".to_owned(),
        None => format!("Kapitel mit {} Szenen", children.len()),
    };
    let Some(title) = ask_copy_title(label, &suggested, &scope, is_valid_document_title)? else {
        return Ok(None);
    };
    // Pass the displayed selection and subtree into the transactional recheck.
    let mut opened = manager.open_project(source.project_id.to_string())?;
    let mut repository = DocumentRepository::new(opened.database_mut());
    Ok(Some(repository.duplicate_manuscript(
        source,
        &children,
        destination,
        &title,
    )?))
}

fn change_title(
    manager: &ProjectManager,
    project: ProjectId,
    parent: Option<EntityId>,
    kind: DocumentKind,
    label: &'static str,
    current: Option<&DocumentNode>,
) -> Result<Option<DocumentNode>> {
    let Some(title) = ask_document_title(
        label,
        current.map(|n| n.title.as_str()),
        is_valid_document_title,
    )?
    else {
        return Ok(None);
    };
    let mut opened = manager.open_project(project.to_string())?;
    let mut repository = DocumentRepository::new(opened.database_mut());
    let changed = match current {
        Some(n) => repository.rename_selected(n, &title),
        None => repository.create(project, parent, kind, &title),
    };
    if let Ok(node) = changed {
        Ok(Some(node))
    } else {
        notify(
            "Die Änderung konnte nicht gespeichert werden. Bitte prüfe die aktualisierte Auswahl.",
        );
        Ok(None)
    }
}

fn list_outline(
    manager: &ProjectManager,
    project: ProjectId,
) -> Result<(String, Vec<DocumentNode>)> {
    let mut opened = manager.open_project(project.to_string())?;
    let name = opened.database().connection().query_row(
        "SELECT name FROM project WHERE id = ?1",
        [project.to_string()],
        |row| row.get(0),
    )?;
    Ok((
        name,
        DocumentRepository::new(opened.database_mut()).list(project)?,
    ))
}

fn remove_document(manager: &ProjectManager, node: &DocumentNode) -> Result<()> {
    let mut project = manager.open_project(node.project_id.to_string())?;
    let mut repository = DocumentRepository::new(project.database_mut());
    let preview = repository.preview_delete(node)?;
    if confirm_removal(&deletion_details(&preview))? {
        repository.confirm_delete(&preview)?;
    }
    Ok(())
}

fn deletion_details(preview: &DeletePreview) -> String {
    fn entry(node: &DocumentNode) -> String {
        let kind = match node.kind {
            DocumentKind::Chapter => "Kapitel",
            DocumentKind::Scene => "Szene",
        };
        // Quote titles to keep imported newlines and control characters inside a single entry.
        format!("{kind} {}: {:?}", u64::from(node.position) + 1, node.title)
    }
    let mut lines = Vec::new();
    if let Some(parent) = preview.parent() {
        lines.push(format!("Zugehöriges {}", entry(parent)));
    }
    lines.push(entry(preview.selected()));
    if preview.selected().kind == DocumentKind::Chapter {
        lines.push(format!("Enthaltene Szenen ({})", preview.children().len()));
        lines.extend(preview.children().iter().map(entry));
    }
    for (label, count) in [
        "Story-Bibel-Verknüpfungen",
        "Plotkarten",
        "Ereignisverknüpfungen",
        "Medienverknüpfungen",
        "Bildprompt-Entwürfe",
        "Profilzuordnungen der Bildprompt-Entwürfe",
    ]
    .into_iter()
    .zip(preview.related_counts())
    {
        if count > 0 {
            lines.push(format!("{label}: {count}"));
        }
    }
    lines.join("\r\n")
}

fn choose_move(
    choice: &OutlineChoice,
    siblings: &[String],
    chapters: &[&DocumentNode],
    current: Option<usize>,
) -> Result<Option<OutlineMove>> {
    match choice {
        OutlineChoice::Position(index) => {
            Ok(choose_position(siblings, *index)?.map(OutlineMove::AtIndex))
        }
        OutlineChoice::Up(_) => Ok(Some(OutlineMove::Earlier)),
        OutlineChoice::Down(_) => Ok(Some(OutlineMove::Later)),
        OutlineChoice::Move(_) => {
            let Some(target) =
                choose_destination(&titles(chapters), current.ok_or("Kapitel fehlt.")?)?
            else {
                return Ok(None);
            };
            Ok(Some(OutlineMove::AppendToChapter(
                chapters.get(target).ok_or("Kapitel fehlt.")?.id,
            )))
        }
        _ => Err("Ungültige Verschiebeaktion.".into()),
    }
}
