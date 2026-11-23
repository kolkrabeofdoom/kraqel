//! Broker provisions one private A/read channel for two verified suspended generations.
use super::*;
use windows_sys::Win32::Storage::FileSystem::{FILE_READ_DATA, FILE_WRITE_DATA, SYNCHRONIZE};

fn install(process: HANDLE, file: fs::File, rights: u32) -> Result<[u8; 8]> {
    let source = file.as_raw_handle();
    let mut remote = null_mut();
    check(
        unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                source,
                process,
                &mut remote,
                rights,
                0,
                0,
            )
        },
        "scene-end-install",
    )?;
    let mut local = null_mut();
    check(
        unsafe {
            DuplicateHandle(
                process,
                remote,
                GetCurrentProcess(),
                &mut local,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        },
        "scene-end-readback",
    )?;
    let local = Handle(local);
    check(
        unsafe { CompareObjectHandles(local.0, source) },
        "scene-end-identity",
    )?;
    Ok((remote as usize as u64).to_le_bytes())
}

pub(in super::super) fn read_scene(
    store: &mut Store,
    root: &Path,
    window: &fs::File,
    service: &fs::File,
    mode: crate::window_receipt::SceneOperation,
    mut revision: u32,
    target: &mut Option<SnapshotFile>,
    baseline: &mut Option<SnapshotFile>,
    checkpoint: bool,
    continued: bool,
    budget: Option<u8>,
) -> Result<Value> {
    use crate::window_receipt::SceneOperation;
    use crate::windows::exchange::ui_restart::pipe;
    let mut discarded = false;
    with_role(
        store,
        root,
        window,
        WINDOW,
        if budget.is_some() {
            crate::window_receipt::budgeted_argument(mode)
        } else if continued {
            crate::window_receipt::continue_argument(mode)
        } else if checkpoint {
            crate::window_receipt::checkpoint_argument(mode)
        } else {
            mode.argument()
        },
        true,
        |store, ui, endpoint, mut to_ui, reply| {
            let mut ui_reply = reply.ok_or(invalid("scene-ui-reply"))?;
            let ui_secret = bootstrap::stage(store, window, ui, endpoint, &mut to_ui, true)?;
            let mut first_window = None;
            let limit = budget.unwrap_or(if continued { 2 } else { 1 });
            for turn in 0..limit {
                let can_continue = turn + 1 < limit;
                let mode = if turn == 0 {
                    mode
                } else {
                    SceneOperation::Revise
                };
                with_role(
                    store,
                    root,
                    service,
                    "pipe-service.exe",
                    mode.argument(),
                    true,
                    |store, data, endpoint, mut to_data, reply| {
                        let mut data_reply = reply.ok_or(invalid("scene-data-reply"))?;
                        let data_secret =
                            bootstrap::stage(store, service, data, endpoint, &mut to_data, true)?;
                        let (request_read, request_write) = pipe()?;
                        let (response_read, response_write) = pipe()?;
                        let read = FILE_READ_DATA | SYNCHRONIZE;
                        let write = FILE_WRITE_DATA | SYNCHRONIZE;
                        let ui_ends = [
                            install(ui.hProcess, response_read, read)?,
                            install(ui.hProcess, request_write, write)?,
                        ];
                        let data_ends = [
                            install(data.hProcess, request_read, read)?,
                            install(data.hProcess, response_write, write)?,
                        ];
                        let mut snapshot_ends = Vec::new();
                        let snapshots = [
                            target.as_ref().map(|s| &s.file),
                            baseline.as_ref().map(|s| &s.file),
                        ];
                        for (index, file) in snapshots.into_iter().enumerate() {
                            let Some(file) = file else { continue };
                            let rights = if matches!(
                                mode,
                                SceneOperation::KeyboardEdit | SceneOperation::SavePersistent
                            ) || mode == SceneOperation::Revise && index == 0
                            {
                                write
                            } else {
                                read
                            };
                            snapshot_ends.push(install(
                                data.hProcess,
                                io("scene-snapshot-clone", file.try_clone())?,
                                rights | FILE_READ_ATTRIBUTES,
                            )?);
                        }
                        if turn == 0 {
                            bootstrap::confirm_grant(ui, &mut to_ui, &mut ui_reply, &ui_secret)?;
                            if let Some(limit) = budget {
                                io("scene-autosave-budget", to_ui.write_all(&[limit]))?;
                            }
                        }
                        bootstrap::confirm_grant(
                            data,
                            &mut to_data,
                            &mut data_reply,
                            &data_secret,
                        )?;
                        let mut material = Zeroizing::new([0u8; 96]);
                        let mut context = Sha256::new();
                        match mode {
                            SceneOperation::Revise
                            | SceneOperation::ReadRevision
                            | SceneOperation::KeyboardEdit
                            | SceneOperation::SavePersistent
                            | SceneOperation::ReadPersistent => {
                                context.update(mode.argument().as_bytes())
                            }
                            SceneOperation::CheckEditConflict => {
                                context.update(b"KRAQEL-managed-A/edit-conflict-v1")
                            }
                            SceneOperation::Once | SceneOperation::CheckDeniedSave => {
                                context.update(b"KRAQEL-managed-A/read-v1")
                            }
                        }
                        if mode == SceneOperation::CheckDeniedSave {
                            context.update(mode.argument().as_bytes());
                        }
                        if mode.revision_bound() {
                            context.update(revision.to_le_bytes());
                        }
                        if turn > 0 {
                            context.update(store.state.head);
                        }
                        context.update(ui_secret);
                        context.update(data_secret);
                        material[..32].copy_from_slice(&context.finalize());
                        getrandom::fill(&mut material[32..]).map_err(|_| invalid("scene-keys"))?;
                        for (control, ends, storage) in [
                            (&mut to_data, data_ends, snapshot_ends.as_slice()),
                            (&mut to_ui, ui_ends, &[][..]),
                        ] {
                            io("scene-channel-keys", control.write_all(&material[..]))?;
                            for end in ends {
                                io("scene-channel-end", control.write_all(&end))?;
                            }
                            if mode.revision_bound() {
                                io(
                                    "scene-bound-revision",
                                    control.write_all(&revision.to_le_bytes()),
                                )?;
                            }
                            for handle in storage {
                                io("scene-snapshot-end", control.write_all(handle))?;
                            }
                        }
                        let ended = if matches!(
                            mode,
                            SceneOperation::KeyboardEdit | SceneOperation::Revise
                        ) {
                            unsafe {
                                WaitForMultipleObjects(
                                    2,
                                    [data.hProcess, ui.hProcess].as_ptr(),
                                    0,
                                    INFINITE,
                                )
                            }
                        } else {
                            unsafe { WaitForSingleObject(data.hProcess, 5000) }
                        };
                        if ended != WAIT_OBJECT_0
                            && !(ended == WAIT_OBJECT_0 + 1
                                && unsafe { WaitForSingleObject(data.hProcess, 5000) }
                                    == WAIT_OBJECT_0)
                        {
                            return Err(invalid("scene-service-timeout"));
                        }
                        let mut exit = 0;
                        check(
                            unsafe { GetExitCodeProcess(data.hProcess, &mut exit) },
                            "scene-service-exit",
                        )?;
                        if exit == crate::window_receipt::DISCARDED_EXIT
                            && matches!(mode, SceneOperation::KeyboardEdit | SceneOperation::Revise)
                        {
                            if unsafe { WaitForSingleObject(ui.hProcess, 5000) } != WAIT_OBJECT_0 {
                                return Err(invalid("scene-discard-ui-timeout"));
                            }
                            let mut ui_exit = 0;
                            check(
                                unsafe { GetExitCodeProcess(ui.hProcess, &mut ui_exit) },
                                "scene-discard-ui-exit",
                            )?;
                            if ui_exit != exit
                                || bootstrap::live_frame::<32>(&mut data_reply, data.hProcess)?
                                    != crate::window_receipt::discard_receipt(&material, false)
                                || bootstrap::live_frame::<32>(&mut ui_reply, ui.hProcess)?
                                    != crate::window_receipt::discard_receipt(&material, true)
                            {
                                return Err(invalid("scene-discard-receipt"));
                            }
                            discarded = true;
                            return Ok(());
                        }
                        if exit != 0 {
                            return Err(invalid("scene-service-denied"));
                        }
                        Ok(())
                    },
                )?;
                if discarded {
                    return Ok(());
                }
                if checkpoint {
                    // The service and its job have ended; only the starter still holds the candidate.
                    let hwnd = bootstrap::verified_surface(ui, &mut ui_reply)
                        .map_err(|_| invalid("scene-checkpoint-live-surface"))?;
                    if first_window.is_some_and(|first| first != hwnd) {
                        return Err(invalid("scene-rebind-window"));
                    }
                    let snapshot = target
                        .as_mut()
                        .ok_or(invalid("scene-checkpoint-target"))?
                        .identity(revision + 1)?;
                    let begin = store.state.head;
                    store.append_with_checked_context(
                        store.state.actor,
                        begin,
                        Action::PublishWin32Scene { begin, snapshot },
                    )?;
                    let mut frame = [0u8; 36];
                    frame[..4].copy_from_slice(&snapshot.revision.to_le_bytes());
                    frame[4..].copy_from_slice(&store.state.head);
                    io("scene-checkpoint-confirm", to_ui.write_all(&frame))?;
                    if can_continue {
                        let receipt = bootstrap::live_frame::<32>(&mut ui_reply, ui.hProcess)?;
                        if receipt == crate::window_receipt::publication_receipt(&frame) {
                            io("scene-confirm-close", to_ui.write_all(&[0]))?;
                        } else if receipt == crate::window_receipt::continue_receipt(&frame) {
                            let publication = store.state.head;
                            revision = snapshot.revision;
                            store.append_with_checked_context(
                                store.state.actor,
                                publication,
                                Action::RebindWin32Scene {
                                    begin,
                                    publication,
                                    revision,
                                },
                            )?;
                            // Same pinned file object, duplicated into the new service with read rights only.
                            *baseline = target.take();
                            *target =
                                Some(SnapshotFile::open(store, &scene_file(revision + 1), true)?);
                            first_window = Some(hwnd);
                            continue;
                        } else {
                            return Err(invalid("scene-rebind-receipt"));
                        }
                    }
                    if unsafe { WaitForSingleObject(ui.hProcess, 5000) } != WAIT_OBJECT_0 {
                        return Err(invalid("scene-checkpoint-window-timeout"));
                    }
                    if !can_continue
                        && bootstrap::live_frame::<32>(&mut ui_reply, ui.hProcess)?
                            != crate::window_receipt::publication_receipt(&frame)
                    {
                        return Err(invalid("scene-checkpoint-receipt"));
                    }
                    let mut exit = 0;
                    check(
                        unsafe { GetExitCodeProcess(ui.hProcess, &mut exit) },
                        "scene-checkpoint-window-exit",
                    )?;
                    return if exit == 40 {
                        Ok(())
                    } else {
                        Err(invalid("scene-checkpoint-window-failed"))
                    };
                }
                return bootstrap::wait_surface(ui, &mut ui_reply, mode == SceneOperation::Once);
            }
            Err(invalid("scene-turn-budget"))
        },
    )?;
    if discarded {
        return Ok(
            json!({"nativePreviewOnly":true,"role":"window","syntheticScene":"A","revision":revision,"discarded":true,"persistent":revision>0,"dataAllowed":false,"activeJobProcessesAtEnd":0}),
        );
    }
    let mut result = json!({"nativePreviewOnly":true,"role":"window","syntheticScene":"A","revision":0,"readOnly":true,"dataAllowed":false,"activeJobProcessesAtEnd":0});
    if mode == SceneOperation::CheckDeniedSave {
        result["saveDenied"] = json!(true);
    }
    if mode == SceneOperation::CheckEditConflict {
        result["readOnly"] = json!(false);
        result["revision"] = json!(1);
        result["staleSaveRejected"] = json!(true);
    }
    if matches!(
        mode,
        SceneOperation::Revise
            | SceneOperation::ReadRevision
            | SceneOperation::KeyboardEdit
            | SceneOperation::SavePersistent
            | SceneOperation::ReadPersistent
    ) {
        result["revision"] = json!(1);
        if mode.revision_bound() {
            result["revision"] = json!(revision + u32::from(mode == SceneOperation::Revise));
        }
        result["persistent"] = json!(true);
        result["readOnly"] = json!(matches!(
            mode,
            SceneOperation::ReadPersistent | SceneOperation::ReadRevision
        ));
    }
    if checkpoint {
        result["revision"] = json!(revision + 1);
        result["publicationAcknowledged"] = json!(true);
    }
    Ok(result)
}
