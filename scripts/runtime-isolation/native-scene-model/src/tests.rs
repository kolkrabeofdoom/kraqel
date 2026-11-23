use super::*;
use serde_json::{Value, to_value};

const READ: &[u8] = br#"{"command":"c4_document_read_v1"}"#;
const STATUS: &[u8] = br#"{"command":"c4_workspace_status_v1"}"#;

fn save(revision: u32, text: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "command": "c4_document_save_v1", "expectedRevision": revision, "text": text
    }))
    .unwrap()
}

fn reply(binding: &mut BoundScene<'_>, bytes: &[u8]) -> Value {
    to_value(binding.handle(bytes)).unwrap()
}

#[test]
fn each_binding_edits_only_its_scene_and_preserves_both_paragraphs() {
    for (scene, other, id, original, edited, untouched, bold) in [
        (
            Scene::A,
            Scene::B,
            "A",
            "Rabe\n42",
            "Rabe\n43",
            "Mond\n7",
            false,
        ),
        (
            Scene::A,
            Scene::B,
            "A",
            "Rabe\n42",
            "Rabe\n43",
            "Mond\n7",
            true,
        ),
        (
            Scene::B,
            Scene::A,
            "B",
            "Mond\n7",
            "Mond\n8",
            "Rabe\n42",
            false,
        ),
    ] {
        let mut catalog = if bold {
            SyntheticCatalog::with_marked_scene_a()
        } else {
            SyntheticCatalog::default()
        };
        let mut binding = catalog.bind(scene, Mode::Edit);
        assert_eq!(
            reply(&mut binding, STATUS),
            json!({"scene":id,"mode":"edit","revision":0})
        );
        let expected_read = |text, revision| {
            let mut answer = json!({"text":text,"revision":revision});
            if bold {
                answer["bold_second"] = json!(true);
            }
            answer
        };
        assert_eq!(reply(&mut binding, READ), expected_read(original, 0));
        assert_eq!(
            reply(&mut binding, &save(0, untouched)),
            json!({"error":"INVALID"})
        );
        assert_eq!(reply(&mut binding, &save(0, edited)), json!({"revision":1}));
        assert_eq!(
            reply(&mut binding, &save(0, original)),
            json!({"error":"CONFLICT"})
        );

        let mut reopened = catalog.bind(scene, Mode::Read);
        let lines: Vec<_> = edited.split('\n').collect();
        assert_eq!(reopened.record.content.version, 1);
        let mut second = json!({"type":"text","text":lines[1]});
        if bold {
            second["marks"] = json!([{"type":"bold"}]);
        }
        assert_eq!(
            reopened.record.content.document,
            json!({"type":"doc","content":[
                {"type":"paragraph","content":[{"type":"text","text":lines[0]}]},
                {"type":"paragraph","content":[second]}
            ]})
        );
        assert_eq!(reply(&mut reopened, READ), expected_read(edited, 1));

        let mut countercheck = catalog.bind(other, Mode::Read);
        assert_eq!(
            reply(&mut countercheck, READ),
            json!({"text":untouched,"revision":0})
        );
    }
}

#[test]
fn read_binding_cannot_write_or_select_another_target_or_mode() {
    let mut catalog = SyntheticCatalog::default();
    let mut binding = catalog.bind(Scene::B, Mode::Read);
    assert_eq!(
        reply(&mut binding, STATUS),
        json!({"scene":"B","mode":"read","revision":0})
    );
    assert_eq!(
        reply(&mut binding, &save(0, "Mond\n8")),
        json!({"error":"DENIED"})
    );
    for body in [
        r#"{"command":"c4_document_read_v1","scene":"A"}"#,
        r#"{"command":"c4_document_save_v1","mode":"edit","expectedRevision":0,"text":"Mond\n8"}"#,
        r#"{"command":"c4_workspace_status_v1","extra":null}"#,
    ] {
        assert_eq!(
            reply(&mut binding, body.as_bytes()),
            json!({"error":"INVALID"})
        );
    }
    assert_eq!(
        reply(&mut binding, READ),
        json!({"text":"Mond\n7","revision":0})
    );

    let formatted = br#"{"command":"c4_document_save_v2","expectedRevision":1,"text":"Rabe\n42","boldSecond":true}"#;
    let before = catalog.snapshot_a();
    for (mode, expected) in [(Mode::Read, "DENIED"), (Mode::Edit, "CONFLICT")] {
        let result = catalog.bind(Scene::A, mode).handle_managed(formatted);
        assert_eq!(to_value(result).unwrap(), json!({"error": expected}));
        assert_eq!(catalog.snapshot_a(), before);
    }
}

#[test]
fn rejects_invalid_frames_without_echo_or_mutation() {
    let mut catalog = SyntheticCatalog::default();
    let mut binding = catalog.bind(Scene::A, Mode::Edit);
    for body in [
        &b"\xff"[..],
        br#"{"command":"c4_document_save_v1","expectedRevision":0,"expectedRevision":1,"text":"Rabe\n43"}"#,
        br#"{"command":"c4_document_read_v1","command":"c4_document_save_v1"}"#,
        br#"{"command":"c4_document_read_v1"}{}"#,
        br#"{"command":"c4_document_save_v1","expectedRevision":-1,"text":"Rabe\n43"}"#,
        br#"{"command":"other"}"#,
        &save(0, "Rabe\r\n43"),
    ] {
        assert_eq!(reply(&mut binding, body), json!({"error":"INVALID"}));
    }
    for body in [
        vec![0; MAX_FRAME_BYTES + 1],
        save(0, &"X".repeat(MAX_TEXT_BYTES + 1)),
        save(MAX_REVISION + 1, "Rabe\n43"),
    ] {
        assert_eq!(reply(&mut binding, &body), json!({"error":"LIMIT"}));
    }
    assert_eq!(
        reply(&mut binding, READ),
        json!({"text":"Rabe\n42","revision":0})
    );
}

