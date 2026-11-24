//! Read-only comparison of logical application tables; only schema names cross IPC.
use super::*;
use kraqel_domain::ProjectId;

impl ProjectManager {
    /// Vergleicht die gespeicherten Arbeitsdaten mit einer ausdrücklich ausgewählten Eingangskopie.
    ///
    /// # Errors
    /// Unbestätigte Datenbankbytes, fremde Projektkennung oder unbekanntes Schema sperren den Vergleich.
    pub fn compare_handoff(
        &self,
        id: ProjectId,
        folder: &str,
        digest: &str,
        include_media: bool,
    ) -> Result<Vec<String>, ProjectError> {
        self.with_inbox_manifest(folder, digest, |dir, mut manifest| {
            if manifest.project_id != id {
                return Err(ProjectError::InvalidProjectMetadata);
            }
            let hash = manifest
                .files
                .remove(DATABASE_FILE_NAME)
                .ok_or(ProjectError::InvalidDatabase)?;
            let incoming = scene_merge::read_package_database(dir, &hash)?;
            let reference = Database::open_in_memory()?;
            handoff::validate_source(&incoming, reference.connection(), id)?;
            self.with_read_snapshot(id, |_, local| {
                handoff::validate_source(local, reference.connection(), id)?;
                let mut stmt = reference.connection().prepare(
                    "SELECT name FROM pragma_table_list WHERE schema='main' AND type='table'
                     AND name NOT GLOB 'sqlite_*' ORDER BY name",
                )?;
                let mut differences = Vec::new();
                for table in stmt.query_map([], |r| r.get::<_, String>(0))? {
                    let table = table?;
                    // Table and column identifiers come exclusively from our own fresh schema.
                    let mut pk = reference
                        .connection()
                        .prepare("SELECT name FROM pragma_table_info(?1) WHERE pk>0 ORDER BY pk")?;
                    let keys = pk
                        .query_map([&table], |r| r.get::<_, String>(0))?
                        .collect::<Result<Vec<_>, _>>()?;
                    if keys.is_empty() {
                        return Err(ProjectError::InvalidDatabase);
                    }
                    let order = keys
                        .iter()
                        .map(|key| format!("\"{key}\""))
                        .collect::<Vec<_>>()
                        .join(",");
                    let query = format!("SELECT * FROM \"{table}\" ORDER BY {order}");
                    if !handoff::same_rows(local, &incoming, &query)? {
                        differences.push(table);
                    }
                }
                if include_media {
                    let external =
                        handoff::media_inventory(&incoming, dir, None, Some(&manifest.files))?;
                    if external != manifest.files {
                        return Err(ProjectError::InvalidDatabase);
                    }
                    let local_files = handoff::media_inventory(
                        local,
                        &self.root.join(id.to_string()),
                        None,
                        None,
                    )?;
                    if local_files != external {
                        differences.push("media_files".into());
                    }
                }
                Ok(differences)
            })
        })
    }
}
