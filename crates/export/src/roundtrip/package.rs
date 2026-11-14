use super::inventory::{CP, W};
use crate::docx_scene::{self, Element};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    io::{Cursor, Write},
};
use zip::{ZipWriter, write::SimpleFileOptions};

pub(super) const REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
pub(super) const CT: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
pub(super) const OFFICE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const XML_MIME: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.";

fn description(name: &str) -> Option<(&'static str, &'static str, String)> {
    let result = match name {
        "word/document.xml" => (W, "document", format!("{XML_MIME}document.main+xml")),
        "word/styles.xml" => (W, "styles", format!("{XML_MIME}styles+xml")),
        "word/stylesWithEffects.xml" => (W, "styles", format!("{XML_MIME}stylesWithEffects+xml")),
        "word/numbering.xml" => (W, "numbering", format!("{XML_MIME}numbering+xml")),
        "word/settings.xml" => (W, "settings", format!("{XML_MIME}settings+xml")),
        "word/webSettings.xml" => (W, "webSettings", format!("{XML_MIME}webSettings+xml")),
        "word/fontTable.xml" => (W, "fonts", format!("{XML_MIME}fontTable+xml")),
        "word/theme/theme1.xml" => (
            "http://schemas.openxmlformats.org/drawingml/2006/main",
            "theme",
            "application/vnd.openxmlformats-officedocument.theme+xml".to_owned(),
        ),
        "docProps/custom.xml" => (
            CP,
            "Properties",
            "application/vnd.openxmlformats-officedocument.custom-properties+xml".to_owned(),
        ),
        "docProps/core.xml" => (
            "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
            "coreProperties",
            "application/vnd.openxmlformats-package.core-properties+xml".to_owned(),
        ),
        "docProps/app.xml" => (
            "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
            "Properties",
            "application/vnd.openxmlformats-officedocument.extended-properties+xml".to_owned(),
        ),
        "docProps/thumbnail.jpeg" => ("", "preview", "image/jpeg".to_owned()),
        "docProps/thumbnail.png" => ("", "preview", "image/png".to_owned()),
        _ => return None,
    };
    Some(result)
}
pub(super) fn read(bytes: &[u8]) -> Result<BTreeMap<String, Element>, ()> {
    let parts = docx_scene::package(bytes)?;
    for (name, node) in &parts {
        if matches!(
            name.as_str(),
            "[Content_Types].xml" | "_rels/.rels" | "word/_rels/document.xml.rels"
        ) {
            continue;
        }
        let (ns, root, _) = description(name).ok_or(())?;
        if !node.is(ns, root) {
            return Err(());
        }
    }
    types(&parts)?;
    relationships(&parts)?;
    Ok(parts)
}
fn types(parts: &BTreeMap<String, Element>) -> Result<(), ()> {
    let root = parts.get("[Content_Types].xml").ok_or(())?;
    root.whitespace()?;
    let mut overrides = BTreeMap::new();
    let mut defaults = BTreeMap::new();
    for entry in &root.children {
        entry.whitespace()?;
        if !entry.children.is_empty() || entry.attrs.len() != 2 {
            return Err(());
        }
        let mime = entry.attr("", "ContentType").ok_or(())?;
        if entry.is(CT, "Override") {
            let name = entry
                .attr("", "PartName")
                .and_then(|s| s.strip_prefix('/'))
                .ok_or(())?;
            if !parts.contains_key(name)
                || description(name).ok_or(())?.2 != mime
                || overrides.insert(name, mime).is_some()
            {
                return Err(());
            }
        } else if entry.is(CT, "Default") {
            let ext = entry.attr("", "Extension").ok_or(())?;
            let expected = match ext {
                "xml" => "application/xml",
                "rels" => "application/vnd.openxmlformats-package.relationships+xml",
                "jpeg" | "jpg" => "image/jpeg",
                "png" => "image/png",
                // LibreOffice declares this unused type; font parts themselves remain forbidden.
                "fntdata" => "application/x-fontdata",
                _ => return Err(()),
            };
            if mime != expected || defaults.insert(ext, mime).is_some() {
                return Err(());
            }
        } else {
            return Err(());
        }
    }
    if defaults.get("rels") != Some(&"application/vnd.openxmlformats-package.relationships+xml") {
        return Err(());
    }
    for name in parts.keys() {
        if matches!(
            name.as_str(),
            "_rels/.rels" | "word/_rels/document.xml.rels" | "[Content_Types].xml"
        ) {
            continue;
        }
        let (_, _, mime) = description(name).ok_or(())?;
        if name.starts_with("docProps/thumbnail.") {
            let ext = name.rsplit('.').next().ok_or(())?;
            if !overrides.contains_key(name.as_str())
                && defaults.get(ext).copied() != Some(mime.as_str())
            {
                return Err(());
            }
        } else if overrides.get(name.as_str()).copied() != Some(mime.as_str()) {
            return Err(());
        }
    }
    Ok(())
}
fn relationship_target(owner: &str, kind: &str, target: &str) -> bool {
    if owner == "_rels/.rels" {
        return match kind.strip_prefix(OFFICE) {
            Some("officeDocument") => target == "word/document.xml",
            Some("custom-properties") => target == "docProps/custom.xml",
            Some("extended-properties") => target == "docProps/app.xml",
            _ => matches!(
                (kind, target),
                (
                    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties",
                    "docProps/core.xml"
                ) | (
                    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
                    "docProps/thumbnail.jpeg" | "docProps/thumbnail.png"
                )
            ),
        };
    }
    if owner != "word/_rels/document.xml.rels" {
        return false;
    }
    match kind.strip_prefix(OFFICE) {
        Some("styles") => target == "word/styles.xml",
        Some("stylesWithEffects") => target == "word/stylesWithEffects.xml",
        Some("numbering") => target == "word/numbering.xml",
        Some("settings") => target == "word/settings.xml",
        Some("webSettings") => target == "word/webSettings.xml",
        Some("fontTable") => target == "word/fontTable.xml",
        Some("theme") => target == "word/theme/theme1.xml",
        _ => {
            kind == "http://schemas.microsoft.com/office/2007/relationships/stylesWithEffects"
                && target == "word/stylesWithEffects.xml"
        }
    }
}
fn relationships(parts: &BTreeMap<String, Element>) -> Result<(), ()> {
    let mut referenced = BTreeSet::new();
    for owner in ["_rels/.rels", "word/_rels/document.xml.rels"] {
        let root = parts.get(owner).ok_or(())?;
        root.whitespace()?;
        if !root.is(REL, "Relationships") || root.children.len() > 128 {
            return Err(());
        }
        let mut ids = BTreeSet::new();
        for rel in &root.children {
            rel.whitespace()?;
            if !rel.children.is_empty()
                || !rel.is(REL, "Relationship")
                || rel.attrs.keys().any(|(ns, key)| {
                    !ns.is_empty()
                        || !matches!(key.as_str(), "Id" | "Type" | "Target" | "TargetMode")
                })
                || rel.attr("", "TargetMode").is_some_and(|s| s != "Internal")
            {
                return Err(());
            }
            let id = rel.attr("", "Id").ok_or(())?;
            if id.is_empty() || !ids.insert(id) {
                return Err(());
            }
            let target = docx_scene::relationship_target(owner, rel.attr("", "Target").ok_or(())?)?;
            if !relationship_target(owner, rel.attr("", "Type").ok_or(())?, &target)
                || !parts.contains_key(&target)
                || !referenced.insert(target)
            {
                return Err(());
            }
        }
    }
    for required in [
        "word/document.xml",
        "word/styles.xml",
        "word/numbering.xml",
        "docProps/custom.xml",
    ] {
        if !referenced.contains(required) {
            return Err(());
        }
    }
    if parts.keys().any(|name| {
        !matches!(
            name.as_str(),
            "[Content_Types].xml" | "_rels/.rels" | "word/_rels/document.xml.rels"
        ) && !referenced.contains(name)
    }) {
        return Err(());
    }
    Ok(())
}

pub(super) fn write(parts: &BTreeMap<String, String>) -> Result<Vec<u8>, ()> {
    let mut entries = parts.clone();
    let mut types = format!(
        "<Types xmlns=\"{CT}\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/>"
    );
    for name in parts
        .keys()
        .filter(|name| name.strip_suffix(".xml").is_some())
    {
        let (_, _, mime) = description(name).ok_or(())?;
        write!(
            types,
            "<Override PartName=\"/{name}\" ContentType=\"{mime}\"/>"
        )
        .map_err(|_| ())?;
    }
    types.push_str("</Types>");
    entries.insert("[Content_Types].xml".to_owned(), types);
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, text) in entries {
        archive.start_file(name, options).map_err(|_| ())?;
        archive.write_all(text.as_bytes()).map_err(|_| ())?;
    }
    archive.finish().map(Cursor::into_inner).map_err(|_| ())
}
