use crate::docx_scene::Element;
use crate::{Block, Inline, Manuscript};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const PROPERTY: &str = "KRAQEL.Manuscript.v1";
pub(super) const PREFIX: &str = "kqm1";
pub(super) const CP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
pub(super) const VT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
pub(super) const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
pub(super) const CHUNK: usize = 200;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Role {
    Manuscript,
    ProjectTitle,
    Chapter,
    ChapterTitle,
    Scene,
    SceneTitle,
    Paragraph,
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    BulletList,
    OrderedList,
    Item,
    Quote,
}
impl Role {
    pub(super) fn token(self) -> &'static str {
        match self {
            Self::Manuscript => "manuscript",
            Self::ProjectTitle => "project-title",
            Self::Chapter => "chapter",
            Self::ChapterTitle => "chapter-title",
            Self::Scene => "scene",
            Self::SceneTitle => "scene-title",
            Self::Paragraph => "paragraph",
            Self::Heading1 => "heading1",
            Self::Heading2 => "heading2",
            Self::Heading3 => "heading3",
            Self::Heading4 => "heading4",
            Self::Heading5 => "heading5",
            Self::Heading6 => "heading6",
            Self::BulletList => "bullet-list",
            Self::OrderedList => "ordered-list",
            Self::Item => "item",
            Self::Quote => "quote",
        }
    }
    pub(super) fn heading(self) -> Option<u8> {
        match self {
            Self::Heading1 => Some(1),
            Self::Heading2 => Some(2),
            Self::Heading3 => Some(3),
            Self::Heading4 => Some(4),
            Self::Heading5 => Some(5),
            Self::Heading6 => Some(6),
            _ => None,
        }
    }
    pub(super) fn leaf(self) -> bool {
        self.heading().is_some()
            || matches!(
                self,
                Self::ProjectTitle | Self::ChapterTitle | Self::SceneTitle | Self::Paragraph
            )
    }
    pub(super) fn block(self) -> bool {
        self.heading().is_some()
            || matches!(
                self,
                Self::Paragraph | Self::BulletList | Self::OrderedList | Self::Quote
            )
    }
    pub(super) fn marker(self, id: &str, digest: &str) -> String {
        format!("{PREFIX}:{}:{id}:{}", self.token(), &digest[..32])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Node {
    pub id: String,
    pub role: Role,
    pub parent: String,
    pub position: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Inventory {
    pub version: u32,
    pub nodes: Vec<Node>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Header {
    pub version: u32,
    pub chunks: usize,
    pub sha256: String,
}
pub(super) fn hash(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

pub(super) struct Record {
    pub node: Node,
    pub runs: Option<Vec<Inline>>,
    pub numbering: Option<usize>,
    pub quote: usize,
}
fn push(
    records: &mut Vec<Record>,
    role: Role,
    parent: Option<usize>,
    position: usize,
    runs: Option<Vec<Inline>>,
    numbering: Option<usize>,
    quote: usize,
) -> usize {
    let index = records.len();
    let parent = parent.map_or_else(String::new, |p| records[p].node.id.clone());
    records.push(Record {
        node: Node {
            id: format!("n{:04}", index + 1),
            role,
            parent,
            position: position.to_string(),
        },
        runs,
        numbering,
        quote,
    });
    index
}
fn title(records: &mut Vec<Record>, role: Role, parent: usize, text: &str) {
    let runs = if text.is_empty() {
        Vec::new()
    } else {
        vec![Inline::Text {
            text: text.to_owned(),
            bold: false,
            italic: false,
        }]
    };
    push(records, role, Some(parent), 0, Some(runs), None, 0);
}
fn emit_blocks(
    records: &mut Vec<Record>,
    source: &[Block],
    parent: usize,
    offset: usize,
    quote: usize,
    numbering: Option<usize>,
) {
    for (position, block) in source.iter().enumerate() {
        let num = if position == 0 { numbering } else { None };
        match block {
            Block::Paragraph(runs) => {
                push(
                    records,
                    Role::Paragraph,
                    Some(parent),
                    position + offset,
                    Some(runs.clone()),
                    num,
                    quote,
                );
            }
            Block::Heading(level, runs) => {
                let role = match level {
                    1 => Role::Heading1,
                    2 => Role::Heading2,
                    3 => Role::Heading3,
                    4 => Role::Heading4,
                    5 => Role::Heading5,
                    _ => Role::Heading6,
                };
                push(
                    records,
                    role,
                    Some(parent),
                    position + offset,
                    Some(runs.clone()),
                    None,
                    quote,
                );
            }
            Block::Quote(children) => {
                let index = push(
                    records,
                    Role::Quote,
                    Some(parent),
                    position + offset,
                    None,
                    None,
                    quote,
                );
                emit_blocks(records, children, index, 0, quote + 1, None);
            }
            Block::BulletList(items) | Block::OrderedList(items) => {
                let role = if matches!(block, Block::BulletList(_)) {
                    Role::BulletList
                } else {
                    Role::OrderedList
                };
                let list = push(
                    records,
                    role,
                    Some(parent),
                    position + offset,
                    None,
                    None,
                    quote,
                );
                for (position, item) in items.iter().enumerate() {
                    let index = push(records, Role::Item, Some(list), position, None, None, quote);
                    emit_blocks(records, item, index, 0, quote, Some(list + 1));
                }
            }
        }
    }
}
pub(super) fn records(manuscript: &Manuscript) -> Vec<Record> {
    let mut output = Vec::new();
    let root = push(&mut output, Role::Manuscript, None, 0, None, None, 0);
    title(&mut output, Role::ProjectTitle, root, &manuscript.title);
    for (position, chapter) in manuscript.chapters.iter().enumerate() {
        let c = push(
            &mut output,
            Role::Chapter,
            Some(root),
            position + 1,
            None,
            None,
            0,
        );
        title(&mut output, Role::ChapterTitle, c, &chapter.title);
        for (position, scene) in chapter.scenes.iter().enumerate() {
            let s = push(
                &mut output,
                Role::Scene,
                Some(c),
                position + 1,
                None,
                None,
                0,
            );
            title(&mut output, Role::SceneTitle, s, &scene.title);
            emit_blocks(&mut output, &scene.blocks, s, 1, 0, None);
        }
    }
    output
}

pub(super) fn payload(nodes: Vec<Node>) -> Result<(String, Header), ()> {
    let payload = serde_json::to_string(&Inventory { version: 1, nodes }).map_err(|_| ())?;
    let header = Header {
        version: 1,
        chunks: payload.len().div_ceil(CHUNK),
        sha256: hash(&payload),
    };
    if !payload.is_ascii() || header.chunks > 2048 {
        return Err(());
    }
    Ok((payload, header))
}

pub(super) fn read(root: &Element) -> Result<(Inventory, String), ()> {
    if !root.is(CP, "Properties") {
        return Err(());
    }
    root.whitespace()?;
    let mut values = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for property in &root.children {
        property.whitespace()?;
        if !property.is(CP, "property") || property.children.len() != 1 {
            return Err(());
        }
        let id = property
            .attr("", "pid")
            .ok_or(())?
            .parse::<u32>()
            .map_err(|_| ())?;
        if id < 2
            || !ids.insert(id)
            || property.attr("", "fmtid") != Some("{D5CDD505-2E9C-101B-9397-08002B2CF9AE}")
        {
            return Err(());
        }
        let name = property.attr("", "name").ok_or(())?;
        if name.len() > 128
            || name.is_empty()
            || property.attrs.keys().any(|(ns, key)| {
                !ns.is_empty() || !matches!(key.as_str(), "pid" | "fmtid" | "name")
            })
        {
            return Err(());
        }
        let value = &property.children[0];
        if !value.is(VT, "lpwstr") || !value.children.is_empty() || !value.attrs.is_empty() {
            return Err(());
        }
        if values.insert(name, value.text.as_str()).is_some() {
            return Err(());
        }
    }
    let header: Header = serde_json::from_str(values.get(PROPERTY).ok_or(())?).map_err(|_| ())?;
    if header.version != 1
        || header.chunks == 0
        || header.chunks > 2048
        || header.sha256.len() != 64
        || !header
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(());
    }
    let mut joined = String::new();
    for index in 1..=header.chunks {
        let key = format!("{PROPERTY}.{index:04}");
        let value = values.get(key.as_str()).ok_or(())?;
        if value.is_empty()
            || value.len() > CHUNK
            || !value.is_ascii()
            || (index < header.chunks && value.len() != CHUNK)
        {
            return Err(());
        }
        joined.push_str(value);
    }
    if values
        .keys()
        .filter(|key| key.starts_with(PROPERTY))
        .count()
        != header.chunks + 1
        || hash(&joined) != header.sha256
    {
        return Err(());
    }
    let inventory: Inventory = serde_json::from_str(&joined).map_err(|_| ())?;
    if inventory.version != 1
        || inventory.nodes.is_empty()
        || inventory.nodes.len() > 4096
        || serde_json::to_string(&inventory).map_err(|_| ())? != joined
    {
        return Err(());
    }
    Ok((inventory, header.sha256))
}

pub(super) fn topology(nodes: &[Node]) -> Result<Vec<Vec<usize>>, ()> {
    let mut ids = BTreeMap::new();
    let mut children = vec![Vec::new(); nodes.len()];
    let mut depths = vec![1; nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        let digits = node.id.strip_prefix('n').ok_or(())?;
        if !(4..=8).contains(&digits.len()) || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(());
        }
        if index == 0 {
            if node.role != Role::Manuscript || !node.parent.is_empty() || node.position != "0" {
                return Err(());
            }
        } else {
            let parent: usize = *ids.get(&node.parent).ok_or(())?;
            if nodes[parent].role.leaf() || node.position != children[parent].len().to_string() {
                return Err(());
            }
            children[parent].push(index);
            depths[index] = depths[parent] + 1;
            if depths[index] > 32 {
                return Err(());
            }
        }
        if ids.insert(node.id.clone(), index).is_some() {
            return Err(());
        }
    }
    let mut pending = vec![0];
    let mut next = 0;
    while let Some(index) = pending.pop() {
        if index != next {
            return Err(());
        }
        next += 1;
        pending.extend(children[index].iter().rev());
        let roles: Vec<_> = children[index].iter().map(|i| nodes[*i].role).collect();
        let valid = match nodes[index].role {
            Role::Manuscript => {
                roles.first() == Some(&Role::ProjectTitle)
                    && roles[1..].iter().all(|r| *r == Role::Chapter)
            }
            Role::Chapter => {
                roles.first() == Some(&Role::ChapterTitle)
                    && roles[1..].iter().all(|r| *r == Role::Scene)
            }
            Role::Scene => {
                roles.first() == Some(&Role::SceneTitle) && roles[1..].iter().all(|r| r.block())
            }
            Role::Item => {
                roles.first() == Some(&Role::Paragraph) && roles.iter().all(|r| r.block())
            }
            Role::Quote => !roles.is_empty() && roles.iter().all(|r| r.block()),
            Role::BulletList | Role::OrderedList => {
                !roles.is_empty() && roles.iter().all(|r| *r == Role::Item)
            }
            _ => roles.is_empty(),
        };
        if !valid {
            return Err(());
        }
    }
    Ok(children)
}
