use super::{
    inventory::{self, Inventory, Role, W},
    numbering::Numbering,
    package,
};
use crate::{
    Block, Chapter, Inline, Manuscript, Scene,
    docx_scene::{self, Element, Styles},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn read(bytes: &[u8]) -> Result<Manuscript, ()> {
    let parts = package::read(bytes)?;
    let (inventory, digest) = inventory::read(parts.get("docProps/custom.xml").ok_or(())?)?;
    let children = inventory::topology(&inventory.nodes)?;
    let root = parts.get("word/document.xml").ok_or(())?;
    root.whitespace()?;
    if root.children.len() != 1 || !root.children[0].is(W, "body") {
        return Err(());
    }
    let body = &root.children[0];
    body.whitespace()?;
    let mut style_root = parts.get("word/styles.xml").ok_or(())?.clone();
    clean_layout(&mut style_root)?;
    let styles = Styles::roundtrip(&style_root)?;
    let mut numbering = Numbering::new(parts.get("word/numbering.xml").ok_or(())?)?;
    let leaves: Vec<_> = inventory
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.role.leaf())
        .collect();
    let mut index = 0;
    let mut blocks = BTreeMap::new();
    let mut text_bytes = 0;
    let ids: BTreeMap<_, _> = inventory
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();
    let mut section = false;
    for node in &body.children {
        if node.is(W, "sectPr") {
            if section || index != leaves.len() {
                return Err(());
            }
            section = true;
            layout_section(node)?;
            continue;
        }
        if section {
            return Err(());
        }
        let &(node_index, expected) = leaves.get(index).ok_or(())?;
        let mut paragraph = control(node, &expected.role.marker(&expected.id, &digest))?;
        let properties = paragraph.child("pPr")?;
        let parent = ids.get(expected.parent.as_str()).copied().ok_or(())?;
        let expected_list =
            if inventory.nodes[parent].role == Role::Item && expected.position == "0" {
                let list = *ids.get(inventory.nodes[parent].parent.as_str()).ok_or(())?;
                Some((list, inventory.nodes[list].role))
            } else {
                None
            };
        numbering.check(
            properties.map(|p| p.child("numPr")).transpose()?.flatten(),
            expected_list,
        )?;
        if let Some(properties) = paragraph.children.iter_mut().find(|p| p.is(W, "pPr")) {
            properties.children.retain(|p| !p.is(W, "numPr"));
        }
        clean_layout(&mut paragraph)?;
        let value = docx_scene::paragraph(&paragraph, &styles, &mut text_bytes)?;
        let block = crate::parse_block(&value).map_err(|_| ())?;
        if let Some(level) = expected.role.heading() {
            if !matches!(&block,Block::Heading(actual,_) if *actual==level) {
                return Err(());
            }
        } else if !matches!(&block, Block::Paragraph(_)) {
            return Err(());
        }
        blocks.insert(node_index, block);
        index += 1;
    }
    if index != leaves.len() {
        return Err(());
    }
    model(&inventory, &children, &blocks)?
        .canonicalized()
        .map_err(|_| ())
}

fn control(node: &Element, wanted: &str) -> Result<Element, ()> {
    node.whitespace()?;
    let (sdt, paragraph) = if node.is(W, "p") {
        let sdt = node.child("sdt")?.ok_or(())?;
        if node
            .children
            .iter()
            .any(|c| !c.is(W, "pPr") && !c.is(W, "sdt"))
        {
            return Err(());
        }
        node.child("pPr")?;
        (sdt, Some(node))
    } else if node.is(W, "sdt") {
        (node, None)
    } else {
        return Err(());
    };
    sdt.whitespace()?;
    if !sdt.attrs.is_empty()
        || sdt
            .children
            .iter()
            .any(|c| c.ns != W || !matches!(c.name.as_str(), "sdtPr" | "sdtContent" | "sdtEndPr"))
    {
        return Err(());
    }
    let props = sdt.child("sdtPr")?.ok_or(())?;
    props.whitespace()?;
    if !props.attrs.is_empty() {
        return Err(());
    }
    let mut names = BTreeSet::new();
    let mut types = 0;
    for property in &props.children {
        property.whitespace()?;
        if property.ns != W
            || !property.children.is_empty()
            || !names.insert(property.name.as_str())
        {
            return Err(());
        }
        match property.name.as_str() {
            "tag" if property.attr(W, "val") == Some(wanted) && property.attrs.len() == 1 => {}
            "alias"
                if property.attr(W, "val").is_some_and(|v| v.len() <= 240)
                    && property.attrs.len() == 1 => {}
            "id" if property
                .attr(W, "val")
                .is_some_and(|v| v.parse::<i32>().is_ok())
                && property.attrs.len() == 1 => {}
            "richText" if property.attrs.is_empty() => {
                types += 1;
            }
            "text"
                if property.attrs.is_empty()
                    || property.attrs.len() == 1
                        && matches!(
                            property.attr(W, "multiLine"),
                            Some("1" | "0" | "true" | "false")
                        ) =>
            {
                types += 1;
            }
            _ => return Err(()),
        }
    }
    if !names.contains("tag") || types > 1 {
        return Err(());
    }
    let content = sdt.child("sdtContent")?.ok_or(())?;
    content.whitespace()?;
    if !content.attrs.is_empty() {
        return Err(());
    }
    end_marker(sdt)?;
    if let Some(paragraph) = paragraph {
        let mut result = paragraph.clone();
        result.children.retain(|c| c.is(W, "pPr"));
        result.children.extend(content.children.iter().cloned());
        Ok(result)
    } else if content.children.len() == 1 && content.children[0].is(W, "p") {
        Ok(content.children[0].clone())
    } else {
        Err(())
    }
}

