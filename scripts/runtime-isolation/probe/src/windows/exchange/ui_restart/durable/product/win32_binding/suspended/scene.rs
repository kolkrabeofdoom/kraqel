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
    store: &Store,
    root: &Path,
    window: &fs::File,
    service: &fs::File,
    mode: crate::window_receipt::SceneRead,
) -> Result<Value> {
    use crate::window_receipt::SceneRead;
    use crate::windows::exchange::ui_restart::pipe;
    with_role(
        store,
        root,
        window,
        WINDOW,
        mode.argument(),
        true,
        |ui, endpoint, mut to_ui, reply| {
            let mut ui_reply = reply.ok_or(invalid("scene-ui-reply"))?;
            let ui_secret = bootstrap::stage(store, window, ui, endpoint, &mut to_ui, true)?;
            with_role(
                store,
                root,
                service,
                "pipe-service.exe",
                mode.argument(),
                true,
                |data, endpoint, mut to_data, reply| {
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
                    bootstrap::confirm_grant(ui, &mut to_ui, &mut ui_reply, &ui_secret)?;
                    bootstrap::confirm_grant(data, &mut to_data, &mut data_reply, &data_secret)?;
                    let mut material = Zeroizing::new([0u8; 96]);
                    let mut context = Sha256::new();
                    context.update(b"KRAQEL-managed-A/read-v1");
                    if mode == SceneRead::CheckDeniedSave {
                        context.update(mode.argument().as_bytes());
                    }
                    context.update(ui_secret);
                    context.update(data_secret);
                    material[..32].copy_from_slice(&context.finalize());
                    getrandom::fill(&mut material[32..]).map_err(|_| invalid("scene-keys"))?;
                    for (mut control, ends) in [(to_data, data_ends), (to_ui, ui_ends)] {
                        io("scene-channel-keys", control.write_all(&material[..]))?;
                        for end in ends {
                            io("scene-channel-end", control.write_all(&end))?;
                        }
                    }
                    if unsafe { WaitForSingleObject(data.hProcess, 5000) } != WAIT_OBJECT_0 {
                        return Err(invalid("scene-service-timeout"));
                    }
                    let mut exit = 0;
                    check(
                        unsafe { GetExitCodeProcess(data.hProcess, &mut exit) },
                        "scene-service-exit",
                    )?;
                    if exit != 0 {
                        return Err(invalid("scene-service-denied"));
                    }
                    Ok(())
                },
            )?;
            bootstrap::wait_surface(ui, &mut ui_reply, mode == SceneRead::Once)
        },
    )?;
    let mut result = json!({"nativePreviewOnly":true,"role":"window","syntheticScene":"A","revision":0,"readOnly":true,"dataAllowed":false,"activeJobProcessesAtEnd":0});
    if mode == SceneRead::CheckDeniedSave {
        result["saveDenied"] = json!(true);
    }
    Ok(result)
}
