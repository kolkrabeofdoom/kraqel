//! Bounded, offline DOCX-to-editor import. See ADR 0024.
use kraqel_domain::{EDITOR_DOCUMENT_VERSION, EditorDocument};
use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{Cursor, Read as _},
    path::Path,
};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const CT: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
const MAX_ARCHIVE: usize = 8 * 1024 * 1024;
const MAX_TOTAL: u64 = 32 * 1024 * 1024;
const MAX_PART: u64 = 8 * 1024 * 1024;
const MAX_TEXT: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub(super) struct Element {
    pub(super) ns: String,
    pub(super) name: String,
    pub(super) attrs: BTreeMap<(String, String), String>,
    pub(super) children: Vec<Self>,
    pub(super) text: String,
}

impl Element {
    pub(super) fn is(&self, ns: &str, name: &str) -> bool {
        self.ns == ns && self.name == name
    }
    pub(super) fn attr(&self, ns: &str, name: &str) -> Option<&str> {
        self.attrs
            .get(&(ns.to_owned(), name.to_owned()))
            .map(String::as_str)
    }
    pub(super) fn child(&self, name: &str) -> Result<Option<&Self>, ()> {
        let mut found = self.children.iter().filter(|e| e.is(W, name));
        let first = found.next();
        if found.next().is_some() {
            return Err(());
        }
        Ok(first)
    }
    pub(super) fn whitespace(&self) -> Result<(), ()> {
        if self.text.trim().is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }
}

fn namespace(value: &ResolveResult<'_>) -> Result<String, ()> {
    match value {
        ResolveResult::Bound(ns) => Ok(ns.as_ref().to_owned()),
        ResolveResult::Unbound => Ok(String::new()),
        ResolveResult::Unknown(_) => Err(()),
    }
}

pub(super) fn valid_text(text: &str) -> bool {
    text.chars().all(|c| {
        matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{fffe}' && c != '\u{ffff}')
    })
}

fn xml(bytes: &[u8]) -> Result<Element, ()> {
    let source = std::str::from_utf8(bytes).map_err(|_| ())?;
    if !valid_text(source) {
        return Err(());
    }
    let mut reader = NsReader::from_str(source);
    reader.config_mut().check_comments = true;
    reader.config_mut().expand_empty_elements = true;
    let mut stack: Vec<Element> = Vec::new();
    let mut root = None;
    let mut count = 0;
    let mut declared = false;
    loop {
        match reader.read_event().map_err(|_| ())? {
            Event::Start(event) | Event::Empty(event) => {
                // Empty events are expanded so every node uses the same close path.
                let (ns, local) = reader.resolver().resolve_element(event.name());
                let mut node = Element {
                    ns: namespace(&ns)?,
                    name: local.as_ref().to_owned(),
                    attrs: BTreeMap::new(),
                    children: Vec::new(),
                    text: String::new(),
                };
                for attribute in event.attributes() {
                    let a = attribute.map_err(|_| ())?;
                    if a.key.as_ref() == "xmlns" || a.key.as_ref().starts_with("xmlns:") {
                        continue;
                    }
                    let (ns, local) = reader.resolver().resolve_attribute(a.key);
                    let key = (namespace(&ns)?, local.as_ref().to_owned());
                    let value = a
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| ())?
                        .into_owned();
                    if !valid_text(&value) || node.attrs.insert(key, value).is_some() {
                        return Err(());
                    }
                }
                count += 1;
                if count > 100_000 || stack.len() >= 64 || (stack.is_empty() && root.is_some()) {
                    return Err(());
                }
                stack.push(node);
            }
            Event::End(_) => {
                let node = stack.pop().ok_or(())?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else if root.replace(node).is_some() {
                    return Err(());
                }
            }
            Event::Text(event) => {
                let text = event.xml10_content();
                if let Some(node) = stack.last_mut() {
                    node.text.push_str(&text);
                } else if !text.trim().is_empty() {
                    return Err(());
                }
            }
            Event::GeneralRef(event) => {
                let entity = event.as_ref();
                let reference = format!("&{entity};");
                let text = quick_xml::escape::unescape(&reference).map_err(|_| ())?;
                if !valid_text(&text) {
                    return Err(());
                }
                stack.last_mut().ok_or(())?.text.push_str(&text);
            }
            Event::CData(event) => {
                let text = event.xml10_content();
                stack.last_mut().ok_or(())?.text.push_str(&text);
            }
            Event::Decl(decl) => {
                if declared || !stack.is_empty() || root.is_some() {
                    return Err(());
                }
                declared = true;
                if let Some(encoding) = decl.encoding()
                    && !encoding.map_err(|_| ())?.eq_ignore_ascii_case("UTF-8")
                {
                    return Err(());
                }
            }
            Event::Comment(_) => {}
            Event::Eof => break,
            // Includes DOCTYPE and processing instructions; no entity/external resolver.
            _ => return Err(()),
        }
    }
    if !stack.is_empty() {
        return Err(());
    }
    root.ok_or(())
}

