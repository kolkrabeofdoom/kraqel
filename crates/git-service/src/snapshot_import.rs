//! Sicherer, nicht-destruktiver Import portabler Snapshot-v3-Archive.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Cursor, Read as _},
    path::Path,
};

use kraqel_asset_service::{PreparedImage, prepare_image, store_image};
use kraqel_domain::{
    CURRENT_SCHEMA_VERSION, EDITOR_DOCUMENT_VERSION, EditorDocument, EntityId, Project, ProjectId,
};
use kraqel_storage::{
    Database,
    project_manager::{ManagedProject, ProjectError, ProjectManager},
};
use rusqlite::{Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;
use zip::ZipArchive;

use crate::GitServiceError;

pub const MAX_SNAPSHOT_ARCHIVE_BYTES: usize = 128 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 256;
const MAX_JSON_BYTES: usize = 4 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 2 * 1024 * 1024;
const FIXED_FILES: [&str; 9] = [
    "manifest.json",
    "schema.json",
    "documents.json",
    "story-bible.json",
    "plotboard.json",
    "timelines.json",
    "visual-profiles.json",
    "prompt-drafts.json",
    "README.md",
];

/// Zusammenfassung einer neu angelegten Snapshotkopie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub project_id: ProjectId,
    pub name: String,
    pub document_count: u32,
    pub entity_count: u32,
    pub asset_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportManifest {
    format: String,
    format_version: u16,
    project: ImportProject,
    files: Vec<String>,
    assets: Vec<ImportAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportProject {
    id: String,
    name: String,
    schema_version: u32,
    settings: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportAsset {
    id: String,
    kind: String,
    path: String,
    original_name: String,
    title: String,
    mime_type: String,
    checksum: String,
    byte_count: u64,
    width: u32,
    height: u32,
    provenance: String,
    tags: Vec<String>,
    links: Vec<ImportAssetLink>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImportAssetLink {
    target_kind: String,
    target_id: String,
    role: String,
}

struct SnapshotData {
    manifest: ImportManifest,
    documents: Vec<Value>,
    story: Value,
    plotboard: Value,
    timelines: Value,
    visuals: Value,
    prompts: Value,
    prepared_assets: BTreeMap<String, PreparedImage>,
    entity_ids: BTreeSet<String>,
}

/// Importiert ein begrenztes Snapshot-v3-ZIP stets als neues, atomar veröffentlichtes Projekt.
///
/// # Errors
/// Liefert einen datenarmen Fehler, wenn Archiv, Inhalte, Referenzen oder Bilder ungültig sind oder
/// das neue lokale Projekt nicht sicher veröffentlicht werden kann.
pub fn import_snapshot_archive(
    manager: &ProjectManager,
    archive: &[u8],
) -> Result<(ManagedProject, ImportSummary), GitServiceError> {
    let data = parse_snapshot(archive)?;
    let project_id = ProjectId::new();
    let project = Project {
        id: project_id,
        name: data.manifest.project.name.clone(),
        schema_version: CURRENT_SCHEMA_VERSION,
    };
    let relative = project_id.to_string();
    let document_count =
        u32::try_from(data.documents.len()).map_err(|_| GitServiceError::InvalidSnapshot)?;
    let entity_count =
        u32::try_from(data.entity_ids.len()).map_err(|_| GitServiceError::InvalidSnapshot)?;
    let asset_count =
        u32::try_from(data.manifest.assets.len()).map_err(|_| GitServiceError::InvalidSnapshot)?;
    let name = project.name.clone();
    let opened = manager
        .create_project_with_initializer(&relative, &project, |database, directory| {
            populate_project(database, directory, project_id, &data)
        })
        .map_err(GitServiceError::Project)?;
    Ok((
        opened,
        ImportSummary {
            project_id,
            name,
            document_count,
            entity_count,
            asset_count,
        },
    ))
}

/// Liest eine reguläre lokale Archivdatei unter derselben harten Größenbegrenzung und importiert
/// sie stets als neues Projekt.
///
/// # Errors
/// Liefert einen datenarmen Fehler, wenn der ausgewählte Pfad kein unverknüpftes reguläres File
/// ist, sich während des Lesens ändert, die Größenbegrenzung verletzt oder kein gültiges
/// Snapshot-v3-Archiv enthält.
pub fn import_snapshot_archive_file(
    manager: &ProjectManager,
    path: &Path,
) -> Result<(ManagedProject, ImportSummary), GitServiceError> {
    let metadata = fs::symlink_metadata(path).map_err(GitServiceError::Io)?;
    let maximum =
        u64::try_from(MAX_SNAPSHOT_ARCHIVE_BYTES).map_err(|_| GitServiceError::InvalidSnapshot)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(GitServiceError::InvalidSnapshot);
    }

    let file = File::open(path).map_err(GitServiceError::Io)?;
    let opened_metadata = file.metadata().map_err(GitServiceError::Io)?;
    if !opened_metadata.is_file() || opened_metadata.len() != metadata.len() {
        return Err(GitServiceError::InvalidSnapshot);
    }
    let expected =
        usize::try_from(opened_metadata.len()).map_err(|_| GitServiceError::InvalidSnapshot)?;
    let mut archive = Vec::with_capacity(expected);
    file.take(maximum + 1)
        .read_to_end(&mut archive)
        .map_err(GitServiceError::Io)?;
    if archive.len() != expected || archive.len() > MAX_SNAPSHOT_ARCHIVE_BYTES {
        return Err(GitServiceError::InvalidSnapshot);
    }
    import_snapshot_archive(manager, &archive)
}

fn parse_snapshot(archive: &[u8]) -> Result<SnapshotData, GitServiceError> {
    let files = read_archive(archive)?;
    let manifest: ImportManifest = parse_json_file(&files, "manifest.json")?;
    if manifest.format != "kraqel-project-snapshot"
        || manifest.format_version != 3
        || manifest.project.schema_version != CURRENT_SCHEMA_VERSION.0
        || canonical_project_id(&manifest.project.id).is_none()
        || !valid_name(&manifest.project.name, 120)
        || !manifest.project.settings.is_object()
        || manifest.files != FIXED_FILES
    {
        return Err(GitServiceError::InvalidSnapshot);
    }
    if files.get("schema.json").map(Vec::as_slice)
        != Some(include_bytes!(
            "../../../docs/data-format/snapshot-v3.schema.json"
        ))
    {
        return Err(GitServiceError::InvalidSnapshot);
    }
    require_utf8(&files, "README.md")?;

    let documents_root: Value = parse_json_file(&files, "documents.json")?;
    require_kind(&documents_root, "documents", &["kind", "documents"])?;
    let documents = require_array_field(&documents_root, "documents")?.clone();
    let document_ids = validate_documents(&documents)?;

    let story: Value = parse_json_file(&files, "story-bible.json")?;
    require_kind(
        &story,
        "storyBible",
        &["kind", "entities", "relations", "links"],
    )?;
    let entity_ids = validate_story(&story, &document_ids)?;

    let plotboard: Value = parse_json_file(&files, "plotboard.json")?;
    require_kind(&plotboard, "plotboard", &["kind", "storylines", "cards"])?;
    validate_plotboard(&plotboard, &document_ids, &entity_ids)?;

    let timelines: Value = parse_json_file(&files, "timelines.json")?;
    require_kind(
        &timelines,
        "timelines",
        &[
            "kind",
            "timelines",
            "events",
            "documentLinks",
            "entityLinks",
        ],
    )?;
    let event_ids = validate_timelines(&timelines, &document_ids, &entity_ids)?;

    let prepared_assets = validate_assets(
        &manifest.assets,
        &files,
        &document_ids,
        &entity_ids,
        &event_ids,
    )?;

    let visuals: Value = parse_json_file(&files, "visual-profiles.json")?;
    require_kind(
        &visuals,
        "visualProfiles",
        &["kind", "profiles", "stylePresets"],
    )?;
    let (profile_ids, preset_ids) = validate_visuals(&visuals, &entity_ids, &manifest.assets)?;

    let prompts: Value = parse_json_file(&files, "prompt-drafts.json")?;
    require_kind(&prompts, "imagePromptDrafts", &["kind", "drafts"])?;
    validate_prompts(&prompts, &document_ids, &profile_ids, &preset_ids)?;

    let mut expected = FIXED_FILES
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    for id in &document_ids {
        let name = format!("manuscript/{id}.md");
        require_utf8(&files, &name)?;
        expected.insert(name);
    }
    expected.extend(manifest.assets.iter().map(|asset| asset.path.clone()));
    if files.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(GitServiceError::InvalidSnapshot);
    }

    Ok(SnapshotData {
        manifest,
        documents,
        story,
        plotboard,
        timelines,
        visuals,
        prompts,
        prepared_assets,
        entity_ids,
    })
}

fn read_archive(archive: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, GitServiceError> {
    if archive.is_empty() || archive.len() > MAX_SNAPSHOT_ARCHIVE_BYTES {
        return Err(GitServiceError::InvalidSnapshot);
    }
    let mut zip =
        ZipArchive::new(Cursor::new(archive)).map_err(|_| GitServiceError::InvalidSnapshot)?;
    if zip.is_empty() || zip.len() > MAX_ARCHIVE_FILES {
        return Err(GitServiceError::InvalidSnapshot);
    }
    let mut files = BTreeMap::new();
    let mut total = 0_u64;
    for index in 0..zip.len() {
        let file = zip
            .by_index(index)
            .map_err(|_| GitServiceError::InvalidSnapshot)?;
        if file.is_dir() || file.is_symlink() {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let enclosed = file
            .enclosed_name()
            .ok_or(GitServiceError::InvalidSnapshot)?;
        let path = enclosed
            .to_str()
            .map(|path| path.replace('\\', "/"))
            .ok_or(GitServiceError::InvalidSnapshot)?;
        if path.is_empty() || path.len() > 320 || path.starts_with('/') || path.contains("//") {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let extension = Path::new(&path)
            .extension()
            .and_then(|value| value.to_str());
        let limit = if extension == Some("json") {
            MAX_JSON_BYTES
        } else if extension == Some("md") {
            MAX_TEXT_BYTES
        } else {
            kraqel_asset_service::MAX_ASSET_BYTES
        };
        let size = usize::try_from(file.size()).map_err(|_| GitServiceError::InvalidSnapshot)?;
        if size == 0 || size > limit {
            return Err(GitServiceError::InvalidSnapshot);
        }
        total = total
            .checked_add(file.size())
            .ok_or(GitServiceError::InvalidSnapshot)?;
        if total > MAX_SNAPSHOT_ARCHIVE_BYTES as u64 {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let mut bytes = Vec::with_capacity(size);
        file.take(u64::try_from(limit + 1).map_err(|_| GitServiceError::InvalidSnapshot)?)
            .read_to_end(&mut bytes)?;
        if bytes.len() != size || files.insert(path, bytes).is_some() {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(files)
}

fn parse_json_file<T: serde::de::DeserializeOwned>(
    files: &BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<T, GitServiceError> {
    serde_json::from_slice(files.get(name).ok_or(GitServiceError::InvalidSnapshot)?)
        .map_err(|_| GitServiceError::InvalidSnapshot)
}

fn require_utf8(files: &BTreeMap<String, Vec<u8>>, name: &str) -> Result<(), GitServiceError> {
    std::str::from_utf8(files.get(name).ok_or(GitServiceError::InvalidSnapshot)?)
        .map(|_| ())
        .map_err(|_| GitServiceError::InvalidSnapshot)
}

fn validate_documents(documents: &[Value]) -> Result<BTreeSet<String>, GitServiceError> {
    let mut ids = BTreeSet::new();
    let mut chapters = BTreeSet::new();
    for document in documents {
        let object = exact_object(
            document,
            &[
                "id",
                "parentId",
                "kind",
                "title",
                "position",
                "revision",
                "content",
                "plainText",
            ],
        )?;
        let id = canonical_entity_id(string(object, "id")?)?;
        if !ids.insert(id.clone()) {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let kind = string(object, "kind")?;
        if !matches!(kind, "chapter" | "scene")
            || string(object, "title")?.chars().count() > 240
            || string(object, "title")?.contains('\0')
            || unsigned(object, "position").is_none()
            || unsigned(object, "revision").is_none()
            || object.get("plainText").and_then(Value::as_str).is_none()
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let editor: EditorDocument = serde_json::from_value(
            object
                .get("content")
                .cloned()
                .ok_or(GitServiceError::InvalidSnapshot)?,
        )
        .map_err(|_| GitServiceError::InvalidSnapshot)?;
        if editor.version != EDITOR_DOCUMENT_VERSION
            || editor.document.get("type").and_then(Value::as_str) != Some("doc")
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        if kind == "chapter" {
            if !object.get("parentId").is_some_and(Value::is_null) {
                return Err(GitServiceError::InvalidSnapshot);
            }
            chapters.insert(id);
        }
    }
    for document in documents {
        let object = document
            .as_object()
            .ok_or(GitServiceError::InvalidSnapshot)?;
        if string(object, "kind")? == "scene"
            && !object
                .get("parentId")
                .and_then(Value::as_str)
                .is_some_and(|id| chapters.contains(id))
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(ids)
}

fn validate_story(
    story: &Value,
    document_ids: &BTreeSet<String>,
) -> Result<BTreeSet<String>, GitServiceError> {
    let entities = require_array_field(story, "entities")?;
    let mut ids = BTreeSet::new();
    for entity in entities {
        let object = exact_object(entity, &["id", "kind", "name", "properties"])?;
        let id = canonical_entity_id(string(object, "id")?)?;
        if !ids.insert(id)
            || !matches!(
                string(object, "kind")?,
                "character" | "location" | "faction" | "item"
            )
            || !valid_name(string(object, "name")?, 240)
            || !valid_string_map(object.get("properties"), 32, 80, 500)
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    let mut relation_ids = BTreeSet::new();
    for relation in require_array_field(story, "relations")? {
        let object = exact_object(
            relation,
            &[
                "id",
                "sourceEntityId",
                "targetEntityId",
                "kind",
                "properties",
            ],
        )?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let source = string(object, "sourceEntityId")?;
        let target = string(object, "targetEntityId")?;
        if !relation_ids.insert(id)
            || source == target
            || !ids.contains(source)
            || !ids.contains(target)
            || !valid_name(string(object, "kind")?, 120)
            || !valid_string_map(object.get("properties"), 32, 80, 500)
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    let mut links = BTreeSet::new();
    for link in require_array_field(story, "links")? {
        let object = exact_object(link, &["documentId", "entityId", "role"])?;
        let document = string(object, "documentId")?;
        let entity = string(object, "entityId")?;
        let role = string(object, "role")?;
        if !document_ids.contains(document)
            || !ids.contains(entity)
            || !valid_name(role, 80)
            || !links.insert((document.to_owned(), entity.to_owned(), role.to_owned()))
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(ids)
}

fn validate_plotboard(
    plotboard: &Value,
    document_ids: &BTreeSet<String>,
    entity_ids: &BTreeSet<String>,
) -> Result<(), GitServiceError> {
    let mut storyline_ids = BTreeSet::new();
    for storyline in require_array_field(plotboard, "storylines")? {
        let object = exact_object(storyline, &["id", "name", "color", "position"])?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let color = string(object, "color")?;
        if !storyline_ids.insert(id)
            || !valid_name(string(object, "name")?, 120)
            || color.len() != 7
            || !color.starts_with('#')
            || !color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
            || unsigned(object, "position").is_none()
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    let mut card_ids = BTreeSet::new();
    for card in require_array_field(plotboard, "cards")? {
        let object = exact_object(
            card,
            &[
                "documentId",
                "storylineId",
                "povEntityId",
                "status",
                "position",
            ],
        )?;
        let document = string(object, "documentId")?;
        if !document_ids.contains(document)
            || !card_ids.insert(document.to_owned())
            || !optional_reference(object.get("storylineId"), &storyline_ids)
            || !optional_reference(object.get("povEntityId"), entity_ids)
            || !matches!(
                string(object, "status")?,
                "planned" | "draft" | "revised" | "done"
            )
            || unsigned(object, "position").is_none()
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(())
}

fn validate_timelines(
    timelines: &Value,
    document_ids: &BTreeSet<String>,
    entity_ids: &BTreeSet<String>,
) -> Result<BTreeSet<String>, GitServiceError> {
    let mut timeline_ids = BTreeSet::new();
    let mut modes = BTreeMap::new();
    for timeline in require_array_field(timelines, "timelines")? {
        let object = exact_object(timeline, &["id", "name", "mode", "position"])?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let mode = string(object, "mode")?;
        if !timeline_ids.insert(id.clone())
            || !valid_name(string(object, "name")?, 120)
            || !matches!(mode, "relative" | "absolute")
            || unsigned(object, "position").is_none()
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        modes.insert(id, mode.to_owned());
    }
    let mut event_ids = BTreeSet::new();
    for event in require_array_field(timelines, "events")? {
        let object = exact_object(
            event,
            &[
                "id",
                "timelineId",
                "title",
                "description",
                "timeValue",
                "position",
            ],
        )?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let timeline = string(object, "timelineId")?;
        let time = optional_string(object.get("timeValue"))?;
        if !event_ids.insert(id)
            || !timeline_ids.contains(timeline)
            || !valid_name(string(object, "title")?, 240)
            || object
                .get("description")
                .and_then(Value::as_str)
                .is_none_or(|value| value.chars().count() > 8_000)
            || modes.get(timeline).is_some_and(|mode| mode == "relative") && time.is_some()
            || modes.get(timeline).is_some_and(|mode| mode == "absolute")
                && time.is_none_or(|value| !valid_name(value, 80))
            || unsigned(object, "position").is_none()
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    validate_pair_links(
        require_array_field(timelines, "documentLinks")?,
        "documentId",
        &event_ids,
        document_ids,
    )?;
    validate_pair_links(
        require_array_field(timelines, "entityLinks")?,
        "entityId",
        &event_ids,
        entity_ids,
    )?;
    Ok(event_ids)
}

fn validate_pair_links(
    links: &[Value],
    target_name: &str,
    event_ids: &BTreeSet<String>,
    target_ids: &BTreeSet<String>,
) -> Result<(), GitServiceError> {
    let mut seen = BTreeSet::new();
    for link in links {
        let object = exact_object(link, &["eventId", target_name])?;
        let event = string(object, "eventId")?;
        let target = string(object, target_name)?;
        if !event_ids.contains(event)
            || !target_ids.contains(target)
            || !seen.insert((event.to_owned(), target.to_owned()))
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(())
}

fn validate_assets(
    assets: &[ImportAsset],
    files: &BTreeMap<String, Vec<u8>>,
    documents: &BTreeSet<String>,
    entities: &BTreeSet<String>,
    events: &BTreeSet<String>,
) -> Result<BTreeMap<String, PreparedImage>, GitServiceError> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut checksums = BTreeSet::new();
    let mut prepared_assets = BTreeMap::new();
    for asset in assets {
        let id = canonical_entity_id(&asset.id)?;
        if !ids.insert(id)
            || asset.kind != "image"
            || asset.provenance != "imported"
            || !valid_name(&asset.original_name, 240)
            || !valid_name(&asset.title, 240)
            || !matches!(
                asset.mime_type.as_str(),
                "image/png" | "image/jpeg" | "image/webp"
            )
            || !paths.insert(asset.path.clone())
            || !checksums.insert(asset.checksum.clone())
            || !valid_tags(&asset.tags)
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let extension = match asset.mime_type.as_str() {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            _ => return Err(GitServiceError::InvalidSnapshot),
        };
        if asset.path != format!("assets/{}.{}", asset.checksum, extension)
            || asset.checksum.len() != 64
            || !asset
                .checksum
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        let bytes = files
            .get(&asset.path)
            .ok_or(GitServiceError::InvalidSnapshot)?;
        let prepared = prepare_image(&asset.original_name, bytes)
            .map_err(|_| GitServiceError::InvalidSnapshot)?;
        if prepared.checksum != asset.checksum
            || prepared.mime_type != asset.mime_type
            || prepared.width != asset.width
            || prepared.height != asset.height
            || asset.byte_count != bytes.len() as u64
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
        prepared_assets.insert(asset.id.clone(), prepared);
        let mut links = BTreeSet::new();
        for link in &asset.links {
            let targets = match link.target_kind.as_str() {
                "document" => documents,
                "entity" => entities,
                "event" => events,
                _ => return Err(GitServiceError::InvalidSnapshot),
            };
            if !targets.contains(&link.target_id)
                || !valid_name(&link.role, 80)
                || !links.insert((
                    link.target_kind.clone(),
                    link.target_id.clone(),
                    link.role.clone(),
                ))
            {
                return Err(GitServiceError::InvalidSnapshot);
            }
        }
    }
    Ok(prepared_assets)
}

fn validate_visuals(
    visuals: &Value,
    entities: &BTreeSet<String>,
    assets: &[ImportAsset],
) -> Result<(BTreeSet<String>, BTreeSet<String>), GitServiceError> {
    let asset_ids = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<BTreeSet<_>>();
    let mut profiles = BTreeSet::new();
    for profile in require_array_field(visuals, "profiles")? {
        let object = exact_object(
            profile,
            &["entityId", "traits", "canonicalAssetId", "exclusions"],
        )?;
        let entity = string(object, "entityId")?;
        if !entities.contains(entity)
            || !profiles.insert(entity.to_owned())
            || !valid_string_map(object.get("traits"), 32, 80, 500)
            || !optional_reference(object.get("canonicalAssetId"), &asset_ids)
            || !valid_string_array(object.get("exclusions"), 32, 500, true)
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    let mut presets = BTreeSet::new();
    let mut names = BTreeSet::new();
    for preset in require_array_field(visuals, "stylePresets")? {
        let object = exact_object(preset, &["id", "name", "visualRules", "applicableKinds"])?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let name = string(object, "name")?;
        if !presets.insert(id)
            || !names.insert(name.to_ascii_lowercase())
            || !valid_name(name, 120)
            || !valid_string_map(object.get("visualRules"), 32, 80, 500)
            || !valid_kinds(object.get("applicableKinds"))
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok((profiles, presets))
}

fn validate_prompts(
    prompts: &Value,
    documents: &BTreeSet<String>,
    profiles: &BTreeSet<String>,
    presets: &BTreeSet<String>,
) -> Result<(), GitServiceError> {
    let mut ids = BTreeSet::new();
    for draft in require_array_field(prompts, "drafts")? {
        let object = exact_object(
            draft,
            &[
                "id",
                "name",
                "sourceDocumentId",
                "sourceDocumentRevision",
                "entityIds",
                "stylePresetId",
                "promptText",
            ],
        )?;
        let id = canonical_entity_id(string(object, "id")?)?;
        let entity_ids = object
            .get("entityIds")
            .and_then(Value::as_array)
            .ok_or(GitServiceError::InvalidSnapshot)?;
        if !ids.insert(id)
            || !valid_name(string(object, "name")?, 120)
            || !documents.contains(string(object, "sourceDocumentId")?)
            || unsigned(object, "sourceDocumentRevision").is_none()
            || entity_ids.is_empty()
            || entity_ids.len() > 32
            || entity_ids
                .iter()
                .any(|id| id.as_str().is_none_or(|id| !profiles.contains(id)))
            || entity_ids
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
                .len()
                != entity_ids.len()
            || !optional_reference(object.get("stylePresetId"), presets)
            || object
                .get("promptText")
                .and_then(Value::as_str)
                .is_none_or(|value| value.trim().is_empty() || value.chars().count() > 200_000)
        {
            return Err(GitServiceError::InvalidSnapshot);
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "Die lineare Importreihenfolge bildet Fremdschlüssel und Rollbackgrenzen sichtbar ab"
)]
fn populate_project(
    database: &mut Database,
    directory: &std::path::Path,
    project_id: ProjectId,
    data: &SnapshotData,
) -> Result<(), ProjectError> {
    let project = project_id.to_string();
    let settings = serde_json::to_string(&data.manifest.project.settings)
        .map_err(|_| ProjectError::InvalidImportedData)?;
    database.connection().execute(
        "UPDATE project SET settings_json = ?1 WHERE id = ?2",
        params![settings, project],
    )?;

    let mut stored_assets = BTreeMap::new();
    for asset in &data.manifest.assets {
        let prepared = data
            .prepared_assets
            .get(&asset.id)
            .ok_or(ProjectError::InvalidImportedData)?;
        let id = EntityId::parse(&asset.id).ok_or(ProjectError::InvalidImportedData)?;
        let paths =
            store_image(directory, id, prepared).map_err(|_| ProjectError::InvalidImportedData)?;
        stored_assets.insert(asset.id.clone(), paths);
    }

    let transaction = database.transaction()?;
    insert_documents(&transaction, &project, &data.documents)?;
    insert_story(&transaction, &project, &data.story)?;
    insert_plotboard(&transaction, &project, &data.plotboard)?;
    insert_timelines(&transaction, &project, &data.timelines)?;
    insert_assets(
        &transaction,
        &project,
        &data.manifest.assets,
        &stored_assets,
    )?;
    insert_visuals(&transaction, &project, &data.visuals)?;
    insert_prompts(&transaction, &project, &data.prompts)?;
    transaction.commit()?;
    Ok(())
}

fn insert_documents(
    transaction: &Transaction<'_>,
    project: &str,
    documents: &[Value],
) -> Result<(), ProjectError> {
    for expected_kind in ["chapter", "scene"] {
        for document in documents {
            let object = document
                .as_object()
                .ok_or(ProjectError::InvalidImportedData)?;
            if string_project(object, "kind")? != expected_kind {
                continue;
            }
            transaction.execute(
                "INSERT INTO document (
                    id, project_id, parent_id, kind, title, position, revision, content_json,
                    plain_text, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'import', 'import')",
                params![
                    string_project(object, "id")?,
                    project,
                    optional_string_project(object.get("parentId"))?,
                    expected_kind,
                    string_project(object, "title")?,
                    integer_project(object, "position")?,
                    integer_project(object, "revision")?,
                    serde_json::to_string(
                        object
                            .get("content")
                            .ok_or(ProjectError::InvalidImportedData)?
                    )
                    .map_err(|_| ProjectError::InvalidImportedData)?,
                    string_project(object, "plainText")?,
                ],
            )?;
        }
    }
    Ok(())
}

fn insert_story(
    transaction: &Transaction<'_>,
    project: &str,
    story: &Value,
) -> Result<(), ProjectError> {
    for entity in array_project(story, "entities")? {
        let object = entity
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO entity (id, project_id, kind, name, properties_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "kind")?,
                string_project(object, "name")?,
                json_text_project(object, "properties")?,
            ],
        )?;
    }
    for relation in array_project(story, "relations")? {
        let object = relation
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO relation (
                id, project_id, source_entity_id, target_entity_id, kind, properties_json,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "sourceEntityId")?,
                string_project(object, "targetEntityId")?,
                string_project(object, "kind")?,
                json_text_project(object, "properties")?,
            ],
        )?;
    }
    for link in array_project(story, "links")? {
        let object = link.as_object().ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO link (id, project_id, document_id, entity_id, role, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'import')",
            params![
                Uuid::new_v4().to_string(),
                project,
                string_project(object, "documentId")?,
                string_project(object, "entityId")?,
                string_project(object, "role")?,
            ],
        )?;
    }
    Ok(())
}

fn insert_plotboard(
    transaction: &Transaction<'_>,
    project: &str,
    plotboard: &Value,
) -> Result<(), ProjectError> {
    for storyline in array_project(plotboard, "storylines")? {
        let object = storyline
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO storyline (id, project_id, name, color, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "name")?,
                string_project(object, "color")?,
                integer_project(object, "position")?,
            ],
        )?;
    }
    for card in array_project(plotboard, "cards")? {
        let object = card.as_object().ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO plot_card (
                project_id, document_id, storyline_id, pov_entity_id, status, position, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'import')",
            params![
                project,
                string_project(object, "documentId")?,
                optional_string_project(object.get("storylineId"))?,
                optional_string_project(object.get("povEntityId"))?,
                string_project(object, "status")?,
                integer_project(object, "position")?,
            ],
        )?;
    }
    Ok(())
}

fn insert_timelines(
    transaction: &Transaction<'_>,
    project: &str,
    timelines: &Value,
) -> Result<(), ProjectError> {
    for timeline in array_project(timelines, "timelines")? {
        let object = timeline
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO timeline (id, project_id, name, mode, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "name")?,
                string_project(object, "mode")?,
                integer_project(object, "position")?,
            ],
        )?;
    }
    for event in array_project(timelines, "events")? {
        let object = event.as_object().ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO event (
                id, project_id, timeline_id, title, description, time_value, position,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "timelineId")?,
                string_project(object, "title")?,
                string_project(object, "description")?,
                optional_string_project(object.get("timeValue"))?,
                integer_project(object, "position")?,
            ],
        )?;
    }
    for (field, table, target) in [
        ("documentLinks", "event_document_link", "document_id"),
        ("entityLinks", "event_entity_link", "entity_id"),
    ] {
        for link in array_project(timelines, field)? {
            let object = link.as_object().ok_or(ProjectError::InvalidImportedData)?;
            let target_field = if target == "document_id" {
                "documentId"
            } else {
                "entityId"
            };
            let sql = format!(
                "INSERT INTO {table} (project_id, event_id, {target}, created_at)
                 VALUES (?1, ?2, ?3, 'import')"
            );
            transaction.execute(
                &sql,
                params![
                    project,
                    string_project(object, "eventId")?,
                    string_project(object, target_field)?,
                ],
            )?;
        }
    }
    Ok(())
}

fn insert_assets(
    transaction: &Transaction<'_>,
    project: &str,
    assets: &[ImportAsset],
    stored: &BTreeMap<String, kraqel_asset_service::StoredAssetFiles>,
) -> Result<(), ProjectError> {
    for asset in assets {
        let paths = stored
            .get(&asset.id)
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO media_asset (
                id, project_id, kind, relative_path, thumbnail_path, original_name, title,
                mime_type, checksum, byte_count, width, height, provenance, tags_json,
                created_at, updated_at
             ) VALUES (?1, ?2, 'image', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
                       'imported', ?12, 'import', 'import')",
            params![
                asset.id,
                project,
                paths.relative_path,
                paths.thumbnail_path,
                asset.original_name,
                asset.title,
                asset.mime_type,
                asset.checksum,
                i64::try_from(asset.byte_count).map_err(|_| ProjectError::InvalidImportedData)?,
                asset.width,
                asset.height,
                serde_json::to_string(&asset.tags)
                    .map_err(|_| ProjectError::InvalidImportedData)?,
            ],
        )?;
        for link in &asset.links {
            transaction.execute(
                "INSERT INTO asset_link (
                    project_id, asset_id, target_type, target_id, role, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'import')",
                params![
                    project,
                    asset.id,
                    link.target_kind,
                    link.target_id,
                    link.role
                ],
            )?;
        }
    }
    Ok(())
}

fn insert_visuals(
    transaction: &Transaction<'_>,
    project: &str,
    visuals: &Value,
) -> Result<(), ProjectError> {
    for profile in array_project(visuals, "profiles")? {
        let object = profile
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO visual_profile (
                entity_id, project_id, traits_json, canonical_asset_id, exclusions_json,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'import', 'import')",
            params![
                string_project(object, "entityId")?,
                project,
                json_text_project(object, "traits")?,
                optional_string_project(object.get("canonicalAssetId"))?,
                json_text_project(object, "exclusions")?,
            ],
        )?;
    }
    for preset in array_project(visuals, "stylePresets")? {
        let object = preset
            .as_object()
            .ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO style_preset (
                id, project_id, name, visual_rules_json, applicable_kinds_json,
                created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "name")?,
                json_text_project(object, "visualRules")?,
                json_text_project(object, "applicableKinds")?,
            ],
        )?;
    }
    Ok(())
}

fn insert_prompts(
    transaction: &Transaction<'_>,
    project: &str,
    prompts: &Value,
) -> Result<(), ProjectError> {
    for draft in array_project(prompts, "drafts")? {
        let object = draft.as_object().ok_or(ProjectError::InvalidImportedData)?;
        transaction.execute(
            "INSERT INTO image_prompt_draft (
                id, project_id, name, source_document_id, source_document_revision,
                style_preset_id, prompt_text, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'import', 'import')",
            params![
                string_project(object, "id")?,
                project,
                string_project(object, "name")?,
                string_project(object, "sourceDocumentId")?,
                integer_project(object, "sourceDocumentRevision")?,
                optional_string_project(object.get("stylePresetId"))?,
                string_project(object, "promptText")?,
            ],
        )?;
        for (position, entity) in object
            .get("entityIds")
            .and_then(Value::as_array)
            .ok_or(ProjectError::InvalidImportedData)?
            .iter()
            .enumerate()
        {
            transaction.execute(
                "INSERT INTO image_prompt_draft_profile (
                    draft_id, project_id, entity_id, position
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![
                    string_project(object, "id")?,
                    project,
                    entity.as_str().ok_or(ProjectError::InvalidImportedData)?,
                    i64::try_from(position).map_err(|_| ProjectError::InvalidImportedData)?,
                ],
            )?;
        }
    }
    Ok(())
}

fn require_kind(value: &Value, kind: &str, keys: &[&str]) -> Result<(), GitServiceError> {
    let object = exact_object(value, keys)?;
    if string(object, "kind")? == kind {
        Ok(())
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn exact_object<'a>(
    value: &'a Value,
    keys: &[&str],
) -> Result<&'a Map<String, Value>, GitServiceError> {
    let object = value.as_object().ok_or(GitServiceError::InvalidSnapshot)?;
    let expected = keys.iter().copied().collect::<BTreeSet<_>>();
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(object)
    } else {
        Err(GitServiceError::InvalidSnapshot)
    }
}

