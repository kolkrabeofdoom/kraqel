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

#[test]
fn docx_embeds_fifty_verified_pngs_and_is_not_a_text_roundtrip() {
    use std::io::Read;
    let root = tempfile::tempdir().unwrap();
    let (project, docs, mut assets, mut selection) = fixture(root.path(), 50);
    selection[0].alt_text = "ÄÖÜ & \"<Rabe>\"".to_owned();
    selection[0].caption = Some("Erste Zeile\nZweite Zeile & <Text>".to_owned());
    let mut repeated = selection[0].clone();
    repeated.scene_id = docs[2].node.id;
    assets[0].links.push(AssetLink {
        target_kind: AssetTargetKind::Document,
        target_id: repeated.scene_id,
        role: "reference".to_owned(),
    });
    selection.push(repeated);
    let first = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Docx,
        &selection,
    )
    .unwrap();
    let second = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Docx,
        &selection,
    )
    .unwrap();
    assert_eq!(first.files, second.files);
    assert_eq!(first.files.len(), 2);
    let bytes = &first.files["manuskript.docx"];
    assert!(crate::read_docx_roundtrip(bytes).is_err());
    let manifest: serde_json::Value = serde_json::from_slice(&first.files["medien.json"]).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    assert_eq!(archive.len(), 55);
    for asset in manifest["assets"].as_array().unwrap() {
        let location = asset["location"].as_str().unwrap();
        assert!(location.starts_with("word/media/"));
        let mut png = Vec::new();
        archive
            .by_name(location)
            .unwrap()
            .read_to_end(&mut png)
            .unwrap();
        assert_eq!(hash(&png), asset["exportSha256"]);
    }
    let read = |archive: &mut zip::ZipArchive<Cursor<&Vec<u8>>>, name: &str| {
        let mut text = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        let mut reader = quick_xml::Reader::from_str(&text);
        loop {
            match reader.read_event().unwrap() {
                quick_xml::events::Event::Eof => break,
                quick_xml::events::Event::DocType(_) => panic!("DTD"),
                _ => {}
            }
        }
        text
    };
    let document = read(&mut archive, "word/document.xml");
    assert_eq!(document.matches("<wp:inline ").count(), 51);
    assert!(!document.contains("wp:anchor"));
    assert!(document.contains("descr=\"ÄÖÜ &amp; &quot;&lt;Rabe&gt;&quot;\""));
    assert!(document.contains("<w:br/>") && document.contains("Zweite Zeile &amp; &lt;Text&gt;"));
    assert!(document.find("Text 0 bleibt").unwrap() < document.find("<wp:inline").unwrap());
    assert!(document.find("<wp:inline").unwrap() < document.find("Text 1 bleibt").unwrap());
    let rels = read(&mut archive, "word/_rels/document.xml.rels");
    assert_eq!(
        rels.matches(&format!(
            "{}/image",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
        ))
        .count(),
        50
    );
    assert!(!rels.contains("TargetMode") && !rels.contains("PRIVATE"));
    let published = publish(root.path(), &first, || Ok(())).unwrap();
    let folder = root.path().join("exports").join(published.package_name);
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 2);
    assert!(!folder.join("assets").exists());
}

