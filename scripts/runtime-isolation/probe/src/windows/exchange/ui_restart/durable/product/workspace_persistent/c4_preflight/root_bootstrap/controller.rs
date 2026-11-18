//! Holds the native lease/job through authenticated bootstrap and blank WebView observation.
#![cfg_attr(not(test), allow(dead_code))]
use super::*;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
mod modules;
struct Observation {
    value: Value,
    process: Handle,
    modules: modules::Observation,
}

struct Process(Handle, Handle);
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0.0, 3);
            WaitForSingleObject(self.0.0, 5000);
        }
    }
}
struct Job(Handle);
impl Job {
    fn count(&self) -> Result<u32> {
        let mut i: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    self.0.0,
                    JobObjectBasicAccountingInformation,
                    (&mut i as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of_val(&i) as u32,
                    null_mut(),
                )
            },
            "c4-root-job-count",
        )?;
        Ok(i.ActiveProcesses)
    }
    fn stop(&self) -> Result<()> {
        check(
            unsafe { TerminateJobObject(self.0.0, 3) },
            "c4-root-job-stop",
        )?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.count()? != 0 {
            if Instant::now() >= deadline {
                return Err(invalid("c4-root-job-drain"));
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}
fn image(process: HANDLE) -> Result<PathBuf> {
    let mut bytes = [0u16; 32768];
    let mut n = bytes.len() as u32;
    check(
        unsafe { QueryFullProcessImageNameW(process, 0, bytes.as_mut_ptr(), &mut n) },
        "c4-root-image",
    )?;
    Ok(PathBuf::from(
        String::from_utf16(&bytes[..n as usize]).map_err(|_| invalid("c4-root-image"))?,
    ))
}
fn same_image(actual: &Path, expected: &Path) -> Result<bool> {
    fn file_id(path: &Path) -> Result<([u8; 16], u64)> {
        let _parents = pin_directories(path.parent().ok_or(invalid("c4-image-parent"))?)?;
        let file = io(
            "c4-image-pin",
            OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(local_path(path)?),
        )?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
        check(
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
            "c4-image-info",
        )?;
        if info.dwFileAttributes & (BAD_ATTRIBUTES | FILE_ATTRIBUTE_DIRECTORY) != 0
            || info.nNumberOfLinks != 1
        {
            return Err(invalid("c4-image-linked-or-wrong-kind"));
        }
        let mut id: FILE_ID_INFO = unsafe { zeroed() };
        check(
            unsafe {
                GetFileInformationByHandleEx(
                    file.as_raw_handle(),
                    FileIdInfo,
                    (&mut id as *mut FILE_ID_INFO).cast(),
                    size_of_val(&id) as u32,
                )
            },
            "c4-image-file-id",
        )?;
        Ok((id.FileId.Identifier, id.VolumeSerialNumber))
    }
    Ok(file_id(actual)? == file_id(expected)?)
}
fn append(
    store: &mut Store,
    e: &mut C4ProbeEvent,
    stage: Stage,
    job: &Job,
    root: Option<&Process>,
    p: &Profile,
) -> Result<()> {
    if e.guardian != identity(unsafe { GetCurrentProcess() })? {
        return Err(invalid("c4-root-guardian-generation"));
    }
    let count = job.count()?;
    if matches!(stage, Stage::Commit | Stage::Drained | Stage::Finish) {
        if count != 0 {
            return Err(invalid("c4-root-record-not-drained"));
        }
    } else {
        if count != 1 {
            return Err(invalid("c4-root-record-group"));
        }
        let root = root.ok_or(invalid("c4-root-record-process"))?;
        let id = identity(root.0.0)?;
        if stage == Stage::RootBound {
            e.root = Some(id);
        } else if e.root != Some(id) {
            return Err(invalid("c4-root-record-generation"));
        }
        let mut yes = 0;
        check(
            unsafe { IsProcessInJob(root.0.0, job.0.0, &mut yes) },
            "c4-root-record-job",
        )?;
        let facts = token_facts(root.0.0, Some(p.sid))?;
        if yes == 0
            || facts["isAppContainer"] != true
            || facts["capabilityCount"] != 0
            || facts["expectedIdentity"] != true
        {
            return Err(invalid("c4-root-record-token"));
        }
    }
    let mut next = e.clone();
    next.stage = stage;
    store.append_with_checked_context(
        store.state.actor,
        store.state.head,
        Action::C4RootProbe {
            event: next.clone(),
        },
    )?;
    *e = next;
    Ok(())
}
fn sample(
    job: &Job,
    root: Identity,
    ui: &Path,
    seen: &mut std::collections::BTreeMap<(u32, u64), Observation>,
) -> Result<()> {
    #[repr(C)]
    struct List {
        assigned: u32,
        listed: u32,
        ids: [usize; 16],
    }
    let mut list: List = unsafe { zeroed() };
    check(
        unsafe {
            QueryInformationJobObject(
                job.0.0,
                JobObjectBasicProcessIdList,
                (&mut list as *mut List).cast(),
                size_of_val(&list) as u32,
                null_mut(),
            )
        },
        "c4-root-job-members",
    )?;
    if list.listed > 16 || list.assigned > 16 || list.assigned != list.listed {
        return Err(invalid("c4-root-job-members"));
    }
    for pid in &list.ids[..list.listed as usize] {
        let process = Handle(unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                *pid as u32,
            )
        });
        if process.0.is_null() {
            return Err(fail("c4-root-child-open"));
        }
        let id = identity(process.0)?;
        if seen.contains_key(&(id.pid, id.created)) {
            continue;
        }
        if seen.len() >= 64 {
            return Err(invalid("c4-observation-lifetime-bound"));
        }
        let file = image(process.0)?;
        let role = if id == root { "root" } else { "runtime" };
        let expected = if id == root {
            ui.join("ui.exe")
        } else {
            ui.join("runtime/msedgewebview2.exe")
        };
        let expected_image = same_image(&file, &expected)?;
        let facts = token_facts(process.0, None)?;
        let parent = if id == root {
            None
        } else {
            Some(parent_in_job(id, job)?)
        };
        let sha = if expected_image {
            json!(hex(&hash(&file)?))
        } else {
            Value::Null
        };
        seen.insert((id.pid,id.created),Observation{value:json!({"pid":id.pid,"created":id.created,"role":role,"image":file,"expectedImagePath":expected,"expectedImage":expected_image,"token":facts,"parent":parent,"imageSha256":sha}),process,modules:modules::Observation::default()});
        if !expected_image {
            return Err(invalid("c4-root-unexpected-image"));
        }
        if facts["isAppContainer"] != true || facts["capabilityCount"] != 0 {
            return Err(invalid("c4-root-child-token"));
        }
    }
    Ok(())
}
fn parent_in_job(child: Identity, job: &Job) -> Result<Identity> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(fail("c4-parent-snapshot"));
    }
    let snapshot = Handle(raw);
    let mut e: PROCESSENTRY32W = unsafe { zeroed() };
    e.dwSize = size_of_val(&e) as u32;
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut e) };
    let mut pid = None;
    while more != 0 {
        if e.th32ProcessID == child.pid {
            pid = Some(e.th32ParentProcessID);
            break;
        }
        more = unsafe { Process32NextW(snapshot.0, &mut e) };
    }
    let parent = Handle(unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid.ok_or(invalid("c4-parent-missing"))?,
        )
    });
    if parent.0.is_null() {
        return Err(fail("c4-parent-open"));
    }
    let id = identity(parent.0)?;
    let mut yes = 0;
    check(
        unsafe { IsProcessInJob(parent.0, job.0.0, &mut yes) },
        "c4-parent-job",
    )?;
    if yes == 0 || id.created > child.created {
        return Err(invalid("c4-parent-generation-or-job"));
    }
    Ok(id)
}
fn outcome(
    events: &mut fs::File,
    deadline: Instant,
    job: &Job,
    root: Identity,
    ui: &Path,
    seen: &mut std::collections::BTreeMap<(u32, u64), Observation>,
) -> Result<Value> {
    let mut bytes = Vec::new();
    let mut goal = 4;
    let mut next_modules = Instant::now();
    while bytes.len() < goal {
        if Instant::now() >= deadline {
            return Err(invalid("c4-root-runtime-deadline"));
        }
        sample(job, root, ui, seen)?;
        if Instant::now() >= next_modules {
            for observation in seen.values_mut() {
                observation.modules.sample(observation.process.0);
            }
            next_modules = Instant::now() + Duration::from_millis(100);
        }
        let mut available = 0;
        check(
            unsafe {
                PeekNamedPipe(
                    events.as_raw_handle(),
                    null_mut(),
                    0,
                    null_mut(),
                    &mut available,
                    null_mut(),
                )
            },
            "c4-root-outcome-peek",
        )?;
        if available == 0 {
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        let mut part = vec![0; (available as usize).min(goal - bytes.len())];
        let n = io("c4-root-outcome-read", events.read(&mut part))?;
        if n == 0 {
            return Err(invalid("c4-root-outcome-eof"));
        }
        bytes.extend(&part[..n]);
        if bytes.len() == 4 {
            let len = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            if len == 0 || len > 4096 {
                return Err(invalid("c4-root-outcome-size"));
            }
            goal = 4 + len;
        }
    }
    serde_json::from_slice(&bytes[4..]).map_err(|_| invalid("c4-root-outcome-json"))
}
fn rx_tree(root: &Path, p: &Profile) -> Result<()> {
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        set_profile_acl(&dir, true, p.sid, RX)?;
        for entry in io("c4-root-acl-list", fs::read_dir(&dir))? {
            let entry = io("c4-root-acl-entry", entry)?;
            let kind = io("c4-root-acl-kind", entry.file_type())?;
            if kind.is_dir() {
                dirs.push(entry.path());
            } else {
                set_profile_acl(&entry.path(), false, p.sid, RX)?;
            }
        }
    }
    Ok(())
}
pub(in super::super::super) fn run(
    pins: &CandidatePins,
    store: &mut Store,
    bad_mac: bool,
) -> Result<Value> {
    let actor = hash(&io("c4-root-self", std::env::current_exe())?)?;
    store.c2_guard(actor, store.state.head)?;
    let _base = store.verify_c2(actor, store.state.head)?;
    if MAX_RECORDS.saturating_sub(store.state.sequence + 1) < 6 {
        return Err(invalid("c4-root-record-budget"));
    }
    environment_clear(std::env::vars_os())?;
    registry_clear()?;
    let id = store.state.c2.as_ref().unwrap().id;
    let p = profile(id)?;
    let ui = data_root(id).join("c4-candidate/ui");
    if root_identity(&ui)? != pins._ui_root
        || local_path(&pins.exe)? != local_path(&ui.join("ui.exe"))?
        || hash(&pins.exe)? != pins.exe_sha256
    {
        return Err(invalid("c4-root-candidate"));
    }
    rx_tree(&ui, &p)?; // Only this new non-secret candidate; no C2/host/control ACL changes.
    let launch = Uuid::new_v4();
    let mut nonce = [0; 32];
    getrandom::fill(&mut nonce).map_err(|_| invalid("c4-root-nonce"))?;
    let data = data_root(id).join(format!("c4-runtime-{}", launch.simple()));
    secure_directory(&data)?;
    set_profile_acl(&data, true, p.sid, MODIFY)?;
    low_label(&data)?;
    let udf = data.join("webview");
    secure_directory(&udf)?;
    set_profile_acl(&udf, true, p.sid, MODIFY)?;
    low_label(&udf)?;
    let raw = unsafe { CreateJobObjectW(null(), store.job_name().as_ptr()) };
    if raw.is_null() {
        return Err(fail("c4-root-job"));
    }
    let job = Job(Handle(raw));
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Err(invalid("c4-root-job-collision"));
    }
    if !(3..=16).contains(&pins.process_limit) {
        return Err(invalid("c4-root-process-policy"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    limits.BasicLimitInformation.ActiveProcessLimit = u32::from(pins.process_limit);
    check(
        unsafe {
            SetInformationJobObject(
                job.0.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&limits) as u32,
            )
        },
        "c4-root-job-policy",
    )?;
    let guardian = identity(unsafe { GetCurrentProcess() })?;
    let mut event = C4ProbeEvent {
        version: 1,
        stage: Stage::Commit,
        launch,
        workspace: id,
        package: pins._package,
        guardian,
        nonce,
        root: None,
    };
    append(store, &mut event, Stage::Commit, &job, None, &p)?;
    let grant = Grant {
        version: 1,
        purpose: "c4-blank-runtime-probe".into(),
        launch,
        workspace: id,
        guardian,
        nonce,
        package: pins._package,
        code: pins.exe_sha256,
        ui: ui.clone(),
        runtime: ui.join("runtime"),
        data: udf,
        profile: name(id),
    };
    let mut key = Zeroizing::new([0; 32]);
    getrandom::fill(key.as_mut()).map_err(|_| invalid("c4-root-key"))?;
    let mut wire = encode(&grant, key.as_ref())?;
    if bad_mac {
        wire[4] ^= 1;
    }
    let boot = prepaid(&wire)?;
    let secret = prepaid(key.as_ref())?;
    let (mut events, write_events) = pipe()?;
    let (go, mut write_go) = pipe()?;
    let handles = [
        boot.as_raw_handle(),
        secret.as_raw_handle(),
        write_events.as_raw_handle(),
        go.as_raw_handle(),
    ];
    inherit(&handles, true)?;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: p.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let attrs = Attributes::with_handles_and_jobs(&caps, &handles, &[job.0.0])?;
    let mut si: STARTUPINFOEXW = unsafe { zeroed() };
    si.StartupInfo.cb = size_of_val(&si) as u32;
    si.lpAttributeList = attrs.ptr;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let mut cmd = wide(format!(
        "\"{}\" c4-root-probe-worker {} {} {} {}",
        pins.exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize,
        handles[3] as usize
    ));
    let env = crate::windows::exchange::ui_restart::environment(&data);
    let ok = unsafe {
        CreateProcessW(
            wide(&pins.exe).as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | DETACHED_PROCESS,
            env.as_ptr().cast(),
            wide(&ui).as_ptr(),
            &si.StartupInfo,
            &mut pi,
        )
    };
    let root = if ok != 0 {
        Some(Process(Handle(pi.hProcess), Handle(pi.hThread)))
    } else {
        None
    };
    let create_error = (ok == 0).then(|| fail("c4-root-create"));
    inherit(&handles, false)?;
    drop(attrs);
    drop(boot);
    drop(secret);
    drop(write_events);
    drop(go);
    let mut seen = std::collections::BTreeMap::new();
    let mut accepted = false;
    let mut continued = false;
    let attempt = (|| {
        if let Some(e) = create_error {
            return Err(e);
        }
        let root = root.as_ref().ok_or(invalid("c4-root-process"))?;
        if !same_image(&image(root.0.0)?, &pins.exe)? {
            return Err(invalid("c4-root-mapped-image"));
        }
        verify_denials(root.0.0, &store.root)?;
        append(store, &mut event, Stage::RootBound, &job, Some(root), &p)?;
        sample(&job, identity(root.0.0)?, &ui, &mut seen)?;
        if unsafe { ResumeThread(root.1.0) } == u32::MAX {
            return Err(fail("c4-root-resume"));
        }
        let deadline = Instant::now() + Duration::from_secs(30);
        let ack: Ack = decode(&receive(&mut events, deadline)?, key.as_ref())?;
        if ack.version != 1
            || ack.sequence != 1
            || ack.process != identity(root.0.0)?
            || ack.grant != grant_hash(&grant)?
        {
            return Err(invalid("c4-root-ack"));
        }
        append(store, &mut event, Stage::Accepted, &job, Some(root), &p)?;
        accepted = true;
        append(store, &mut event, Stage::Running, &job, Some(root), &p)?;
        io(
            "c4-root-continue",
            write_go.write_all(&encode(
                &Ack {
                    version: 1,
                    sequence: 2,
                    grant: grant_hash(&grant)?,
                    process: guardian,
                },
                key.as_ref(),
            )?),
        )?;
        continued = true;
        let result = outcome(&mut events, deadline, &job, ack.process, &ui, &mut seen)?;
        Ok(result)
    })();
    drop(write_go);
    key.zeroize();
    drop(key);
    job.stop()?;
    let (outcome, error) = match attempt {
        Ok(v) => (v, Value::Null),
        Err(e) => (Value::Null, json!({"stage":e.stage,"code":e.code})),
    };
    let end_deadline = Instant::now() + Duration::from_secs(5);
    for observation in seen.values_mut() {
        let wait_ms = end_deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(5000) as u32;
        if unsafe { WaitForSingleObject(observation.process.0, wait_ms) } != WAIT_OBJECT_0 {
            return Err(invalid("c4-observed-process-not-ended"));
        }
        let mut code = 0;
        check(
            unsafe { GetExitCodeProcess(observation.process.0, &mut code) },
            "c4-observed-exit-code",
        )?;
        observation.value["ended"] = json!(true);
        observation.value["exitCode"] = json!(code);
        observation.value["moduleObservation"] = observation.modules.value();
    }
    let mut totals: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
    check(
        unsafe {
            QueryInformationJobObject(
                job.0.0,
                JobObjectBasicAccountingInformation,
                (&mut totals as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of_val(&totals) as u32,
                null_mut(),
            )
        },
        "c4-root-job-final-accounting",
    )?;
    append(store, &mut event, Stage::Drained, &job, None, &p)?;
    append(store, &mut event, Stage::Finish, &job, None, &p)?;
    let mut root_exit = 0;
    if let Some(root) = &root {
        check(
            unsafe { GetExitCodeProcess(root.0.0, &mut root_exit) },
            "c4-root-exit-code",
        )?;
    }
    drop(root);
    Ok(
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"rootAuthenticated":accepted,"continueSent":continued,"badMac":bad_mac,"runtimeOutcome":outcome,"controllerError":error,"processObservations":seen.values().map(|o|&o.value).collect::<Vec<_>>(),"jobTotalProcesses":totals.TotalProcesses,"jobTerminatedProcesses":totals.TotalTerminatedProcesses,"unobservedProcessCount":totals.TotalProcesses.saturating_sub(seen.len() as u32),"rootExitCode":root_exit,"drained":true,"controlFinished":!store.state.c4_probe.pending(),"uiAbiSupported":0,"networkMeasurement":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c4_image_identity_accepts_case_alias_but_not_equal_content_copy() {
        let root = std::env::temp_dir().join(format!(
            "kraqel-isolation-controller-{}",
            Uuid::new_v4().simple()
        ));
        secure_directory(&root).unwrap();
        let a = root.join("image.exe");
        let b = root.join("copy.exe");
        write_new(&a, b"synthetic same content, never executed").unwrap();
        write_new(&b, b"synthetic same content, never executed").unwrap();
        assert!(same_image(&a, &root.join("IMAGE.EXE")).unwrap());
        assert_eq!(hash(&a).unwrap(), hash(&b).unwrap());
        assert!(!same_image(&a, &b).unwrap());
        let link = root.join("alias.exe");
        fs::hard_link(&a, &link).unwrap();
        assert!(same_image(&a, &link).is_err());
        let resolved = root.canonicalize().unwrap();
        assert_eq!(
            resolved.parent(),
            Some(std::env::temp_dir().canonicalize().unwrap().as_path())
        );
        assert!(crate::policy::valid_root(&resolved, true));
        fs::remove_dir_all(resolved).unwrap();
    }
}
