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
