//! Native metadata navigation. The separate authenticated roles own the manuscript control.
use super::{DocumentKind, DocumentNode, DocumentRepository, ProjectId, ProjectManager};
use kraqel_domain::EntityId;
use kraqel_native_start::{
    OutlineChoice, SceneAction, ask_document_title, choose_chapter, choose_destination,
    choose_scene, confirm_removal, notify,
};
use kraqel_storage::document_repository::{DeletePreview, OutlineMove, is_valid_document_title};
use kraqel_storage::native_scene::{NativeScene, SceneReadGrant};
use std::{error::Error, path::Path};
use zeroize::Zeroizing;

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
    editable: bool,
) -> Result<Option<String>> {
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
        editable,
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
        if isolated(home, scene, &text, false)?.is_some() {
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
    let Some(draft) = isolated(home, scene, &text, true)? else {
        return Ok(());
    };
    let draft = Zeroizing::new(draft);
    let saved = manager
        .open_project(scene.project_id.to_string())
        .ok()
        .and_then(|mut project| {
            grant
                .save(&mut DocumentRepository::new(project.database_mut()), &draft)
                .ok()
        });
    if saved.is_some() {
        notify("Die Szene wurde gespeichert.");
    } else {
        notify(
            "Die Szene konnte nicht unter der gelesenen Revision gespeichert werden. Der Entwurf wird noch einmal im isolierten Lesefenster angezeigt. Mit dessen Schließen verwirfst du diesen Entwurf.",
        );
        isolated(home, scene, &draft, false)?;
    }
    Ok(())
}

pub(super) fn browse(home: &Path, manager: &ProjectManager, project: ProjectId) -> Result<()> {
    let mut parent: Option<EntityId> = None;
    let mut selected = None;
    loop {
        let outline = list_outline(manager, project)?;
        let chapters: Vec<_> = outline
            .iter()
            .filter(|n| n.kind == DocumentKind::Chapter && n.parent_id.is_none())
            .collect();
        let chapter_index = chapters.iter().position(|n| Some(n.id) == parent);
        if parent.is_some() && chapter_index.is_none() {
            parent = None;
            selected = None;
        }
        let (kind, label, nodes) = if parent.is_some() {
            (
                DocumentKind::Scene,
                "Szene",
                outline
                    .iter()
                    .filter(|n| n.kind == DocumentKind::Scene && n.parent_id == parent)
                    .collect::<Vec<_>>(),
            )
        } else {
            (DocumentKind::Chapter, "Kapitel", chapters.clone())
        };
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
            }
            OutlineChoice::Up(_) | OutlineChoice::Down(_) | OutlineChoice::Move(_) => {
                let id = node.ok_or("Dokumentauswahl fehlt.")?.id;
                let Some(direction) = choose_move(&choice, &chapters, chapter_index)? else {
                    continue;
                };
                let mut opened = manager.open_project(project.to_string())?;
                match DocumentRepository::new(opened.database_mut())
                    .move_in_outline(project, &outline, id, direction)
                {
                    Ok(()) => {
                        if let OutlineMove::AppendToChapter(target) = direction {
                            parent = Some(target);
                        }
                    }
                    Err(_) => notify(
                        "Der Eintrag wurde nicht verschoben. Bitte prüfe die aktualisierte Gliederung.",
                    ),
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

fn list_outline(manager: &ProjectManager, project: ProjectId) -> Result<Vec<DocumentNode>> {
    let mut opened = manager.open_project(project.to_string())?;
    Ok(DocumentRepository::new(opened.database_mut()).list(project)?)
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
    chapters: &[&DocumentNode],
    current: Option<usize>,
) -> Result<Option<OutlineMove>> {
    match choice {
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
