use super::*;
use crate::{Mode, Scene, SyntheticCatalog};
const SAVE: &[u8] = br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#;
const READ: &[u8] = br#"{"command":"c4_document_read_v1"}"#;

#[test]
fn conflict_closes_the_service_to_further_writes() {
    let binding = Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    let stale = br#"{"command":"c4_document_save_v1","expectedRevision":1,"text":"Rabe\n43"}"#;
    let mut requests = Vec::new();
    for (seq, body) in [(1, READ), (2, stale.as_slice()), (3, SAVE)] {
        requests.extend(binding.encode(Kind::Request, seq, body).unwrap());
    }
    let mut catalog = SyntheticCatalog::default();
    let mut scene = catalog.bind(Scene::A, Mode::Edit);
    let mut dispatched = 0;
    let mut replies = Vec::new();
    assert_eq!(
        binding.revision_service(
            false,
            0,
            true,
            &mut requests.as_slice(),
            &mut replies,
            |body| {
                dispatched += 1;
                Ok(scene.handle(body))
            }
        ),
        Err(Closed)
    );
    assert_eq!(dispatched, 2);
    let mut frames = replies.as_slice();
    let _ = read_frame(&mut frames).unwrap();
    let conflict = read_frame(&mut frames).unwrap();
    let verifier = Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    Binding::expect_error(
        verifier.decode(&conflict, Kind::Response, 2).unwrap(),
        "CONFLICT",
    )
    .unwrap();
    assert!(frames.is_empty());
}

