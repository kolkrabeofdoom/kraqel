//! In-memory simulation of two protocol endpoints. No IPC, native credentials or timers.
use super::{BoundScene, MAX_FRAME_BYTES};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

const PREPARE_WITH_REFRESH: &[u8] = b"prepare-reviewed-with-refresh-v1";
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReviewedWrite {
    Unavailable,
    Once,
    Repeat,
}
const PREPARE_COMPARISON: &[u8] = b"prepare-compared-snapshot-v1";
pub const COMPARE_READ: &[u8] = br#"{"command":"c4_document_read_v1"}"#;
const DISCARD: &[u8] = br#"{"command":"discard_draft_v1"}"#;
const DISCARDED: &[u8] = br#"{"discarded":true}"#;
const HEADER: usize = 50;
const TAG: usize = 32;
type Key = [u8; 32];

#[derive(Debug, PartialEq, Eq)]
pub struct Closed;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Kind {
    Hello = 1,
    Ready,
    Request,
    Response,
}

#[derive(Clone, Copy)]
pub enum Handshake {
    Hello,
    Ready,
}
impl Handshake {
    fn kind(self) -> Kind {
        match self {
            Self::Hello => Kind::Hello,
            Self::Ready => Kind::Ready,
        }
    }
}
pub const HANDSHAKE_BYTES: usize = HEADER + TAG;

pub struct Binding {
    id: [u8; 32],
    request: Zeroizing<Key>,
    response: Zeroizing<Key>,
}

#[derive(Clone, Copy)]
enum Expected {
    Read,
    Denied,
    Saved(u32),
    Conflict,
}

#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneView {
    pub text: String,
    pub revision: u32,
    #[serde(default)]
    pub bold_second: bool,
}

