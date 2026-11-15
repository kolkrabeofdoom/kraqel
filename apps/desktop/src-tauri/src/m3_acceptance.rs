//! Synthetic integration corpus and explicitly activated native QA steps; never compiled into the app.
use super::*;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use kraqel_export::illustrated::{IllustratedFormat, ImagePlacement};
use serde_json::{Value, json};
use std::io::Cursor;

#[derive(Serialize, Deserialize)]
struct Corpus {
    project: String,
    scene: String,
    character: String,
}
fn state(root: &Path) -> AppState {
    AppState {
        project_root: root.join("projects"),
        ai_releases: Mutex::new(HashMap::new()),
    }
}
fn save(root: &Path, name: &str, value: &impl Serialize) {
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn corpus(root: &Path) -> Corpus {
    serde_json::from_slice(&fs::read(root.join("corpus.json")).unwrap()).unwrap()
}
fn seed(root: &Path) -> Corpus {
    fs::create_dir_all(root).unwrap();
    assert!(
        !root.join("corpus.json").exists(),
        "QA corpus already exists"
    );
    let app = state(root);
    fs::create_dir_all(&app.project_root).unwrap();
    let id = uuid::Uuid::new_v4();
    let workspace = create_project_impl(
        &app,
        id,
        CreateProjectRequest {
            name: "M3 Gesamttest".into(),
        },
    )
    .unwrap();
    let project = workspace.project.id;
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let mut opened = manager.open_project(project.to_string()).unwrap();
    let scenes = seed_documents(&mut opened, project, &workspace);
    let character = seed_story(&mut opened, project, &scenes);
    let event = seed_structure(&mut opened, project, &scenes, character);
    drop(opened);
    let assets = seed_assets(&app, project, &scenes, character, event);
    let mut opened = manager.open_project(project.to_string()).unwrap();
    let preset = {
        let mut repo = VisualRepository::new(opened.database_mut());
        repo.upsert_profile(
            project,
            character,
            &BTreeMap::from([("Augen".into(), "bernsteinfarben".into())]),
            Some(assets[0]),
            &["blaue Augen".into()],
        )
        .unwrap();
        repo.create_preset(
            project,
            "Tusche",
            &BTreeMap::from([("Linien".into(), "klar".into())]),
            &[EntityKind::Character],
        )
        .unwrap()
    };
    let mut repo = PromptRepository::new(opened.database_mut());
    let composition = repo
        .compose(project, scenes[0], &[character], Some(preset.id))
        .unwrap();
    repo.create_draft(
        project,
        "M3 Prompt",
        &composition,
        "Nur synthetischer lokaler Entwurf",
    )
    .unwrap();
    drop(opened);
    let result = Corpus {
        project: project.to_string(),
        scene: scenes[0].to_string(),
        character: character.to_string(),
    };
    save(root, "corpus.json", &result);
    save(
        root,
        "inventory-before.json",
        &inventory(&app, &result.project),
    );
    archive(&app, &result.project, &root.join("before.kraqel"));
    result
}
fn seed_documents(
    opened: &mut kraqel_storage::project_manager::ManagedProject,
    project: ProjectId,
    workspace: &WorkspaceResponse,
) -> Vec<EntityId> {
    let mut chapters = vec![
        workspace
            .documents
            .iter()
            .find(|d| d.kind == DocumentKind::Chapter)
            .unwrap()
            .id,
    ];
    let mut scenes = vec![
        workspace
            .documents
            .iter()
            .find(|d| d.kind == DocumentKind::Scene)
            .unwrap()
            .id,
    ];
    {
        let mut repo = DocumentRepository::new(opened.database_mut());
        for n in 2..=3 {
            chapters.push(
                repo.create(
                    project,
                    None,
                    DocumentKind::Chapter,
                    &format!("Kapitel {n}"),
                )
                .unwrap()
                .id,
            );
        }
        for n in 1..100 {
            scenes.push(
                repo.create(
                    project,
                    Some(chapters[n / 34]),
                    DocumentKind::Scene,
                    &format!("Szene {:03}", n + 1),
                )
                .unwrap()
                .id,
            );
        }
        for (n, scene) in scenes.iter().enumerate() {
            let content = EditorDocument {
                version: EDITOR_DOCUMENT_VERSION,
                document: json!({"type":"doc","content":[
                    {"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":format!("Prüfszene {}",n+1)}]},
                    {"type":"paragraph","content":[{"type":"text","text":format!("M3 Text {}: ÄÖÜ ß Ελληνικά Кириллица €. ",n+1)},
                        {"type":"text","text":"Fett","marks":[{"type":"bold"}]},{"type":"hardBreak"},
                        {"type":"text","text":"Kursiv","marks":[{"type":"italic"}]}]}
                ]}),
            };
            repo.save_scene(project, *scene, 0, &content).unwrap();
        }
    }
    scenes
}

