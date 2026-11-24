//! Product-only comparison. The caller supplies the two role images embedded at desktop build time.
use super::*;
use kraqel_storage::project_manager::{ReviewChoice, ReviewedScene, SceneMergeSelection};

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
    review(parent, project, plan, left, right, service, window)
        .map_err(|failure| std::io::Error::other(failure.stage))
}

fn review(
    parent: &Path,
    project: kraqel_domain::ProjectId,
    plan: &SceneMergeSelection,
    left: &ReviewedScene,
    right: &ReviewedScene,
    service: &[u8],
    window: &[u8],
) -> Result<ReviewChoice> {
    let [scene] = plan.scenes.as_slice() else {
        return Err(invalid("review-scene"));
    };
    let mut row = Zeroizing::new(Vec::new());
    row.extend_from_slice(project.as_uuid().as_bytes());
    row.extend_from_slice(scene.as_uuid().as_bytes());
    // The local frame binds the full reviewed database basis, the external side its inventory.
    for hash in [&plan.local, &plan.manifest] {
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
    for (i, v) in [left, right].iter().enumerate() {
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
        .chain_update(plan.base.as_bytes())
        .chain_update(plan.incoming.as_bytes())
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
        true,
        || Ok(()),
    )?;
    store.append_with_checked_context(actor, store.state.head, Action::EndWin32Window { begin })?;
    drop((store, service, window));
    retire(&active)?;
    match choice {
        0 => Ok(ReviewChoice::Cancel),
        1 => Ok(ReviewChoice::Local),
        2 => Ok(ReviewChoice::Incoming),
        _ => Err(invalid("review-choice")),
    }
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
