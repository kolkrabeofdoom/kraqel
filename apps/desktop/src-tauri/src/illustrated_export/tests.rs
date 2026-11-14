use super::*;
use crate::{AssetLinkRequest, CreateProjectRequest, ImportMediaRequest};
use kraqel_domain::{AssetTargetKind, DocumentKind};
use std::{collections::HashMap, fs, path::Path};

fn setup(root: &Path) -> (AppState, IllustratedRequest) {
    fs::create_dir_all(root).unwrap();
    let state = AppState {
        project_root: root.to_path_buf(),
        ai_releases: Mutex::new(HashMap::new()),
    };
    let request_id = uuid::Uuid::new_v4();
    let workspace = crate::create_project_impl(
        &state,
        request_id,
        CreateProjectRequest {
            name: "Bilder am Szenenende".to_owned(),
        },
    )
    .unwrap();
    let scene = workspace
        .documents
        .iter()
        .find(|d| d.kind == DocumentKind::Scene)
        .unwrap();
    let media = crate::import_media_asset_impl(
        &state,
        request_id,
        &ImportMediaRequest {
            project_id: workspace.project.id.to_string(),
            file_name: "PRIVATE-original.png".to_owned(),
            title: "Abend am Schreibtisch".to_owned(),
            tags: vec!["PRIVATE-tag".to_owned()],
            bytes: include_bytes!("../../../../../crates/asset-service/tests/fixtures/scene.png")
                .to_vec(),
        },
    )
    .unwrap();
    crate::change_media_link_impl(
        &state,
        request_id,
        &AssetLinkRequest {
            project_id: workspace.project.id.to_string(),
            asset_id: media.asset.id.to_string(),
            target_kind: AssetTargetKind::Document,
            target_id: scene.id.to_string(),
            role: "reference".to_owned(),
        },
        true,
    )
    .unwrap();
    let request = IllustratedRequest {
        project_id: workspace.project.id.to_string(),
        format: IllustratedFormat::Html,
        placements: vec![ImagePlacement {
            scene_id: scene.id,
            asset_id: media.asset.id,
            expected_checksum: media.asset.checksum,
            expected_scene_revision: 0,
            alt_text: "Eine helle Szene".to_owned(),
            caption: Some("Bild am Ende der Szene".to_owned()),
        }],
    };
    (state, request)
}
#[test]
fn command_publishes_independent_packages_and_sanitizes_bad_sources() {
    let root = tempfile::tempdir().unwrap();
    let (state, mut request) = setup(root.path());
    let id = uuid::Uuid::new_v4();
    for format in [IllustratedFormat::Html, IllustratedFormat::Markdown] {
        request.format = format;
        let result = export(&state, id, &request).unwrap();
        let directory = root
            .path()
            .join(&request.project_id)
            .join("exports")
            .join(result.package_name);
        assert!(directory.join("medien.json").exists());
        assert_eq!(result.image_count, 1);
    }
    request.placements[0].expected_scene_revision = 99;
    let error = export(&state, id, &request).unwrap_err();
    assert!(!serde_json::to_string(&error).unwrap().contains("PRIVATE"));
}
#[test]
fn concurrent_metadata_changes_conflict_before_directory_publication() {
    let root = tempfile::tempdir().unwrap();
    let (state, request) = setup(root.path());
    let request_id = uuid::Uuid::new_v4();
    let id = ProjectId::parse(&request.project_id).unwrap();
    let (snapshot, directory) = frozen(&state, id, request_id, &request.placements).unwrap();
    let package = illustrated::prepare(
        &directory,
        &snapshot.project,
        &snapshot.documents,
        &snapshot.assets,
        request.format,
        &request.placements,
    )
    .unwrap();
    let manager = manager(&state, request_id).unwrap();
    let opened = manager.open_project(id.to_string()).unwrap();
    opened
        .database()
        .connection()
        .execute(
            "UPDATE document SET title='Geändert' WHERE id=?1",
            [request.placements[0].scene_id.to_string()],
        )
        .unwrap();
    drop(opened);
    assert_eq!(
        publish_checked(
            &state,
            id,
            request_id,
            &request.placements,
            &directory,
            &package,
            &snapshot.fingerprint()
        )
        .unwrap_err(),
        IllustratedError::Conflict
    );
    assert_eq!(
        fs::read_dir(directory.join("exports/mit-bildern"))
            .unwrap()
            .count(),
        0
    );
}
#[test]
#[ignore = "seeds only an explicitly selected synthetic QA project root"]
fn seed_illustrated_native_qa() {
    let path = std::env::var_os("KRAQEL_ILLUSTRATED_QA_ROOT").expect("explicit QA directory");
    let (state, _) = setup(Path::new(&path));
    assert_eq!(fs::read_dir(state.project_root).unwrap().count(), 1);
}