fn seed_story(
    opened: &mut kraqel_storage::project_manager::ManagedProject,
    project: ProjectId,
    scenes: &[EntityId],
) -> EntityId {
    let mut repo = StoryBibleRepository::new(opened.database_mut());
    let character = repo
        .create_entity(
            project,
            EntityKind::Character,
            "Rhea",
            &BTreeMap::from([("Rolle".into(), "Botin".into())]),
        )
        .unwrap();
    let place = repo
        .create_entity(
            project,
            EntityKind::Location,
            "Bibliothek",
            &BTreeMap::new(),
        )
        .unwrap();
    repo.create_relation(project, character.id, place.id, "kennt", &BTreeMap::new())
        .unwrap();
    for scene in &scenes[..2] {
        repo.link_document(project, *scene, character.id).unwrap();
    }
    character.id
}

fn seed_structure(
    opened: &mut kraqel_storage::project_manager::ManagedProject,
    project: ProjectId,
    scenes: &[EntityId],
    character: EntityId,
) -> EntityId {
    let mut repo = StructureRepository::new(opened.database_mut());
    let story = repo
        .create_storyline(project, "Die Nachricht", "#008080")
        .unwrap();
    repo.plot_board(project).unwrap();
    repo.update_card(
        project,
        scenes[0],
        PlotStatus::Draft,
        Some(story.id),
        Some(character),
    )
    .unwrap();
    let relative = repo
        .create_timeline(project, "Chronologie", TimelineMode::Relative)
        .unwrap();
    let absolute = repo
        .create_timeline(project, "Erzählreihenfolge", TimelineMode::Absolute)
        .unwrap();
    let event = repo
        .create_event(
            project,
            relative.id,
            "Ankunft",
            "Synthetische Prüfung",
            None,
        )
        .unwrap();
    repo.create_event(
        project,
        absolute.id,
        "Rückblick",
        "Zweite Zeitebene",
        Some("2026-11-15"),
    )
    .unwrap();
    repo.link_event(project, event.id, Some(scenes[0]), None)
        .unwrap();
    repo.link_event(project, event.id, None, Some(character))
        .unwrap();
    event.id
}

fn seed_assets(
    app: &AppState,
    project: ProjectId,
    scenes: &[EntityId],
    character: EntityId,
    event: EntityId,
) -> Vec<EntityId> {
    let id = uuid::Uuid::new_v4();
    let mut assets = Vec::new();
    for n in 0_u8..50 {
        let (format, ext) = match n % 3 {
            0 => (ImageFormat::Png, "png"),
            1 => (ImageFormat::Jpeg, "jpg"),
            _ => (ImageFormat::WebP, "webp"),
        };
        let pixels = RgbImage::from_fn(160 + u32::from(n), 100, |x, y| {
            Rgb([
                n.wrapping_mul(5),
                u8::try_from(x % 256).unwrap(),
                u8::try_from(y).unwrap(),
            ])
        });
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(pixels)
            .write_to(&mut bytes, format)
            .unwrap();
        let asset = import_media_asset_impl(
            app,
            id,
            &ImportMediaRequest {
                project_id: project.to_string(),
                file_name: format!("synthetic-{n:02}.{ext}"),
                title: format!("M3 Bild {:02}", n + 1),
                tags: vec!["synthetisch".into()],
                bytes: bytes.into_inner(),
            },
        )
        .unwrap()
        .asset;
        for target in [scenes[usize::from(n)], scenes[usize::from(n) + 50]] {
            change_media_link_impl(
                app,
                id,
                &AssetLinkRequest {
                    project_id: project.to_string(),
                    asset_id: asset.id.to_string(),
                    target_kind: AssetTargetKind::Document,
                    target_id: target.to_string(),
                    role: "reference".into(),
                },
                true,
            )
            .unwrap();
        }
        if n == 0 {
            for (kind, target) in [
                (AssetTargetKind::Entity, character),
                (AssetTargetKind::Event, event),
            ] {
                change_media_link_impl(
                    app,
                    id,
                    &AssetLinkRequest {
                        project_id: project.to_string(),
                        asset_id: asset.id.to_string(),
                        target_kind: kind,
                        target_id: target.to_string(),
                        role: "reference".into(),
                    },
                    true,
                )
                .unwrap();
            }
        }
        assets.push(asset.id);
    }
    assets
}