pub(super) fn parse_xml(bytes: &[u8]) -> Result<Element, ()> {
    // Expand empty elements into matching End events, retaining strict name checks.
    // This configuration is local to the single parser below.
    xml(bytes)
}

pub(super) fn read_bytes(path: &Path) -> Result<Vec<u8>, ()> {
    if !path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("docx"))
    {
        return Err(());
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAX_ARCHIVE as u64
    {
        return Err(());
    }
    let file = fs::File::open(path).map_err(|_| ())?;
    let opened = file.metadata().map_err(|_| ())?;
    if !opened.is_file() || opened.len() != metadata.len() {
        return Err(());
    }
    let mut bytes = Vec::new();
    file.take(MAX_ARCHIVE as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 != metadata.len() || bytes.len() > MAX_ARCHIVE {
        return Err(());
    }
    Ok(bytes)
}

pub(super) fn read_file(path: &Path) -> Result<(String, EditorDocument), ()> {
    let bytes = read_bytes(path)?;
    let title: String = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or(())?
        .trim()
        .chars()
        .take(240)
        .collect();
    if title.is_empty() {
        return Err(());
    }
    Ok((title, parse(&bytes)?))
}

pub(super) fn package(bytes: &[u8]) -> Result<BTreeMap<String, Element>, ()> {
    if bytes.len() > MAX_ARCHIVE {
        return Err(());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| ())?;
    if archive.len() > 128 || archive.has_duplicate_names() {
        return Err(());
    }
    archive.validate_header_consistency().map_err(|_| ())?;
    let mut parts = BTreeMap::new();
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0_u64;
    let mut total_nodes = 0;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|_| ())?;
        let name = entry.name().map_err(|_| ())?.into_owned();
        if !names.insert(name.clone())
            || name.contains(['\\', ':'])
            || name.starts_with('/')
            || name
                .trim_end_matches('/')
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
            || entry.is_symlink()
        {
            return Err(());
        }
        total = total.checked_add(entry.size()).ok_or(())?;
        if total > MAX_TOTAL
            || entry.size() > MAX_PART
            || entry.size() > entry.compressed_size().saturating_mul(100)
        {
            return Err(());
        }
        if entry.is_dir() {
            continue;
        }
        let allowed = matches!(
            name.as_str(),
            "[Content_Types].xml"
                | "_rels/.rels"
                | "word/document.xml"
                | "word/styles.xml"
                | "word/stylesWithEffects.xml"
                | "word/settings.xml"
                | "word/webSettings.xml"
                | "word/fontTable.xml"
                | "word/theme/theme1.xml"
                | "word/numbering.xml"
                | "word/_rels/document.xml.rels"
                | "docProps/core.xml"
                | "docProps/app.xml"
                | "docProps/custom.xml"
                | "docProps/thumbnail.jpeg"
                | "docProps/thumbnail.png"
        ) || auxiliary_part(&name);
        if !allowed {
            return Err(());
        }
        let expected = entry.size();
        let mut data = Vec::new();
        entry
            .by_ref()
            .take(MAX_PART + 1)
            .read_to_end(&mut data)
            .map_err(|_| ())?;
        if data.len() as u64 != expected {
            return Err(());
        }
        // A package preview is metadata, never an imported document image.
        let root = if name.starts_with("docProps/thumbnail.") {
            parse_xml(b"<preview/>")?
        } else {
            parse_xml(&data)?
        };
        total_nodes += node_count(&root);
        if total_nodes > 100_000 {
            return Err(());
        }
        parts.insert(name, root);
    }
    validate_package(&parts)?;
    Ok(parts)
}

