//! Replay-bound, explicit project scene comparison over the authenticated role pipes.
use super::*;
use crate::window_receipt as wire;
use manuscript::{SceneRequest, SceneSaveStatus, SceneWriter};
use std::io::Read;
use std::time::{Duration, Instant};

const MAX_BYTES: usize = kraqel_storage::native_scene::NATIVE_SCENE_TEXT_BYTES;

struct Review {
    generation: u64,
    revision: u64,
    local: Zeroizing<String>,
    remote: Zeroizing<String>,
}

impl Review {
    fn merge(&self, choices: &[u8]) -> Option<Zeroizing<String>> {
        let mut local = self.local.split('\n');
        let mut remote = self.remote.split('\n');
        let mut merged = Zeroizing::new(String::new());
        let mut first = true;
        for choice in choices {
            let (a, b) = (local.next(), remote.next());
            if a.is_none() && b.is_none() {
                return None;
            }
            let chosen = match choice {
                0 => a,
                1 => b,
                _ => return None,
            };
            if let Some(line) = chosen {
                if !first {
                    merged.push('\n');
                }
                merged.push_str(line);
                first = false;
                if merged.len() > MAX_BYTES {
                    return None;
                }
            }
        }
        (local.next().is_none() && remote.next().is_none()).then_some(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_distinguishes_missing_and_empty_paragraphs() {
        let cases: &[(&str, &str, &[u8], Option<&str>)] = &[
            ("a\n\nz", "b", &[1, 0, 0], Some("b\n\nz")),
            ("a\n\nz", "b", &[1, 1, 0], Some("b\nz")),
            ("a", "b\n", &[0, 1], Some("a\n")),
            ("a", "b\n", &[0], None),
            ("a", "b", &[0, 0], None),
            ("a", "b", &[2], None),
        ];
        for &(local, remote, choices, expected) in cases {
            let review = Review {
                generation: 1,
                revision: 1,
                local: Zeroizing::new(local.to_owned()),
                remote: Zeroizing::new(remote.to_owned()),
            };
            let merged = review.merge(choices);
            assert_eq!(merged.as_ref().map(|s| s.as_str()), expected);
        }
    }
}

struct Frame<'a> {
    file: &'a mut fs::File,
    deadline: Instant,
}

impl<'a> Frame<'a> {
    fn new(file: &'a mut fs::File) -> Self {
        Self {
            file,
            deadline: Instant::now() + Duration::from_secs(15),
        }
    }

    // The deadline begins after an explicit decision, never while the person is editing.
    fn fill(&mut self, mut target: &mut [u8]) -> Result<()> {
        use windows_sys::Win32::System::Pipes::PeekNamedPipe;
        while !target.is_empty() {
            if Instant::now() >= self.deadline {
                return Err(invalid("scene-edit-timeout"));
            }
            let mut available = 0;
            check(
                unsafe {
                    PeekNamedPipe(
                        self.file.as_raw_handle(),
                        null_mut(),
                        0,
                        null_mut(),
                        &mut available,
                        null_mut(),
                    )
                },
                "scene-edit-pipe",
            )?;
            if available == 0 {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            let size = target.len().min(available as usize);
            let (chunk, rest) = target.split_at_mut(size);
            io("scene-edit-bytes", self.file.read_exact(chunk))?;
            target = rest;
        }
        Ok(())
    }

    fn number(&mut self) -> Result<u64> {
        let mut field = [0; 8];
        self.fill(&mut field)?;
        Ok(u64::from_le_bytes(field))
    }

    fn bytes(&mut self, max: usize) -> Result<Vec<u8>> {
        let mut field = [0; 4];
        self.fill(&mut field)?;
        let length = u32::from_le_bytes(field) as usize;
        if length > max {
            return Err(invalid("scene-edit-length"));
        }
        let mut bytes = vec![0; length];
        self.fill(&mut bytes)?;
        Ok(bytes)
    }

    fn text(&mut self) -> Result<Zeroizing<String>> {
        let text =
            String::from_utf8(self.bytes(MAX_BYTES)?).map_err(|_| invalid("scene-edit-utf8"))?;
        if text.contains(['\0', '\r']) {
            return Err(invalid("scene-edit-text"));
        }
        Ok(Zeroizing::new(text))
    }
}

pub(super) fn serve_edit(
    reply: &mut fs::File,
    acknowledgement: &mut fs::File,
    save: SceneWriter<'_>,
) -> Result<u8> {
    let mut conflicted = false;
    let mut review: Option<Review> = None;
    let mut generation = 0_u64;
    loop {
        let mut decision = [0];
        io("scene-edit-decision", reply.read_exact(&mut decision))?;
        let mut frame = Frame::new(reply);
        let mut local = None;
        let result = match decision[0] {
            0 => return Ok(0),
            1 => {
                let draft = frame.text()?;
                if conflicted {
                    SceneSaveStatus::Conflict
                } else {
                    save(SceneRequest::Save(&draft))
                }
            }
            wire::SCENE_COMPARE if conflicted => {
                review = None;
                local = Some(frame.text()?);
                save(SceneRequest::ReadLatest)
            }
            wire::SCENE_CANCEL_REVIEW if conflicted => {
                let ticket = frame.number()?;
                let canceled = review.take().is_some_and(|r| r.generation == ticket);
                let ack = if canceled {
                    wire::SAVE_ACK_CANCELED
                } else {
                    wire::SAVE_ACK_FAILED
                };
                io("scene-edit-cancel", acknowledgement.write_all(&[ack]))?;
                continue;
            }
            wire::SCENE_CONFIRM if conflicted => {
                let ticket = frame.number()?;
                let revision = frame.number()?;
                let choices = frame.bytes(MAX_BYTES + 1)?;
                if let Some(base) = review
                    .take()
                    .filter(|r| r.generation == ticket && r.revision == revision)
                    && let Some(merged) = base.merge(&choices)
                {
                    let outcome = save(SceneRequest::Confirm {
                        revision,
                        reviewed: &base.remote,
                        merged: &merged,
                    });
                    local = Some(base.local);
                    outcome
                } else {
                    SceneSaveStatus::Failed
                }
            }
            _ => return Err(invalid("scene-edit-decision")),
        };
        let byte = match result {
            SceneSaveStatus::Committed => wire::SAVE_ACK_COMMITTED,
            SceneSaveStatus::Conflict => {
                conflicted = true;
                wire::SAVE_ACK_CONFLICT
            }
            SceneSaveStatus::Failed => wire::SAVE_ACK_FAILED,
            SceneSaveStatus::Review { revision, text } => {
                let local = local.ok_or(invalid("scene-edit-unsolicited-review"))?;
                if text.len() > MAX_BYTES {
                    return Err(invalid("scene-edit-length"));
                }
                generation = generation
                    .checked_add(1)
                    .ok_or(invalid("scene-edit-generation"))?;
                let mut header = [0; 21];
                header[0] = wire::SAVE_ACK_REVIEW;
                header[1..9].copy_from_slice(&generation.to_le_bytes());
                header[9..17].copy_from_slice(&revision.to_le_bytes());
                header[17..].copy_from_slice(&(text.len() as u32).to_le_bytes());
                io("scene-edit-review", acknowledgement.write_all(&header))?;
                io(
                    "scene-edit-review",
                    acknowledgement.write_all(text.as_bytes()),
                )?;
                review = Some(Review {
                    generation,
                    revision,
                    local,
                    remote: text,
                });
                continue;
            }
        };
        io("scene-edit-ack", acknowledgement.write_all(&[byte]))?;
        if byte == wire::SAVE_ACK_COMMITTED {
            return Ok(1);
        }
    }
}