fn require_array_field<'a>(
    value: &'a Value,
    name: &str,
) -> Result<&'a Vec<Value>, GitServiceError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or(GitServiceError::InvalidSnapshot)
}

fn string<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, GitServiceError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or(GitServiceError::InvalidSnapshot)
}

fn unsigned(object: &Map<String, Value>, name: &str) -> Option<u64> {
    object.get(name).and_then(Value::as_u64)
}

fn optional_string(value: Option<&Value>) -> Result<Option<&str>, GitServiceError> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(GitServiceError::InvalidSnapshot),
    }
}

fn canonical_project_id(value: &str) -> Option<ProjectId> {
    ProjectId::parse(value).filter(|id| id.to_string() == value)
}

fn canonical_entity_id(value: &str) -> Result<String, GitServiceError> {
    EntityId::parse(value)
        .filter(|id| id.to_string() == value)
        .map(|id| id.to_string())
        .ok_or(GitServiceError::InvalidSnapshot)
}

fn valid_name(value: &str, maximum: usize) -> bool {
    value.trim() == value
        && !value.is_empty()
        && value.chars().count() <= maximum
        && !value.chars().any(char::is_control)
}

fn valid_string_map(
    value: Option<&Value>,
    maximum: usize,
    key_maximum: usize,
    value_maximum: usize,
) -> bool {
    value.and_then(Value::as_object).is_some_and(|map| {
        map.len() <= maximum
            && map.iter().all(|(key, value)| {
                valid_name(key, key_maximum)
                    && value
                        .as_str()
                        .is_some_and(|value| valid_name(value, value_maximum))
            })
    })
}

