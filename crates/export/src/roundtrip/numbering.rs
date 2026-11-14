use super::inventory::{Role, W};
use crate::docx_scene::Element;
use std::collections::BTreeMap;

pub(super) struct Numbering<'a> {
    abstracts: BTreeMap<&'a str, &'a Element>,
    numbers: BTreeMap<&'a str, &'a Element>,
    used: BTreeMap<usize, String>,
    ordered_owners: BTreeMap<String, usize>,
}
pub(super) fn scalar(node: &Element) -> Result<&str, ()> {
    node.whitespace()?;
    if !node.children.is_empty() || node.attrs.len() != 1 {
        return Err(());
    }
    node.attr(W, "val").ok_or(())
}
impl<'a> Numbering<'a> {
    pub(super) fn new(root: &'a Element) -> Result<Self, ()> {
        root.whitespace()?;
        let mut result = Self {
            abstracts: BTreeMap::new(),
            numbers: BTreeMap::new(),
            used: BTreeMap::new(),
            ordered_owners: BTreeMap::new(),
        };
        for node in &root.children {
            if node.is(W, "abstractNum") {
                let key = node.attr(W, "abstractNumId").ok_or(())?;
                key.parse::<u32>().map_err(|_| ())?;
                if result.abstracts.insert(key, node).is_some() {
                    return Err(());
                }
            } else if node.is(W, "num") {
                let key = node.attr(W, "numId").ok_or(())?;
                if key.parse::<u32>().map_err(|_| ())? == 0
                    || result.numbers.insert(key, node).is_some()
                {
                    return Err(());
                }
            } else if node.is(W, "numIdMacAtCleanup") {
                scalar(node)?.parse::<u32>().map_err(|_| ())?;
            } else {
                return Err(());
            }
        }
        Ok(result)
    }
    pub(super) fn check(
        &mut self,
        node: Option<&Element>,
        expected: Option<(usize, Role)>,
    ) -> Result<(), ()> {
        let Some((list, role)) = expected else {
            return if node.is_none() { Ok(()) } else { Err(()) };
        };
        let node = node.ok_or(())?;
        node.whitespace()?;
        if !node.attrs.is_empty()
            || node
                .children
                .iter()
                .any(|c| !c.is(W, "numId") && !c.is(W, "ilvl"))
        {
            return Err(());
        }
        if let Some(level) = node.child("ilvl")?
            && scalar(level)? != "0"
        {
            return Err(());
        }
        let key = scalar(node.child("numId")?.ok_or(())?)?;
        let num = *self.numbers.get(key).ok_or(())?;
        num.whitespace()?;
        let abstract_key = scalar(num.child("abstractNumId")?.ok_or(())?)?;
        let abstract_node = *self.abstracts.get(abstract_key).ok_or(())?;
        abstract_node.whitespace()?;
        let mut levels = BTreeMap::new();
        for child in &abstract_node.children {
            if child.is(W, "lvl") {
                let level = child
                    .attr(W, "ilvl")
                    .ok_or(())?
                    .parse::<u8>()
                    .map_err(|_| ())?;
                if level > 8 || levels.insert(level, child).is_some() {
                    return Err(());
                }
            } else if !matches!(
                child.name.as_str(),
                "nsid" | "multiLevelType" | "tmpl" | "name"
            ) || child.ns != W
                || !child.children.is_empty()
            {
                return Err(());
            }
        }
        Self::level(levels.get(&0).copied().ok_or(())?, role)?;
        let mut seen = false;
        for child in &num.children {
            if child.is(W, "abstractNumId") {
                continue;
            }
            if !child.is(W, "lvlOverride") || child.attr(W, "ilvl") != Some("0") || seen {
                return Err(());
            }
            seen = true;
            child.whitespace()?;
            if child.children.iter().any(|c| !c.is(W, "startOverride"))
                || scalar(child.child("startOverride")?.ok_or(())?)? != "1"
            {
                return Err(());
            }
        }
        if self.used.get(&list).is_some_and(|id| id != key) {
            return Err(());
        }
        self.used.insert(list, key.to_owned());
        if role == Role::OrderedList {
            if self
                .ordered_owners
                .get(key)
                .is_some_and(|owner| *owner != list)
            {
                return Err(());
            }
            self.ordered_owners.insert(key.to_owned(), list);
        }
        Ok(())
    }
    fn level(node: &Element, role: Role) -> Result<(), ()> {
        node.whitespace()?;
        if scalar(node.child("start")?.ok_or(())?)? != "1" {
            return Err(());
        }
        if scalar(node.child("numFmt")?.ok_or(())?)?
            != if role == Role::BulletList {
                "bullet"
            } else {
                "decimal"
            }
        {
            return Err(());
        }
        let marker = scalar(node.child("lvlText")?.ok_or(())?)?;
        if (role == Role::OrderedList && marker != "%1.") || marker.is_empty() || marker.len() > 16
        {
            return Err(());
        }
        let mut seen = std::collections::BTreeSet::new();
        for child in &node.children {
            child.whitespace()?;
            if child.ns != W || !seen.insert(child.name.as_str()) {
                return Err(());
            }
            match child.name.as_str() {
                "start" | "numFmt" | "lvlText" | "lvlJc" | "suff" => {
                    scalar(child)?;
                }
                "pPr" => {
                    for property in &child.children {
                        if property.ns != W
                            || !matches!(property.name.as_str(), "ind" | "tabs" | "jc" | "spacing")
                        {
                            return Err(());
                        }
                        property.whitespace()?;
                        if property.name == "tabs" {
                            if property.children.iter().any(|c| {
                                !c.is(W, "tab")
                                    || !c.children.is_empty()
                                    || !c.text.trim().is_empty()
                            }) {
                                return Err(());
                            }
                        } else if !property.children.is_empty() {
                            return Err(());
                        }
                    }
                }
                "rPr" => {
                    for property in &child.children {
                        if property.ns != W
                            || !matches!(
                                property.name.as_str(),
                                "b" | "i"
                                    | "bCs"
                                    | "iCs"
                                    | "rFonts"
                                    | "sz"
                                    | "szCs"
                                    | "color"
                                    | "lang"
                                    | "hint"
                                    | "spacing"
                                    | "position"
                                    | "kern"
                            )
                            || !property.children.is_empty()
                            || !property.text.trim().is_empty()
                        {
                            return Err(());
                        }
                    }
                }
                _ => return Err(()),
            }
        }
        Ok(())
    }
}
