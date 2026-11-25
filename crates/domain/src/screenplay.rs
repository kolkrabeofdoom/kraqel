//! Speichergrammatik und getrennte, strengere Druckgrammatik für Drehbuchszenen.
use super::{EDITOR_DOCUMENT_VERSION, EditorDocument, SCREENPLAY_DOCUMENT_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScreenplayRole {
    SceneHeading,
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
}

fn fields(value: &Value, names: &[&str]) -> bool {
    value
        .as_object()
        .is_some_and(|o| o.keys().all(|k| names.contains(&k.as_str())))
}

fn contains_screenplay(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("screenplayBlock")
        || value
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|c| c.iter().any(contains_screenplay))
}

fn valid_inline(value: &Value) -> bool {
    match value.get("type").and_then(Value::as_str) {
        Some("text") => {
            if !fields(value, &["type", "text", "marks"])
                || !value["text"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty() && !s.contains('\0'))
            {
                return false;
            }
            let Some(marks) = value.get("marks") else {
                return true;
            };
            marks.as_array().is_some_and(|m| {
                m.len() <= 2
                    && m.iter().enumerate().all(|(i, mark)| {
                        fields(mark, &["type"])
                            && matches!(mark["type"].as_str(), Some("bold" | "italic"))
                            && !m[..i].contains(mark)
                    })
            })
        }
        Some("hardBreak") => fields(value, &["type"]),
        _ => false,
    }
}

fn valid_block(value: &Value) -> bool {
    value["type"].as_str() == Some("screenplayBlock")
        && fields(value, &["type", "attrs", "content"])
        && fields(&value["attrs"], &["role"])
        && serde_json::from_value::<ScreenplayRole>(value["attrs"]["role"].clone()).is_ok()
        && value
            .get("content")
            .is_none_or(|c| c.as_array().is_some_and(|a| a.iter().all(valid_inline)))
}

impl EditorDocument {
    /// Prüft die Speichergrammatik. Dialogentwürfe dürfen beim Tippen unvollständig sein.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        if self.document["type"].as_str() != Some("doc") {
            return false;
        }
        match self.version {
            EDITOR_DOCUMENT_VERSION => {
                self.document.get("content").is_none_or(Value::is_array)
                    && !contains_screenplay(&self.document)
            }
            SCREENPLAY_DOCUMENT_VERSION => {
                fields(&self.document, &["type", "content"])
                    && self.document["content"]
                        .as_array()
                        .is_some_and(|a| !a.is_empty() && a.iter().all(valid_block))
            }
            _ => false,
        }
    }
}

/// Prüft Rollen in Dokumentreihenfolge. Der boolesche Wert bezeichnet sichtbaren Text.
///
/// # Errors
/// Liefert ausschließlich einen statischen, inhaltsfreien Druckhinweis.
pub fn validate_sequence(blocks: &[(ScreenplayRole, bool)]) -> Result<(), &'static str> {
    use ScreenplayRole::{Action, Character, Dialogue, Parenthetical, SceneHeading};
    if blocks.iter().all(|(_, text)| !text)
        && blocks
            .iter()
            .all(|(role, _)| matches!(role, SceneHeading | Action))
    {
        return Ok(());
    }
    if !matches!(
        blocks.iter().find(|(_, text)| *text),
        Some((SceneHeading, true))
    ) {
        return Err("Der erste nichtleere Drehbuchabsatz muss ein Szenenkopf sein.");
    }
    let mut heading = false;
    let mut previous = None;
    for &(role, text) in blocks {
        if !text && role != Action {
            return Err("Diese Drehbuchrolle braucht einen Text.");
        }
        if matches!(previous, Some(Character | Parenthetical))
            && !matches!(role, Dialogue | Parenthetical)
        {
            return Err("Ein Dialog fehlt nach Figur oder Klammeranweisung.");
        }
        match role {
            SceneHeading if heading => {
                return Err("Eine Szene darf nur einen nichtleeren Szenenkopf enthalten.");
            }
            SceneHeading => heading = true,
            Dialogue if !matches!(previous, Some(Character | Parenthetical | Dialogue)) => {
                return Err("Dialog benötigt eine Figur.");
            }
            Parenthetical if !matches!(previous, Some(Character | Dialogue)) => {
                return Err("Eine Klammeranweisung benötigt eine Figur und anschließenden Dialog.");
            }
            _ => {}
        }
        previous = Some(role);
    }
    if matches!(previous, Some(Character | Parenthetical)) {
        return Err("Der letzte Dialogblock ist unvollständig.");
    }
    Ok(())
}