fn clean_layout(node: &mut Element) -> Result<(), ()> {
    let mut remove = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let bidi = node.is(W, "pPr") && child.is(W, "bidi");
        let ligatures = node.is(W, "rPr")
            && child.is(
                "http://schemas.microsoft.com/office/word/2010/wordml",
                "ligatures",
            );
        if bidi || ligatures {
            child.whitespace()?;
            if !child.children.is_empty()
                || child.attrs.len() != 1
                || remove
                    .iter()
                    .any(|previous: &usize| node.children[*previous].name == child.name)
            {
                return Err(());
            }
            if bidi && !matches!(child.attr(W, "val"), Some("0" | "false" | "off")) {
                return Err(());
            }
            if ligatures
                && child
                    .attr(
                        "http://schemas.microsoft.com/office/word/2010/wordml",
                        "val",
                    )
                    .is_none()
            {
                return Err(());
            }
            remove.push(index);
        }
    }
    for index in remove.into_iter().rev() {
        node.children.remove(index);
    }
    for child in &mut node.children {
        clean_layout(child)?;
    }
    Ok(())
}
fn layout_section(node: &Element) -> Result<(), ()> {
    node.whitespace()?;
    let mut seen = BTreeSet::new();
    for child in &node.children {
        child.whitespace()?;
        if child.is(W, "formProt")
            && !matches!(super::numbering::scalar(child)?, "0" | "false" | "off")
        {
            return Err(());
        }
        if child.ns != W
            || !seen.insert(&child.name)
            || !child.children.is_empty()
            || !matches!(
                child.name.as_str(),
                "pgSz"
                    | "pgMar"
                    | "cols"
                    | "docGrid"
                    | "type"
                    | "titlePg"
                    | "pgNumType"
                    | "textDirection"
                    | "vAlign"
                    | "paperSrc"
                    | "formProt"
            )
        {
            return Err(());
        }
    }
    Ok(())
}

fn title(index: usize, blocks: &BTreeMap<usize, Block>) -> Result<String, ()> {
    let Block::Paragraph(runs) = blocks.get(&index).ok_or(())? else {
        return Err(());
    };
    let mut title = String::new();
    for run in runs {
        let Inline::Text {
            text,
            bold: false,
            italic: false,
        } = run
        else {
            return Err(());
        };
        title.push_str(text);
    }
    Ok(title)
}
fn block(
    index: usize,
    inventory: &Inventory,
    children: &[Vec<usize>],
    blocks: &BTreeMap<usize, Block>,
) -> Result<Block, ()> {
    Ok(match inventory.nodes[index].role {
        Role::Paragraph
        | Role::Heading1
        | Role::Heading2
        | Role::Heading3
        | Role::Heading4
        | Role::Heading5
        | Role::Heading6 => blocks.get(&index).ok_or(())?.clone(),
        Role::Quote => Block::Quote(
            children[index]
                .iter()
                .map(|i| block(*i, inventory, children, blocks))
                .collect::<Result<_, _>>()?,
        ),
        Role::BulletList | Role::OrderedList => {
            let items = children[index]
                .iter()
                .map(|item| {
                    children[*item]
                        .iter()
                        .map(|i| block(*i, inventory, children, blocks))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<_, _>>()?;
            if inventory.nodes[index].role == Role::BulletList {
                Block::BulletList(items)
            } else {
                Block::OrderedList(items)
            }
        }
        _ => return Err(()),
    })
}
fn model(
    inventory: &Inventory,
    children: &[Vec<usize>],
    blocks: &BTreeMap<usize, Block>,
) -> Result<Manuscript, ()> {
    let mut chapters = Vec::new();
    for c in &children[0][1..] {
        let mut scenes = Vec::new();
        for s in &children[*c][1..] {
            scenes.push(Scene {
                title: title(children[*s][0], blocks)?,
                blocks: children[*s][1..]
                    .iter()
                    .map(|i| block(*i, inventory, children, blocks))
                    .collect::<Result<_, _>>()?,
            });
        }
        chapters.push(Chapter {
            title: title(children[*c][0], blocks)?,
            scenes,
        });
    }
    Ok(Manuscript {
        title: title(children[0][0], blocks)?,
        chapters,
    })
}

fn end_marker(sdt: &Element) -> Result<(), ()> {
    if let Some(end) = sdt.child("sdtEndPr")? {
        end.whitespace()?;
        if !end.attrs.is_empty()
            || end.children.len() > 1
            || end.children.iter().any(|c| !c.is(W, "rPr"))
        {
            return Err(());
        }
        // End-marker run formatting is layout only, but cannot contain arbitrary content.
        for rpr in &end.children {
            rpr.whitespace()?;
            if rpr.children.iter().any(|c| {
                c.ns != W
                    || !matches!(
                        c.name.as_str(),
                        "rFonts" | "b" | "i" | "bCs" | "iCs" | "sz" | "szCs" | "lang" | "color"
                    )
                    || !c.children.is_empty()
                    || !c.text.trim().is_empty()
            }) {
                return Err(());
            }
        }
    }
    Ok(())
}
