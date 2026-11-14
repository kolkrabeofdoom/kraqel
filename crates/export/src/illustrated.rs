//! Illustrated manuscript profile v1: static scenes, relative PNGs, immutable generations.
use crate::{ExportSourceDocument, Manuscript, render};
use kraqel_domain::{AssetTargetKind, DocumentKind, EntityId, MediaAsset, Project};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

pub const MAX_PACKAGE: usize = 128 * 1024 * 1024;

struct SourceBudget(usize);
impl std::io::Write for SourceBudget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("source limit"))?;
        if self.0 > 32 * 1024 * 1024 {
            return Err(std::io::Error::other("source limit"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IllustratedFormat {
    Markdown,
    Html,
}
impl IllustratedFormat {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Html => "html",
        }
    }
    const fn file(self) -> &'static str {
        match self {
            Self::Markdown => "manuskript.md",
            Self::Html => "manuskript.html",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImagePlacement {
    pub scene_id: EntityId,
    pub asset_id: EntityId,
    pub expected_checksum: String,
    pub expected_scene_revision: u64,
    pub alt_text: String,
    pub caption: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllustratedError {
    Invalid,
    Conflict,
    Internal,
}
impl std::fmt::Display for IllustratedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => {
                "Die Auswahl oder eine Bilddatei ist ungültig oder überschreitet die Exportgrenzen."
            }
            Self::Conflict => "Projekt oder Bildauswahl wurden zwischenzeitlich verändert.",
            Self::Internal => "Das Bildpaket konnte nicht veröffentlicht werden.",
        })
    }
}
impl std::error::Error for IllustratedError {}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn text_valid(value: &str, max: usize, multiline: bool) -> bool {
    value.chars().count() <= max
        && value
            .chars()
            .all(|c| !c.is_control() || multiline && c == '\n')
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetRecord {
    id: String,
    source_sha256: String,
    source_mime_type: String,
    source_byte_count: u64,
    source_width: u32,
    source_height: u32,
    provenance: &'static str,
    export_sha256: String,
    export_mime_type: &'static str,
    export_byte_count: usize,
    export_width: u32,
    export_height: u32,
    location: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlacementRecord {
    chapter_position: usize,
    scene_position: usize,
    position: usize,
    asset_id: String,
    title: String,
    alt_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    caption: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    format: &'static str,
    version: u8,
    manuscript_format: IllustratedFormat,
    manuscript_file: &'static str,
    assets: Vec<AssetRecord>,
    placements: Vec<PlacementRecord>,
}

pub struct PreparedPackage {
    files: BTreeMap<String, Vec<u8>>,
    format: IllustratedFormat,
    image_count: usize,
    placement_count: usize,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustratedResult {
    pub package_name: String,
    pub format: IllustratedFormat,
    pub file_count: usize,
    pub byte_count: usize,
    pub image_count: usize,
    pub placement_count: usize,
}

/// Validates a frozen model and its explicit source selection before producing an unpublished package.
/// # Errors
/// Rejects foreign/changed images, ambiguous selection, unsupported sources and all budget overruns.
pub fn prepare(
    root: &Path,
    project: &Project,
    documents: &[ExportSourceDocument],
    assets: &[MediaAsset],
    format: IllustratedFormat,
    selection: &[ImagePlacement],
) -> Result<PreparedPackage, IllustratedError> {
    let invalid = || IllustratedError::Invalid;
    if selection.is_empty() || selection.len() > 200 {
        return Err(invalid());
    }
    validate_source_budget(project, documents)?;
    let manuscript =
        Manuscript::from_export_documents(project, documents).map_err(|_| invalid())?;
    let coordinates = coordinates(documents);
    let mut pairs = BTreeSet::new();
    let mut sources = BTreeMap::new();
    for item in selection {
        if !pairs.insert((item.scene_id, item.asset_id))
            || !text_valid(&item.alt_text, 500, false)
            || item.alt_text.trim().is_empty()
            || item
                .caption
                .as_ref()
                .is_some_and(|s| !text_valid(s, 1200, true))
        {
            return Err(invalid());
        }
        let (_, _, revision) = coordinates.get(&item.scene_id).ok_or_else(invalid)?;
        if *revision != item.expected_scene_revision {
            return Err(IllustratedError::Conflict);
        }
        let asset = assets
            .iter()
            .find(|a| a.id == item.asset_id && a.project_id == project.id)
            .ok_or_else(invalid)?;
        if !asset
            .links
            .iter()
            .any(|l| l.target_kind == AssetTargetKind::Document && l.target_id == item.scene_id)
            || !text_valid(&asset.title, 240, false)
        {
            return Err(invalid());
        }
        if asset.checksum != item.expected_checksum {
            return Err(IllustratedError::Conflict);
        }
        sources.insert(asset.checksum.clone(), asset);
    }
    if sources.len() > 50 {
        return Err(invalid());
    }
    let PreparedAssets {
        mut files,
        records,
        ids,
    } = prepare_assets(root, &sources)?;
    let mut placements = Vec::new();
    let mut positions = BTreeMap::new();
    for item in selection {
        let &(c, s, _) = coordinates.get(&item.scene_id).ok_or_else(invalid)?;
        let position = positions.entry((c, s)).or_insert(0usize);
        let asset = assets
            .iter()
            .find(|a| a.id == item.asset_id)
            .ok_or_else(invalid)?;
        placements.push(PlacementRecord {
            chapter_position: c,
            scene_position: s,
            position: *position,
            asset_id: ids[&asset.checksum].clone(),
            title: asset.title.clone(),
            alt_text: item.alt_text.trim().to_owned(),
            caption: item.caption.clone(),
        });
        *position += 1;
    }
    placements.sort_by_key(|p| (p.chapter_position, p.scene_position, p.position));
    let manifest = Manifest {
        format: "kraqel-illustrated-manuscript",
        version: 1,
        manuscript_format: format,
        manuscript_file: format.file(),
        assets: records,
        placements,
    };
    let document = render_document(&manuscript, &manifest)?;
    let value = serde_json::to_value(&manifest).map_err(|_| invalid())?;
    // serde_json's default Map uses sorted keys. No UUIDs or dates enter this manifest.
    let mut json = serde_json::to_vec_pretty(&value).map_err(|_| invalid())?;
    json.push(b'\n');
    if json.len() > 1024 * 1024 {
        return Err(invalid());
    }
    files.insert("medien.json".to_owned(), json);
    files.insert(format.file().to_owned(), document.into_bytes());
    if files.len() > 64 || files.values().map(Vec::len).sum::<usize>() > MAX_PACKAGE {
        return Err(invalid());
    }
    Ok(PreparedPackage {
        files,
        format,
        image_count: manifest.assets.len(),
        placement_count: manifest.placements.len(),
    })
}

fn validate_source_budget(
    project: &Project,
    documents: &[ExportSourceDocument],
) -> Result<(), IllustratedError> {
    if documents.len() > 10000 {
        return Err(IllustratedError::Invalid);
    }
    let mut budget = SourceBudget(0);
    budget
        .write_all(project.name.as_bytes())
        .map_err(|_| IllustratedError::Invalid)?;
    for document in documents {
        budget
            .write_all(document.node.title.as_bytes())
            .map_err(|_| IllustratedError::Invalid)?;
        serde_json::to_writer(&mut budget, &document.content)
            .map_err(|_| IllustratedError::Invalid)?;
    }
    Ok(())
}

fn coordinates(documents: &[ExportSourceDocument]) -> BTreeMap<EntityId, (usize, usize, u64)> {
    let mut chapters: Vec<_> = documents
        .iter()
        .filter(|d| d.node.kind == DocumentKind::Chapter)
        .collect();
    chapters.sort_by_key(|d| (d.node.position, d.node.id.to_string()));
    let mut coordinates = BTreeMap::new();
    for (c, chapter) in chapters.iter().enumerate() {
        let mut scenes: Vec<_> = documents
            .iter()
            .filter(|d| d.node.parent_id == Some(chapter.node.id))
            .collect();
        scenes.sort_by_key(|d| (d.node.position, d.node.id.to_string()));
        for (s, scene) in scenes.iter().enumerate() {
            coordinates.insert(scene.node.id, (c, s, scene.node.revision));
        }
    }
    coordinates
}
struct PreparedAssets {
    files: BTreeMap<String, Vec<u8>>,
    records: Vec<AssetRecord>,
    ids: BTreeMap<String, String>,
}
fn prepare_assets(
    root: &Path,
    sources: &BTreeMap<String, &MediaAsset>,
) -> Result<PreparedAssets, IllustratedError> {
    let invalid = || IllustratedError::Invalid;
    let mut files = BTreeMap::new();
    let mut records = Vec::new();
    let mut source_bytes = 0u64;
    let mut pixels = 0u64;
    let mut ids = BTreeMap::new();
    for (index, (checksum, asset)) in sources.iter().enumerate() {
        source_bytes = source_bytes
            .checked_add(asset.byte_count)
            .ok_or_else(invalid)?;
        pixels = pixels
            .checked_add(u64::from(asset.width) * u64::from(asset.height))
            .ok_or_else(invalid)?;
        if source_bytes > MAX_PACKAGE as u64 || pixels > 200_000_000 {
            return Err(invalid());
        }
        let bytes = kraqel_asset_service::read_export_original(root, &asset.relative_path)
            .map_err(|_| invalid())?;
        if bytes.len() as u64 != asset.byte_count || hash(&bytes) != **checksum {
            return Err(invalid());
        }
        let image = kraqel_asset_service::prepare_export_image(&bytes, &asset.mime_type)
            .map_err(|_| invalid())?;
        if image.source_width != asset.width || image.source_height != asset.height {
            return Err(invalid());
        }
        let export_sha256 = hash(&image.png);
        let location = format!("assets/{export_sha256}.png");
        let id = format!("m{:04}", index + 1);
        let record = AssetRecord {
            id: id.clone(),
            source_sha256: (*checksum).clone(),
            source_mime_type: asset.mime_type.clone(),
            source_byte_count: asset.byte_count,
            source_width: asset.width,
            source_height: asset.height,
            provenance: "imported",
            export_sha256,
            export_mime_type: "image/png",
            export_byte_count: image.png.len(),
            export_width: image.width,
            export_height: image.height,
            location: location.clone(),
        };
        files.insert(location, image.png);
        ids.insert((*checksum).clone(), id);
        records.push(record);
        if files.values().map(Vec::len).sum::<usize>() > MAX_PACKAGE {
            return Err(invalid());
        }
    }
    Ok(PreparedAssets {
        files,
        records,
        ids,
    })
}
fn md(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| {
            if c.is_ascii_punctuation() {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}
fn render_document(
    manuscript: &Manuscript,
    manifest: &Manifest,
) -> Result<String, IllustratedError> {
    let html = manifest.manuscript_format == IllustratedFormat::Html;
    let escape = |s: &str| if html { render::escape_html(s) } else { md(s) };
    let mut body = if html {
        format!("<h1>{}</h1>\n", escape(&manuscript.title))
    } else {
        format!("# {}\n\n", escape(&manuscript.title))
    };
    for (c, chapter) in manuscript.chapters.iter().enumerate() {
        if html {
            let _ = writeln!(body, "<section><h2>{}</h2>", escape(&chapter.title));
        } else {
            let _ = write!(body, "## {}\n\n", escape(&chapter.title));
        }
        for (s, scene) in chapter.scenes.iter().enumerate() {
            if html {
                let _ = writeln!(body, "<article><h3>{}</h3>", escape(&scene.title));
            } else {
                let _ = write!(body, "### {}\n\n", escape(&scene.title));
            }
            if html {
                render::html_blocks(&mut body, &scene.blocks);
            } else {
                render::markdown_blocks(&mut body, &scene.blocks, 0, "");
            }
            for item in manifest
                .placements
                .iter()
                .filter(|p| p.chapter_position == c && p.scene_position == s)
            {
                let asset = manifest
                    .assets
                    .iter()
                    .find(|a| a.id == item.asset_id)
                    .ok_or(IllustratedError::Invalid)?;
                if html {
                    let _ = write!(
                        body,
                        "<figure><img src=\"{}\" alt=\"{}\" width=\"{}\" height=\"{}\">",
                        asset.location,
                        escape(&item.alt_text),
                        asset.export_width,
                        asset.export_height
                    );
                    if let Some(caption) = &item.caption {
                        let _ = write!(
                            body,
                            "<figcaption>{}</figcaption>",
                            escape(caption).replace('\n', "<br>")
                        );
                    }
                    body.push_str("</figure>\n");
                } else {
                    let _ = write!(
                        body,
                        "![{}]({})\n\n",
                        escape(&item.alt_text),
                        asset.location
                    );
                    if let Some(caption) = &item.caption {
                        body.push_str(&escape(caption).replace('\n', "  \n"));
                        body.push_str("\n\n");
                    }
                }
            }
            if html {
                body.push_str("</article>\n");
            }
        }
        if html {
            body.push_str("</section>\n");
        }
    }
    if html {
        Ok(format!(
            "<!doctype html>\n<html lang=\"de\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src 'self'; style-src 'unsafe-inline'\"><title>{}</title><style>body{{max-width:44rem;margin:3rem auto;padding:0 1rem;font:1rem/1.6 Georgia,serif;color:#182829}}img{{max-width:100%;height:auto}}figure{{margin:2rem 0;break-inside:avoid}}figcaption{{white-space:pre-wrap}}blockquote{{border-left:2px solid #888;padding-left:1rem}}</style></head><body>{body}</body></html>\n",
            escape(&manuscript.title)
        ))
    } else {
        Ok(body)
    }
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Publishes one validated package as a new directory after the caller's final state check.
/// # Errors
/// Rejects unsafe destinations, changed staging bytes and failed final validation/publication.
pub fn publish(
    root: &Path,
    package: &PreparedPackage,
    check: impl FnOnce() -> Result<(), IllustratedError>,
) -> Result<IllustratedResult, IllustratedError> {
    let directory = crate::prepare_export_directory(root)
        .and_then(|p| crate::prepare_child_directory(&p, "mit-bildern"))
        .map_err(|_| IllustratedError::Internal)?;
    let generation = format!("{}-{}", package.format.token(), EntityId::new());
    let temporary = directory.join(format!(".new-{generation}"));
    let destination = directory.join(&generation);
    fs::create_dir(&temporary).map_err(|_| IllustratedError::Internal)?;
    let staging = Staging(temporary);
    fs::create_dir(staging.0.join("assets")).map_err(|_| IllustratedError::Internal)?;
    let mut total = 0;
    for (name, bytes) in &package.files {
        let path = staging.0.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| IllustratedError::Internal)?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| IllustratedError::Internal)?;
        if fs::read(&path).map_err(|_| IllustratedError::Internal)? != *bytes {
            return Err(IllustratedError::Internal);
        }
        total += bytes.len();
    }
    check()?;
    if fs::symlink_metadata(&destination).is_ok() {
        return Err(IllustratedError::Internal);
    }
    fs::rename(&staging.0, &destination).map_err(|_| IllustratedError::Internal)?;
    Ok(IllustratedResult {
        package_name: format!("mit-bildern/{generation}"),
        format: package.format,
        file_count: package.files.len(),
        byte_count: total,
        image_count: package.image_count,
        placement_count: package.placement_count,
    })
}