impl SceneView {
    pub fn reviewed_receipt(&self) -> [u8; 36] {
        use sha2::Digest;
        let mut hash = Sha256::new();
        hash.update(b"KRAQEL-reviewed-scene-A-v1");
        hash.update(self.revision.to_le_bytes());
        hash.update([u8::from(self.bold_second)]);
        hash.update(self.text.as_bytes());
        let mut frame = [0; 36];
        frame[..4].copy_from_slice(&self.revision.to_le_bytes());
        frame[4..].copy_from_slice(&hash.finalize());
        frame
    }
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextPreview {
    pub preview_text: String,
    pub revision: u32,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum OpenedScene {
    Editable(SceneView),
    Preview(TextPreview),
}

pub enum EditorEvent<'a> {
    Loaded(&'a SceneView),
    Preview(&'a TextPreview),
    RevisionConflict(Option<&'a SceneView>),
    RefreshedComparison(&'a SceneView),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServiceEnd {
    Complete,
    Discarded,
    PrepareComparison,
    PrepareReviewed([u8; 36]),
}

enum SaveReply {
    Saved,
    Conflict,
}

impl Binding {
    /// Bootstrap material supplied by the trusted launcher, or by a synthetic model driver.
    pub fn from_keys(id: [u8; 32], request: Key, response: Key) -> Result<Self, Closed> {
        if request == response {
            return Err(Closed);
        }
        Ok(Self {
            id,
            request: Zeroizing::new(request),
            response: Zeroizing::new(response),
        })
    }
    pub fn handshake_frame(&self, message: Handshake) -> Result<Vec<u8>, Closed> {
        self.encode(message.kind(), 0, &[])
    }
    pub fn verify_handshake(&self, message: Handshake, bytes: &[u8]) -> Result<(), Closed> {
        self.decode(bytes, message.kind(), 0).map(|_| ())
    }

    /// Single-use guest exchange after READY. Each endpoint consumes its binding.
    pub fn read_scene(
        self,
        pipes: (&mut impl std::io::Read, &mut impl std::io::Write),
    ) -> Result<String, Closed> {
        self.read_at(1, pipes.0, pipes.1, 0, "Rabe\n42")
    }

    fn exchange(
        &self,
        sequence: u64,
        command: &[u8],
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<Vec<u8>, Closed> {
        writer
            .write_all(&self.encode(Kind::Request, sequence, command)?)
            .map_err(|_| Closed)?;
        let frame = read_frame(reader)?;
        Ok(self.decode(&frame, Kind::Response, sequence)?.to_vec())
    }

    fn read_at(
        &self,
        sequence: u64,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        revision: u32,
        text: &str,
    ) -> Result<String, Closed> {
        let view = self.read_view(sequence, reader, writer, revision, text)?;
        if view.bold_second {
            return Err(Closed);
        }
        Ok(view.text)
    }

    fn read_view(
        &self,
        sequence: u64,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        revision: u32,
        text: &str,
    ) -> Result<SceneView, Closed> {
        let answer = self.read_bound_view(sequence, reader, writer, revision)?;
        if answer.text != text {
            return Err(Closed);
        }
        Ok(answer)
    }
    pub fn read_bound_view(
        &self,
        sequence: u64,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        revision: u32,
    ) -> Result<SceneView, Closed> {
        match self.open_bound_scene(sequence, reader, writer, revision)? {
            OpenedScene::Editable(view) => Ok(view),
            OpenedScene::Preview(_) => Err(Closed),
        }
    }

    pub fn open_bound_scene(
        &self,
        sequence: u64,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        revision: u32,
    ) -> Result<OpenedScene, Closed> {
        let body = self.exchange(
            sequence,
            br#"{"command":"c4_document_read_v1"}"#,
            reader,
            writer,
        )?;
        let answer: OpenedScene = serde_json::from_slice(&body).map_err(|_| Closed)?;
        let valid = match &answer {
            OpenedScene::Editable(view) => {
                view.revision == revision && super::valid_paragraphs(&view.text)
            }
            OpenedScene::Preview(view) => {
                revision > 0
                    && view.revision == revision
                    && super::valid_preview_text(&view.preview_text)
            }
        };
        if !valid {
            return Err(Closed);
        }
        Ok(answer)
    }

    pub fn serve_scene_read(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        let mut catalog = super::SyntheticCatalog::default();
        let mut scene = catalog.bind(super::Scene::A, super::Mode::Read);
        self.reply_at(1, &mut scene, reader, writer, Expected::Read)
    }

    pub fn check_denied_save(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<String, Closed> {
        let before = self.read_at(1, reader, writer, 0, "Rabe\n42")?;
        let denied = self.exchange(
            2,
            br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#,
            reader,
            writer,
        )?;
        Self::expect_error(&denied, "DENIED")?;
        let after = self.read_at(3, reader, writer, 0, "Rabe\n42")?;
        if before != after {
            return Err(Closed);
        }
        Ok(after)
    }

    fn expect_error(body: &[u8], expected: &str) -> Result<(), Closed> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ErrorReply {
            error: String,
        }
        if serde_json::from_slice::<ErrorReply>(body)
            .map_err(|_| Closed)?
            .error
            != expected
        {
            return Err(Closed);
        }
        Ok(())
    }

    pub fn check_edit_conflict(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<String, Closed> {
        self.read_at(1, reader, writer, 0, "Rabe\n42")?;
        self.save_first(reader, writer)?;
        let stale = self.exchange(
            3,
            br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n42"}"#,
            reader,
            writer,
        )?;
        Self::expect_error(&stale, "CONFLICT")?;
        self.read_at(4, reader, writer, 1, "Rabe\n43")
    }

    fn save_first(
        &self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        match self.save_text(reader, writer, 0, "Rabe\n43", None)? {
            SaveReply::Saved => Ok(()),
            SaveReply::Conflict => Err(Closed),
        }
    }
    fn save_text(
        &self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        revision: u32,
        text: &str,
        bold: Option<bool>,
    ) -> Result<SaveReply, Closed> {
        let body = match bold {
            Some(mark) => {
                serde_json::json!({"command":"c4_document_save_v2","expectedRevision":revision,"text":text,"boldSecond":mark})
            }
            None => {
                serde_json::json!({"command":"c4_document_save_v1","expectedRevision":revision,"text":text})
            }
        };
        let body = serde_json::to_vec(&body).map_err(|_| Closed)?;
        let saved = self.exchange(2, &body, reader, writer)?;
        if Self::expect_error(&saved, "CONFLICT").is_ok() {
            return Ok(SaveReply::Conflict);
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            revision: u32,
        }
        if serde_json::from_slice::<Saved>(&saved)
            .map_err(|_| Closed)?
            .revision
            != revision.checked_add(1).ok_or(Closed)?
        {
            return Err(Closed);
        }
        Ok(SaveReply::Saved)
    }
    /// Send only the header and first body byte of an otherwise valid second request.
    pub fn partial_save(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        self.read_at(1, reader, writer, 0, "Rabe\n42")?;
        let frame = self.encode(
            Kind::Request,
            2,
            br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#,
        )?;
        writer
            .write_all(&frame[..PARTIAL_SAVE_BYTES])
            .map_err(|_| Closed)
    }
    pub fn send_pending_save(
        &self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        self.read_at(1, reader, writer, 0, "Rabe\n42")?;
        let body = br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#;
        writer
            .write_all(&self.encode(Kind::Request, 2, body)?)
            .map_err(|_| Closed)
    }
    /// One authenticated save, with a distinct outcome only for disconnect BEFORE any response byte.
    pub fn unacknowledged_save(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        self.send_pending_save(reader, writer)?;
        match reader.read(&mut [0u8; 1]) {
            Ok(0) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(Closed),
        }
    }
    pub fn durable_window(
        self,
        reopening: bool,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<String, Closed> {
        let view = self.durable_view(reopening, reader, writer)?;
        if view.text != "Rabe\n43" {
            return Err(Closed);
        }
        Ok(view.text)
    }
    pub fn durable_view(
        self,
        reopening: bool,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<SceneView, Closed> {
        if reopening {
            return self.read_bound_view(1, reader, writer, 1);
        }
        self.read_view(1, reader, writer, 0, "Rabe\n42")?;
        self.save_first(reader, writer)?;
        self.read_view(3, reader, writer, 1, "Rabe\n43")
    }
    // Each edit turn already has unique launcher keys. This one-shot child grant
    // has its own identifier, direction keys, sequence space and read-only grammar.
    fn comparison_binding(&self, basis: u32) -> Result<Self, Closed> {
        self.conflict_binding(basis, b"KRAQEL-conflict-A/read-v1")
    }
    fn conflict_binding(&self, basis: u32, domain: &[u8]) -> Result<Self, Closed> {
        let derive = |kind, purpose: &[u8]| {
            let mut mac = self.mac(kind);
            mac.update(domain);
            mac.update(&self.id);
            mac.update(&basis.to_le_bytes());
            mac.update(purpose);
            mac.finalize().into_bytes().into()
        };
        Self::from_keys(
            derive(Kind::Request, b"id"),
            derive(Kind::Request, b"request"),
            derive(Kind::Response, b"response"),
        )
    }

    fn read_comparison(
        &self,
        basis: u32,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<SceneView, Closed> {
        let grant = self.comparison_binding(basis)?;
        let body = grant.exchange(1, COMPARE_READ, reader, writer)?;
        let current: SceneView = serde_json::from_slice(&body).map_err(|_| Closed)?;
        if current.revision <= basis || !super::valid_paragraphs(&current.text) {
            return Err(Closed);
        }
        Ok(current)
    }

    pub fn keyboard_view(
        self,
        revision: u32,
        compare: bool,
        refresh: bool,
        reviewed_write: ReviewedWrite,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        mut edit: impl FnMut(EditorEvent<'_>) -> Result<Option<(String, bool)>, Closed>,
    ) -> Result<Option<SceneView>, Closed> {
        let selection = match self.open_bound_scene(1, reader, writer, revision)? {
            OpenedScene::Editable(original) => {
                if revision == 0 && (original.text != "Rabe\n42" || !original.bold_second) {
                    return Err(Closed);
                }
                edit(EditorEvent::Loaded(&original))?.map(|choice| (original, choice))
            }
            OpenedScene::Preview(view) => {
                if edit(EditorEvent::Preview(&view))?.is_some() {
                    return Err(Closed);
                }
                None
            }
        };
        let Some((original, (text, bold))) = selection else {
            let ack = self.exchange(2, DISCARD, reader, writer)?;
            return if ack == DISCARDED {
                Ok(None)
            } else {
                Err(Closed)
            };
        };
        if !super::valid_paragraphs(&text) {
            return Err(Closed);
        }
        if let SaveReply::Conflict =
            self.save_text(reader, writer, original.revision, &text, Some(bold))?
        {
            let comparison = compare
                .then(|| self.read_comparison(revision, reader, writer))
                .transpose();
            let selected = edit(EditorEvent::RevisionConflict(
                comparison.as_ref().ok().and_then(Option::as_ref),
            ))?;
            let compared = comparison?;
            if selected.is_some() {
                let view = compared.ok_or(Closed)?;
                if Some(view.revision) != revision.checked_add(1) {
                    return Err(Closed);
                }
                let prepare =
                    self.conflict_binding(view.revision, b"KRAQEL-conflict-A/prepare-v1")?;
                let body = prepare.exchange(1, PREPARE_COMPARISON, reader, writer)?;
                if Self::expect_error(&body, "CONFLICT").is_ok() {
                    let mut basis = view.revision;
                    let mut can_refresh = refresh;
                    loop {
                        let updated = can_refresh
                            .then(|| self.read_comparison(basis, reader, writer))
                            .transpose();
                        let event = match updated.as_ref() {
                            Ok(Some(current)) => EditorEvent::RefreshedComparison(current),
                            _ => EditorEvent::RevisionConflict(None),
                        };
                        let approved = edit(event)?.is_some();
                        let current = updated?;
                        if !approved {
                            break;
                        }
                        let current = current
                            .filter(|_| reviewed_write != ReviewedWrite::Unavailable)
                            .ok_or(Closed)?;
                        let grant = self
                            .conflict_binding(current.revision, b"KRAQEL-reviewed-A/prepare-v1")?;
                        can_refresh = reviewed_write == ReviewedWrite::Repeat;
                        let command = if can_refresh {
                            PREPARE_WITH_REFRESH
                        } else {
                            PREPARE_COMPARISON
                        };
                        let response = grant.exchange(1, command, reader, writer)?;
                        if Self::expect_error(&response, "CONFLICT").is_ok() {
                            basis = current.revision;
                            continue;
                        }
                        let verified: SceneView =
                            serde_json::from_slice(&response).map_err(|_| Closed)?;
                        if verified.reviewed_receipt() != current.reviewed_receipt() {
                            return Err(Closed);
                        }
                        return Ok(Some(current));
                    }
                } else {
                    let confirmed: SceneView = serde_json::from_slice(&body).map_err(|_| Closed)?;
                    if confirmed.text != view.text
                        || confirmed.bold_second != view.bold_second
                        || confirmed.revision != view.revision
                    {
                        return Err(Closed);
                    }
                    return Ok(Some(view));
                }
            }
            let ack = self.exchange(3, DISCARD, reader, writer)?;
            return if ack == DISCARDED {
                Ok(None)
            } else {
                Err(Closed)
            };
        }
        let saved = self.read_view(
            3,
            reader,
            writer,
            revision.checked_add(1).ok_or(Closed)?,
            &text,
        )?;
        if saved.bold_second != bold {
            return Err(Closed);
        }
        Ok(Some(saved))
    }
    /// Storage dispatch returns a Saved reply only after its disk transaction commits.
    pub fn durable_service(
        self,
        reopening: bool,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        dispatch: impl FnMut(&[u8]) -> Result<super::Reply, Closed>,
    ) -> Result<(), Closed> {
        self.revision_service(
            reopening,
            0,
            false,
            false,
            ReviewedWrite::Unavailable,
            reader,
            writer,
            dispatch,
        )
        .map(|_| ())
    }
    pub fn revision_service(
        self,
        reopening: bool,
        revision: u32,
        allow_discard: bool,
        allow_prepare: bool,
        reviewed_write: ReviewedWrite,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        mut dispatch: impl FnMut(&[u8]) -> Result<super::Reply, Closed>,
    ) -> Result<ServiceEnd, Closed> {
        let steps: &[Expected] = if reopening {
            &[Expected::Read]
        } else {
            &[
                Expected::Read,
                Expected::Saved(revision.checked_add(1).ok_or(Closed)?),
                Expected::Read,
            ]
        };
        for (index, expected) in steps.iter().enumerate() {
            let seq = index as u64 + 1;
            let frame = read_frame(reader)?;
            let body = self.decode(&frame, Kind::Request, seq)?;
            if allow_discard && !reopening && seq == 2 && body == DISCARD {
                writer
                    .write_all(&self.encode(Kind::Response, seq, DISCARDED)?)
                    .map_err(|_| Closed)?;
                return Ok(ServiceEnd::Discarded);
            }
            let answer = dispatch(body)?;
            if let super::Reply::Preview { .. } = &answer {
                if seq != 1
                    || body != br#"{"command":"c4_document_read_v1"}"#
                    || !(reopening || allow_discard)
                {
                    return Err(Closed);
                }
                let payload = serde_json::to_vec(&answer).map_err(|_| Closed)?;
                writer
                    .write_all(&self.encode(Kind::Response, seq, &payload)?)
                    .map_err(|_| Closed)?;
                if reopening {
                    return Ok(ServiceEnd::Complete);
                }
                let frame = read_frame(reader)?;
                if self.decode(&frame, Kind::Request, 2)? != DISCARD {
                    return Err(Closed);
                }
                writer
                    .write_all(&self.encode(Kind::Response, 2, DISCARDED)?)
                    .map_err(|_| Closed)?;
                return Ok(ServiceEnd::Discarded);
            }

            if allow_discard
                && seq == 2
                && matches!(
                    answer,
                    super::Reply::Failure {
                        error: super::Failure::Conflict
                    }
                )
            {
                self.write_reply(seq, answer, writer, Expected::Conflict)?;
                let frame = read_frame(reader)?;
                let grant = self.comparison_binding(revision)?;
                let final_frame = if grant.decode(&frame, Kind::Request, 1) == Ok(COMPARE_READ) {
                    let answer = dispatch(COMPARE_READ)?;
                    let compared = serde_json::to_vec(&answer).map_err(|_| Closed)?;
                    grant.write_reply(1, answer, writer, Expected::Read)?;
                    let next = read_frame(reader)?;
                    if allow_prepare {
                        let basis = revision.checked_add(1).ok_or(Closed)?;
                        let prepare =
                            self.conflict_binding(basis, b"KRAQEL-conflict-A/prepare-v1")?;
                        if prepare.decode(&next, Kind::Request, 1) == Ok(PREPARE_COMPARISON) {
                            let current = dispatch(COMPARE_READ)?;
                            if matches!(&current,super::Reply::Read{revision,..} if *revision==basis)
                                && serde_json::to_vec(&current).map_err(|_| Closed)? == compared
                            {
                                prepare.write_reply(1, current, writer, Expected::Read)?;
                                return Ok(ServiceEnd::PrepareComparison);
                            }
                            prepare.write_reply(
                                1,
                                super::Reply::Failure {
                                    error: super::Failure::Conflict,
                                },
                                writer,
                                Expected::Conflict,
                            )?;
                            return self.serve_reviewed_comparisons(
                                basis,
                                reviewed_write,
                                reader,
                                writer,
                                &mut dispatch,
                            );
                        } else {
                            next
                        }
                    } else {
                        next
                    }
                } else {
                    frame
                };
                if self.decode(&final_frame, Kind::Request, 3)? != DISCARD {
                    return Err(Closed);
                }
                writer
                    .write_all(&self.encode(Kind::Response, 3, DISCARDED)?)
                    .map_err(|_| Closed)?;
                return Ok(ServiceEnd::Discarded);
            }
            self.write_reply(seq, answer, writer, *expected)?;
        }
        Ok(ServiceEnd::Complete)
    }
    fn serve_reviewed_comparisons(
        &self,
        mut basis: u32,
        reviewed_write: ReviewedWrite,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        dispatch: &mut impl FnMut(&[u8]) -> Result<super::Reply, Closed>,
    ) -> Result<ServiceEnd, Closed> {
        let mut frame = read_frame(reader)?;
        loop {
            let refresh = self.comparison_binding(basis)?;
            if refresh.decode(&frame, Kind::Request, 1) != Ok(COMPARE_READ) {
                break;
            }
            let reply = dispatch(COMPARE_READ)?;
            let offered = serde_json::to_vec(&reply).map_err(|_| Closed)?;
            refresh.write_reply(1, reply, writer, Expected::Read)?;
            frame = read_frame(reader)?;
            let view: SceneView = serde_json::from_slice(&offered).map_err(|_| Closed)?;
            if reviewed_write == ReviewedWrite::Unavailable || view.revision <= basis {
                break;
            }
            let grant = self.conflict_binding(view.revision, b"KRAQEL-reviewed-A/prepare-v1")?;
            let repeat = match grant.decode(&frame, Kind::Request, 1) {
                Ok(PREPARE_WITH_REFRESH) if reviewed_write == ReviewedWrite::Repeat => true,
                Ok(PREPARE_COMPARISON) => false,
                _ => break,
            };
            let latest = dispatch(COMPARE_READ)?;
            if serde_json::to_vec(&latest).map_err(|_| Closed)? == offered {
                grant.write_reply(1, latest, writer, Expected::Read)?;
                return Ok(ServiceEnd::PrepareReviewed(view.reviewed_receipt()));
            }
            grant.write_reply(
                1,
                super::Reply::Failure {
                    error: super::Failure::Conflict,
                },
                writer,
                Expected::Conflict,
            )?;
            frame = read_frame(reader)?;
            if !repeat {
                break;
            }
            basis = view.revision;
        }
        if self.decode(&frame, Kind::Request, 3)? != DISCARD {
            return Err(Closed);
        }
        writer
            .write_all(&self.encode(Kind::Response, 3, DISCARDED)?)
            .map_err(|_| Closed)?;
        Ok(ServiceEnd::Discarded)
    }

    pub fn serve_edit_conflict(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        let mut catalog = super::SyntheticCatalog::default();
        let mut scene = catalog.bind(super::Scene::A, super::Mode::Edit);
        for (n, expected) in [
            (1, Expected::Read),
            (2, Expected::Saved(1)),
            (3, Expected::Conflict),
            (4, Expected::Read),
        ] {
            self.reply_at(n, &mut scene, reader, writer, expected)?;
        }
        Ok(())
    }

    pub fn serve_denied_save(
        self,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Closed> {
        let mut catalog = super::SyntheticCatalog::default();
        let mut scene = catalog.bind(super::Scene::A, super::Mode::Read);
        self.reply_at(1, &mut scene, reader, writer, Expected::Read)?;
        self.reply_at(2, &mut scene, reader, writer, Expected::Denied)?;
        self.reply_at(3, &mut scene, reader, writer, Expected::Read)
    }

    fn reply_at(
        &self,
        sequence: u64,
        scene: &mut BoundScene,
        reader: &mut impl std::io::Read,
        writer: &mut impl std::io::Write,
        expected: Expected,
    ) -> Result<(), Closed> {
        let frame = read_frame(reader)?;
        let body = self.decode(&frame, Kind::Request, sequence)?;
        let answer = scene.handle(body);
        self.write_reply(sequence, answer, writer, expected)
    }
    fn write_reply(
        &self,
        sequence: u64,
        answer: super::Reply,
        writer: &mut impl std::io::Write,
        expected: Expected,
    ) -> Result<(), Closed> {
        let saved = matches!((&answer,expected),(super::Reply::Saved{revision},Expected::Saved(n)) if *revision==n);
        if !saved
            && !matches!(
                (&answer, expected),
                (super::Reply::Read { .. }, Expected::Read)
                    | (
                        super::Reply::Failure {
                            error: super::Failure::Conflict
                        },
                        Expected::Conflict
                    )
                    | (
                        super::Reply::Failure {
                            error: super::Failure::Denied
                        },
                        Expected::Denied
                    )
            )
        {
            return Err(Closed);
        }
        let payload = serde_json::to_vec(&answer).map_err(|_| Closed)?;
        writer
            .write_all(&self.encode(Kind::Response, sequence, &payload)?)
            .map_err(|_| Closed)
    }

    fn mac(&self, kind: Kind) -> Hmac<Sha256> {
        let key = match kind {
            Kind::Hello | Kind::Request => &self.request,
            Kind::Ready | Kind::Response => &self.response,
        };
        Hmac::new_from_slice(key.as_slice()).expect("fixed-size HMAC key")
    }

    fn encode(&self, kind: Kind, number: u64, body: &[u8]) -> Result<Vec<u8>, Closed> {
        if body.len() > MAX_FRAME_BYTES {
            return Err(Closed);
        }
        let mut frame = Vec::with_capacity(HEADER + body.len() + TAG);
        frame.extend_from_slice(b"KQSC");
        frame.extend_from_slice(&[1, kind as u8]);
        frame.extend_from_slice(&self.id);
        frame.extend_from_slice(&number.to_le_bytes());
        frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
        frame.extend_from_slice(body);
        let mut mac = self.mac(kind);
        mac.update(&frame);
        frame.extend_from_slice(&mac.finalize().into_bytes());
        Ok(frame)
    }

    fn decode<'a>(&self, frame: &'a [u8], kind: Kind, number: u64) -> Result<&'a [u8], Closed> {
        let header = frame.get(..HEADER).ok_or(Closed)?;
        let length = u32::from_le_bytes(header[46..50].try_into().unwrap()) as usize;
        if length > MAX_FRAME_BYTES
            || frame.len() != HEADER + length + TAG
            || &header[..4] != b"KQSC"
            || header[4..6] != [1, kind as u8]
            || header[6..38] != self.id
            || header[38..46] != number.to_le_bytes()
            || (matches!(kind, Kind::Hello | Kind::Ready) && length != 0)
        {
            return Err(Closed);
        }
        let (signed, tag) = frame.split_at(HEADER + length);
        let mut mac = self.mac(kind);
        mac.update(signed);
        mac.verify_slice(tag).map_err(|_| Closed)?;
        Ok(&signed[HEADER..])
    }
}

pub const PARTIAL_SAVE_BYTES: usize = HEADER + 1;

fn read_frame(reader: &mut impl std::io::Read) -> Result<Vec<u8>, Closed> {
    let mut header = [0; HEADER];
    reader.read_exact(&mut header).map_err(|_| Closed)?;
    let len = u32::from_le_bytes(header[46..50].try_into().unwrap()) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(Closed);
    }
    let mut frame = vec![0; HEADER + len + TAG];
    frame[..HEADER].copy_from_slice(&header);
    reader
        .read_exact(&mut frame[HEADER..])
        .map_err(|_| Closed)?;
    Ok(frame)
}

#[derive(Clone, Copy)]
enum Ui {
    New,
    AwaitReady,
    Idle(u64),
    Waiting(u64),
    Ended,
}

/// The endpoints have independent sequencing. Losing either endpoint closes the pair.
pub struct Channel<'a> {
    binding: Option<Binding>,
    ui: Ui,
    // 0 expects HELLO; None means the service exhausted its sequence space.
    service_next: Option<u64>,
    scene: BoundScene<'a>,
}

impl<'a> Channel<'a> {
    /// The model driver supplies a fresh synthetic binding per simulated launch.
    pub fn new(
        scene: BoundScene<'a>,
        id: [u8; 32],
        request: Key,
        response: Key,
    ) -> Result<Self, Closed> {
        if request == response {
            return Err(Closed);
        }
        Ok(Self {
            binding: Some(Binding {
                id,
                request: Zeroizing::new(request),
                response: Zeroizing::new(response),
            }),
            ui: Ui::New,
            service_next: Some(0),
            scene,
        })
    }

    pub fn close(&mut self) {
        self.binding = None;
    }

    // Take the binding before validating: every error path drops both directional keys.
    fn step<T>(
        &mut self,
        action: impl FnOnce(&mut Self, &Binding) -> Result<T, Closed>,
    ) -> Result<T, Closed> {
        let binding = self.binding.take().ok_or(Closed)?;
        let value = action(self, &binding)?;
        if !matches!(self.ui, Ui::Ended) {
            self.binding = Some(binding);
        }
        Ok(value)
    }

    pub fn hello(&mut self) -> Result<Vec<u8>, Closed> {
        self.step(|s, b| {
            if !matches!(s.ui, Ui::New) {
                return Err(Closed);
            }
            s.ui = Ui::AwaitReady;
            b.encode(Kind::Hello, 0, &[])
        })
    }

    pub fn request(&mut self, body: &[u8]) -> Result<Vec<u8>, Closed> {
        self.step(|s, b| {
            let Ui::Idle(n) = s.ui else {
                return Err(Closed);
            };
            s.ui = Ui::Waiting(n);
            b.encode(Kind::Request, n, body)
        })
    }

    pub fn service_receive(&mut self, frame: &[u8]) -> Result<Vec<u8>, Closed> {
        self.step(|s, b| {
            let n = s.service_next.ok_or(Closed)?;
            if n == 0 {
                b.decode(frame, Kind::Hello, 0)?;
                s.service_next = Some(1);
                return b.encode(Kind::Ready, 0, &[]);
            }
            let body = b.decode(frame, Kind::Request, n)?;
            s.service_next = n.checked_add(1);
            let reply = serde_json::to_vec(&s.scene.handle(body)).map_err(|_| Closed)?;
            b.encode(Kind::Response, n, &reply)
        })
    }

    /// READY yields an empty body; RESPONSE yields only its authenticated payload.
    pub fn ui_receive<'f>(&mut self, frame: &'f [u8]) -> Result<&'f [u8], Closed> {
        self.step(|s, b| match s.ui {
            Ui::AwaitReady => {
                let empty = b.decode(frame, Kind::Ready, 0)?;
                s.ui = Ui::Idle(1);
                Ok(empty)
            }
            Ui::Waiting(n) => {
                let body = b.decode(frame, Kind::Response, n)?;
                s.ui = n.checked_add(1).map_or(Ui::Ended, Ui::Idle);
                Ok(body)
            }
            _ => Err(Closed),
        })
    }
}

#[cfg(test)]
mod tests;
