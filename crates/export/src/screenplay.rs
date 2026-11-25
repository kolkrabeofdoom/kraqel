//! Shared validation and presentation of screenplay paragraphs.
use crate::{ExportError, Inline, parse_inlines};
use kraqel_domain::{
    EditorDocument, SCREENPLAY_DOCUMENT_VERSION, ScreenplayRole as Role,
    screenplay::validate_sequence,
};
const INVALID: ExportError = ExportError::InvalidDocument("Die Drehbuchszene ist ungültig.");

pub(crate) struct Paragraph {
    pub role: Role,
    pub runs: Vec<Inline>,
}

pub(crate) fn paragraphs(content: &EditorDocument) -> Result<Vec<Paragraph>, ExportError> {
    if content.version != SCREENPLAY_DOCUMENT_VERSION || !content.is_valid() {
        return Err(INVALID);
    }
    let nodes = crate::array(
        content.document.get("content"),
        "Die Drehbuchszene ist ungültig.",
    )?;
    let mut blocks = Vec::with_capacity(nodes.len());
    let mut sequence = Vec::with_capacity(nodes.len());
    for node in nodes {
        let role: Role =
            serde_json::from_value(node["attrs"]["role"].clone()).map_err(|_| INVALID)?;
        let runs = parse_inlines(node.get("content"))?;
        let has_text = runs
            .iter()
            .any(|r| matches!(r,Inline::Text{text,..} if !text.trim().is_empty()));
        sequence.push((role, has_text));
        blocks.push(Paragraph { role, runs });
    }
    validate_sequence(&sequence).map_err(ExportError::InvalidDocument)?;
    Ok(blocks)
}

pub(crate) struct Presentation {
    pub key: &'static str,
    pub label: &'static str,
    pub style: &'static str,
}

pub(crate) const ROLES: [Role; 6] = [
    Role::SceneHeading,
    Role::Action,
    Role::Character,
    Role::Parenthetical,
    Role::Dialogue,
    Role::Transition,
];

pub(crate) fn presentation(role: Role) -> Presentation {
    let (key, label, style) = match role {
        Role::SceneHeading => ("sceneHeading", "Szenenkopf", "KQScreenplaySceneHeading"),
        Role::Action => ("action", "Handlung", "KQScreenplayAction"),
        Role::Character => ("character", "Figur", "KQScreenplayCharacter"),
        Role::Parenthetical => (
            "parenthetical",
            "Klammeranweisung",
            "KQScreenplayParenthetical",
        ),
        Role::Dialogue => ("dialogue", "Dialog", "KQScreenplayDialogue"),
        Role::Transition => ("transition", "Übergang", "KQScreenplayTransition"),
    };
    Presentation { key, label, style }
}
