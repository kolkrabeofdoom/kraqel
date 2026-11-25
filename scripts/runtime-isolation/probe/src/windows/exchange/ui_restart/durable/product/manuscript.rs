//! Product-only comparison. The caller supplies the two role images embedded at desktop build time.
use super::*;
use kraqel_storage::project_manager::{ReviewChoice, ReviewedScene, SceneMergeSelection};

/// Operations scoped to the project and scene selected before the authenticated session.
pub enum SceneRequest<'a> {
    Save(&'a str),
    ReadLatest,
    Confirm {
        revision: u64,
        reviewed: &'a str,
        merged: &'a str,
    },
}

/// Result from the trusted project store.
pub enum SceneSaveStatus {
    Committed,
    Conflict,
    Failed,
    Review {
        revision: u64,
        text: Zeroizing<String>,
    },
}

pub(super) type SceneWriter<'a> = &'a mut dyn FnMut(SceneRequest<'_>) -> SceneSaveStatus;

/// Runs a read-only selection against an ordered and verified pair.
/// Only a confirmed choice after both roles have exited can escape the private channel.
pub fn open_review(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    plan: &SceneMergeSelection,
    left: &ReviewedScene,
    right: &ReviewedScene,
    service: &[u8],
    window: &[u8],
) -> std::io::Result<ReviewChoice> {
    let [scene] = plan.scenes.as_slice() else {
        return Err(std::io::Error::other("review-scene"));
    };
    let context = PairContext {
        subject: *scene.as_uuid().as_bytes(),
        local: &plan.local,
        manifest: &plan.manifest,
        scope: [plan.base.as_bytes(), plan.incoming.as_bytes()].concat(),
    };
    compare(parent, project, &context, [left, right], [service, window])
        .map_err(|failure| std::io::Error::other(failure.stage))
}

struct PairContext<'a> {
    subject: [u8; 16],
    local: &'a str,
    manifest: &'a str,
    scope: Vec<u8>,
}

/// Runs the same authenticated roles for an item pair. Its domain-separated binding
/// includes the complete composite key, both reviewed row hashes and the inventory.
pub fn open_item_review(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    plan: &kraqel_storage::project_manager::ItemSelection,
    local: Option<String>,
    incoming: String,
    images: [&[u8]; 2],
) -> std::io::Result<ReviewChoice> {
    let identity = serde_json::to_vec(&("kraqel-item-pair-v1", project, plan))?;
    let digest = Sha256::digest(&identity);
    let local_binding = format!("{digest:x}");
    let context = PairContext {
        subject: digest[..16].try_into().unwrap(),
        local: &local_binding,
        manifest: &plan.manifest,
        scope: identity,
    };
    let left = ReviewedScene {
        title: "Lokaler Datensatz · Vergleichsformat".into(),
        text: local.unwrap_or_else(|| "Am Ziel noch nicht vorhanden.".into()),
        revision: 1,
    };
    let right = ReviewedScene {
        title: "Externer Datensatz · Vergleichsformat".into(),
        text: incoming,
        revision: 1,
    };
    compare(parent, project, &context, [&left, &right], images)
        .map_err(|failure| std::io::Error::other(failure.stage))
}

fn compare(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    context: &PairContext<'_>,
    pair: [&ReviewedScene; 2],
    [service, window]: [&[u8]; 2],
) -> Result<ReviewChoice> {
    let choice = run_pair(
        parent,
        project,
        context,
        pair,
        [service, window],
        crate::window_receipt::MANUSCRIPT_COMPARE,
        None,
    )?;
    match choice {
        0 => Ok(ReviewChoice::Cancel),
        1 => Ok(ReviewChoice::Local),
        2 => Ok(ReviewChoice::Incoming),
        _ => Err(invalid("review-choice")),
    }
}