fn valid_string_array(
    value: Option<&Value>,
    maximum: usize,
    item_maximum: usize,
    unique: bool,
) -> bool {
    value.and_then(Value::as_array).is_some_and(|items| {
        items.len() <= maximum
            && items.iter().all(|item| {
                item.as_str()
                    .is_some_and(|item| valid_name(item, item_maximum))
            })
            && (!unique
                || items
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == items.len())
    })
}

fn valid_tags(tags: &[String]) -> bool {
    tags.len() <= 32
        && tags.iter().all(|tag| valid_name(tag, 80))
        && tags.iter().collect::<BTreeSet<_>>().len() == tags.len()
}

fn valid_kinds(value: Option<&Value>) -> bool {
    value.and_then(Value::as_array).is_some_and(|items| {
        !items.is_empty()
            && items.len() <= 4
            && items.iter().all(|item| {
                item.as_str().is_some_and(|item| {
                    matches!(item, "character" | "location" | "faction" | "item")
                })
            })
            && items
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
                .len()
                == items.len()
    })
}

fn optional_reference(value: Option<&Value>, ids: &BTreeSet<String>) -> bool {
    matches!(value, Some(Value::Null))
        || value
            .and_then(Value::as_str)
            .is_some_and(|value| ids.contains(value))
}

