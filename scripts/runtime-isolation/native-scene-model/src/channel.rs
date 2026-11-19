//! In-memory simulation of two protocol endpoints. No IPC, native credentials or timers.
use super::{BoundScene, MAX_FRAME_BYTES};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::Zeroizing;

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
        let body = self.exchange(
            sequence,
            br#"{"command":"c4_document_read_v1"}"#,
            reader,
            writer,
        )?;
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Answer {
            text: String,
            revision: u32,
        }
        let answer: Answer = serde_json::from_slice(&body).map_err(|_| Closed)?;
        if answer.revision != revision || answer.text != text {
            return Err(Closed);
        }
        Ok(answer.text)
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
        let saved = self.exchange(
            2,
            br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n43"}"#,
            reader,
            writer,
        )?;
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            revision: u32,
        }
        if serde_json::from_slice::<Saved>(&saved)
            .map_err(|_| Closed)?
            .revision
            != 1
        {
            return Err(Closed);
        }
        let stale = self.exchange(
            3,
            br#"{"command":"c4_document_save_v1","expectedRevision":0,"text":"Rabe\n42"}"#,
            reader,
            writer,
        )?;
        Self::expect_error(&stale, "CONFLICT")?;
        self.read_at(4, reader, writer, 1, "Rabe\n43")
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
            (2, Expected::Saved),
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
        if !matches!(
            (&answer, expected),
            (super::Reply::Read { .. }, Expected::Read)
                | (super::Reply::Saved { revision: 1 }, Expected::Saved)
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
        ) {
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
