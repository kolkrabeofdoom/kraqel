//! Native window identity gate and optional receipt-only resume.
use super::*;
mod bootstrap;
mod scene;
pub(super) use scene::read_scene;

pub(super) const WINDOW: &str = "native-leaf.exe";

pub(super) fn check_window(
    store: &mut Store,
    root: &Path,
    expected: &fs::File,
    mode: WindowMode,
) -> Result<Value> {
    let resume = mode != WindowMode::Suspended;
    let interactive = mode == WindowMode::Interactive;
    let surface = interactive || mode == WindowMode::Surface;
    with_role(
        store,
        root,
        expected,
        WINDOW,
        if surface {
            crate::window_receipt::UI_ARG
        } else {
            crate::window_receipt::ARG
        },
        resume,
        |store, pi, endpoint, mut writer, reply| {
            let bootstrap_value =
                bootstrap::stage(store, expected, pi, endpoint, &mut writer, surface)?;
            if let Some(reply) = reply {
                if surface {
                    bootstrap::confirm_and_open(pi, writer, reply, bootstrap_value, interactive)?;
                } else {
                    drop(writer);
                    bootstrap::resume_and_confirm(
                        pi,
                        &reply,
                        crate::window_receipt::receipt(&bootstrap_value),
                    )?;
                }
            }
            Ok(())
        },
    )?;
    Ok(
        json!({"nativePreviewOnly":true,"role":"window","suspendedIdentityVerified":true,
        "bootstrapBoundBeforeResume":true,"roleResumed":resume,"bootstrapReceiptVerified":resume,"syntheticWindowVerified":surface,"dataAllowed":false,"activeJobProcessesAtEnd":0,
        "headSha256":hex(&store.state.head)}),
    )
}

pub(in super::super) fn deliver_p3(
    store: &mut Store,
    root: &Path,
    image: &fs::File,
    window: &fs::File,
    binding: [u8; 32],
    row: &[u8],
    interactive: bool,
    product_argument: Option<&'static str>,
    visible: impl FnOnce() -> Result<()>,
) -> Result<(u8, Option<String>)> {
    use crate::window_receipt as protocol;
    let mut choice = 0;
    let mut edited = None;
    use windows_sys::Win32::Storage::FileSystem::{FILE_READ_DATA, FILE_WRITE_DATA, SYNCHRONIZE};
    let argument = if let Some(argument) = product_argument {
        argument
    } else if row.len() != protocol::P3_LEN {
        protocol::P3_COMPARE
    } else {
        protocol::P3_READ
    };
    with_role(
        store,
        root,
        window,
        WINDOW,
        argument,
        true,
        |store, ui, endpoint, mut to_ui, reply| {
            let mut reply = reply.ok_or(invalid("p3-window-pipe"))?;
            let ui_grant = bootstrap::stage_bound(
                store,
                window,
                ui,
                endpoint,
                &mut to_ui,
                true,
                (store.state.actor, binding),
            )?;
            let (rx, tx) = crate::windows::exchange::ui_restart::pipe()?;
            let read_end = scene::install(ui.hProcess, rx, FILE_READ_DATA | SYNCHRONIZE)?;
            bootstrap::confirm_grant(ui, &mut to_ui, &mut reply, &ui_grant)?;
            io("p3-window-end", to_ui.write_all(&read_end))?;
            io(
                "p3-window-binding",
                to_ui.write_all(&protocol::p3_receipt(row)),
            )?;
            drop(to_ui);
            with_role(
                store,
                root,
                image,
                "pipe-service.exe",
                argument,
                true,
                |store, pi, endpoint, mut writer, reply| {
                    let mut reply = reply.ok_or(invalid("p3-service-pipe"))?;
                    let grant = bootstrap::stage_bound(
                        store,
                        image,
                        pi,
                        endpoint,
                        &mut writer,
                        true,
                        (store.state.actor, binding),
                    )?;
                    let write_end = scene::install(pi.hProcess, tx, FILE_WRITE_DATA | SYNCHRONIZE)?;
                    bootstrap::confirm_grant(pi, &mut writer, &mut reply, &grant)?;
                    io("p3-service-end", writer.write_all(&write_end))?;
                    io(
                        "p3-service-send",
                        protocol::write_p3(&mut writer, argument != protocol::P3_READ, row),
                    )?;
                    drop(writer);
                    bootstrap::confirm_exit_receipt(pi, &reply, protocol::p3_receipt(row))
                },
            )?;
            let hwnd = bootstrap::verified_surface(ui, &mut reply)?;
            if interactive {
                visible()?;
            }
            // The isolated editor writes its draft before exiting. Drain its private response
            // pipe while it is alive so a draft larger than the pipe buffer cannot deadlock.
            if argument == protocol::PROJECT_SCENE_EDIT {
                use std::io::Read;
                let mut decision = [0];
                io("scene-edit-decision", reply.read_exact(&mut decision))?;
                match decision[0] {
                    0 => {}
                    1 => edited = Some(read_edited_scene(&mut reply)?),
                    _ => return Err(invalid("scene-edit-decision")),
                }
            }
            bootstrap::wait_for_close(ui, hwnd, interactive)?;
            if argument != protocol::P3_READ && argument != protocol::PROJECT_SCENE_EDIT {
                use std::io::Read;
                let mut decision = [0u8];
                io("p3-window-choice", reply.read_exact(&mut decision))?;
                if decision[0] > 2
                    || ((!interactive || argument == protocol::PROJECT_SCENE_READ)
                        && decision[0] != 0)
                {
                    return Err(invalid("p3-window-choice"));
                }
                choice = decision[0];
            }
            Ok(())
        },
    )?;
    Ok((choice, edited))
}