/// Opens one authenticated isolated read or edit window. The full revision and mode bind
/// the session even though the legacy comparison frame has a 32-bit revision field.
pub fn open_project_scene(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    scene: kraqel_domain::EntityId,
    revision: u64,
    text: &str,
    save: Option<SceneWriter<'_>>,
    images: [&[u8]; 2],
) -> std::io::Result<bool> {
    use crate::window_receipt::{PROJECT_SCENE_EDIT, PROJECT_SCENE_READ};
    let argument = if save.is_some() {
        PROJECT_SCENE_EDIT
    } else {
        PROJECT_SCENE_READ
    };
    let scope = serde_json::to_vec(&(argument, project, scene, revision))?;
    let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
    let anchor = format!("{:x}", Sha256::digest(&scope));
    let context = PairContext {
        subject: *scene.as_uuid().as_bytes(),
        local: &digest,
        manifest: &anchor,
        scope,
    };
    let revision = u32::try_from(revision).map_err(|_| std::io::Error::other("scene-revision"))?;
    let left = ReviewedScene {
        title: "Szene".into(),
        text: text.to_owned(),
        revision: i64::from(revision),
    };
    let right = ReviewedScene {
        title: "Auftrag".into(),
        text: argument.into(),
        revision: i64::from(revision),
    };
    let decision = run_pair(
        parent,
        project,
        &context,
        [&left, &right],
        images,
        argument,
        save,
    )
    .map_err(|failure| std::io::Error::other(failure.stage))?;
    Ok(decision == 1)
}
fn run_pair(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    context: &PairContext<'_>,
    pair: [&ReviewedScene; 2],
    [service, window]: [&[u8]; 2],
    argument: &'static str,
    save: Option<SceneWriter<'_>>,
) -> Result<u8> {
    let mut row = Zeroizing::new(Vec::new());
    row.extend_from_slice(project.as_uuid().as_bytes());
    row.extend_from_slice(&context.subject);
    for hash in [context.local, context.manifest] {
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid("review-digest"));
        }
        for pair in hash.as_bytes().chunks_exact(2) {
            row.push(
                u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
                    .map_err(|_| invalid("review-digest"))?,
            );
        }
    }
    for (i, v) in pair.iter().enumerate() {
        if v.title.is_empty()
            || v.title.chars().any(char::is_control)
            || v.text.contains('\0')
            || row.len() + v.title.len() + v.text.len() + 9 > 2 * 1024 * 1024
        {
            return Err(invalid("review-size-or-text"));
        }
        if i == 0 {
            row.extend_from_slice(&((5 + v.title.len() + v.text.len()) as u32).to_le_bytes());
        }
        row.extend_from_slice(
            &u32::try_from(v.revision)
                .map_err(|_| invalid("review-revision"))?
                .to_le_bytes(),
        );
        row.extend_from_slice(v.title.as_bytes());
        row.push(0);
        row.extend_from_slice(v.text.as_bytes());
    }
    let _parents = pin_directories(&local_path(parent)?)?;
    let home = parent.join("KRAQEL-Manuskriptvergleich");
    if !home.exists() {
        secure_directory(&home)?;
    }
    let _home = pin_directories(&home)?;
    private_acl(&home, true)?;
    let _lease = io(
        "review-exclusive",
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(home.join("owner.lock")),
    )?;
    let active = home.join("active");
    // Never reuse an uncertain session, including after a desktop crash.
    let actor: [u8; 32] = Sha256::new()
        .chain_update(b"KRAQEL-DESKTOP-REVIEW-v1")
        .chain_update(Sha256::digest(service))
        .chain_update(Sha256::digest(window))
        .finalize()
        .into();
    if active.exists() {
        retire(&active)?;
    }
    let mut store = Store::initialize(&active, actor)?;
    let bin = active.join("bin");
    let install = |name: &str, bytes: &[u8]| -> Result<_> {
        let path = bin.join(name);
        let mut out = io(
            "review-image-create",
            OpenOptions::new().write(true).create_new(true).open(&path),
        )?;
        io("review-image-write", out.write_all(bytes))?;
        io("review-image-sync", out.sync_all())?;
        drop(out);
        let pin = binary_pin(&path)?;
        if pin.2[..] != Sha256::digest(bytes)[..] {
            return Err(invalid("review-image-digest"));
        }
        Ok(pin)
    };
    let service = install("pipe-service.exe", service)?;
    let window = install("native-leaf.exe", window)?;
    let binding: [u8; 32] = Sha256::new()
        .chain_update(b"KRAQEL-MANUSCRIPT-PAIR-v1")
        .chain_update(actor)
        .chain_update(store.state.installation.as_bytes())
        .chain_update(&context.scope)
        .chain_update(&*row)
        .finalize()
        .into();
    for (slot, digest) in [
        binding,
        row[32..64].try_into().unwrap(),
        row[64..96].try_into().unwrap(),
    ]
    .into_iter()
    .enumerate()
    {
        store.append(
            actor,
            store.state.head,
            Action::P3FixtureEvidence {
                slot: slot as u8,
                digest,
            },
        )?;
    }
    store.append(
        actor,
        store.state.head,
        Action::BeginP3PipePreview {
            binding,
            packages: p3_fixture::packages_folder()?.1,
        },
    )?;
    let begin = store.state.head;
    let choice = win32_binding::deliver_p3(
        &mut store,
        &bin,
        &service.1,
        &window.1,
        binding,
        &row,
        true,
        Some(argument),
        save,
        || Ok(()),
    )?;
    store.append_with_checked_context(actor, store.state.head, Action::EndWin32Window { begin })?;
    drop((store, service, window));
    retire(&active)?;
    Ok(choice)
}

fn retire(path: &Path) -> Result<()> {
    let store = Store::open(path)?;
    if store.state.p3_pipe_begin.is_none() || store.state.p3_profiles != [None; 2] {
        return Err(invalid("review-unfinished"));
    }
    store.state.window_session.require_closed()?;
    store.idle()?;
    Store::job_idle(&wide(format!(
        "Local\\KRAQEL.Scene.{}",
        store.state.installation.simple()
    )))?;
    let sequence = store.state.sequence;
    let retired = path.with_file_name(format!("closed-{}", store.state.installation.simple()));
    drop(store);
    io("review-retire", fs::rename(path, &retired))?;
    for i in 0..=sequence {
        io(
            "review-record-remove",
            fs::remove_file(retired.join(format!("control/records/{i:04}.mac"))),
        )?;
    }
    for name in [
        "bin/native-leaf.exe",
        "bin/pipe-service.exe",
        "control/head.mac",
        "control/authority.key",
        "control/owner.lock",
    ] {
        io("review-file-remove", fs::remove_file(retired.join(name)))?;
    }
    for name in ["control/records", "control", "bin", "data", ""] {
        io(
            "review-directory-remove",
            fs::remove_dir(retired.join(name)),
        )?;
    }
    Ok(())
}
