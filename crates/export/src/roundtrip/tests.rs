use super::{
    inventory::{self, Header, Inventory, PROPERTY, Role},
    package, writer,
};
use crate::{Block, Chapter, Inline, Manuscript, Scene, read_docx_roundtrip, write_docx_roundtrip};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
};

fn text(value: &str) -> Inline {
    Inline::Text {
        text: value.to_owned(),
        bold: false,
        italic: false,
    }
}
fn paragraph(value: &str) -> Block {
    Block::Paragraph(if value.is_empty() {
        vec![]
    } else {
        vec![text(value)]
    })
}
pub(super) fn sample() -> Manuscript {
    let nested = Block::BulletList(vec![vec![
        paragraph("Verschachtelte Aufzählung"),
        Block::Quote(vec![paragraph("Zitat innerhalb der Liste")]),
    ]]);
    let ordered = Block::OrderedList(vec![
        vec![
            paragraph("Erstes Listenelement"),
            paragraph("Fortsetzung desselben Elements"),
            nested,
        ],
        vec![paragraph("Zweites Listenelement")],
    ]);
    let quote = Block::Quote(vec![
        paragraph("Äußeres Zitat"),
        Block::Quote(vec![paragraph("Inneres Zitat")]),
        Block::BulletList(vec![vec![paragraph("Liste im Zitat")]]),
    ]);
    let mut blocks = vec![
        paragraph(
            "Nur ein synthetisches Testmanuskript. Strukturen und Text sollen nach erneutem Speichern erhalten bleiben.",
        ),
        Block::Paragraph(vec![
            text("EDITIERBAR Original"),
            text("  äöü ß Ω 😀  "),
            Inline::Text {
                text: "fett".to_owned(),
                bold: true,
                italic: false,
            },
            Inline::Text {
                text: "kursiv".to_owned(),
                bold: false,
                italic: true,
            },
            Inline::Text {
                text: "beides".to_owned(),
                bold: true,
                italic: true,
            },
            Inline::HardBreak,
            text("Zweite Zeile"),
        ]),
    ];
    blocks.extend((1..=6).map(|level| {
        Block::Heading(
            level,
            vec![text(&format!("Szenenüberschrift Ebene {level}"))],
        )
    }));
    blocks.extend([paragraph(""), ordered, quote]);
    Manuscript {
        title: "KRAQEL Strukturprobe".to_owned(),
        chapters: vec![
            Chapter {
                title: "Kapitel Eins".to_owned(),
                scenes: vec![
                    Scene {
                        title: "Am Schreibtisch".to_owned(),
                        blocks,
                    },
                    Scene {
                        title: "Am Schreibtisch".to_owned(),
                        blocks: vec![],
                    },
                ],
            },
            Chapter {
                title: String::new(),
                scenes: vec![],
            },
            Chapter {
                title: "Kapitel Eins".to_owned(),
                scenes: vec![Scene {
                    title: String::new(),
                    blocks: vec![],
                }],
            },
        ],
    }
}

