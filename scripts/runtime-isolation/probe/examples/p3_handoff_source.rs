//! Produces only the fixed synthetic project used by the P3 receiver exercise.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use kraqel_domain::{CURRENT_SCHEMA_VERSION, EntityId, Project, ProjectId};
    use kraqel_storage::project_manager::ProjectManager;
    use sha2::{Digest, Sha256};
    use std::fs;

    // The harness owns this working directory. Never open a pre-existing catalog.
    fs::create_dir("catalog")?;
    let manager = ProjectManager::new("catalog")?;
    let id = ProjectId::new();
    let scene = EntityId::new().to_string();
    let asset = EntityId::new().to_string();
    let project = manager.create_project(
        id.to_string(),
        &Project {
            id,
            name: "P3 Hafen".into(),
            schema_version: CURRENT_SCHEMA_VERSION,
        },
    )?;
    let db = project.database().connection();
    let id_text = id.to_string();
    db.execute(
        "INSERT INTO document(id,project_id,kind,title,plain_text,position,created_at,updated_at)
        VALUES(?1,?2,'scene','Szene B','Vor dem WAL-Commit',0,'t','t')",
        [&scene, &id_text],
    )?;
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA wal_autocheckpoint=0")?;
    db.execute(
        "UPDATE document SET plain_text='Wind an der Suedkueste.' WHERE id=?1",
        [&scene],
    )?;
    let data = b"P3 synthetic asset";
    let root = project.directory();
    fs::create_dir_all(root.join("assets/nested"))?;
    fs::create_dir_all(root.join("assets/thumbnails"))?;
    fs::write(root.join("assets/nested/hafen.png"), data)?;
    fs::write(
        root.join("assets/thumbnails/hafen.png"),
        b"P3 synthetic thumbnail",
    )?;
    db.execute("INSERT INTO media_asset(id,project_id,kind,relative_path,thumbnail_path,original_name,title,
        mime_type,checksum,byte_count,width,height,provenance,created_at,updated_at)
        VALUES(?1,?2,'image','assets/nested/hafen.png','assets/thumbnails/hafen.png','hafen.png','Hafen',
        'image/png',?3,?4,1,1,'imported','t','t')",
        (&asset,&id_text,format!("{:x}",Sha256::digest(data)),i64::try_from(data.len())?))?;
    db.execute(
        "INSERT INTO asset_link(project_id,asset_id,target_type,target_id,created_at)
        VALUES(?1,?2,'document',?3,'t')",
        [&id_text, &asset, &scene],
    )?;
    if std::env::args().nth(1).as_deref() == Some("--divergent") {
        // Seed an independent offline workspace from one committed logical baseline.
        let baseline = manager.prepare_handoff(id)?;
        let backup = root.join("backups");
        let seed = backup.join(baseline.folder);
        fs::create_dir("second-catalog")?;
        let other_manager = ProjectManager::new("second-catalog")?;
        let other_root = std::path::Path::new("second-catalog").join(&id_text);
        for relative in ["backups", "assets/nested", "assets/thumbnails"] {
            fs::create_dir_all(other_root.join(relative))?;
        }
        for relative in [
            "kraqel.sqlite3",
            "assets/nested/hafen.png",
            "assets/thumbnails/hafen.png",
        ] {
            fs::copy(seed.join(relative), other_root.join(relative))?;
        }
        let other_project = other_manager.open_project(&id_text)?;
        for (conn, title, text) in [
            (db, "Szene B", "Die Flut erreicht den Hafen."),
            (
                other_project.database().connection(),
                "Szene A",
                "Der Hafen liegt im Nebel.",
            ),
        ] {
            let content = serde_json::json!({"type":"doc","content":[{"type":"paragraph",
                "content":[{"type":"text","text":text}]}]})
            .to_string();
            conn.execute(
                "UPDATE document SET title=?1,plain_text=?2,content_json=?3,revision=revision+1 WHERE id=?4",
                [title, text, &content, &scene],
            )?;
        }
        let receipt = manager.prepare_handoff(id)?;
        let followup = other_manager.prepare_handoff(id)?;
        // Collect only the prepared copy, leaving both workspaces intact.
        fs::rename(
            other_root.join("backups").join(&followup.folder),
            backup.join(&followup.folder),
        )?;
        println!(
            "{}",
            serde_json::json!({"project":id,"scene":scene,"asset":asset,
                                          "receipt":receipt,"followup":followup})
        );
        return Ok(());
    }
    let receipt = manager.prepare_handoff(id)?;
    let followup = if std::env::args().nth(1).as_deref() == Some("--followup") {
        db.execute(
            "UPDATE document SET title='Szene A',plain_text='Text aus der Testkopie.' WHERE id=?1",
            [&scene],
        )?;
        Some(manager.prepare_handoff(id)?)
    } else {
        None
    };
    println!(
        "{}",
        serde_json::json!({"project":id,"scene":scene,"asset":asset,"receipt":receipt,"followup":followup})
    );
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    panic!("Windows fixture required");
}