fn node_count(node: &Element) -> usize {
    1 + node.children.iter().map(node_count).sum::<usize>()
}

fn auxiliary_part(name: &str) -> bool {
    let Some(part) = name.strip_prefix("customXml/") else {
        return false;
    };
    let digits = part
        .strip_prefix("itemProps")
        .and_then(|s| s.strip_suffix(".xml"))
        .or_else(|| {
            part.strip_prefix("item")
                .and_then(|s| s.strip_suffix(".xml"))
        })
        .or_else(|| {
            part.strip_prefix("_rels/item")
                .and_then(|s| s.strip_suffix(".xml.rels"))
        });
    digits.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

pub(super) fn relationship_target(owner: &str, target: &str) -> Result<String, ()> {
    if target.starts_with('/') || target.contains(['\\', ':', '%', '?', '#']) {
        return Err(());
    }
    let base = owner
        .split("/_rels/")
        .next()
        .filter(|_| owner != "_rels/.rels");
    let mut segments: Vec<&str> = base.into_iter().collect();
    for segment in target.split('/') {
        match segment {
            "" => return Err(()),
            "." => {}
            ".." => {
                segments.pop().ok_or(())?;
            }
            _ => segments.push(segment),
        }
    }
    Ok(segments.join("/"))
}

fn validate_package(parts: &BTreeMap<String, Element>) -> Result<(), ()> {
    let types = parts.get("[Content_Types].xml").ok_or(())?;
    if !types.is(CT, "Types") {
        return Err(());
    }
    let mut main_type = false;
    for child in &types.children {
        if child.is(CT, "Override") {
            let kind = child.attr("", "ContentType").ok_or(())?;
            if kind.contains("macroEnabled") {
                return Err(());
            }
            if child.attr("", "PartName") == Some("/word/document.xml") {
                if main_type
                    || kind
                        != "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
                {
                    return Err(());
                }
                main_type = true;
            }
        } else if !child.is(CT, "Default") {
            return Err(());
        }
    }
    if !main_type {
        return Err(());
    }
    let mut main_relationship = false;
    for (name, relationships) in parts {
        if name.strip_suffix(".rels").is_none() {
            continue;
        }
        if !relationships.is(REL, "Relationships") {
            return Err(());
        }
        let mut ids = std::collections::BTreeSet::new();
        for relationship in &relationships.children {
            if !relationship.is(REL, "Relationship")
                || relationship
                    .attr("", "TargetMode")
                    .is_some_and(|m| m != "Internal")
            {
                return Err(());
            }
            if !ids.insert(relationship.attr("", "Id").ok_or(())?) {
                return Err(());
            }
            let target = relationship.attr("", "Target").ok_or(())?;
            let kind = relationship.attr("", "Type").ok_or(())?;
            let resolved = relationship_target(name, target)?;
            if !parts.contains_key(&resolved) {
                return Err(());
            }
            let suffix = kind.strip_prefix(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/",
            );
            let core = matches!(
                kind,
                "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"
                    | "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"
                    | "http://schemas.microsoft.com/office/2007/relationships/stylesWithEffects"
            );
            if !core
                && !matches!(
                    suffix,
                    Some(
                        "officeDocument"
                            | "styles"
                            | "stylesWithEffects"
                            | "settings"
                            | "webSettings"
                            | "fontTable"
                            | "theme"
                            | "extended-properties"
                            | "custom-properties"
                            | "numbering"
                            | "customXml"
                            | "customXmlProps"
                    )
                )
            {
                return Err(());
            }
            if suffix == Some("officeDocument") {
                if name != "_rels/.rels" || target != "word/document.xml" || main_relationship {
                    return Err(());
                }
                main_relationship = true;
            }
        }
    }
    if !main_relationship {
        return Err(());
    }
    Ok(())
}

#[derive(Clone, Copy, Default)]
struct Format {
    bold: bool,
    italic: bool,
    heading: Option<u8>,
}

fn toggle(node: &Element) -> Result<bool, ()> {
    match node.attr(W, "val") {
        None | Some("true" | "1" | "on") => Ok(true),
        Some("false" | "0" | "off") => Ok(false),
        _ => Err(()),
    }
}

fn run_properties(node: Option<&Element>, mut format: Format) -> Result<Format, ()> {
    if let Some(node) = node {
        node.whitespace()?;
        let mut seen = std::collections::BTreeSet::new();
        let mut complex_bold = None;
        let mut complex_italic = None;
        for property in &node.children {
            if property.ns != W || !seen.insert(property.name.as_str()) {
                return Err(());
            }
            match property.name.as_str() {
                "b" => format.bold = toggle(property)?,
                "i" => format.italic = toggle(property)?,
                "bCs" => complex_bold = Some(toggle(property)?),
                "iCs" => complex_italic = Some(toggle(property)?),
                "rStyle" | "rFonts" | "sz" | "szCs" | "color" | "lang" | "kern" | "spacing"
                | "position" | "w" | "hint" => {}
                // Other character semantics must not be silently dropped.
                _ => return Err(()),
            }
            if !property.children.is_empty() || !property.text.trim().is_empty() {
                return Err(());
            }
        }
        if complex_bold.is_some_and(|v| v != format.bold)
            || complex_italic.is_some_and(|v| v != format.italic)
        {
            return Err(());
        }
    }
    Ok(format)
}

fn style_run_properties(node: Option<&Element>, format: Format) -> Result<Format, ()> {
    let mut result = run_properties(node, format)?;
    if let Some(node) = node {
        if node.child("rStyle")?.is_some() {
            return Err(());
        }
        if let Some(bold) = node.child("b")? {
            result.bold = format.bold ^ toggle(bold)?;
        }
        if let Some(italic) = node.child("i")? {
            result.italic = format.italic ^ toggle(italic)?;
        }
    }
    Ok(result)
}

fn paragraph_properties(node: Option<&Element>, mut format: Format) -> Result<Format, ()> {
    if let Some(node) = node {
        node.whitespace()?;
        let mut seen = std::collections::BTreeSet::new();
        for property in &node.children {
            if property.ns != W || !seen.insert(property.name.as_str()) {
                return Err(());
            }
            property.whitespace()?;
            if property.name != "rPr" && !property.children.is_empty() {
                return Err(());
            }
            match property.name.as_str() {
                "outlineLvl" => {
                    let level: u8 = property.attr(W, "val").ok_or(())?.parse().map_err(|_| ())?;
                    format.heading = match level {
                        0..=5 => Some(level + 1),
                        9 => None,
                        _ => return Err(()),
                    };
                }
                "rPr" => {
                    run_properties(Some(property), format)?;
                }
                "pStyle"
                | "spacing"
                | "ind"
                | "jc"
                | "keepNext"
                | "keepLines"
                | "pageBreakBefore"
                | "widowControl"
                | "contextualSpacing"
                | "snapToGrid"
                | "suppressAutoHyphens" => {}
                _ => return Err(()),
            }
        }
    }
    Ok(format)
}

pub(super) struct Styles<'a> {
    definitions: BTreeMap<&'a str, &'a Element>,
    defaults: Format,
    normal: Option<&'a str>,
    character: Option<&'a str>,
    strict: bool,
}
impl<'a> Styles<'a> {
    pub(super) fn new(root: Option<&'a Element>) -> Result<Self, ()> {
        let mut result = Self {
            definitions: BTreeMap::new(),
            defaults: Format::default(),
            normal: None,
            character: None,
            strict: false,
        };
        if let Some(root) = root {
            if !root.is(W, "styles") {
                return Err(());
            }
            for node in &root.children {
                if node.is(W, "style") {
                    let id = node.attr(W, "styleId").ok_or(())?;
                    if result.definitions.insert(id, node).is_some() {
                        return Err(());
                    }
                    if node.attr(W, "type") == Some("paragraph")
                        && matches!(node.attr(W, "default"), Some("1" | "true" | "on"))
                        && result.normal.replace(id).is_some()
                    {
                        return Err(());
                    }
                    if node.attr(W, "type") == Some("character")
                        && matches!(node.attr(W, "default"), Some("1" | "true" | "on"))
                        && result.character.replace(id).is_some()
                    {
                        return Err(());
                    }
                } else if node.is(W, "docDefaults") {
                    let rpr = node
                        .child("rPrDefault")?
                        .map(|n| n.child("rPr"))
                        .transpose()?
                        .flatten();
                    result.defaults = run_properties(rpr, result.defaults)?;
                    let ppr = node
                        .child("pPrDefault")?
                        .map(|n| n.child("pPr"))
                        .transpose()?
                        .flatten();
                    result.defaults = paragraph_properties(ppr, result.defaults)?;
                } else if !node.is(W, "latentStyles") {
                    return Err(());
                }
            }
        }
        Ok(result)
    }
    pub(super) fn roundtrip(root: &'a Element) -> Result<Self, ()> {
        let mut styles = Self::new(Some(root))?;
        styles.strict = true;
        root.whitespace()?;
        let mut defaults = false;
        for node in &root.children {
            if node.is(W, "docDefaults") {
                if defaults {
                    return Err(());
                }
                defaults = true;
                node.whitespace()?;
                for group in &node.children {
                    group.whitespace()?;
                    let property = if group.is(W, "rPrDefault") {
                        "rPr"
                    } else if group.is(W, "pPrDefault") {
                        "pPr"
                    } else {
                        return Err(());
                    };
                    if group.children.len() > 1 || group.children.iter().any(|p| !p.is(W, property))
                    {
                        return Err(());
                    }
                }
            }
        }
        Ok(styles)
    }
    fn resolve(
        &self,
        id: Option<&str>,
        kind: &str,
        format: Format,
        chain: &mut Vec<String>,
    ) -> Result<Format, ()> {
        let Some(id) = id else {
            return Ok(format);
        };
        if chain.len() >= 32 || chain.iter().any(|s| s == id) {
            return Err(());
        }
        let node = self.definitions.get(id).ok_or(())?;
        if self.strict {
            node.whitespace()?;
            let mut names = std::collections::BTreeSet::new();
            for child in &node.children {
                child.whitespace()?;
                if child.ns != W || !names.insert(child.name.as_str()) {
                    return Err(());
                }
                match child.name.as_str() {
                    "pPr" | "rPr" => {}
                    "name" | "basedOn" | "next" | "link" | "aliases" | "uiPriority" | "qFormat"
                    | "semiHidden" | "unhideWhenUsed" | "locked" | "personal"
                    | "personalCompose" | "personalReply" | "rsid" | "autoRedefine" | "hidden"
                        if child.children.is_empty() => {}
                    _ => return Err(()),
                }
            }
        }
        if node.attr(W, "type") != Some(kind) {
            return Err(());
        }
        chain.push(id.to_owned());
        let base = node
            .child("basedOn")?
            .map(|n| n.attr(W, "val").ok_or(()))
            .transpose()?;
        let mut inherited = self.resolve(base, kind, format, chain)?;
        if kind == "paragraph"
            && let Some(level) = id
                .strip_prefix("Heading")
                .or_else(|| id.strip_prefix("heading"))
                .and_then(|v| v.parse::<u8>().ok())
                .filter(|v| (1..=6).contains(v))
        {
            inherited.heading = Some(level);
        }
        let result = style_run_properties(
            node.child("rPr")?,
            paragraph_properties(node.child("pPr")?, inherited)?,
        )?;
        chain.pop();
        Ok(result)
    }
}