fn unpack(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut parts = BTreeMap::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        parts.insert(file.name().unwrap().into_owned(), data);
    }
    parts
}
fn pack(parts: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in parts {
        zip.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}
fn mutate(bytes: &[u8], name: &str, change: impl FnOnce(String) -> String) -> Vec<u8> {
    let mut parts = unpack(bytes);
    let value = String::from_utf8(parts.remove(name).unwrap()).unwrap();
    parts.insert(name.to_owned(), change(value).into_bytes());
    pack(&parts)
}
fn rebind(bytes: &[u8], change: impl FnOnce(&mut Inventory)) -> Vec<u8> {
    let parts = package::read(bytes).unwrap();
    let (mut inv, old) = inventory::read(&parts["docProps/custom.xml"]).unwrap();
    change(&mut inv);
    let (props, digest) = writer::properties(inv.nodes).unwrap();
    let mut parts = unpack(bytes);
    parts.insert("docProps/custom.xml".to_owned(), props.into_bytes());
    let body = String::from_utf8(parts.remove("word/document.xml").unwrap())
        .unwrap()
        .replace(&old[..32], &digest[..32]);
    parts.insert("word/document.xml".to_owned(), body.into_bytes());
    pack(&parts)
}

/// Test-only migration of metadata. The public reader never accepts prototype profiles.
fn stamp_prototype(bytes: &[u8]) -> Vec<u8> {
    let mut parts = unpack(bytes);
    let props = crate::docx_scene::parse_xml(&parts["docProps/custom.xml"]).unwrap();
    let values: BTreeMap<_, _> = props
        .children
        .iter()
        .map(|p| (p.attr("", "name").unwrap(), p.children[0].text.as_str()))
        .collect();
    let header: Header = serde_json::from_str(values["KRAQEL.Roundtrip.Prototype2"]).unwrap();
    let payload = (1..=header.chunks)
        .map(|i| values[format!("KRAQEL.Roundtrip.Prototype2.{i:04}").as_str()])
        .collect::<String>();
    let inv: Inventory = serde_json::from_str(&payload).unwrap();
    let (props, digest) = writer::properties(inv.nodes).unwrap();
    parts.insert("docProps/custom.xml".to_owned(), props.into_bytes());
    let body = String::from_utf8(parts.remove("word/document.xml").unwrap()).unwrap();
    assert_eq!(body.matches("kqf2:").count(), 24);
    parts.insert(
        "word/document.xml".to_owned(),
        body.replace("kqf2:", "kqm1:")
            .replace(&header.sha256[..32], &digest[..32])
            .into_bytes(),
    );
    pack(&parts)
}
const FIXTURES: [(&str, &[u8]); 6] = [
    (
        "generated",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/generated.docx"),
    ),
    (
        "word-resaved",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/word-resaved.docx"),
    ),
    (
        "libreoffice-resaved",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/libreoffice-resaved.docx"),
    ),
    (
        "word-to-libreoffice",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/word-to-libreoffice.docx"),
    ),
    (
        "word-edited",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/word-edited.docx"),
    ),
    (
        "libreoffice-edited",
        include_bytes!("../../../../scripts/docx-prototype/fixtures/flat/libreoffice-edited.docx"),
    ),
];

#[test]
fn native_writer_is_deterministic_and_preserves_every_model_field() {
    let model = sample().canonicalized().unwrap();
    let bytes = write_docx_roundtrip(&model).expect("native writer self-check");
    assert_eq!(read_docx_roundtrip(&bytes).unwrap(), model);
    assert_eq!(write_docx_roundtrip(&model).unwrap(), bytes);
    for chapter in &model.chapters {
        for scene in &chapter.scenes {
            let doc = scene.editor_document().unwrap();
            assert_eq!(crate::parse_editor_document(&doc).unwrap(), scene.blocks);
        }
    }
}
#[test]
fn real_office_xml_is_read_after_explicit_test_only_profile_stamping() {
    for (name, source) in FIXTURES {
        assert!(
            read_docx_roundtrip(source).is_err(),
            "prototype must not be a production profile"
        );
        let bytes = stamp_prototype(source);
        let parts = package::read(&bytes).unwrap_or_else(|()| panic!("package: {name}"));
        inventory::read(&parts["docProps/custom.xml"])
            .unwrap_or_else(|()| panic!("inventory: {name}"));
        let mut expected = sample().canonicalized().unwrap();
        let replacement = match name {
            "word-edited" => Some("x"),
            "libreoffice-edited" => Some("y"),
            _ => None,
        };
        if let Some(replacement) = replacement
            && let Block::Paragraph(runs) = &mut expected.chapters[0].scenes[0].blocks[1]
            && let Inline::Text { text, .. } = &mut runs[0]
        {
            *text = text.replace("Original", replacement);
        }
        assert_eq!(
            read_docx_roundtrip(&bytes).unwrap_or_else(|_| panic!("content: {name}")),
            expected,
            "{name}"
        );
    }
}
#[test]
fn rejects_profile_inventory_and_marker_corruption() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    for (part, from, to) in [
        ("word/document.xml", "kqm1:", "kqm2:"),
        ("docProps/custom.xml", PROPERTY, "KRAQEL.Other.v1"),
        ("docProps/custom.xml", ".0001", ".0000"),
        (
            "docProps/custom.xml",
            "&quot;version&quot;:1",
            "&quot;version&quot;:2",
        ),
    ] {
        let changed = mutate(&bytes, part, |s| {
            assert!(s.contains(from));
            s.replace(from, to)
        });
        assert!(read_docx_roundtrip(&changed).is_err());
    }
}
#[test]
fn rejects_invalid_tree_even_when_all_digests_are_updated() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    for mode in 0..4 {
        let changed = rebind(&bytes, |inv| match mode {
            0 => inv.nodes[1].parent = inv.nodes[1].id.clone(),
            1 => inv.nodes[1].id = inv.nodes[0].id.clone(),
            2 => inv.nodes[1].position = "99".to_owned(),
            _ => inv.nodes[0].role = Role::Chapter,
        });
        assert!(read_docx_roundtrip(&changed).is_err());
    }
}
#[test]
fn rejects_unknown_content_relationships_and_mime_spoofing() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    for (part, from, to) in [
        (
            "word/document.xml",
            "<w:r><w:br/></w:r>",
            "<w:fldSimple w:instr=\"DATE\"/>",
        ),
        (
            "word/document.xml",
            "</w:sdtContent>",
            "<w:drawing/></w:sdtContent>",
        ),
        (
            "word/document.xml",
            "</w:sdt></w:p>",
            "</w:sdt><w:r><w:t>extra</w:t></w:r></w:p>",
        ),
        (
            "_rels/.rels",
            "Target=\"docProps/custom.xml\"",
            "Target=\"https://example.invalid\" TargetMode=\"External\"",
        ),
        (
            "word/_rels/document.xml.rels",
            "Target=\"numbering.xml\"",
            "Target=\"styles.xml\"",
        ),
        (
            "[Content_Types].xml",
            "wordprocessingml.document.main+xml",
            "wordprocessingml.template.main+xml",
        ),
        (
            "word/numbering.xml",
            "w:val=\"decimal\"",
            "w:val=\"upperRoman\"",
        ),
        (
            "word/numbering.xml",
            "<w:start w:val=\"1\"/>",
            "<w:start w:val=\"2\"/>",
        ),
    ] {
        let changed = mutate(&bytes, part, |s| {
            assert!(s.contains(from));
            s.replace(from, to)
        });
        assert!(read_docx_roundtrip(&changed).is_err(), "{from}");
    }
    let mut parts = unpack(&bytes);
    parts.insert("word/unknown.xml".to_owned(), b"<x/>".to_vec());
    assert!(read_docx_roundtrip(&pack(&parts)).is_err());
}
#[test]
fn rejects_duplicate_and_removed_empty_title_markers() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    let records = inventory::records(&sample());
    let empty = records
        .iter()
        .find(|r| r.node.role == Role::ChapterTitle && r.runs.as_ref().is_some_and(Vec::is_empty))
        .unwrap();
    let marker = format!("kqm1:chapter-title:{}:", empty.node.id);
    let changed = mutate(&bytes, "word/document.xml", |mut s| {
        let tag = s.find(&marker).unwrap();
        let start = s[..tag].rfind("<w:p>").unwrap();
        let end = tag + s[tag..].find("</w:p>").unwrap() + 6;
        s.replace_range(start..end, "");
        s
    });
    assert!(read_docx_roundtrip(&changed).is_err());
    let changed = mutate(&bytes, "word/document.xml", |s| {
        s.replace("kqm1:chapter-title:n0004:", "kqm1:project-title:n0002:")
    });
    assert!(read_docx_roundtrip(&changed).is_err());
}
#[test]
fn canonical_model_bounds_and_hidden_editor_attributes_are_enforced() {
    let mut model = sample();
    model.chapters[0].scenes[0].blocks[0] = Block::Heading(7, vec![]);
    assert!(write_docx_roundtrip(&model).is_err());
    let mut model = sample();
    model.title = "x".repeat(2 * 1024 * 1024 + 1);
    assert!(write_docx_roundtrip(&model).is_err());
    let mut model = sample();
    model.title = "bad\0title".to_owned();
    assert!(write_docx_roundtrip(&model).is_err());
    let (project, mut docs) = crate::tests::fixture();
    assert!(Manuscript::from_documents(&project, &docs).is_ok());
    docs[1].content.document["content"][0]["attrs"] = serde_json::json!({"textAlign":"center"});
    assert!(Manuscript::from_documents(&project, &docs).is_err());
    let (project, mut docs) = crate::tests::fixture();
    docs.push(docs[0].clone());
    assert!(Manuscript::from_documents(&project, &docs).is_err());
}
#[test]
fn failed_export_preserves_the_previous_roundtrip_file() {
    let root = tempfile::tempdir().unwrap();
    let (project, mut docs) = crate::tests::fixture();
    let result = crate::export_project_roundtrip(root.path(), &project, &docs).unwrap();
    let path = root.path().join("exports").join(result.file_name);
    let original = std::fs::read(&path).unwrap();
    read_docx_roundtrip(&original).unwrap();
    docs[1].content.document["content"][0]["attrs"] = serde_json::json!({"textAlign":"center"});
    assert!(crate::export_project_roundtrip(root.path(), &project, &docs).is_err());
    assert_eq!(std::fs::read(path).unwrap(), original);
}
#[test]
#[ignore = "writes a synthetic native DOCX to the explicitly selected QA file"]
fn generate_native_roundtrip_fixture() {
    let path = std::env::var_os("KRAQEL_ROUNDTRIP_QA_FILE")
        .expect("set KRAQEL_ROUNDTRIP_QA_FILE to a test file");
    std::fs::write(path, write_docx_roundtrip(&sample()).unwrap()).unwrap();
}

