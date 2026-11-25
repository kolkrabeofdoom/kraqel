//! Native metadata navigation. The separate authenticated roles own the manuscript control.
use super::{DocumentKind, DocumentNode, DocumentRepository, ProjectId, ProjectManager};
use kraqel_native_start::{ManuscriptLevel, choose_document, confirm_edit, notify};
use kraqel_storage::native_scene::{NativeScene, SceneReadGrant};
use std::{error::Error, path::Path};
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn children(
    manager: &ProjectManager,
    project: ProjectId,
    parent: Option<kraqel_domain::EntityId>,
) -> Result<Vec<DocumentNode>> {
    let mut opened = manager.open_project(project.to_string())?;
    let kind = if parent.is_some() {
        DocumentKind::Scene
    } else {
        DocumentKind::Chapter
    };
    Ok(DocumentRepository::new(opened.database_mut())
        .list(project)?
        .into_iter()
        .filter(|doc| doc.parent_id == parent && doc.kind == kind)
        .collect())
}

fn pick(level: ManuscriptLevel, documents: &[DocumentNode]) -> Result<Option<&DocumentNode>> {
    let names: Vec<_> = documents.iter().map(|doc| doc.title.clone()).collect();
    Ok(choose_document(level, &names)?.and_then(|index| documents.get(index)))
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

fn show(home: &Path, manager: &ProjectManager, scene: &DocumentNode) -> Result<()> {
    let viewed = {
        let NativeScene { text, edit } = read(manager, scene)?;
        let text = Zeroizing::new(text);
        if edit.is_none() {
            notify(
                "Diese Szene öffnet sich als schreibgeschützte Textansicht. Ihre Formatierung bleibt im Projekt erhalten.",
            );
        }
        if isolated(home, scene, &text, false)?.is_some() {
            return Err("Ungültige Antwort auf die Lesefreigabe.".into());
        }
        if edit.is_none() || !confirm_edit()? {
            return Ok(());
        }
        text
    };

    // Revalidate the exact presented revision after the explicit edit decision.
    let NativeScene { text, edit } = read(manager, scene)?;
    if text.as_str() != viewed.as_str() {
        return Err("Die angezeigte Fassung hat sich geändert.".into());
    }
    drop(viewed);
    let grant = edit.ok_or("Die Szene lässt sich in dieser Ansicht nur lesen.")?;
    let Some(draft) = isolated(home, scene, &Zeroizing::new(text), true)? else {
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
    loop {
        let chapters = children(manager, project, None)?;
        let Some(chapter) = pick(ManuscriptLevel::Chapters, &chapters)? else {
            return Ok(());
        };
        loop {
            let scenes = children(manager, project, Some(chapter.id))?;
            let Some(scene) = pick(ManuscriptLevel::Scenes, &scenes)? else {
                break;
            };
            if show(home, manager, scene).is_err() {
                notify(
                    "Die gebundene Szenensitzung konnte nicht abgeschlossen werden. Bitte prüfe die aktuelle Auswahl.",
                );
            }
        }
    }
}