pub(super) fn paragraph(
    node: &Element,
    styles: &Styles<'_>,
    text_bytes: &mut usize,
) -> Result<Value, ()> {
    node.whitespace()?;
    let properties = node.child("pPr")?;
    let style = properties
        .map(|n| n.child("pStyle"))
        .transpose()?
        .flatten()
        .map(|n| n.attr(W, "val").ok_or(()))
        .transpose()?
        .or(styles.normal);
    let inherited = styles.resolve(style, "paragraph", styles.defaults, &mut Vec::new())?;
    let format = paragraph_properties(properties, inherited)?;
    let mut content = Vec::new();
    for run in &node.children {
        if run.is(W, "pPr") {
            continue;
        }
        if matches!(
            run.name.as_str(),
            "bookmarkStart" | "bookmarkEnd" | "proofErr"
        ) && run.ns == W
            && run.children.is_empty()
        {
            continue;
        }
        if !run.is(W, "r") {
            return Err(());
        }
        run.whitespace()?;
        let rpr = run.child("rPr")?;
        let rstyle = rpr
            .map(|n| n.child("rStyle"))
            .transpose()?
            .flatten()
            .map(|n| n.attr(W, "val").ok_or(()))
            .transpose()?;
        let run_format = styles.resolve(
            rstyle.or(styles.character),
            "character",
            format,
            &mut Vec::new(),
        )?;
        let run_format = run_properties(rpr, run_format)?;
        for item in &run.children {
            if item.is(W, "rPr") {
                continue;
            }
            if item.is(W, "t") && item.children.is_empty() {
                *text_bytes = text_bytes.checked_add(item.text.len()).ok_or(())?;
                if *text_bytes > MAX_TEXT {
                    return Err(());
                }
                if !item.text.is_empty() {
                    let mut marks = Vec::new();
                    if run_format.bold {
                        marks.push(json!({"type":"bold"}));
                    }
                    if run_format.italic {
                        marks.push(json!({"type":"italic"}));
                    }
                    let mut text = json!({"type":"text", "text":item.text});
                    if !marks.is_empty() {
                        text["marks"] = json!(marks);
                    }
                    content.push(text);
                }
            } else if item.is(W, "br")
                && item.attr(W, "type").is_none_or(|v| v == "textWrapping")
                && item.children.is_empty()
                && item.text.is_empty()
            {
                content.push(json!({"type":"hardBreak"}));
            } else if item.is(W, "lastRenderedPageBreak") && item.children.is_empty() { /* layout only */
            } else {
                return Err(());
            }
        }
    }
    let mut block = if let Some(level) = format.heading {
        json!({"type":"heading", "attrs":{"level":level}})
    } else {
        json!({"type":"paragraph"})
    };
    if !content.is_empty() {
        block["content"] = json!(content);
    }
    Ok(block)
}

