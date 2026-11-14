use super::*;
use kraqel_domain::{
    AssetLink, CURRENT_SCHEMA_VERSION, DocumentNode, EditorDocument, MediaKind, MediaProvenance,
    ProjectId,
};
use serde_json::json;
use std::io::Cursor;

fn fixture(
    root: &Path,
    count: usize,
) -> (
    Project,
    Vec<ExportSourceDocument>,
    Vec<MediaAsset>,
    Vec<ImagePlacement>,
) {
    let project = Project {
        id: ProjectId::new(),
        name: "Szenenbilder".to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    let chapter = EntityId::new();
    let content = |text: &str| EditorDocument {
        version: 1,
        document: json!({"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":text}]}]}),
    };
    let mut documents = vec![ExportSourceDocument {
        node: DocumentNode {
            id: chapter,
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: "Kapitel".to_owned(),
            position: 0,
            revision: 0,
        },
        content: EditorDocument {
            version: 1,
            document: json!({"type":"doc","content":[]}),
        },
    }];
    let mut assets = Vec::new();
    let mut selection = Vec::new();
    for index in 0..count * 2 {
        let id = EntityId::new();
        documents.push(ExportSourceDocument {
            node: DocumentNode {
                id,
                project_id: project.id,
                parent_id: Some(chapter),
                kind: DocumentKind::Scene,
                title: format!("Szene {index}"),
                position: u32::try_from(index).unwrap(),
                revision: 3,
            },
            content: content(&format!("Text {index} bleibt vor dem Bild.")),
        });
        if index % 2 != 0 {
            continue;
        }
        let mut cursor = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            3,
            2,
            image::Rgba([u8::try_from(index).unwrap(), 30, 100, 128]),
        ))
        .write_to(&mut cursor, image::ImageFormat::Png)
        .unwrap();
        let prepared =
            kraqel_asset_service::prepare_image("private-source.png", &cursor.into_inner())
                .unwrap();
        let asset_id = EntityId::new();
        let stored = kraqel_asset_service::store_image(root, asset_id, &prepared).unwrap();
        let asset = MediaAsset {
            id: asset_id,
            project_id: project.id,
            kind: MediaKind::Image,
            relative_path: stored.relative_path,
            thumbnail_path: stored.thumbnail_path,
            original_name: "private-source.png".to_owned(),
            title: format!("Bild {index}"),
            mime_type: prepared.mime_type.to_owned(),
            checksum: prepared.checksum,
            byte_count: prepared.original_bytes.len() as u64,
            width: prepared.width,
            height: prepared.height,
            provenance: MediaProvenance::Imported,
            tags: vec!["PRIVATE-TAG".to_owned()],
            links: vec![AssetLink {
                target_kind: AssetTargetKind::Document,
                target_id: id,
                role: "reference".to_owned(),
            }],
        };
        selection.push(ImagePlacement {
            scene_id: id,
            asset_id,
            expected_checksum: asset.checksum.clone(),
            expected_scene_revision: 3,
            alt_text: format!("Alternativtext {index}"),
            caption: Some(format!("Beschriftung {index}")),
        });
        assets.push(asset);
    }
    (project, documents, assets, selection)
}
#[test]
fn hundred_scenes_fifty_images_are_complete_deterministic_private_and_portable() {
    let root = tempfile::tempdir().unwrap();
    let (project, docs, assets, selection) = fixture(root.path(), 50);
    for format in [IllustratedFormat::Markdown, IllustratedFormat::Html] {
        let first = prepare(root.path(), &project, &docs, &assets, format, &selection).unwrap();
        let second = prepare(root.path(), &project, &docs, &assets, format, &selection).unwrap();
        assert_eq!(first.files, second.files);
        assert_eq!(first.files.len(), 52);
        let manifest = String::from_utf8(first.files["medien.json"].clone()).unwrap();
        for secret in [
            "private-source",
            "PRIVATE-TAG",
            &project.id.to_string(),
            &docs[1].node.id.to_string(),
        ] {
            assert!(!manifest.contains(secret));
        }
        let value: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(value["placements"].as_array().unwrap().len(), 50);
        for asset in value["assets"].as_array().unwrap() {
            let path = asset["location"].as_str().unwrap();
            assert_eq!(hash(&first.files[path]), asset["exportSha256"]);
        }
        let text = String::from_utf8(first.files[format.file()].clone()).unwrap();
        assert!(text.find("Text 0 bleibt").unwrap() < text.find("Alternativtext 0").unwrap());
        assert!(text.find("Alternativtext 0").unwrap() < text.find("Text 1 bleibt").unwrap());
        let published = publish(root.path(), &first, || Ok(())).unwrap();
        assert_eq!(published.image_count, 50);
        let folder = root.path().join("exports").join(&published.package_name);
        assert_eq!(
            fs::read(folder.join("medien.json")).unwrap(),
            first.files["medien.json"]
        );
        assert!(publish(root.path(), &second, || Err(IllustratedError::Conflict)).is_err());
        assert!(folder.exists());
    }
    assert_eq!(
        fs::read_dir(root.path().join("exports/mit-bildern"))
            .unwrap()
            .count(),
        2
    );
}
#[test]
fn changed_foreign_duplicate_unlinked_and_budget_inputs_fail_without_output() {
    let root = tempfile::tempdir().unwrap();
    let (project, docs, assets, selection) = fixture(root.path(), 2);
    let check = |a: &[MediaAsset], s: &[ImagePlacement]| {
        prepare(root.path(), &project, &docs, a, IllustratedFormat::Html, s).is_err()
    };
    assert!(check(&assets, &[]));
    let mut changed = selection.clone();
    changed[0].expected_scene_revision += 1;
    assert!(check(&assets, &changed));
    let mut changed = selection.clone();
    changed[0].expected_checksum = "0".repeat(64);
    assert!(check(&assets, &changed));
    let mut changed = selection.clone();
    changed.push(changed[0].clone());
    assert!(check(&assets, &changed));
    let mut changed = assets.clone();
    changed[0].links.clear();
    assert!(check(&changed, &selection));
    let mut changed = assets.clone();
    changed[0].project_id = ProjectId::new();
    assert!(check(&changed, &selection));
    let mut changed = selection.clone();
    changed[0].alt_text = "bad\nalt".to_owned();
    assert!(check(&assets, &changed));
    let mut changed = assets.clone();
    changed[0].width = 40000;
    assert!(check(&changed, &selection));
    assert!(check(&assets, &vec![selection[0].clone(); 201]));
    let mut changed = assets.clone();
    changed[0].byte_count = MAX_PACKAGE as u64 + 1;
    assert!(check(&changed, &selection));
    let mut changed = assets.clone();
    changed[0].width = 16000;
    changed[0].height = 16000;
    assert!(check(&changed, &selection));
    assert!(validate_source_budget(&project, &vec![docs[0].clone(); 10001]).is_err());
    let mut budget = SourceBudget(32 * 1024 * 1024);
    assert!(budget.write_all(b"x").is_err());
    let other_root = tempfile::tempdir().unwrap();
    let (other_project, other_docs, other_assets, other_selection) = fixture(other_root.path(), 51);
    assert!(
        prepare(
            other_root.path(),
            &other_project,
            &other_docs,
            &other_assets,
            IllustratedFormat::Html,
            &other_selection,
        )
        .is_err()
    );
    fs::write(root.path().join(&assets[0].relative_path), b"changed").unwrap();
    assert!(check(&assets, &selection));
    assert!(!root.path().join("exports").exists());
}
#[test]
fn repeated_source_reordered_captions_are_escaped_without_implicit_images() {
    let root = tempfile::tempdir().unwrap();
    let (project, docs, mut assets, mut selection) = fixture(root.path(), 2);
    let scene = selection[0].scene_id;
    assets[1].links.push(AssetLink {
        target_kind: AssetTargetKind::Document,
        target_id: scene,
        role: "another-role".to_owned(),
    });
    let mut other = selection[1].clone();
    other.scene_id = scene;
    other.alt_text = "\"<script>&[]".to_owned();
    other.caption = Some("![x](https://evil)\n<script>".to_owned());
    selection.insert(0, other);
    let package = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Html,
        &selection,
    )
    .unwrap();
    assert_eq!(package.files.len(), 4);
    assert_eq!(package.placement_count, 3);
    let text = String::from_utf8(package.files["manuskript.html"].clone()).unwrap();
    assert!(!text.contains("<script>"));
    assert!(text.contains("&quot;&lt;script&gt;&amp;[]"));
    let value: serde_json::Value = serde_json::from_slice(&package.files["medien.json"]).unwrap();
    assert_eq!(
        value["placements"][0]["assetId"],
        value["placements"][2]["assetId"]
    );
    let markdown = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Markdown,
        &selection,
    )
    .unwrap();
    let text = String::from_utf8(markdown.files["manuskript.md"].clone()).unwrap();
    assert!(!text.contains("![x](https://evil)"));
}