#[test]
#[ignore = "reads an explicitly selected native QA DOCX; no app data is modified"]
fn verify_native_roundtrip_fixture() {
    let path = std::env::var_os("KRAQEL_ROUNDTRIP_QA_FILE").expect("set KRAQEL_ROUNDTRIP_QA_FILE");
    let mut expected = sample().canonicalized().unwrap();
    if let Ok(replacement) = std::env::var("KRAQEL_ROUNDTRIP_QA_EDIT")
        && let Block::Paragraph(runs) = &mut expected.chapters[0].scenes[0].blocks[1]
        && let Inline::Text { text, .. } = &mut runs[0]
    {
        *text = text.replace("Original", &replacement);
    }
    assert_eq!(
        crate::read_docx_roundtrip_file(std::path::Path::new(&path)).unwrap(),
        expected
    );
}

#[test]
fn reads_only_bounded_docx_files_and_sanitizes_errors() {
    let root = tempfile::tempdir().unwrap();
    let good = root.path().join("sample.docx");
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    std::fs::write(&good, &bytes).unwrap();
    assert_eq!(
        crate::read_docx_roundtrip_file(&good).unwrap(),
        sample().canonicalized().unwrap()
    );
    let bad = root.path().join("private-manuscript.docm");
    std::fs::write(&bad, &bytes).unwrap();
    let error = crate::read_docx_roundtrip_file(&bad).unwrap_err();
    assert!(!error.to_string().contains("private-manuscript"));
    assert!(crate::read_docx_roundtrip_file(root.path()).is_err());
    std::fs::write(&good, vec![0; 8 * 1024 * 1024 + 1]).unwrap();
    assert!(crate::read_docx_roundtrip_file(&good).is_err());
}