fn parse(bytes: &[u8]) -> Result<EditorDocument, ()> {
    let parts = package(bytes)?;
    let document = parts.get("word/document.xml").ok_or(())?;
    if !document.is(W, "document") || document.children.len() != 1 {
        return Err(());
    }
    document.whitespace()?;
    let body = &document.children[0];
    if !body.is(W, "body") {
        return Err(());
    }
    body.whitespace()?;
    let styles = Styles::new(parts.get("word/styles.xml"))?;
    let mut blocks = Vec::new();
    let mut text_bytes = 0;
    for node in &body.children {
        if node.is(W, "p") {
            blocks.push(paragraph(node, &styles, &mut text_bytes)?);
        } else if node.is(W, "sectPr") {
            node.whitespace()?;
            for layout in &node.children {
                if layout.ns != W
                    || !matches!(
                        layout.name.as_str(),
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
                    )
                    || !layout.children.is_empty()
                    || !layout.text.trim().is_empty()
                {
                    return Err(());
                }
            }
        } else {
            return Err(());
        }
    }
    if blocks.is_empty() {
        blocks.push(json!({"type":"paragraph"}));
    }
    Ok(EditorDocument {
        version: EDITOR_DOCUMENT_VERSION,
        document: json!({"type":"doc", "content":blocks}),
    })
}