fn array_project<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, ProjectError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .ok_or(ProjectError::InvalidImportedData)
}

fn string_project<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a str, ProjectError> {
    object
        .get(name)
        .and_then(Value::as_str)
        .ok_or(ProjectError::InvalidImportedData)
}

fn optional_string_project(value: Option<&Value>) -> Result<Option<&str>, ProjectError> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(ProjectError::InvalidImportedData),
    }
}

fn integer_project(object: &Map<String, Value>, name: &str) -> Result<i64, ProjectError> {
    object
        .get(name)
        .and_then(Value::as_i64)
        .ok_or(ProjectError::InvalidImportedData)
}

fn json_text_project(object: &Map<String, Value>, name: &str) -> Result<String, ProjectError> {
    serde_json::to_string(object.get(name).ok_or(ProjectError::InvalidImportedData)?)
        .map_err(|_| ProjectError::InvalidImportedData)
}

#[cfg(test)]
mod tests {
    use std::{fs, fs::File};

    use super::{
        MAX_SNAPSHOT_ARCHIVE_BYTES, import_snapshot_archive, import_snapshot_archive_file,
    };
    use kraqel_storage::project_manager::ProjectManager;
    use tempfile::tempdir;

    const EXAMPLE: &[u8] =
        include_bytes!("../../../examples/die-stadt-die-den-dienstag-verkaufte.kraqel");