#[test]
fn inherited_character_styles_are_resolved_and_unsupported_styles_rejected() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    let bytes = mutate(&bytes, "word/styles.xml", |s| {
        s.replace("</w:styles>","<w:style w:type=\"character\" w:styleId=\"BaseMark\"><w:name w:val=\"BaseMark\"/><w:rPr><w:b/></w:rPr></w:style><w:style w:type=\"character\" w:styleId=\"Derived\"><w:name w:val=\"Derived\"/><w:basedOn w:val=\"BaseMark\"/><w:rPr><w:i/></w:rPr></w:style></w:styles>")
    });
    let bytes = mutate(&bytes, "word/document.xml", |s| {
        s.replace(
            "<w:r><w:t xml:space=\"preserve\">Nur ein",
            "<w:r><w:rPr><w:rStyle w:val=\"Derived\"/></w:rPr><w:t xml:space=\"preserve\">Nur ein",
        )
    });
    let model = read_docx_roundtrip(&bytes).unwrap();
    assert!(
        matches!(&model.chapters[0].scenes[0].blocks[0],Block::Paragraph(runs) if matches!(&runs[0],Inline::Text{bold:true,italic:true,..}))
    );
    // Create a real basedOn cycle without changing the visible document or inventory.
    let cycle = mutate(&bytes, "word/styles.xml", |s| {
        s.replace(
            "<w:name w:val=\"BaseMark\"/>",
            "<w:name w:val=\"BaseMark\"/><w:basedOn w:val=\"Derived\"/>",
        )
    });
    assert!(read_docx_roundtrip(&cycle).is_err());
    let unknown = mutate(&bytes, "word/styles.xml", |s| {
        s.replace(
            "<w:rPr><w:i/></w:rPr></w:style>",
            "<w:rPr><w:i/><w:u w:val=\"single\"/></w:rPr></w:style>",
        )
    });
    assert!(read_docx_roundtrip(&unknown).is_err());
    let unknown = mutate(&bytes, "word/styles.xml", |s| {
        s.replace(
            "<w:name w:val=\"Derived\"/>",
            "<w:name w:val=\"Derived\"/><w:unknownStyleBehavior/>",
        )
    });
    assert!(read_docx_roundtrip(&unknown).is_err());
}

