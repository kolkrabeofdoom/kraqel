//! Native metadata navigation. The separate authenticated roles own the manuscript control.
use super::{DocumentKind, DocumentNode, DocumentRepository, ProjectId, ProjectManager};
use kraqel_domain::EntityId;
use kraqel_native_start::{
    OutlineChoice, SceneAction, ask_document_title, choose_chapter, choose_scene, notify,
};
use kraqel_storage::document_repository::is_valid_document_title;
use kraqel_storage::native_scene::{NativeScene, SceneReadGrant};
use std::{error::Error, path::Path};
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn children(
    manager: &ProjectManager,
    project: ProjectId,
    parent: Option<EntityId>,
    kind: DocumentKind,
) -> Result<Vec<DocumentNode>> {
    let mut opened = manager.open_project(project.to_string())?;
    Ok(DocumentRepository::new(opened.database_mut())
        .list(project)?
        .into_iter()
        .filter(|doc| doc.parent_id == parent && doc.kind == kind)
        .collect())
}

fn titles(documents: &[DocumentNode]) -> Vec<String> {
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
    browse_level(home, manager, project, None)
}

fn browse_level(
    home: &Path,
    manager: &ProjectManager,
    project: ProjectId,
    parent: Option<EntityId>,
) -> Result<()> {
    let (kind, label) = if parent.is_some() {
        (DocumentKind::Scene, "Szene")
    } else {
        (DocumentKind::Chapter, "Kapitel")
    };
    let mut selected = None;
    loop {
        let nodes = children(manager, project, parent, kind)?;
        let names = titles(&nodes);
        let index = nodes.iter().position(|node| Some(node.id) == selected);
        let choice = match kind {
            DocumentKind::Chapter => choose_chapter(&names, index)?,
            DocumentKind::Scene => choose_scene(&names, index)?,
        };
        let selected_index = match choice {
            OutlineChoice::Back => None,
            OutlineChoice::Create(index) => index,
            OutlineChoice::Rename(i) | OutlineChoice::Open(i) | OutlineChoice::Edit(i) => Some(i),
        };
        let node = selected_index
            .map(|i| nodes.get(i).ok_or("Ungültige Dokumentauswahl."))
            .transpose()?;
        selected = node.map(|n| n.id);
        match choice {
            OutlineChoice::Create(_) | OutlineChoice::Rename(_) => {
                let rename = matches!(choice, OutlineChoice::Rename(_));
                let current = node.filter(|_| rename);
                let Some(title) = ask_document_title(
                    label,
                    current.map(|n| n.title.as_str()),
                    is_valid_document_title,
                )?
                else {
                    continue;
                };
                let mut opened = manager.open_project(project.to_string())?;
                let mut repository = DocumentRepository::new(opened.database_mut());
                let changed = match current {
                    Some(n) => repository.rename_selected(n, &title),
                    None => repository.create(project, parent, kind, &title),
                };
                match changed {
                    Ok(n) => selected = Some(n.id),
                    Err(_) => notify(
                        "Die Änderung konnte nicht gespeichert werden. Bitte prüfe die aktualisierte Auswahl.",
                    ),
                }
            }
            OutlineChoice::Open(_) | OutlineChoice::Edit(_) => {
                let node = node.ok_or("Dokumentauswahl fehlt.")?;
                if node.kind == DocumentKind::Chapter {
                    browse_level(home, manager, project, Some(node.id))?;
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
            OutlineChoice::Back => return Ok(()),
        }
    }
}