    #[test]
    fn imports_bundled_example_as_independent_project_copies() {
        let root = tempdir().expect("temporary project root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");

        let (first, first_summary) =
            import_snapshot_archive(&manager, EXAMPLE).expect("bundled example should import");
        assert_eq!(first_summary.document_count, 20);
        assert_eq!(first_summary.entity_count, 36);
        assert_eq!(first_summary.asset_count, 21);
        assert_eq!(
            first
                .database()
                .connection()
                .query_row("SELECT count(*) FROM event", [], |row| row.get::<_, u32>(0))
                .unwrap(),
            25
        );
        assert_eq!(
            first
                .database()
                .connection()
                .query_row("SELECT count(*) FROM visual_profile", [], |row| {
                    row.get::<_, u32>(0)
                })
                .unwrap(),
            13
        );
        assert!(first.directory().join("assets/thumbnails").is_dir());
        drop(first);

        let (_, second_summary) = import_snapshot_archive(&manager, EXAMPLE)
            .expect("a second independent copy should import");
        assert_ne!(first_summary.project_id, second_summary.project_id);
        assert_eq!(manager.list_projects().unwrap().len(), 2);
    }

    #[test]
    fn rejects_tampered_archive_without_publishing_a_project() {
        let root = tempdir().expect("temporary project root should exist");
        let manager = ProjectManager::new(root.path()).expect("manager should open");
        let mut tampered = EXAMPLE.to_vec();
        let middle = tampered.len() / 2;
        tampered[middle] ^= 0xff;

        assert!(import_snapshot_archive(&manager, &tampered).is_err());
        assert_eq!(manager.list_projects().unwrap().len(), 0);
    }

