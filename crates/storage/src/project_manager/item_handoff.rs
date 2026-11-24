//! Explicit one-row adoption in a logical work copy.
use super::*;
use kraqel_domain::{EntityId, ProjectId};
use rusqlite::{
    params_from_iter,
    types::{Value as SqlValue, ValueRef},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemKind {
    Entity,
    Relation,
    Link,
    PlotCard,
    Event,
    EventDocumentLink,
    EventEntityLink,
}

impl ItemKind {
    pub(super) fn table(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relation => "relation",
            Self::Link => "link",
            Self::PlotCard => "plot_card",
            Self::Event => "event",
            Self::EventDocumentLink => "event_document_link",
            Self::EventEntityLink => "event_entity_link",
        }
    }
    pub(super) fn columns(self) -> &'static [&'static str] {
        match self {
            Self::PlotCard => &["document_id"],
            Self::EventDocumentLink => &["event_id", "document_id"],
            Self::EventEntityLink => &["event_id", "entity_id"],
            _ => &["id"],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemKey {
    pub kind: ItemKind,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemChange {
    pub key: ItemKey,
    pub local: Option<String>,
    pub incoming: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemSelection {
    pub folder: String,
    pub manifest: String,
    pub change: ItemChange,
}

pub(super) type ItemRow = BTreeMap<String, Value>;

fn invalid() -> ProjectError {
    ProjectError::InvalidDatabase
}
fn validate_key(key: &ItemKey) -> Result<(), ProjectError> {
    if key.ids.len() != key.kind.columns().len()
        || key
            .ids
            .iter()
            .any(|s| EntityId::parse(s).is_none_or(|id| id.to_string() != *s))
    {
        return Err(invalid());
    }
    Ok(())
}
fn where_key(key: &ItemKey) -> String {
    let mut sql = "project_id=?".to_owned();
    for col in key.kind.columns() {
        sql.push_str(" AND ");
        sql.push_str(col);
        sql.push_str("=?");
    }
    sql
}
fn args(id: ProjectId, key: &ItemKey) -> Vec<String> {
    let mut values = vec![id.to_string()];
    values.extend_from_slice(&key.ids);
    values
}
fn item_row(row: &rusqlite::Row<'_>) -> Result<ItemRow, ProjectError> {
    let mut values = ItemRow::new();
    for (index, name) in row.as_ref().column_names().into_iter().enumerate() {
        let value = match row.get_ref(index)? {
            ValueRef::Integer(n) => Value::from(n),
            ValueRef::Text(s) => Value::from(std::str::from_utf8(s).map_err(|_| invalid())?),
            ValueRef::Null => Value::Null,
            _ => return Err(invalid()),
        };
        values.insert(name.to_owned(), value);
    }
    Ok(values)
}
pub(super) fn load_item(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
) -> Result<Option<ItemRow>, ProjectError> {
    validate_key(key)?;
    let mut query = db.prepare(&format!(
        "SELECT * FROM {} WHERE {}",
        key.kind.table(),
        where_key(key)
    ))?;
    let mut rows = query.query(params_from_iter(args(id, key)))?;
    rows.next()?.map(item_row).transpose()
}
pub(super) fn row_hash(row: &ItemRow) -> Result<String, ProjectError> {
    Ok(scene_basis::hash(
        &serde_json::to_vec(row).map_err(|_| invalid())?,
    ))
}
fn inventory(
    db: &Connection,
    id: ProjectId,
    kind: ItemKind,
) -> Result<BTreeMap<Vec<String>, String>, ProjectError> {
    let mut query = db.prepare(&format!(
        "SELECT * FROM {} WHERE project_id=?1",
        kind.table()
    ))?;
    let mut rows = query.query([id.to_string()])?;
    let mut out = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let key = ItemKey {
            kind,
            ids: kind
                .columns()
                .iter()
                .map(|c| row.get(*c))
                .collect::<Result<_, _>>()?,
        };
        validate_key(&key)?;
        out.insert(key.ids, row_hash(&item_row(row)?)?);
    }
    Ok(out)
}

fn checked_rows(
    local: &Connection,
    src: &Connection,
    id: ProjectId,
    change: &ItemChange,
) -> Result<(Option<ItemRow>, ItemRow), ProjectError> {
    let before = load_item(local, id, &change.key)?;
    let after = load_item(src, id, &change.key)?.ok_or_else(invalid)?;
    if before.as_ref() == Some(&after)
        || before.as_ref().map(row_hash).transpose()? != change.local
        || Some(row_hash(&after)?) != change.incoming
    {
        return Err(invalid());
    }
    Ok((before, after))
}

pub(super) fn write_item(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    row: &ItemRow,
) -> Result<(), ProjectError> {
    validate_key(key)?;
    let table = key.kind.table();
    let columns = db
        .prepare(&format!("SELECT * FROM {table} LIMIT 0"))?
        .column_names()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if columns.len() != row.len()
        || row.get("project_id") != Some(&Value::from(id.to_string()))
        || key
            .kind
            .columns()
            .iter()
            .zip(&key.ids)
            .any(|(c, s)| row.get(*c).and_then(Value::as_str) != Some(s.as_str()))
    {
        return Err(invalid());
    }
    let values = columns
        .iter()
        .map(|c| match row.get(c) {
            Some(Value::Null) => Ok(SqlValue::Null),
            Some(Value::String(s)) => Ok(SqlValue::Text(s.clone())),
            Some(Value::Number(n)) => n.as_i64().map(SqlValue::Integer).ok_or_else(invalid),
            _ => Err(invalid()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_fields(db, id, key, row)?;
    let assignments = columns
        .iter()
        .filter(|c| c.as_str() != "project_id" && !key.kind.columns().contains(&c.as_str()))
        .map(|c| format!("{c}=excluded.{c}"))
        .collect::<Vec<_>>()
        .join(",");
    let placeholders = vec!["?"; columns.len()].join(",");
    let sql = format!(
        "INSERT INTO {table} ({}) VALUES ({placeholders}) ON CONFLICT ({}) DO UPDATE SET {assignments} WHERE {table}.project_id=excluded.project_id",
        columns.join(","),
        key.kind.columns().join(",")
    );
    if db.execute(&sql, params_from_iter(values))? != 1
        || load_item(db, id, key)?.as_ref() != Some(row)
    {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn validate_fields(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
    row: &ItemRow,
) -> Result<(), ProjectError> {
    use crate::{story_bible_repository as bible, structure_repository as structure};
    let text = |field: &str| row.get(field).and_then(Value::as_str).ok_or_else(invalid);
    let entity_id = |s: &str| {
        EntityId::parse(s)
            .filter(|id| id.to_string() == s)
            .ok_or_else(invalid)
    };
    if matches!(key.kind, ItemKind::Event | ItemKind::PlotCard) {
        let position = row
            .get("position")
            .and_then(Value::as_u64)
            .ok_or_else(invalid)?;
        u32::try_from(position).map_err(|_| invalid())?;
    }
    match key.kind {
        ItemKind::Entity => {
            bible::parse_entity_kind(text("kind")?).map_err(|_| invalid())?;
            bible::validate_name(text("name")?).map_err(|_| invalid())?;
            bible::parse_stored_properties(text("properties_json")?).map_err(|_| invalid())?;
            if let Some(old) = load_item(db, id, key)?
                && old["kind"] != row["kind"]
            {
                return Err(invalid());
            }
        }
        ItemKind::Relation => {
            bible::validate_short_text(text("kind")?, bible::MAX_RELATION_KIND_CHARS)
                .map_err(|_| invalid())?;
            bible::parse_stored_properties(text("properties_json")?).map_err(|_| invalid())?;
        }
        ItemKind::PlotCard => {
            structure::require_scene(db, id, entity_id(&key.ids[0])?).map_err(|_| invalid())?;
            if let Some(value) = row["storyline_id"].as_str() {
                structure::require_storyline(db, id, entity_id(value)?).map_err(|_| invalid())?;
            }
            if let Some(value) = row["pov_entity_id"].as_str() {
                structure::require_character(db, id, entity_id(value)?).map_err(|_| invalid())?;
            }
        }
        ItemKind::Event => {
            structure::validate_text(text("title")?, 240).map_err(|_| invalid())?;
            if text("description")?.chars().count() > 8000 {
                return Err(invalid());
            }
            let mode = structure::require_timeline(db, id, entity_id(text("timeline_id")?)?)
                .map_err(|_| invalid())?;
            structure::validate_event_time(mode, row["time_value"].as_str())
                .map_err(|_| invalid())?;
        }
        ItemKind::EventDocumentLink => {
            structure::require_scene(db, id, entity_id(&key.ids[1])?).map_err(|_| invalid())?;
        }
        ItemKind::Link | ItemKind::EventEntityLink => {}
    }
    Ok(())
}

pub(super) fn remove_item(
    db: &Connection,
    id: ProjectId,
    key: &ItemKey,
) -> Result<(), ProjectError> {
    validate_key(key)?;
    if db.execute(
        &format!("DELETE FROM {} WHERE {}", key.kind.table(), where_key(key)),
        params_from_iter(args(id, key)),
    )? != 1
    {
        return Err(invalid());
    }
    Ok(())
}

impl ProjectManager {
    pub(super) fn with_item_source<T>(
        &self,
        id: ProjectId,
        folder: &str,
        manifest: &str,
        run: impl FnOnce(&Connection) -> Result<T, ProjectError>,
    ) -> Result<T, ProjectError> {
        self.with_inbox_manifest(folder, manifest, |dir, inventory| {
            if inventory.project_id != id {
                return Err(invalid());
            }
            let db = scene_merge::read_package_database(
                dir,
                inventory
                    .files
                    .get(DATABASE_FILE_NAME)
                    .ok_or_else(invalid)?,
            )?;
            let reference = Database::open_in_memory()?;
            handoff::validate_source(&db, reference.connection(), id)?;
            run(&db)
        })
    }

    /// Lists changed keys and content commitments without transferring row values.
    ///
    /// # Errors
    /// Unconfirmed input, invalid identities or unknown schemas reject the operation.
    pub fn list_handoff_items(
        &self,
        id: ProjectId,
        folder: &str,
        manifest: &str,
        kind: ItemKind,
    ) -> Result<Vec<ItemChange>, ProjectError> {
        self.with_item_source(id, folder, manifest, |src| {
            self.with_read_snapshot(id, |_, local| {
                handoff::validate_source(local, src, id)?;
                let mut incoming = inventory(src, id, kind)?;
                let mut result = Vec::new();
                for (ids, here) in inventory(local, id, kind)? {
                    let there = incoming.remove(&ids);
                    if there.as_ref() != Some(&here) {
                        result.push(ItemChange {
                            key: ItemKey { kind, ids },
                            local: Some(here),
                            incoming: there,
                        });
                    }
                }
                result.extend(incoming.into_iter().map(|(ids, h)| ItemChange {
                    key: ItemKey { kind, ids },
                    local: None,
                    incoming: Some(h),
                }));
                Ok(result)
            })
        })
    }

    /// Reads both native-only row presentations following an explicit review request.
    ///
    /// # Errors
    /// Stale commitments or unavailable incoming rows reject review.
    pub fn review_handoff_item(
        &self,
        id: ProjectId,
        s: &ItemSelection,
    ) -> Result<(Option<String>, String), ProjectError> {
        self.with_item_source(id, &s.folder, &s.manifest, |src| {
            self.with_read_snapshot(id, |_, local| {
                handoff::validate_source(local, src, id)?;
                let (before, after) = checked_rows(local, src, id, &s.change)?;
                let render =
                    |row: &ItemRow| serde_json::to_string_pretty(row).map_err(|_| invalid());
                Ok((before.as_ref().map(render).transpose()?, render(&after)?))
            })
        })
    }

    /// Prepares exactly the row selected by the native confirmation.
    ///
    /// # Errors
    /// Stale values or unavailable dependencies leave the candidate unpublished.
    pub fn prepare_handoff_item(
        &self,
        id: ProjectId,
        s: &ItemSelection,
    ) -> Result<PreparedHandoff, ProjectError> {
        self.prepare_changed_handoff(id, None, Some(s))
    }

    pub(super) fn apply_item_selection(
        &self,
        id: ProjectId,
        s: &ItemSelection,
        target: &mut Database,
    ) -> Result<(), ProjectError> {
        self.with_item_source(id, &s.folder, &s.manifest, |src| {
            let tx = target.transaction()?;
            let (_, after) = checked_rows(&tx, src, id, &s.change)?;
            write_item(&tx, id, &s.change.key, &after)?;
            tx.commit()?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    fn uid() -> String {
        EntityId::new().to_string()
    }

    #[test]
    fn selected_item_is_reviewed_isolated_prepared_switched_and_restored() {
        let root = tempdir().unwrap();
        let manager = ProjectManager::new(root.path()).unwrap();
        let project = Project {
            id: ProjectId::new(),
            name: "Einträge".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
        };
        let id = project.id;
        let project_name = id.to_string();
        let live = manager.create_project(&project_name, &project).unwrap();
        let db = live.database().connection();
        let (main, other, new) = (uid(), uid(), uid());
        for (key, name) in [
            (&main, "Ferne Fassung"),
            (&other, "Zweite Figur"),
            (&new, "Neu"),
        ] {
            db.execute("INSERT INTO entity(id,project_id,kind,name,created_at,updated_at) VALUES (?1,?2,'character',?3,'t','t')",params![key,project_name,name]).unwrap();
        }
        let relation = uid();
        db.execute("INSERT INTO relation(id,project_id,source_entity_id,target_entity_id,kind,created_at,updated_at) VALUES (?1,?2,?3,?4,'Freundschaft','t','t')",params![relation,project_name,main,other]).unwrap();
        let receipt = manager.prepare_handoff(id).unwrap();
        let inbox = root.path().join("empfang");
        fs::create_dir(&inbox).unwrap();
        fs::rename(
            live.directory()
                .join(BACKUP_DIRECTORY_NAME)
                .join(&receipt.folder),
            inbox.join(&receipt.folder),
        )
        .unwrap();
        db.execute("UPDATE entity SET name='Lokal' WHERE id=?1", [&main])
            .unwrap();
        db.execute("DELETE FROM entity WHERE id=?1", [&new])
            .unwrap();
        let changes = manager
            .list_handoff_items(
                id,
                &receipt.folder,
                &receipt.manifest_sha256,
                ItemKind::Entity,
            )
            .unwrap();
        assert_eq!(changes.len(), 2);
        for key in [&main, &new] {
            let change = changes
                .iter()
                .find(|c| &c.key.ids[0] == key)
                .unwrap()
                .clone();
            let s = ItemSelection {
                folder: receipt.folder.clone(),
                manifest: receipt.manifest_sha256.clone(),
                change,
            };
            let original = load_item(db, id, &s.change.key).unwrap();
            let (_, view) = manager.review_handoff_item(id, &s).unwrap();
            assert!(view.contains(if key == &main { "Ferne Fassung" } else { "Neu" }));
            if key == &main {
                db.execute(
                    "UPDATE entity SET name='Zwischenstand' WHERE id=?1",
                    [&main],
                )
                .unwrap();
                assert!(manager.prepare_handoff_item(id, &s).is_err());
                db.execute("UPDATE entity SET name='Lokal' WHERE id=?1", [&main])
                    .unwrap();
                // A SQL-valid legacy row may still be unacceptable to the domain writer.
                db.execute(
                    "UPDATE entity SET properties_json='[]' WHERE id=?1",
                    [&main],
                )
                .unwrap();
                let mut legacy = s.clone();
                let prior = load_item(db, id, &s.change.key).unwrap().unwrap();
                legacy.change.local = Some(row_hash(&prior).unwrap());
                let prepared = manager.prepare_handoff_item(id, &legacy).unwrap();
                assert!(manager.secure_handoff_item(id, &legacy, &prepared).is_err());
                assert_eq!(load_item(db, id, &s.change.key).unwrap(), Some(prior));
                db.execute(
                    "UPDATE entity SET properties_json='{}' WHERE id=?1",
                    [&main],
                )
                .unwrap();
            }
            let candidate = manager.prepare_handoff_item(id, &s).unwrap();
            assert_eq!(load_item(db, id, &s.change.key).unwrap(), original);
            let ticket = manager.secure_handoff_item(id, &s, &candidate).unwrap();
            manager.switch_handoff_item(id, &ticket, false).unwrap();
            assert_eq!(
                load_item(db, id, &s.change.key)
                    .unwrap()
                    .as_ref()
                    .map(row_hash)
                    .transpose()
                    .unwrap(),
                s.change.incoming
            );
            if key == &new {
                item_activation::check_linked_undo(&manager, id, &ticket, &new, &other);
            }
            manager.switch_handoff_item(id, &ticket, true).unwrap();
            manager.switch_handoff_item(id, &ticket, true).unwrap();
            assert!(manager.switch_handoff_item(id, &ticket, false).is_err());
            assert_eq!(load_item(db, id, &s.change.key).unwrap(), original);
        }
        let related:String=db.query_row("SELECT e.name FROM relation r JOIN entity e ON r.target_entity_id=e.id WHERE r.id=?1 AND r.source_entity_id=?2",params![relation,main],|r|r.get(0)).unwrap();
        assert_eq!(related, "Zweite Figur");
    }
}

#[cfg(test)]
mod structure_item_tests {
    use super::*;

    #[test]
    fn structured_rows_bind_their_actual_keys_and_local_dependencies() {
        let database = Database::open_in_memory().unwrap();
        let db = database.connection();
        let id = ProjectId::new();
        let project = id.to_string();
        let [a, b, scene, timeline, event, relation, link] =
            std::array::from_fn(|_| EntityId::new().to_string());
        db.execute("INSERT INTO project(id,name,schema_version,created_at,updated_at) VALUES(?,'Projekt',1,'t','t')",[&project]).unwrap();
        for entity in [&a, &b] {
            db.execute("INSERT INTO entity(id,project_id,kind,name,created_at,updated_at) VALUES(?,?,'character','Figur','t','t')",params![entity,project]).unwrap();
        }
        db.execute("INSERT INTO document(id,project_id,kind,title,position,created_at,updated_at) VALUES(?,?,'scene','Szene',0,'t','t')",params![scene,project]).unwrap();
        db.execute("INSERT INTO timeline(id,project_id,name,mode,position,created_at,updated_at) VALUES(?,?,'Tage','relative',0,'t','t')",params![timeline,project]).unwrap();
        db.execute("INSERT INTO event(id,project_id,timeline_id,title,position,created_at,updated_at) VALUES(?,?,?,'Ein Ereignis',0,'t','t')",params![event,project,timeline]).unwrap();
        db.execute("INSERT INTO relation(id,project_id,source_entity_id,target_entity_id,kind,created_at,updated_at) VALUES(?,?,?,?,'kennt','t','t')",params![relation,project,a,b]).unwrap();
        db.execute(
            "INSERT INTO link(id,project_id,document_id,entity_id,created_at) VALUES(?,?,?,?,'t')",
            params![link, project, scene, a],
        )
        .unwrap();
        db.execute("INSERT INTO plot_card(project_id,document_id,pov_entity_id,position,updated_at) VALUES(?,?,?,0,'t')",params![project,scene,a]).unwrap();
        db.execute(
            "INSERT INTO event_document_link VALUES(?,?,?,'t')",
            params![project, event, scene],
        )
        .unwrap();
        db.execute(
            "INSERT INTO event_entity_link VALUES(?,?,?,'t')",
            params![project, event, b],
        )
        .unwrap();
        for (kind, ids) in [
            (ItemKind::Relation, vec![relation]),
            (ItemKind::Link, vec![link]),
            (ItemKind::PlotCard, vec![scene.clone()]),
            (ItemKind::EventDocumentLink, vec![event.clone(), scene]),
            (ItemKind::EventEntityLink, vec![event.clone(), b]),
            (ItemKind::Event, vec![event]),
        ] {
            let key = ItemKey { kind, ids };
            let mut row = load_item(db, id, &key).unwrap().unwrap();
            // Force a real update in every supported structure, including composite-key links.
            let timestamp = if row.contains_key("updated_at") {
                "updated_at"
            } else {
                "created_at"
            };
            row.insert(timestamp.into(), Value::from("new"));
            write_item(db, id, &key, &row).unwrap();
            if matches!(kind, ItemKind::PlotCard | ItemKind::Event) {
                let mut oversized = row.clone();
                oversized.insert("position".into(), Value::from(i64::from(u32::MAX) + 1));
                assert!(write_item(db, id, &key, &oversized).is_err());
            }
            if kind == ItemKind::PlotCard {
                db.execute("UPDATE entity SET kind='location' WHERE id=?", [&a])
                    .unwrap();
                assert!(write_item(db, id, &key, &row).is_err());
            }
            if kind == ItemKind::Event {
                db.execute(
                    "UPDATE timeline SET mode='absolute' WHERE id=?",
                    [&timeline],
                )
                .unwrap();
                assert!(write_item(db, id, &key, &row).is_err());
            }
        }
    }
}