#[test]
fn foreign_inventory_and_duplicate_property_headers_are_rejected() {
    let bytes = write_docx_roundtrip(&sample()).unwrap();
    let parsed = package::read(&bytes).unwrap();
    let (mut inv, _) = inventory::read(&parsed["docProps/custom.xml"]).unwrap();
    inv.nodes[1].id = "n9999".to_owned();
    let (props, _) = writer::properties(inv.nodes).unwrap();
    let mut parts = unpack(&bytes);
    parts.insert("docProps/custom.xml".to_owned(), props.into_bytes());
    assert!(read_docx_roundtrip(&pack(&parts)).is_err());
    let duplicate = mutate(&bytes, "docProps/custom.xml", |s| {
        let start = s.find("<property").unwrap();
        let end = s[start..].find("</property>").unwrap() + start + 11;
        let item = s[start..end].to_owned();
        s.replace("</Properties>", &format!("{item}</Properties>"))
    });
    assert!(read_docx_roundtrip(&duplicate).is_err());
    let duplicate = mutate(&bytes, "docProps/custom.xml", |s| {
        s.replace(
            "&quot;version&quot;:1",
            "&quot;version&quot;:1,&quot;version&quot;:1",
        )
    });
    assert!(read_docx_roundtrip(&duplicate).is_err());
}