fn read_edited_scene(reply: &mut fs::File) -> Result<String> {
    use std::{
        io::Read,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;

    let deadline = Instant::now() + Duration::from_secs(15);
    let mut fill = |mut target: &mut [u8]| -> Result<()> {
        while !target.is_empty() {
            if Instant::now() >= deadline {
                return Err(invalid("scene-edit-timeout"));
            }
            let mut count = 0;
            check(
                unsafe {
                    PeekNamedPipe(
                        reply.as_raw_handle(),
                        null_mut(),
                        0,
                        null_mut(),
                        &mut count,
                        null_mut(),
                    )
                },
                "scene-edit-pipe",
            )?;
            if count == 0 {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            let bytes = target.len().min(count as usize);
            let (chunk, remaining) = target.split_at_mut(bytes);
            io("scene-edit-bytes", reply.read_exact(chunk))?;
            target = remaining;
        }
        Ok(())
    };
    let mut length = [0; 4];
    fill(&mut length)?;
    let size = u32::from_le_bytes(length) as usize;
    if size > 64 * 1024 {
        return Err(invalid("scene-edit-length"));
    }
    let mut bytes = vec![0; size];
    fill(&mut bytes)?;
    let text = String::from_utf8(bytes).map_err(|_| invalid("scene-edit-utf8"))?;
    if text.contains(['\0', '\r']) {
        return Err(invalid("scene-edit-text"));
    }
    Ok(text)
}

fn with_role(
    store: &mut Store,
    root: &Path,
    expected: &fs::File,
    name: &str,
    argument: &str,
    resume: bool,
    run: impl FnOnce(&mut Store, &PROCESS_INFORMATION, HANDLE, fs::File, Option<fs::File>) -> Result<()>,
) -> Result<()> {
    let image = root.join(name);
    let data = name == "pipe-service.exe";
    let role = if data { "data" } else { "ui" };
    let begin = if crate::window_receipt::p3_comparison(Some(argument)).is_some()
        || matches!(
            argument,
            crate::window_receipt::MANUSCRIPT_COMPARE
                | crate::window_receipt::PROJECT_SCENE_READ
                | crate::window_receipt::PROJECT_SCENE_EDIT
        ) {
        let begin = store
            .state
            .p3_pipe_begin
            .ok_or(invalid("p3-profile-generation"))?;
        if store.state.window_session != Session::Open(begin) {
            return Err(invalid("p3-profile-session"));
        }
        Some(begin)
    } else {
        None
    };
    let id = begin.map_or_else(
        || format!("{role}.{}", Uuid::new_v4().simple()),
        |b| p3_fixture::profile_id(b, data),
    );
    let mut profile = Profile::create(&id)?;
    let witness = if let Some(begin) = begin {
        let folder = p3_fixture::folder_identity(store, &profile)?
            .ok_or(invalid("p3-created-profile-missing"))?;
        store.append_with_checked_context(
            store.state.actor,
            store.state.head,
            Action::P3Profile {
                begin,
                data,
                folder,
                removed: false,
            },
        )?;
        // From here the journal owns retirement, including any error during role startup.
        profile.active = false;
        Some((begin, folder))
    } else {
        None
    };
    let job_name = if data {
        wide(format!(
            "Local\\KRAQEL.Scene.{}",
            store.state.installation.simple()
        ))
    } else {
        store.job_name()
    };
    let job = Handle(unsafe { CreateJobObjectW(null(), job_name.as_ptr()) });
    if job.0.is_null() {
        return Err(fail("win32-window-job"));
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Err(invalid("win32-window-job-collision"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    check(
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of_val(&limits) as u32,
            )
        },
        "win32-window-limits",
    )?;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let jobs = [job.0];
    let dynamic = 1u64 << 36;
    let no_children = 1u32;
    // Pin both directions in the handle list. The receipt-only mode has no command channel.
    let (reader, writer) = crate::windows::exchange::ui_restart::pipe()?;
    let response = resume
        .then(crate::windows::exchange::ui_restart::pipe)
        .transpose()?;
    let (response_reader, response_writer) = response.unzip();
    let mut endpoints = vec![reader.as_raw_handle()];
    if let Some(out) = &response_writer {
        endpoints.push(out.as_raw_handle());
    }
    for endpoint in &endpoints {
        check(
            unsafe { SetHandleInformation(*endpoint, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
            "win32-window-bootstrap-inherit",
        )?;
    }
    let attrs =
        Attributes::with_creation_policy(&caps, &endpoints, &jobs, Some((&dynamic, &no_children)))?;
    let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
    startup.StartupInfo.cb = size_of_val(&startup) as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = endpoints[0];
    if let Some(out) = &response_writer {
        startup.StartupInfo.hStdOutput = out.as_raw_handle();
    }
    let profile_name = String::from_utf16_lossy(&profile.name[..profile.name.len() - 1]);
    let mut command = wide(format!("{name} {argument} {profile_name}"));
    startup.lpAttributeList = attrs.ptr;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let environment = crate::windows::exchange::ui_restart::environment(root);
    check(
        unsafe {
            CreateProcessW(
                path_wide(&image)?.as_ptr(),
                if resume {
                    command.as_mut_ptr()
                } else {
                    null_mut()
                },
                null(),
                null(),
                1,
                CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT,
                environment.as_ptr().cast(),
                path_wide(root)?.as_ptr(),
                &startup.StartupInfo,
                &mut pi,
            )
        },
        "win32-window-create",
    )?;
    drop(reader);
    drop(response_writer);
    let process = Handle(pi.hProcess);
    let primary = Handle(pi.hThread);
    let verified = (|| {
        let mut member = 0;
        check(
            unsafe { IsProcessInJob(process.0, job.0, &mut member) },
            "win32-window-job-readback",
        )?;
        if member == 0
            || pi.dwProcessId == 0
            || unsafe { GetProcessId(process.0) } != pi.dwProcessId
            || unsafe { GetProcessIdOfThread(primary.0) } != pi.dwProcessId
            || unsafe { GetThreadId(primary.0) } != pi.dwThreadId
        {
            return Err(invalid("win32-window-generation"));
        }
        workspace_probe::lifetime::verify_single_process_job(job.0)?;
        starter::verify_start_lease(&store.root, &job_name)?;
        let facts = token_facts(process.0, Some(profile.sid))?;
        if facts["isAppContainer"] != true
            || facts["capabilityCount"] != 0
            || facts["expectedIdentity"] != true
        {
            return Err(invalid("win32-window-token"));
        }
        for policy in [ProcessDynamicCodePolicy, ProcessChildProcessPolicy] {
            let mut bits = 0u32;
            check(
                unsafe {
                    GetProcessMitigationPolicy(
                        process.0,
                        policy,
                        (&raw mut bits).cast(),
                        size_of_val(&bits),
                    )
                },
                "win32-window-mitigation",
            )?;
            if bits != 1 {
                return Err(invalid("win32-window-mitigation-mismatch"));
            }
        }
        let mut path = vec![0u16; 32768];
        let mut len = path.len() as u32;
        check(
            unsafe { QueryFullProcessImageNameW(process.0, 0, path.as_mut_ptr(), &mut len) },
            "win32-window-image",
        )?;
        let actual = PathBuf::from(
            String::from_utf16(&path[..len as usize])
                .map_err(|_| invalid("win32-window-image-path"))?,
        );
        if local_path(&actual)? != local_path(&image)? {
            return Err(invalid("win32-window-image-path"));
        }
        let actual = protected_file(&image)?;
        if secure_fs::file_id(&actual)? != secure_fs::file_id(expected)? {
            return Err(invalid("win32-window-image-identity"));
        }
        run(store, &pi, endpoints[0], writer, response_reader)
    })();
    // Release neither package pins nor owner lease while this generation is still live.
    check(unsafe { TerminateJobObject(job.0, 0) }, "win32-window-stop")?;
    if unsafe { WaitForSingleObject(process.0, 5000) } != WAIT_OBJECT_0 {
        return Err(invalid("win32-window-stop-timeout"));
    }
    if data {
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicAccountingInformation,
                    (&raw mut info).cast(),
                    size_of_val(&info) as u32,
                    null_mut(),
                )
            },
            "scene-job-drain",
        )?;
        if info.ActiveProcesses != 0 {
            return Err(invalid("scene-job-not-empty"));
        }
    } else {
        store.idle()?;
    }
    let removed = if let Some((begin, folder)) = witness {
        p3_fixture::retire_profile(store, begin, data, &profile, folder)?;
        true
    } else {
        profile.remove()
    };
    verified?;
    if !removed {
        return Err(invalid("win32-window-profile-cleanup"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_bytes_from_a_different_file_cannot_authorize_the_suspended_window() {
        let root =
            std::env::temp_dir().join(format!("kraqel-suspended-{}", Uuid::new_v4().simple()));
        let mut store = Store::initialize(&root, [1; 32]).unwrap();
        let bin = root.join("bin");
        let role = bin.join(WINDOW);
        let other = bin.join("other.exe");
        let bytes = fs::read(std::env::current_exe().unwrap()).unwrap();
        fs::write(&role, &bytes).unwrap();
        fs::write(&other, &bytes).unwrap();
        let wrong_pin = protected_file(&other).unwrap();
        assert_eq!(
            check_window(&mut store, &bin, &wrong_pin, WindowMode::Surface)
                .unwrap_err()
                .stage,
            "win32-window-image-identity"
        );
        store.idle().unwrap();
        drop(wrong_pin);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
