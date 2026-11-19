//! Native window identity gate and optional receipt-only resume.
use super::*;
mod bootstrap;

pub(super) const WINDOW: &str = "native-leaf.exe";

pub(super) fn check_window(
    store: &Store,
    root: &Path,
    expected: &fs::File,
    resume: bool,
) -> Result<Value> {
    let image = root.join(WINDOW);
    let mut profile = Profile::create(&format!("ui.{}", Uuid::new_v4().simple()))?;
    let job_name = store.job_name();
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
    let (reader, mut writer) = crate::windows::exchange::ui_restart::pipe()?;
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
    let mut command = wide(format!("native-leaf.exe {}", crate::window_receipt::ARG));
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
        let expected_ack = bootstrap::stage(store, expected, &pi, endpoints[0], &mut writer)?;
        drop(writer);
        if let Some(reply) = response_reader {
            bootstrap::resume_and_confirm(&pi, &reply, expected_ack)?;
        }
        Ok(())
    })();
    // Release neither package pins nor owner lease while this generation is still live.
    check(unsafe { TerminateJobObject(job.0, 0) }, "win32-window-stop")?;
    if unsafe { WaitForSingleObject(process.0, 5000) } != WAIT_OBJECT_0 {
        return Err(invalid("win32-window-stop-timeout"));
    }
    store.idle()?;
    let removed = profile.remove();
    verified?;
    if !removed {
        return Err(invalid("win32-window-profile-cleanup"));
    }
    Ok(
        json!({"nativePreviewOnly":true,"role":"window","suspendedIdentityVerified":true,
        "bootstrapBoundBeforeResume":true,"roleResumed":resume,"bootstrapReceiptVerified":resume,"dataAllowed":false,"activeJobProcessesAtEnd":0,
        "headSha256":hex(&store.state.head)}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_bytes_from_a_different_file_cannot_authorize_the_suspended_window() {
        let root =
            std::env::temp_dir().join(format!("kraqel-suspended-{}", Uuid::new_v4().simple()));
        let store = Store::initialize(&root, [1; 32]).unwrap();
        let bin = root.join("bin");
        let role = bin.join(WINDOW);
        let other = bin.join("other.exe");
        let bytes = fs::read(std::env::current_exe().unwrap()).unwrap();
        fs::write(&role, &bytes).unwrap();
        fs::write(&other, &bytes).unwrap();
        let wrong_pin = protected_file(&other).unwrap();
        assert_eq!(
            check_window(&store, &bin, &wrong_pin, true)
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
