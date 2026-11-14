//! Ordinary illustrated DOCX. Deliberately contains no roundtrip markers or inventory.
use super::{IllustratedError, MAX_PACKAGE, Manifest, PlacementRecord};
use crate::{Manuscript, render};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    io::{Cursor, Read, Write},
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const REL: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const PIC: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";

fn xml(value: &str) -> String {
    render::escape_html(value).replace('\n', "&#10;")
}

// 96 dpi intrinsic size; Letter with 1-inch margins, 600 pt maximum image height.
// Keep the final 48 pt of a content page available for inline spacing and a short caption.
fn extent(width: u32, height: u32) -> (u64, u64) {
    let (mut w, mut h) = (u64::from(width) * 9525, u64::from(height) * 9525);
    if w > 5_943_600 {
        h = h * 5_943_600 / w;
        w = 5_943_600;
    }
    if h > 7_620_000 {
        w = w * 7_620_000 / h;
        h = 7_620_000;
    }
    (w.max(1), h.max(1))
}

fn figure(
    output: &mut String,
    item: &PlacementRecord,
    manifest: &Manifest,
    id: usize,
) -> Result<(), IllustratedError> {
    let asset = manifest
        .assets
        .iter()
        .find(|a| a.id == item.asset_id)
        .ok_or(IllustratedError::Invalid)?;
    let (cx, cy) = extent(asset.export_width, asset.export_height);
    let keep = if item.caption.is_some() {
        "<w:keepNext/>"
    } else {
        ""
    };
    let title = xml(&item.title);
    let alt = xml(&item.alt_text);
    let rel = format!("image{}", asset.id);
    let _ = write!(
        output,
        r#"<w:p><w:pPr>{keep}<w:spacing w:before="160" w:after="80"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="{id}" name="Bild {id}" title="{title}" descr="{alt}"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri="{PIC}"><pic:pic><pic:nvPicPr><pic:cNvPr id="{id}" name="Bild {id}" descr="{alt}"/><pic:cNvPicPr><a:picLocks noChangeAspect="1"/></pic:cNvPicPr></pic:nvPicPr><pic:blipFill><a:blip r:embed="{rel}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    );
    if let Some(caption) = &item.caption {
        output.push_str(
            "<w:p><w:pPr><w:pStyle w:val=\"Caption\"/><w:keepLines w:val=\"0\"/></w:pPr>",
        );
        for (index, line) in caption.split('\n').enumerate() {
            if index > 0 {
                output.push_str("<w:r><w:br/></w:r>");
            }
            let _ = write!(
                output,
                "<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r>",
                xml(line)
            );
        }
        output.push_str("</w:p>");
    }
    Ok(())
}

fn document(manuscript: &Manuscript, manifest: &Manifest) -> Result<String, IllustratedError> {
    let mut output = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="{W}" xmlns:r="{R}" xmlns:wp="{WP}" xmlns:a="{A}" xmlns:pic="{PIC}"><w:body>"#
    );
    render::docx_text_paragraph(
        &mut output,
        &manuscript.title,
        Some("Title"),
        true,
        false,
        Some(44),
    );
    let mut drawing = 0;
    for (c, chapter) in manuscript.chapters.iter().enumerate() {
        render::docx_text_paragraph(
            &mut output,
            &chapter.title,
            Some("Heading1"),
            true,
            false,
            Some(32),
        );
        for (s, scene) in chapter.scenes.iter().enumerate() {
            render::docx_text_paragraph(
                &mut output,
                &scene.title,
                Some("Heading2"),
                true,
                false,
                Some(26),
            );
            render::docx_blocks(&mut output, &scene.blocks, 0);
            for item in manifest
                .placements
                .iter()
                .filter(|p| p.chapter_position == c && p.scene_position == s)
            {
                drawing += 1;
                figure(&mut output, item, manifest, drawing)?;
            }
        }
    }
    output.push_str(r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>"#);
    Ok(output)
}

fn styles() -> String {
    let mut output = format!(
        r#"<w:styles xmlns:w="{W}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/><w:color w:val="000000"/><w:sz w:val="24"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>"#
    );
    for (id, name) in [
        ("Title", "Title"),
        ("Heading1", "heading 1"),
        ("Heading2", "heading 2"),
        ("Heading3", "heading 3"),
        ("Heading4", "heading 4"),
        ("Heading5", "heading 5"),
        ("Heading6", "heading 6"),
    ] {
        let _ = write!(
            output,
            r#"<w:style w:type="paragraph" w:styleId="{id}"><w:name w:val="{name}"/><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/></w:pPr><w:rPr><w:b/><w:color w:val="000000"/></w:rPr></w:style>"#
        );
    }
    output.push_str(r#"<w:style w:type="paragraph" w:styleId="Caption"><w:name w:val="caption"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="180"/></w:pPr></w:style></w:styles>"#);
    output
}

pub(super) fn write(
    manuscript: &Manuscript,
    manifest: &Manifest,
    images: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, IllustratedError> {
    let mut entries = BTreeMap::new();
    entries.insert("[Content_Types].xml".to_owned(), br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#.to_vec());
    entries.insert("_rels/.rels".to_owned(), format!(r#"<Relationships xmlns="{REL}"><Relationship Id="office" Type="{R}/officeDocument" Target="word/document.xml"/></Relationships>"#).into_bytes());
    let mut rels = format!(
        r#"<Relationships xmlns="{REL}"><Relationship Id="styles" Type="{R}/styles" Target="styles.xml"/>"#
    );
    for asset in &manifest.assets {
        let bytes = images
            .get(&format!("assets/{}.png", asset.export_sha256))
            .ok_or(IllustratedError::Invalid)?;
        entries.insert(asset.location.clone(), bytes.clone());
        let _ = write!(
            rels,
            r#"<Relationship Id="image{}" Type="{R}/image" Target="media/{}.png"/>"#,
            asset.id, asset.export_sha256
        );
    }
    rels.push_str("</Relationships>");
    entries.insert("word/_rels/document.xml.rels".to_owned(), rels.into_bytes());
    entries.insert(
        "word/document.xml".to_owned(),
        document(manuscript, manifest)?.into_bytes(),
    );
    entries.insert("word/styles.xml".to_owned(), styles().into_bytes());
    package(&entries)
}

fn package(entries: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, IllustratedError> {
    let invalid = |_| IllustratedError::Invalid;
    if entries.len() > 64 || entries.values().map(Vec::len).sum::<usize>() > MAX_PACKAGE {
        return Err(IllustratedError::Invalid);
    }
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644)
        .last_modified_time(zip::DateTime::default());
    for (name, bytes) in entries {
        archive.start_file(name, options).map_err(invalid)?;
        archive
            .write_all(bytes)
            .map_err(|_| IllustratedError::Invalid)?;
    }
    let bytes = archive.finish().map_err(invalid)?.into_inner();
    if bytes.len() > MAX_PACKAGE {
        return Err(IllustratedError::Invalid);
    }
    // Self-read exactly the generated allowlist and bytes before the outer staging publication.
    let mut readback = ZipArchive::new(Cursor::new(&bytes)).map_err(invalid)?;
    if readback.len() != entries.len() {
        return Err(IllustratedError::Invalid);
    }
    for (name, expected) in entries {
        let mut part = readback.by_name(name).map_err(invalid)?;
        if part.size() != expected.len() as u64 {
            return Err(IllustratedError::Invalid);
        }
        let mut actual = Vec::new();
        part.read_to_end(&mut actual)
            .map_err(|_| IllustratedError::Invalid)?;
        if actual != *expected {
            return Err(IllustratedError::Invalid);
        }
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intrinsic_sizes_are_bounded_and_keep_aspect() {
        assert_eq!(extent(320, 180), (3_048_000, 1_714_500));
        assert_eq!(extent(2400, 1200), (5_943_600, 2_971_800));
        assert_eq!(extent(1200, 2400), (3_810_000, 7_620_000));
    }
}