#[cfg(test)]
pub(super) fn fixture(body: &str, styles: Option<&str>, extra: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write as _;
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let main = format!("<w:document xmlns:w=\"{W}\"><w:body>{body}</w:body></w:document>");
    let content_types = format!(
        "<Types xmlns=\"{CT}\"><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>"
    );
    let rels = format!(
        "<Relationships xmlns=\"{REL}\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>"
    );
    let mut entries = vec![
        ("[Content_Types].xml", content_types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        ("word/document.xml", main.as_bytes()),
    ];
    if let Some(styles) = styles {
        entries.push(("word/styles.xml", styles.as_bytes()));
    }
    entries.extend_from_slice(extra);
    for (name, data) in entries {
        zip.start_file(name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_independently_generated_docx_with_standard_template_parts() {
        let document = parse(include_bytes!(
            "../../../apps/desktop/src-tauri/tests/fixtures/basic.docx"
        ))
        .unwrap();
        assert_eq!(document.document["content"][0]["attrs"]["level"], 2);
        assert_eq!(
            document.document["content"][1]["content"][0]["text"],
            "Grüße aus Köln: Fett und kursiv."
        );
        assert_eq!(
            document.document["content"][1]["content"][0]["marks"],
            json!([{"type":"bold"},{"type":"italic"}])
        );
        assert_eq!(
            document.document["content"][1]["content"][1]["type"],
            "hardBreak"
        );
    }

    #[test]
    fn rejects_archive_entry_count_compression_amplification_and_duplicate_names() {
        use std::io::Write as _;
        let make = |count: usize, deflate: bool| {
            let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
            let method = if deflate {
                zip::CompressionMethod::Deflated
            } else {
                zip::CompressionMethod::Stored
            };
            for i in 0..count {
                zip.start_file(
                    format!("customXml/item{i}.xml"),
                    zip::write::SimpleFileOptions::default().compression_method(method),
                )
                .unwrap();
                let content = if deflate {
                    vec![b'a'; 200_000]
                } else {
                    b"<a/>".to_vec()
                };
                zip.write_all(&content).unwrap();
            }
            zip.finish().unwrap().into_inner()
        };
        assert!(package(&make(129, false)).is_err());
        assert!(package(&make(1, true)).is_err());
        let styles = format!("<w:styles xmlns:w=\"{W}\"/>");
        let mut duplicate = fixture("<w:p/>", Some(&styles), &[("word/foobar.xml", b"<a/>")]);
        for offset in (0..duplicate.len() - 15).rev() {
            if &duplicate[offset..offset + 15] == b"word/foobar.xml" {
                duplicate[offset..offset + 15].copy_from_slice(b"word/styles.xml");
            }
        }
        assert!(package(&duplicate).is_err());
    }

    #[test]
    fn bounds_nodes_and_resolves_only_internal_package_targets() {
        let many = format!("<a>{}</a>", "<b/>".repeat(100_000));
        assert!(parse_xml(many.as_bytes()).is_err());
        assert_eq!(
            relationship_target("word/_rels/document.xml.rels", "../customXml/item1.xml").unwrap(),
            "customXml/item1.xml"
        );
        for target in [
            "../../outside.xml",
            "file:///private",
            "https://example.invalid",
            "/absolute",
            "..%2Foutside",
        ] {
            assert!(relationship_target("word/_rels/document.xml.rels", target).is_err());
        }
    }

    #[test]
    fn preserves_unicode_empty_paragraphs_breaks_and_combined_marks() {
        let doc = parse(&fixture("<w:p/><w:p><w:r><w:rPr><w:b/><w:i/></w:rPr><w:t xml:space=\"preserve\"> Grüße &amp; &lt;Welt&gt; &#x1F426; </w:t><w:br/><w:t>Ende</w:t></w:r></w:p>", None, &[])).unwrap();
        assert_eq!(doc.document["content"][0], json!({"type":"paragraph"}));
        assert_eq!(
            doc.document["content"][1]["content"][0]["text"],
            " Grüße & <Welt> 🐦 "
        );
        assert_eq!(
            doc.document["content"][1]["content"][0]["marks"],
            json!([{"type":"bold"},{"type":"italic"}])
        );
        assert_eq!(
            doc.document["content"][1]["content"][1]["type"],
            "hardBreak"
        );
    }

    #[test]
    fn resolves_inherited_styles_and_direct_overrides() {
        let styles = format!(
            "<w:styles xmlns:w=\"{W}\"><w:style w:type=\"paragraph\" w:styleId=\"Base\"><w:rPr><w:b/></w:rPr></w:style><w:style w:type=\"paragraph\" w:styleId=\"Titel\"><w:basedOn w:val=\"Base\"/><w:pPr><w:outlineLvl w:val=\"2\"/></w:pPr></w:style></w:styles>"
        );
        let doc = parse(&fixture("<w:p><w:pPr><w:pStyle w:val=\"Titel\"/></w:pPr><w:r><w:rPr><w:b w:val=\"0\"/><w:i/></w:rPr><w:t>Text</w:t></w:r></w:p>", Some(&styles), &[])).unwrap();
        assert_eq!(doc.document["content"][0]["attrs"]["level"], 3);
        assert_eq!(
            doc.document["content"][0]["content"][0]["marks"],
            json!([{"type":"italic"}])
        );
    }

    #[test]
    fn handles_all_six_heading_levels_and_namespace_aliases() {
        for level in 0..6 {
            let body = format!(
                "<x:p xmlns:x=\"{W}\"><x:pPr><x:outlineLvl x:val=\"{level}\"/></x:pPr><x:r><x:t>Text</x:t></x:r></x:p>"
            );
            let doc = parse(&fixture(&body, None, &[])).unwrap();
            assert_eq!(doc.document["content"][0]["attrs"]["level"], level + 1);
        }
    }

    #[test]
    fn rejects_unsupported_content_instead_of_dropping_it() {
        for body in [
            "<w:tbl/>",
            "<w:p><w:hyperlink/></w:p>",
            "<w:p><w:r><w:drawing/></w:r></w:p>",
            "<w:p><w:r><w:footnoteReference/></w:r></w:p>",
            "<w:p><w:r><w:instrText>FIELD</w:instrText></w:r></w:p>",
            "<w:p><w:ins><w:r><w:t>neu</w:t></w:r></w:ins></w:p>",
            "<w:p><w:pPr><w:numPr/></w:pPr></w:p>",
            "<w:p><w:r><w:rPr><w:u/></w:rPr><w:t>Text</w:t></w:r></w:p>",
            "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>",
        ] {
            assert!(parse(&fixture(body, None, &[])).is_err(), "accepted {body}");
        }
    }

    #[test]
    fn rejects_unknown_and_cyclic_styles() {
        let body = "<w:p><w:pPr><w:pStyle w:val=\"Cycle\"/></w:pPr></w:p>";
        assert!(parse(&fixture(body, None, &[])).is_err());
        let styles = format!(
            "<w:styles xmlns:w=\"{W}\"><w:style w:type=\"paragraph\" w:styleId=\"Cycle\"><w:basedOn w:val=\"Cycle\"/></w:style></w:styles>"
        );
        assert!(parse(&fixture(body, Some(&styles), &[])).is_err());
    }

    #[test]
    fn rejects_external_relationships_macros_and_unsupported_parts() {
        let rels = format!(
            "<Relationships xmlns=\"{REL}\"><Relationship Id=\"r1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink\" Target=\"https://example.invalid\" TargetMode=\"External\"/></Relationships>"
        );
        assert!(
            parse(&fixture(
                "<w:p/>",
                None,
                &[("word/_rels/document.xml.rels", rels.as_bytes())]
            ))
            .is_err()
        );
        for name in [
            "word/vbaProject.bin",
            "word/media/image1.png",
            "../escape.xml",
            "word/footnotes.xml",
        ] {
            assert!(parse(&fixture("<w:p/>", None, &[(name, b"x")])).is_err());
        }
    }

    #[test]
    fn rejects_malformed_xml_dtd_unknown_entities_depth_and_namespace_spoofing() {
        for source in [
            "<a><b></a>",
            "<!DOCTYPE a [<!ENTITY x 'text'>]><a>&x;</a>",
            "<a>&unknown;</a>",
            "<a>&#0;</a>",
            "<a/><b/>",
        ] {
            assert!(parse_xml(source.as_bytes()).is_err(), "accepted {source}");
        }
        let deep = format!("{}{}", "<a>".repeat(65), "</a>".repeat(65));
        assert!(parse_xml(deep.as_bytes()).is_err());
        assert!(parse(&fixture("<w:p xmlns:w=\"urn:fake\"/>", None, &[])).is_err());
    }

    #[test]
    fn rejects_oversized_text_and_input() {
        let body = format!(
            "<w:p><w:r><w:t>{}</w:t></w:r></w:p>",
            "a".repeat(MAX_TEXT + 1)
        );
        assert!(parse(&fixture(&body, None, &[])).is_err());
        assert!(parse(&vec![0; MAX_ARCHIVE + 1]).is_err());
    }

    #[test]
    fn checks_extension_file_type_and_title() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("Grüße.DOCX");
        fs::write(&path, fixture("<w:p/>", None, &[])).unwrap();
        assert_eq!(read_file(&path).unwrap().0, "Grüße");
        assert!(read_file(temp.path()).is_err());
        let wrong = temp.path().join("falsch.docm");
        fs::copy(&path, &wrong).unwrap();
        assert!(read_file(&wrong).is_err());
    }
}