fn archive(app: &AppState, project: &str, destination: &Path) {
    export_project_archive_file_impl(app, uuid::Uuid::new_v4(), project, destination).unwrap();
}
fn normalized(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            if fields.contains_key("projectId") {
                fields.insert("projectId".into(), json!("PROJECT"));
            }
            for v in fields.values_mut() {
                normalized(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                normalized(v);
            }
        }
        _ => {}
    }
}
fn inventory(app: &AppState, value: &str) -> Value {
    let id = ProjectId::parse(value).unwrap();
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let mut opened = manager.open_project(value).unwrap();
    let service = GitService::new(opened.directory());
    let summary = service.create_snapshot(opened.database(), id).unwrap();
    assert_eq!(summary.format_version, 3);
    let mut result = serde_json::Map::new();
    for file in [
        "documents.json",
        "story-bible.json",
        "plotboard.json",
        "timelines.json",
        "visual-profiles.json",
        "prompt-drafts.json",
    ] {
        let mut data: Value = serde_json::from_slice(
            &fs::read(opened.directory().join("snapshot").join(file)).unwrap(),
        )
        .unwrap();
        normalized(&mut data);
        result.insert(file.into(), data);
    }
    let manifest: Value = serde_json::from_slice(
        &fs::read(opened.directory().join("snapshot/manifest.json")).unwrap(),
    )
    .unwrap();
    result.insert("assets".into(), manifest["assets"].clone());
    let scenes = DocumentRepository::new(opened.database_mut())
        .list(id)
        .unwrap();
    assert_eq!(
        scenes
            .iter()
            .filter(|n| n.kind == DocumentKind::Scene)
            .count(),
        100
    );
    assert_eq!(
        scenes
            .iter()
            .filter(|n| n.kind == DocumentKind::Chapter)
            .count(),
        3
    );
    let assets = MediaRepository::new(opened.database_mut())
        .list(id)
        .unwrap();
    assert_eq!(assets.len(), 50);
    assert_eq!(assets.iter().map(|a| a.links.len()).sum::<usize>(), 102);
    let character = StoryBibleRepository::new(opened.database_mut())
        .list(id, None, Some(EntityKind::Character))
        .unwrap()[0]
        .id;
    assert_eq!(
        StoryBibleRepository::new(opened.database_mut())
            .load(id, character)
            .unwrap()
            .backlinks
            .len(),
        2
    );
    assert_eq!(
        StructureRepository::new(opened.database_mut())
            .list_timelines(id)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        VisualRepository::new(opened.database_mut())
            .list_profiles(id)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        PromptRepository::new(opened.database_mut())
            .list_drafts(id)
            .unwrap()
            .len(),
        1
    );
    Value::Object(result)
}
fn expected_renames(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            if fields.get("title") == Some(&json!("Szene 1")) {
                fields.insert("title".into(), json!("Szene 1 geändert"));
            }
            if fields.get("name") == Some(&json!("Rhea")) {
                fields.insert("name".into(), json!("Rhea geändert"));
            }
            for v in fields.values_mut() {
                expected_renames(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                expected_renames(v);
            }
        }
        _ => {}
    }
}
fn check_renamed(root: &Path) {
    let c = corpus(root);
    let mut expected: Value =
        serde_json::from_slice(&fs::read(root.join("inventory-before.json")).unwrap()).unwrap();
    expected_renames(&mut expected);
    let actual = inventory(&state(root), &c.project);
    assert_eq!(
        actual, expected,
        "Only the two deliberate native UI renames may change semantics"
    );
    save(root, "inventory-after.json", &actual);
}
fn backup_recovery(root: &Path) {
    check_renamed(root);
    let c = corpus(root);
    let manager = ProjectManager::new(root).unwrap();
    fs::create_dir_all(root.join("recovery/projects")).unwrap();
    let backup = manager
        .backup_project(Path::new("projects").join(&c.project), "m3-native")
        .unwrap();
    let bytes = fs::read(&backup).unwrap();
    drop(
        manager
            .restore_backup(
                Path::new("projects").join(&c.project),
                "m3-native",
                Path::new("recovery/projects").join(&c.project),
            )
            .unwrap(),
    );
    let recovered = state(&root.join("recovery"));
    assert_eq!(
        inventory(&recovered, &c.project),
        inventory(&state(root), &c.project)
    );
    assert_eq!(
        fs::read(backup).unwrap(),
        bytes,
        "Backup must remain byte-identical"
    );
    save(
        root,
        "recovery.json",
        &json!({"passed":true,"project":c.project,"sameSemanticInventory":true,"backupUnchanged":true}),
    );
}
fn export_all(root: &Path) {
    let c = corpus(root);
    let app = state(root);
    let id = ProjectId::parse(&c.project).unwrap();
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let mut opened = manager.open_project(&c.project).unwrap();
    let assets = MediaRepository::new(opened.database_mut())
        .list(id)
        .unwrap();
    let nodes = DocumentRepository::new(opened.database_mut())
        .list(id)
        .unwrap();
    let placements: Vec<_> = assets
        .iter()
        .flat_map(|asset| {
            asset
                .links
                .iter()
                .filter(|l| l.target_kind == AssetTargetKind::Document)
                .map(|link| ImagePlacement {
                    scene_id: link.target_id,
                    asset_id: asset.id,
                    expected_checksum: asset.checksum.clone(),
                    expected_scene_revision: nodes
                        .iter()
                        .find(|n| n.id == link.target_id)
                        .unwrap()
                        .revision,
                    alt_text: format!("Synthetische Illustration {}", asset.title),
                    caption: Some("M3 Prüfabbildung ÄÖÜ & <Text>".into()),
                })
        })
        .collect();
    assert_eq!(placements.len(), 100);
    drop(opened);
    let mut packages = Vec::new();
    for format in [
        IllustratedFormat::Markdown,
        IllustratedFormat::Html,
        IllustratedFormat::Docx,
        IllustratedFormat::Pdf,
    ] {
        let result = illustrated_export::export(
            &app,
            uuid::Uuid::new_v4(),
            &illustrated_export::IllustratedRequest {
                project_id: c.project.clone(),
                format,
                placements: placements.clone(),
            },
        )
        .unwrap();
        assert_eq!(result.image_count, 50);
        assert_eq!(result.placement_count, 100);
        packages.push(serde_json::to_value(result).unwrap());
    }
    save(root, "exports.json", &packages);
}
fn check_imports(root: &Path, native: bool) {
    let c = corpus(root);
    let app = state(root);
    let expected = inventory(&app, &c.project);
    let file = root.join(if native {
        "native.kraqel"
    } else {
        "after.kraqel"
    });
    if !native {
        archive(&app, &c.project, &file);
    }
    let before = fs::read(&file).unwrap();
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let imported: Vec<_> = manager
        .list_projects()
        .unwrap()
        .into_iter()
        .filter(|p| p.id.to_string() != c.project)
        .collect();
    if native {
        assert_eq!(imported.len(), 2, "Two native dialog imports required");
    } else {
        for _ in 0..2 {
            drop(import_snapshot_archive_file(&manager, &file).unwrap());
        }
    }
    let projects = manager.list_projects().unwrap();
    assert_eq!(projects.len(), 3);
    for p in &projects {
        assert_eq!(inventory(&app, &p.id.to_string()), expected);
    }
    assert_eq!(fs::read(file).unwrap(), before);
    save(
        root,
        "imports.json",
        &json!({"passed":true,"projects":projects.iter().map(|p|p.id.to_string()).collect::<Vec<_>>(),"sameSemanticInventory":true,"archiveUnchanged":true}),
    );
}
fn negative_cases(root: &Path) {
    let c = corpus(root);
    let app = state(root);
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let before = inventory(&app, &c.project);
    let (mut faulty, summary) =
        import_snapshot_archive_file(&manager, &root.join("after.kraqel")).unwrap();
    let id = summary.project_id;
    let assets = MediaRepository::new(faulty.database_mut())
        .list(id)
        .unwrap();
    let first_scene = assets[0]
        .links
        .iter()
        .find(|link| link.target_kind == AssetTargetKind::Document)
        .unwrap()
        .target_id;
    let revision = DocumentRepository::new(faulty.database_mut())
        .load(id, first_scene)
        .unwrap()
        .node
        .revision;
    let request = illustrated_export::IllustratedRequest {
        project_id: id.to_string(),
        format: IllustratedFormat::Html,
        placements: vec![ImagePlacement {
            scene_id: first_scene,
            asset_id: assets[0].id,
            expected_checksum: assets[0].checksum.clone(),
            expected_scene_revision: revision,
            alt_text: "Synthetischer Fehlerfall".into(),
            caption: None,
        }],
    };
    drop(faulty);
    illustrated_export::export(&app, uuid::Uuid::new_v4(), &request).unwrap();
    let faulty = manager.open_project(id.to_string()).unwrap();
    let previous = tree_bytes(&faulty.directory().join("exports"));
    faulty
        .database()
        .connection()
        .execute("UPDATE project SET name='M3 Fehlerprobe'", [])
        .unwrap();
    fs::rename(
        faulty.directory().join(&assets[0].relative_path),
        faulty.directory().join("missing-original.saved"),
    )
    .unwrap();
    fs::rename(
        faulty.directory().join(&assets[1].thumbnail_path),
        faulty.directory().join("missing-thumbnail.saved"),
    )
    .unwrap();
    let changed = faulty.directory().join(&assets[2].relative_path);
    let mut bytes = fs::read(&changed).unwrap();
    bytes.push(1);
    fs::write(changed, bytes).unwrap();
    drop(faulty);
    let listed = list_media_assets_impl(&app, uuid::Uuid::new_v4(), &id.to_string()).unwrap();
    for asset in &assets[..3] {
        assert!(
            !listed
                .iter()
                .find(|v| v.asset.id == asset.id)
                .unwrap()
                .file_available
        );
    }
    assert!(manager.backup_project(id.to_string(), "must-fail").is_err());
    assert!(illustrated_export::export(&app, uuid::Uuid::new_v4(), &request).is_err());
    assert_eq!(
        tree_bytes(&app.project_root.join(id.to_string()).join("exports")),
        previous
    );
    let count = manager.list_projects().unwrap().len();
    let invalid = root.join("invalid.kraqel");
    fs::write(&invalid, b"not a ZIP").unwrap();
    assert!(import_snapshot_archive_file(&manager, &invalid).is_err());
    let large = root.join("oversized.kraqel");
    fs::File::create(&large)
        .unwrap()
        .set_len(128 * 1024 * 1024 + 1)
        .unwrap();
    assert!(import_snapshot_archive_file(&manager, &large).is_err());
    let docx = root.join("invalid.docx");
    fs::write(&docx, b"not a DOCX").unwrap();
    assert!(import_docx_project_file_impl(&app, uuid::Uuid::new_v4(), &docx).is_err());
    assert_eq!(manager.list_projects().unwrap().len(), count);
    assert_eq!(inventory(&app, &c.project), before);
    save(
        root,
        "negative.json",
        &json!({"passed":true,"faultProject":id.to_string(),"unavailable":3,"backupRejected":true,"exportRejected":true,"previousPackageUnchanged":true,"invalidArchiveRejected":true,"oversizedArchiveRejected":true,"invalidDocxRejected":true,"sourceUnchanged":true,"projectCount":count}),
    );
}

