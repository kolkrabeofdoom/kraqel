//! BP4a research model. The caller supplies a simulated binding, never a native grant.
#![forbid(unsafe_code)]

pub mod channel;

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

// Only the second text node in this fixed scene may carry a bold mark.
fn marked_document(text: &str) -> EditorDocument {
    let mut doc = document(text);
    doc.document["content"][1]["content"][0]["marks"] = json!([{"type":"bold"}]);
    doc
}

impl Scene {
    fn documents(self) -> impl Iterator<Item = (&'static str, EditorDocument)> {
        self.versions().into_iter().flat_map(move |text| {
            [
                Some(document(text)),
                (self == Scene::A).then(|| marked_document(text)),
            ]
            .into_iter()
            .flatten()
            .map(move |doc| (text, doc))
        })
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
    pub fn with_marked_scene_a() -> Self {
        let mut catalog = Self::default();
        catalog.a.content = marked_document(Scene::A.versions()[0]);
        catalog
    }

    /// Restore only canonical scene A fixtures from the separate BP4b database.
    pub fn restore_a(revision: u32, encoded: &str) -> Option<Self> {
        if revision > MAX_REVISION {
            return None;
        }
        let content = Scene::A
            .documents()
            .map(|(_, doc)| doc)
            .find(|d| serde_json::to_string(d).is_ok_and(|json| json == encoded))?;
        Some(Self {
            a: Record { content, revision },
            ..Self::default()
        })
    }
    pub fn snapshot_a(&self) -> (String, u32) {
        (
            serde_json::to_string(&self.a.content).expect("catalog JSON"),
            self.a.revision,
        )
    }

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
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        bold_second: bool,
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
                bold_second: self.record.content.document["content"][1]["content"][0]
                    .get("marks")
                    .is_some(),
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
                // Both accepted versions share the first paragraph. Change only the
                // second text leaf after validating the entire stored tree above.
                let (_, replacement) = text.split_once('\n').ok_or(Failure::Invalid)?;
                self.record.content.document["content"][1]["content"][0]["text"] =
                    json!(replacement);
                self.record.revision = revision;
                Ok(Reply::Saved { revision })
            }
        }
    }

    // Match the entire stored tree, including the envelope version. A plain-text
    // projection could hide marks, attributes, or content belonging to another scene.
    fn current_text(&self) -> Result<&'static str, Failure> {
        self.scene
            .documents()
            .find(|(_, doc)| *doc == self.record.content)
            .map(|(text, _)| text)
            .ok_or(Failure::Invalid)
    }
}

#[cfg(test)]
mod tests;
