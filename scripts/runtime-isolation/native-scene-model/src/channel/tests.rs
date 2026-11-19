use super::*;
use crate::{Mode, Scene, SyntheticCatalog};
const SAVE: &[u8] = br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#;
const READ: &[u8] = br#"{"command":"c4_document_read_v1"}"#;

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