fn tree_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, items: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            assert!(!path.is_symlink());
            if path.is_dir() {
                visit(root, &path, items);
            } else {
                items.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn check_history(root: &Path, native: bool) {
    let expected: Value =
        serde_json::from_slice(&fs::read(root.join("inventory-before.json")).unwrap()).unwrap();
    let app = if native {
        state(root)
    } else {
        state(&root.join("history"))
    };
    fs::create_dir_all(&app.project_root).unwrap();
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let id = if native {
        let imported: Value =
            serde_json::from_slice(&fs::read(root.join("imports.json")).unwrap()).unwrap();
        let projects = manager.list_projects().unwrap();
        assert_eq!(projects.len(), 4);
        projects
            .iter()
            .find(|p| {
                !imported["projects"]
                    .as_array()
                    .unwrap()
                    .contains(&json!(p.id.to_string()))
            })
            .unwrap()
            .id
    } else {
        import_snapshot_archive_file(&manager, &root.join("before.kraqel"))
            .unwrap()
            .1
            .project_id
    };
    assert_eq!(inventory(&app, &id.to_string()), expected);
    save(
        root,
        "history.json",
        &json!({"passed":true,"project":id.to_string(),"originalSceneAndCharacterNames":true,"sameBeforeInventory":true}),
    );
}

#[test]
fn integrated_m3_corpus_preserves_backlinks_through_backup_and_two_imports() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let c = seed(root);
    let app = state(root);
    let request_id = uuid::Uuid::new_v4();
    rename_document_impl(
        &app,
        request_id,
        RenameDocumentRequest {
            project_id: c.project.clone(),
            document_id: c.scene.clone(),
            expected_title: "Szene 1".into(),
            title: "Szene 1 geändert".into(),
        },
    )
    .unwrap();
    let manager = ProjectManager::new(&app.project_root).unwrap();
    let mut opened = manager.open_project(&c.project).unwrap();
    StoryBibleRepository::new(opened.database_mut())
        .update_entity(
            ProjectId::parse(&c.project).unwrap(),
            EntityId::parse(&c.character).unwrap(),
            EntityKind::Character,
            "Rhea geändert",
            &BTreeMap::from([("Rolle".into(), "Botin".into())]),
        )
        .unwrap();
    drop(opened);
    backup_recovery(root);
    check_imports(root, false);
    check_history(root, false);
    export_all(root);
    negative_cases(root);
}

#[test]
#[ignore = "explicit synthetic native M3 QA only; requires a dedicated temporary root and step"]
fn run_native_m3_qa_step() {
    let root =
        PathBuf::from(std::env::var_os("KRAQEL_M3_QA_ROOT").expect("KRAQEL_M3_QA_ROOT required"));
    assert!(root.is_absolute());
    let temp = std::env::temp_dir().canonicalize().unwrap();
    assert_eq!(root.parent().unwrap().canonicalize().unwrap(), temp);
    assert!(
        root.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("kraqel-m3-acceptance-")
    );
    let step = std::env::var("KRAQEL_M3_QA_STEP").expect("explicit QA step required");
    if step != "seed" {
        assert!(root.join("corpus.json").is_file());
    }
    match step.as_str() {
        "seed" => {
            seed(&root);
        }
        "verify-renamed" => check_renamed(&root),
        "backup" => backup_recovery(&root),
        "exports" => export_all(&root),
        "verify-imports" => {
            check_imports(&root, true);
            fs::copy(root.join("native.kraqel"), root.join("after.kraqel")).unwrap();
        }
        "negative" => negative_cases(&root),
        "verify-history" => check_history(&root, true),
        _ => panic!("Unknown QA step"),
    }
}