#[test]
fn excessive_model_depth_and_nonempty_chapter_bodies_are_rejected() {
    let mut block = paragraph("deep");
    for _ in 0..40 {
        block = Block::Quote(vec![block]);
    }
    let mut model = sample();
    model.chapters[0].scenes[0].blocks = vec![block];
    assert!(write_docx_roundtrip(&model).is_err());
    let mut model = sample();
    model.chapters[0].scenes[0].blocks = vec![paragraph(""); 4097];
    assert!(write_docx_roundtrip(&model).is_err());
    let (project, mut docs) = crate::tests::fixture();
    docs[0].content.document["content"] = serde_json::json!([{"type":"paragraph","content":[{"type":"text","text":"Must not disappear"}]}]);
    assert!(Manuscript::from_documents(&project, &docs).is_err());
}

#[test]
fn separate_ordered_lists_restart_and_xml_text_is_preserved() {
    let mut model = sample();
    model.chapters[0].scenes[0]
        .blocks
        .push(Block::OrderedList(vec![
            vec![paragraph("A")],
            vec![paragraph("B")],
        ]));
    model.chapters[0].scenes[0]
        .blocks
        .push(paragraph("& < > \" \r\n e\u{301} 😀"));
    let bytes = write_docx_roundtrip(&model).unwrap();
    assert_eq!(
        read_docx_roundtrip(&bytes).unwrap(),
        model.canonicalized().unwrap()
    );
}

#[test]
fn unsupported_editor_attributes_are_rejected_for_every_export_format() {
    let root = tempfile::tempdir().unwrap();
    let (project, mut docs) = crate::tests::fixture();
    docs[1].content.document["content"] = serde_json::json!([{"type":"orderedList","attrs":{"start":2},"content":[{"type":"listItem","content":[{"type":"paragraph"}]}]}]);
    for format in [
        crate::ExportFormat::Markdown,
        crate::ExportFormat::Html,
        crate::ExportFormat::Docx,
        crate::ExportFormat::Pdf,
    ] {
        assert!(crate::export_project(root.path(), &project, &docs, format).is_err());
    }
    assert!(!root.path().join("exports").exists());
}

#[test]
fn existing_exports_do_not_inherit_the_new_roundtrip_size_limit() {
    let root = tempfile::tempdir().unwrap();
    let (project, mut docs) = crate::tests::fixture();
    let large = "x".repeat(2 * 1024 * 1024 + 1);
    docs[1].content.document["content"] =
        serde_json::json!([{"type":"paragraph","content":[{"type":"text","text":large}]}]);
    assert!(Manuscript::from_documents(&project, &docs).is_err());
    let result =
        crate::export_project(root.path(), &project, &docs, crate::ExportFormat::Markdown).unwrap();
    assert!(result.byte_count > 2 * 1024 * 1024);
}

#[test]
fn native_v1_writer_survives_real_word_and_libreoffice_saves() {
    let expected = sample().canonicalized().unwrap();
    let generated = include_bytes!("../../tests/fixtures/roundtrip-v1/generated.docx");
    assert_eq!(write_docx_roundtrip(&sample()).unwrap(), generated);
    for bytes in [
        generated.as_slice(),
        include_bytes!("../../tests/fixtures/roundtrip-v1/word-resaved.docx").as_slice(),
        include_bytes!("../../tests/fixtures/roundtrip-v1/libreoffice-resaved.docx").as_slice(),
    ] {
        assert_eq!(read_docx_roundtrip(bytes).unwrap(), expected);
    }
    let mut edited = expected;
    let Block::Paragraph(runs) = &mut edited.chapters[0].scenes[0].blocks[1] else {
        panic!("fixture paragraph");
    };
    let Inline::Text { text, .. } = &mut runs[0] else {
        panic!("fixture text");
    };
    *text = text.replace("Original", "x");
    assert_eq!(
        read_docx_roundtrip(include_bytes!(
            "../../tests/fixtures/roundtrip-v1/word-edited.docx"
        ))
        .unwrap(),
        edited
    );
}