#[test]
fn pdf_has_fifty_white_proof_pngs_matching_embedded_rgb_and_atomic_failure() {
    let root = tempfile::tempdir().unwrap();
    let (project, docs, assets, selection) = fixture(root.path(), 50);
    let first = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Pdf,
        &selection,
    )
    .unwrap();
    let second = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Pdf,
        &selection,
    )
    .unwrap();
    assert_eq!(first.files.len(), 52);
    assert_eq!(first.files["medien.json"], second.files["medien.json"]);
    let manifest: serde_json::Value = serde_json::from_slice(&first.files["medien.json"]).unwrap();
    let document = printpdf::lopdf::Document::load_mem(&first.files["manuskript.pdf"]).unwrap();
    let embedded: Vec<_> = document
        .objects
        .values()
        .filter_map(|o| o.as_stream().ok())
        .filter(|s| s.dict.has(b"KRAQELImageId"))
        .collect();
    assert_eq!(embedded.len(), 50);
    for asset in manifest["assets"].as_array().unwrap() {
        let location = asset["location"].as_str().unwrap();
        let png = &first.files[location];
        assert_eq!(hash(png), asset["exportSha256"]);
        assert_eq!(*png, second.files[location]);
        let pixels = image::load_from_memory(png).unwrap().to_rgba8();
        assert!(pixels.pixels().all(|p| p[3] == 255));
        let rgb = image::load_from_memory(png).unwrap().to_rgb8();
        let id = asset["pdfImageId"].as_str().unwrap().as_bytes();
        let found = embedded
            .iter()
            .find(|s| s.dict.get(b"KRAQELImageId").unwrap().as_name().unwrap() == id)
            .unwrap();
        assert_eq!(
            found
                .decompressed_content()
                .unwrap_or_else(|_| found.content.clone()),
            rgb.into_raw()
        );
    }
    let result = publish(root.path(), &first, || Ok(())).unwrap();
    let old = root.path().join("exports").join(result.package_name);
    let mut invalid = selection.clone();
    invalid[0].alt_text = "Nicht unterstützt 🙂".to_owned();
    assert!(
        prepare(
            root.path(),
            &project,
            &docs,
            &assets,
            IllustratedFormat::Pdf,
            &invalid
        )
        .is_err()
    );
    assert!(publish(root.path(), &first, || Err(IllustratedError::Conflict)).is_err());
    assert!(old.join("manuskript.pdf").exists());
    assert_eq!(
        fs::read_dir(root.path().join("exports/mit-bildern"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn pdf_distinguishes_equal_rgb_bytes_with_different_image_dimensions() {
    let root = tempfile::tempdir().unwrap();
    let (project, docs, mut assets, mut selection) = fixture(root.path(), 2);
    let pixels = image::RgbaImage::from_pixel(2, 3, image::Rgba([0, 30, 100, 128]));
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let prepared =
        kraqel_asset_service::prepare_image("synthetic.png", &bytes.into_inner()).unwrap();
    let stored =
        kraqel_asset_service::store_image(root.path(), EntityId::new(), &prepared).unwrap();
    assets[1].relative_path = stored.relative_path;
    assets[1].checksum = prepared.checksum;
    assets[1].byte_count = prepared.original_bytes.len() as u64;
    assets[1].width = 2;
    assets[1].height = 3;
    selection[1].expected_checksum = assets[1].checksum.clone();
    let package = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Pdf,
        &selection,
    )
    .unwrap();
    assert_eq!(package.image_count, 2);
    let document = printpdf::lopdf::Document::load_mem(&package.files["manuskript.pdf"]).unwrap();
    assert_eq!(
        document
            .objects
            .values()
            .filter_map(|o| o.as_stream().ok())
            .filter(|s| s.dict.has(b"KRAQELImageId"))
            .count(),
        2
    );
}

#[test]
#[ignore = "writes synthetic illustrated DOCX only to an explicitly selected QA folder"]
fn generate_illustrated_docx_qa() {
    let output = std::env::var_os("KRAQEL_DOCX_MEDIA_QA_ROOT").expect("explicit QA folder");
    let output = Path::new(&output);
    fs::create_dir_all(output).unwrap();
    let root = tempfile::tempdir().unwrap();
    let (mut project, mut docs, mut assets, mut selection) = fixture(root.path(), 2);
    project.name = "Szenenbilder im Manuskript".to_owned();
    docs[0].node.title = "Kapitel mit Illustrationen".to_owned();
    for (index, asset) in assets.iter_mut().enumerate() {
        let (width, height) = if index == 0 { (640, 320) } else { (600, 1000) };
        let pixels = image::RgbaImage::from_fn(width, height, |x, y| {
            if x < width / 2 && y < height / 2 {
                image::Rgba([190, 50, 45, 255])
            } else if x >= width / 2 && y < height / 2 {
                image::Rgba([40, 130, 80, 170])
            } else if x < width / 2 {
                image::Rgba([40, 80, 170, 255])
            } else {
                image::Rgba([230, 180, 60, 100])
            }
        });
        let mut source = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(pixels)
            .write_to(&mut source, image::ImageFormat::Png)
            .unwrap();
        let prepared =
            kraqel_asset_service::prepare_image("PRIVATE-fixture.png", &source.into_inner())
                .unwrap();
        let stored =
            kraqel_asset_service::store_image(root.path(), EntityId::new(), &prepared).unwrap();
        asset.relative_path = stored.relative_path;
        asset.checksum = prepared.checksum;
        asset.width = width;
        asset.height = height;
        asset.byte_count = prepared.original_bytes.len() as u64;
        asset.title = if index == 0 {
            "Querformat mit Transparenz"
        } else {
            "Hochformat"
        }
        .to_owned();
        selection[index].expected_checksum = asset.checksum.clone();
        selection[index].alt_text = format!(
            "{}: Rot links oben, Grün rechts oben, Blau links unten, Gold rechts unten.",
            asset.title
        );
        selection[index].caption = Some(format!(
            "Abbildung {}: ÄÖÜ, ß, & und <Klartext>.\nZweite Zeile bleibt sichtbar.",
            index + 1
        ));
    }
    let mut repeated = selection[0].clone();
    repeated.scene_id = docs[4].node.id;
    repeated.caption = Some(
        "Lange Beschriftung mit Umlauten ÄÖÜ und Satzzeichen & <Text>. "
            .repeat(20)
            .chars()
            .take(1200)
            .collect(),
    );
    assets[0].links.push(AssetLink {
        target_kind: AssetTargetKind::Document,
        target_id: repeated.scene_id,
        role: "reference".to_owned(),
    });
    selection.push(repeated);
    let package = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Docx,
        &selection,
    )
    .unwrap();
    for (name, bytes) in package.files {
        fs::write(output.join(name), bytes).unwrap();
    }
}

#[test]
#[ignore = "writes synthetic illustrated PDF only to an explicitly selected QA folder"]
fn generate_illustrated_pdf_qa() {
    let output = std::env::var_os("KRAQEL_PDF_MEDIA_QA_ROOT").expect("explicit QA folder");
    let output = Path::new(&output);
    fs::create_dir_all(output).unwrap();
    let root = tempfile::tempdir().unwrap();
    let (mut project, mut docs, mut assets, mut selection) = fixture(root.path(), 2);
    project.name = "PDF Schrift und Szenenbilder ÄÖÜ".to_owned();
    docs[1].content.document = json!({"type":"doc","content":[
        {"type":"paragraph","content":[{"type":"text","text":"Unicode: A\u{308}ÖÜ ß é Ελληνικά Кириллица € ","marks":[{"type":"bold"}]},{"type":"text","text":"kursiv und fett kursiv", "marks":[{"type":"italic"},{"type":"bold"}]}]},
        {"type":"blockquote","content":[{"type":"paragraph","content":[{"type":"text","text":"Zitat mit sichtbaren Leerzeichen und Umlauten."}]}]},
        {"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"Ein Listeneintrag mit Überlänge: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"}]}]}]}
    ]});
    docs[0].node.title = "Kapitel mit Illustrationen".to_owned();
    for (index, asset) in assets.iter_mut().enumerate() {
        let (width, height) = if index == 0 { (640, 320) } else { (600, 1000) };
        let pixels = image::RgbaImage::from_fn(width, height, |x, y| {
            if x < width / 2 && y < height / 2 {
                image::Rgba([190, 50, 45, 255])
            } else if x >= width / 2 && y < height / 2 {
                image::Rgba([40, 130, 80, 170])
            } else if x < width / 2 {
                image::Rgba([40, 80, 170, 255])
            } else {
                image::Rgba([230, 180, 60, 100])
            }
        });
        let mut source = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(pixels)
            .write_to(&mut source, image::ImageFormat::Png)
            .unwrap();
        let prepared =
            kraqel_asset_service::prepare_image("PRIVATE-fixture.png", &source.into_inner())
                .unwrap();
        let stored =
            kraqel_asset_service::store_image(root.path(), EntityId::new(), &prepared).unwrap();
        asset.relative_path = stored.relative_path;
        asset.checksum = prepared.checksum;
        asset.width = width;
        asset.height = height;
        asset.byte_count = prepared.original_bytes.len() as u64;
        asset.title = if index == 0 {
            "Querformat mit Transparenz"
        } else {
            "Hochformat"
        }
        .to_owned();
        selection[index].expected_checksum = asset.checksum.clone();
        selection[index].alt_text = format!(
            "{}: Rot links oben, Grün rechts oben, Blau links unten, Gold rechts unten.",
            asset.title
        );
        selection[index].caption = Some(format!(
            "Abbildung {}: ÄÖÜ, ß, & und <Klartext>.\nZweite Zeile bleibt sichtbar.",
            index + 1
        ));
    }
    let mut repeated = selection[0].clone();
    repeated.scene_id = docs[4].node.id;
    repeated.caption = Some(
        "Lange Beschriftung mit Umlauten ÄÖÜ und Satzzeichen & <Text>. "
            .repeat(20)
            .chars()
            .take(1200)
            .collect(),
    );
    assets[0].links.push(AssetLink {
        target_kind: AssetTargetKind::Document,
        target_id: repeated.scene_id,
        role: "reference".to_owned(),
    });
    selection.push(repeated);
    let package = prepare(
        root.path(),
        &project,
        &docs,
        &assets,
        IllustratedFormat::Pdf,
        &selection,
    )
    .unwrap();
    fs::create_dir_all(output.join("assets")).unwrap();
    for (name, bytes) in package.files {
        fs::write(output.join(name), bytes).unwrap();
    }
}