#[test]
fn interrupted_save_requires_a_verified_initial_read_and_zero_response_bytes() {
    let binding = || Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    let first = binding()
        .encode(Kind::Response, 1, br#"{"text":"Rabe\n42","revision":0}"#)
        .unwrap();
    assert!(
        binding()
            .unacknowledged_save(&mut first.as_slice(), &mut Vec::new())
            .is_ok()
    );
    let mut partial = first;
    partial.push(b'K');
    assert_eq!(
        binding().unacknowledged_save(&mut partial.as_slice(), &mut Vec::new()),
        Err(Closed)
    );
    assert_eq!(
        binding().unacknowledged_save(&mut &[][..], &mut Vec::new()),
        Err(Closed)
    );
}

#[test]
fn edit_probe_checks_saved_revision_conflict_and_final_content() {
    let binding = || Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    let initial = br#"{"text":"Rabe\n42","revision":0}"#.as_slice();
    let saved = br#"{"revision":1}"#.as_slice();
    let conflict = br#"{"error":"CONFLICT"}"#.as_slice();
    let updated = br#"{"text":"Rabe\n43","revision":1}"#.as_slice();
    for (ack, retry, last, ok) in [
        (saved, conflict, updated, true),
        (br#"{"revision":2}"#, conflict, updated, false),
        (saved, br#"{"error":"DENIED"}"#, updated, false),
        (
            saved,
            conflict,
            br#"{"text":"Rabe\n42","revision":1}"#,
            false,
        ),
        (
            saved,
            conflict,
            br#"{"text":"Rabe\n43","revision":2}"#,
            false,
        ),
    ] {
        let mut answers = Vec::new();
        for (n, body) in [(1, initial), (2, ack), (3, retry), (4, last)] {
            answers.extend(binding().encode(Kind::Response, n, body).unwrap());
        }
        let mut requests = Vec::new();
        assert_eq!(
            binding()
                .check_edit_conflict(&mut answers.as_slice(), &mut requests)
                .is_ok(),
            ok
        );
        if ok {
            let mut actual = Vec::new();
            binding()
                .serve_edit_conflict(&mut requests.as_slice(), &mut actual)
                .unwrap();
            assert_eq!(actual, answers);
        }
    }
}

#[test]
fn write_probe_requires_denied_and_checks_the_final_revision() {
    let binding = || Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    let original = br#"{"text":"Rabe\n42","revision":0}"#.as_slice();
    for (middle, last, success) in [
        (br#"{"error":"DENIED"}"#.as_slice(), original, true),
        (br#"{"error":"CONFLICT"}"#, original, false),
        (br#"{"revision":1}"#, original, false),
        (
            br#"{"error":"DENIED"}"#,
            br#"{"text":"Rabe\n42","revision":1}"#,
            false,
        ),
        (
            br#"{"error":"DENIED"}"#,
            br#"{"text":"Rabe\n43","revision":0}"#,
            false,
        ),
    ] {
        let mut answers = Vec::new();
        for (n, body) in [(1, original), (2, middle), (3, last)] {
            answers.extend(binding().encode(Kind::Response, n, body).unwrap());
        }
        let mut requests = Vec::new();
        let result = binding().check_denied_save(&mut answers.as_slice(), &mut requests);
        assert_eq!(result.is_ok(), success);
        if success {
            let mut actual = Vec::new();
            binding()
                .serve_denied_save(&mut requests.as_slice(), &mut actual)
                .unwrap();
            assert_eq!(actual, answers);
        }
    }
}

#[test]
fn bound_guest_read_rejects_other_operations_and_invalid_frames_without_reply() {
    let binding = || Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    let request = |body: &[u8]| binding().encode(Kind::Request, 1, body).unwrap();
    let mut bad_tag = request(READ);
    *bad_tag.last_mut().unwrap() ^= 1;
    let mut oversized = request(READ);
    oversized[46..50].copy_from_slice(&u32::MAX.to_le_bytes());
    for frame in [
        request(SAVE),
        request(br#"{"command":"c4_document_read_v1","scene":"B"}"#),
        bad_tag,
        oversized,
        request(READ)[..HEADER + 1].to_vec(),
    ] {
        let mut response = Vec::new();
        assert_eq!(
            binding().serve_scene_read(&mut frame.as_slice(), &mut response),
            Err(Closed)
        );
        assert!(response.is_empty());
    }
}

#[test]
fn bound_guest_read_checks_the_authenticated_answer_before_display() {
    let binding = || Binding::from_keys([1; 32], [2; 32], [3; 32]).unwrap();
    for body in [
        br#"{"text":"Mond\n7","revision":0}"#.as_slice(),
        br#"{"text":"Rabe\n42","revision":1}"#,
        br#"{"text":"Rabe\n42","revision":0,"extra":1}"#,
    ] {
        let frame = binding().encode(Kind::Response, 1, body).unwrap();
        assert_eq!(
            binding().read_scene((&mut frame.as_slice(), &mut Vec::new())),
            Err(Closed)
        );
    }
}

fn pair(catalog: &mut SyntheticCatalog) -> Channel<'_> {
    Channel::new(
        catalog.bind(Scene::A, Mode::Edit),
        std::array::from_fn(|i| i as u8),
        std::array::from_fn(|i| 32 + i as u8),
        [93; 32],
    )
    .unwrap()
}
fn handshake(c: &mut Channel) {
    let hello = c.hello().unwrap();
    let ready = c.service_receive(&hello).unwrap();
    assert_eq!(c.ui_receive(&ready).unwrap(), b"");
}
fn exchange(c: &mut Channel, body: &[u8]) -> (Vec<u8>, Vec<u8>, serde_json::Value) {
    let request = c.request(body).unwrap();
    let reply = c.service_receive(&request).unwrap();
    let value = serde_json::from_slice(c.ui_receive(&reply).unwrap()).unwrap();
    (request, reply, value)
}

#[test]
fn hello_matches_independent_vector() {
    let mut store = SyntheticCatalog::default();
    let hello = pair(&mut store).hello().unwrap();
    assert_eq!(hello.len(), 82);
    let tag = hello[50..]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        tag,
        "fa777fcddf15349fc2f06e72cada2c0366e7588656f7b05bd167ac5ac2d4fbe9"
    );
}
#[test]
fn save_replay_never_reaches_scene() {
    let mut store = SyntheticCatalog::default();
    let mut c = pair(&mut store);
    handshake(&mut c);
    assert_eq!(
        exchange(&mut c, READ).2,
        serde_json::json!({"revision":0,"text":"Rabe\n42"})
    );
    assert_eq!(
        exchange(&mut c, b"bad").2,
        serde_json::json!({"error":"INVALID"})
    );
    let (request, _, value) = exchange(&mut c, SAVE);
    assert_eq!(value, serde_json::json!({"revision":1}));
    assert_eq!(c.service_receive(&request), Err(Closed));
    assert_eq!(c.request(READ), Err(Closed));
    assert_eq!(store.a.revision, 1);
    assert_eq!(store.b.revision, 0);
}
#[test]
fn invalid_service_frames_close_both_ends_before_dispatch() {
    for case in [
        "gap",
        "reflected",
        "wrong key",
        "tag",
        "length",
        "truncated",
        "trailing",
        "old pair",
        "version",
        "second hello",
    ] {
        let mut store = SyntheticCatalog::default();
        let mut c = pair(&mut store);
        handshake(&mut c);
        let mut request = c.request(SAVE).unwrap();
        let b = c.binding.as_ref().unwrap();
        match case {
            "gap" => request = b.encode(Kind::Request, 2, SAVE).unwrap(),
            "reflected" => request = b.encode(Kind::Response, 1, SAVE).unwrap(),
            "wrong key" => {
                let mut mac = b.mac(Kind::Response);
                let end = request.len() - TAG;
                mac.update(&request[..end]);
                request[end..].copy_from_slice(&mac.finalize().into_bytes());
            }
            "tag" => *request.last_mut().unwrap() ^= 1,
            "length" => request[46..50].copy_from_slice(&u32::MAX.to_le_bytes()),
            "truncated" => request.truncate(HEADER - 1),
            "trailing" => request.push(0),
            "old pair" => {
                let mut old_store = SyntheticCatalog::default();
                let mut old = Channel::new(
                    old_store.bind(Scene::A, Mode::Edit),
                    [8; 32],
                    [9; 32],
                    [10; 32],
                )
                .unwrap();
                handshake(&mut old);
                request = old.request(SAVE).unwrap();
                old.close();
            }
            "version" => request[4] = 2,
            "second hello" => request = b.encode(Kind::Hello, 0, &[]).unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(c.service_receive(&request), Err(Closed), "{case}");
        assert_eq!(c.ui_receive(&[]), Err(Closed));
        assert_eq!(store.a.revision, 0);
    }
}
#[test]
fn stale_response_cannot_complete_later_request() {
    let mut store = SyntheticCatalog::default();
    let mut c = pair(&mut store);
    handshake(&mut c);
    let (_, reply, _) = exchange(&mut c, READ);
    let pending = c.request(SAVE).unwrap();
    assert_eq!(c.ui_receive(&reply), Err(Closed));
    assert_eq!(c.service_receive(&pending), Err(Closed));
    assert_eq!(store.a.revision, 0);
}
#[test]
fn only_one_request_and_no_sequence_wrap() {
    let mut store = SyntheticCatalog::default();
    let mut c = pair(&mut store);
    handshake(&mut c);
    let pending = c.request(READ).unwrap();
    assert_eq!(c.request(READ), Err(Closed));
    assert_eq!(c.service_receive(&pending), Err(Closed));
    let mut fresh = pair(&mut store);
    handshake(&mut fresh);
    fresh.service_next = Some(u64::MAX);
    fresh.ui = Ui::Idle(u64::MAX);
    assert_eq!(exchange(&mut fresh, READ).2["revision"], 0);
    assert_eq!(fresh.hello(), Err(Closed));
}
