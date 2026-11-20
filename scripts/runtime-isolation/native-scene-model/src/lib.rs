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

/// Two short paragraphs are the whole synthetic editing surface.
pub const MAX_PARAGRAPH_UTF16: usize = 48;

pub fn valid_paragraph(text: &str) -> bool {
    !text.is_empty() && text.encode_utf16().count() <= MAX_PARAGRAPH_UTF16 && !text.chars().any(char::is_control)
}

pub fn valid_paragraphs(text: &str) -> bool {
    text.split_once('\n')
        .is_some_and(|(first, second)| valid_paragraph(first) && valid_paragraph(second))
}

fn managed_text(doc: &EditorDocument, revision: u32) -> Option<String> {
    let paragraphs = &doc.document["content"];
    let text = format!(
        "{}\n{}",
        paragraphs[0]["content"][0]["text"].as_str()?,
        paragraphs[1]["content"][0]["text"].as_str()?
    );
    // Unmarked v1 fixtures had fixed text until their first interactive follow-up edit.
    let plain_allowed = revision >= 2 || Scene::A.versions().contains(&text.as_str());
    (valid_paragraphs(&text)
        && (marked_document(&text) == *doc || plain_allowed && document(&text) == *doc))
        .then_some(text)
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
    pub fn restore_managed_a(revision: u32, encoded: &str) -> Option<Self> {
        if let Some(old) = Self::restore_a(revision, encoded) {
            return Some(old);
        }
        let content = serde_json::from_str(encoded).ok()?;
        managed_text(&content, revision)?;
        (revision <= MAX_REVISION).then(|| Self {
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
        text: String,
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
        self.apply(body, false)
            .unwrap_or_else(|error| Reply::Failure { error })
    }

    pub fn handle_managed(&mut self, body: &[u8]) -> Reply {
        self.apply(body, true)
            .unwrap_or_else(|error| Reply::Failure { error })
    }

    fn apply(&mut self, body: &[u8], managed: bool) -> Result<Reply, Failure> {
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
        let editable_fixture = managed && self.scene == Scene::A;
        match request {
            Request::Status {} => Ok(Reply::Status {
                scene: self.scene,
                mode: self.mode,
                revision,
            }),
            Request::Read {} => Ok(Reply::Read {
                text: self.current_text(editable_fixture)?,
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
                self.current_text(editable_fixture)?;
                if !(self.scene.versions().contains(&text.as_str())
                    || editable_fixture
                        && valid_paragraphs(&text)
                        && (revision >= 1
                            || self.record.content.document["content"][1]["content"][0]
                                .get("marks")
                                .is_some()))
                {
                    return Err(Failure::Invalid);
                }
                if revision != expected_revision {
                    return Err(Failure::Conflict);
                }
                if revision == MAX_REVISION {
                    return Err(Failure::Limit);
                }
                let revision = revision + 1;
                let (first, second) = text.split_once('\n').ok_or(Failure::Invalid)?;
                let paragraphs = &mut self.record.content.document["content"];
                paragraphs[0]["content"][0]["text"] = json!(first);
                paragraphs[1]["content"][0]["text"] = json!(second);
                self.record.revision = revision;
                Ok(Reply::Saved { revision })
            }
        }
    }

    // Match the entire stored tree, including the envelope version. A plain-text
    // projection could hide marks, attributes, or content belonging to another scene.
    fn current_text(&self, editable_fixture: bool) -> Result<String, Failure> {
        if editable_fixture {
            return managed_text(&self.record.content, self.record.revision)
                .ok_or(Failure::Invalid);
        }
        self.scene
            .documents()
            .find(|(_, doc)| *doc == self.record.content)
            .map(|(text, _)| text.to_owned())
            .ok_or(Failure::Invalid)
    }
}

#[cfg(test)]
mod tests;