#[test]
fn unsupported_stored_documents_cannot_be_read_or_replaced() {
    let mut marked = document("Rabe\n42");
    marked.document["content"][0]["content"][0]["marks"] = json!([{"type":"bold"}]);
    let mut extended = to_value(document("Rabe\n42")).unwrap();
    extended["annotations"] = json!(["synthetic"]);
    assert!(SyntheticCatalog::restore_managed_a(1, &extended.to_string()).is_none());
    let mut wrong_version = document("Rabe\n42");
    wrong_version.version = 2;
    for damaged in [marked, wrong_version, document("Mond\n7")] {
        let mut catalog = SyntheticCatalog::default();
        let encoded = serde_json::to_string(&damaged).unwrap();
        assert!(SyntheticCatalog::restore_managed_a(0, &encoded).is_none());
        catalog.a.content = damaged.clone();
        let mut binding = catalog.bind(Scene::A, Mode::Edit);
        assert_eq!(reply(&mut binding, READ), json!({"error":"INVALID"}));
        assert_eq!(
            reply(&mut binding, &save(0, "Rabe\n43")),
            json!({"error":"INVALID"})
        );
        assert_eq!(catalog.a.content, damaged);
        assert_eq!(catalog.a.revision, 0);
    }
}

#[test]
fn known_inline_marks_open_only_as_plain_text_preview() {
    let mut doc = document("Kolkrabe\n42");
    doc.document["content"][0]["content"] = json!([
        {"type":"text","text":"Kolk","marks":[{"type":"italic"},{"type":"bold"}]},
        {"type":"text","text":"rabe","marks":[{"type":"strike"}]}
    ]);
    let encoded = serde_json::to_string(&doc).unwrap();
    let mut catalog = SyntheticCatalog::restore_managed_a(1, &encoded).unwrap();
    for mode in [Mode::Read, Mode::Edit] {
        let mut scene = catalog.bind(Scene::A, mode);
        assert_eq!(
            to_value(scene.handle_managed(READ)).unwrap(),
            json!({"preview_text":"Kolkrabe\n42","revision":1})
        );
        for body in [save(1, "Rabe\n43"), br#"{"command":"c4_document_save_v2","expectedRevision":1,"text":"Rabe\n43","boldSecond":true}"#.to_vec()] {
            assert_eq!(to_value(scene.handle_managed(&body)).unwrap(), json!({"error":"DENIED"}));
        }
        assert_eq!(catalog.snapshot_a(), (encoded.clone(), 1));
    }
    for unsupported in [
        json!({"type":"italic","attrs":{}}),
        json!({"type":"future"}),
    ] {
        doc.document["content"][0]["content"][0]["marks"] = json!([unsupported]);
        assert!(
            SyntheticCatalog::restore_managed_a(1, &serde_json::to_string(&doc).unwrap()).is_none()
        );
    }
}

#[test]
fn preview_budget_counts_paragraph_breaks_and_utf16_without_enabling_writes() {
    let last = "🐦".repeat((MAX_PREVIEW_UTF16 - 4) / 2);
    let text = format!("AB\n\n{last}");
    let mut doc = document(&text);
    doc.document["content"][1] = json!({"type":"paragraph"});
    let mut catalog =
        SyntheticCatalog::restore_managed_a(1, &serde_json::to_string(&doc).unwrap()).unwrap();
    let mut scene = catalog.bind(Scene::A, Mode::Edit);
    assert_eq!(
        to_value(scene.handle_managed(READ)).unwrap(),
        json!({"preview_text":text,"revision":1})
    );
    assert_eq!(
        to_value(scene.handle_managed(&save(1, "Rabe\n43"))).unwrap(),
        json!({"error":"DENIED"})
    );
    assert!(valid_preview_text(&text));
    let over = format!("{last}X");
    doc.document["content"][2]["content"][0]["text"] = json!(over);
    assert!(!valid_preview_text(&format!("AB\n\n{over}")));
    assert!(
        SyntheticCatalog::restore_managed_a(1, &serde_json::to_string(&doc).unwrap()).is_none()
    );
}

#[test]
fn revision_limit_is_readable_and_cannot_roll_over() {
    let mut catalog = SyntheticCatalog::default();
    catalog.a.revision = MAX_REVISION - 1;
    let mut binding = catalog.bind(Scene::A, Mode::Edit);
    assert_eq!(
        reply(&mut binding, &save(MAX_REVISION - 1, "Rabe\n43")),
        json!({"revision":MAX_REVISION})
    );
    assert_eq!(
        reply(&mut binding, &save(MAX_REVISION, "Rabe\n42")),
        json!({"error":"LIMIT"})
    );
    assert_eq!(
        reply(&mut binding, READ),
        json!({"text":"Rabe\n43","revision":MAX_REVISION})
    );
}