    #[test]
    fn imports_a_regular_bounded_archive_file() {
        let root = tempdir().expect("temporary root should exist");
        let project_root = root.path().join("projects");
        fs::create_dir(&project_root).expect("project root should exist");
        let source = root.path().join("source.kraqel");
        fs::write(&source, EXAMPLE).expect("fixture archive should be written");
        let manager = ProjectManager::new(&project_root).expect("manager should open");

        let (_, summary) = import_snapshot_archive_file(&manager, &source)
            .expect("regular bounded archive should import");

        assert_eq!(summary.document_count, 20);
        assert_eq!(manager.list_projects().unwrap().len(), 1);
    }

    #[test]
    fn rejects_non_files_and_oversized_archive_files_before_import() {
        let root = tempdir().expect("temporary root should exist");
        let project_root = root.path().join("projects");
        fs::create_dir(&project_root).expect("project root should exist");
        let manager = ProjectManager::new(&project_root).expect("manager should open");

        assert!(import_snapshot_archive_file(&manager, root.path()).is_err());

        let oversized = root.path().join("oversized.kraqel");
        let file = File::create(&oversized).expect("oversized fixture should be created");
        file.set_len(u64::try_from(MAX_SNAPSHOT_ARCHIVE_BYTES).unwrap() + 1)
            .expect("sparse fixture size should be set");
        assert!(import_snapshot_archive_file(&manager, &oversized).is_err());
        assert_eq!(manager.list_projects().unwrap().len(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_archive_file() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary root should exist");
        let project_root = root.path().join("projects");
        fs::create_dir(&project_root).expect("project root should exist");
        let source = root.path().join("source.kraqel");
        let link = root.path().join("linked.kraqel");
        fs::write(&source, EXAMPLE).expect("fixture archive should be written");
        symlink(&source, &link).expect("fixture symlink should be created");
        let manager = ProjectManager::new(&project_root).expect("manager should open");

        assert!(import_snapshot_archive_file(&manager, &link).is_err());
        assert_eq!(manager.list_projects().unwrap().len(), 0);
    }
}
