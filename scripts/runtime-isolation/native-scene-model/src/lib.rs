//! BP4a research model. The caller supplies a simulated binding, never a native grant.
#![forbid(unsafe_code)]

use kraqel_domain::EditorDocument;
use serde::{Deserialize, Serialize};
use serde_json::json;

const MAX_FRAME_BYTES: usize = 80 * 1024;
const MAX_TEXT_BYTES: usize = 64 * 1024;
const MAX_REVISION: u32 = i32::MAX as u32;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Scene {
    A,
    B,
}

impl Scene {
    fn versions(self) -> [&'static str; 2] {
        match self {
            Self::A => ["Rabe\n42", "Rabe\n43"],
            Self::B => ["Mond\n7", "Mond\n8"],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Read,
    Edit,
}

fn document(text: &str) -> EditorDocument {
    let paragraphs: Vec<_> = text
        .lines()
        .map(|line| json!({"type": "paragraph", "content": [{"type": "text", "text": line}]}))
        .collect();
    EditorDocument {
        version: 1,
        document: json!({"type": "doc", "content": paragraphs}),
    }
}

struct Record {
    content: EditorDocument,
    revision: u32,
}

impl Record {
    fn initial(scene: Scene) -> Self {
        Self {
            content: document(scene.versions()[0]),
            revision: 0,
        }
    }
}

/// Owned fixture storage. No constructor accepts a project or external document.
pub struct SyntheticCatalog {
    a: Record,
    b: Record,
}

impl Default for SyntheticCatalog {
    fn default() -> Self {
        Self {
            a: Record::initial(Scene::A),
            b: Record::initial(Scene::B),
        }
    }
}

impl SyntheticCatalog {
    /// Only the model driver selects the scene and rights. The exclusive borrow lasts
    /// until this binding ends, including across reads and failed saves.
    ///
    /// ```compile_fail
    /// use kraqel_native_scene_model::{Mode, Scene, SyntheticCatalog};
    /// let mut catalog = SyntheticCatalog::default();
    /// let mut old = catalog.bind(Scene::A, Mode::Read);
    /// let _new = catalog.bind(Scene::B, Mode::Edit);
    /// old.handle(br#"{"command":"c4_document_read_v1"}"#);
    /// ```
    pub fn bind(&mut self, scene: Scene, mode: Mode) -> BoundScene<'_> {
        let record = match scene {
            Scene::A => &mut self.a,
            Scene::B => &mut self.b,
        };
        BoundScene {
            scene,
            mode,
            record,
        }
    }
}

pub struct BoundScene<'a> {
    scene: Scene,
    mode: Mode,
    record: &'a mut Record,
}

#[derive(Deserialize)]
#[serde(tag = "command", deny_unknown_fields)]
enum Request {
    #[serde(rename = "c4_workspace_status_v1")]
    Status {},
    #[serde(rename = "c4_document_read_v1")]
    Read {},
    #[serde(rename = "c4_document_save_v1")]
    Save {
        #[serde(rename = "expectedRevision")]
        expected_revision: u32,
        text: String,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum Reply {
    Status {
        scene: Scene,
        mode: Mode,
        revision: u32,
    },
    Read {
        text: &'static str,
        revision: u32,
    },
    Saved {
        revision: u32,
    },
    Failure {
        error: Failure,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Failure {
    Invalid,
    Denied,
    Conflict,
    Limit,
}

impl BoundScene<'_> {
    /// One complete v1 message body. Authentication and framing belong to BP4b.
    pub fn handle(&mut self, body: &[u8]) -> Reply {
        self.apply(body)
            .unwrap_or_else(|error| Reply::Failure { error })
    }

    fn apply(&mut self, body: &[u8]) -> Result<Reply, Failure> {
        if body.len() > MAX_FRAME_BYTES {
            return Err(Failure::Limit);
        }
        let mut stream = serde_json::Deserializer::from_slice(body).into_iter::<Request>();
        let request = stream
            .next()
            .ok_or(Failure::Invalid)?
            .map_err(|_| Failure::Invalid)?;
        if stream.byte_offset() != body.len() {
            return Err(Failure::Invalid);
        }

        let revision = self.record.revision;
        match request {
            Request::Status {} => Ok(Reply::Status {
                scene: self.scene,
                mode: self.mode,
                revision,
            }),
            Request::Read {} => Ok(Reply::Read {
                text: self.current_text()?,
                revision,
            }),
            Request::Save {
                expected_revision,
                text,
            } => {
                if self.mode != Mode::Edit {
                    return Err(Failure::Denied);
                }
                if text.len() > MAX_TEXT_BYTES || expected_revision > MAX_REVISION {
                    return Err(Failure::Limit);
                }
                self.current_text()?;
                if !self.scene.versions().contains(&text.as_str()) {
                    return Err(Failure::Invalid);
                }
                if revision != expected_revision {
                    return Err(Failure::Conflict);
                }
                if revision == MAX_REVISION {
                    return Err(Failure::Limit);
                }
                let revision = revision + 1;
                *self.record = Record {
                    content: document(&text),
                    revision,
                };
                Ok(Reply::Saved { revision })
            }
        }
    }

    // Match the entire stored tree, including the envelope version. A plain-text
    // projection could hide marks, attributes, or content belonging to another scene.
    fn current_text(&self) -> Result<&'static str, Failure> {
        self.scene
            .versions()
            .into_iter()
            .find(|text| document(text) == self.record.content)
            .ok_or(Failure::Invalid)
    }
}

#[cfg(test)]
mod tests;
