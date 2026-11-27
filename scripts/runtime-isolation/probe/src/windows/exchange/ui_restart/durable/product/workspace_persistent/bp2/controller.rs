//! Native BP2 controller, callable only by the explicit synthetic test.
#![cfg_attr(not(test), allow(dead_code))]
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum Mode {
    Normal,
    UnsupportedRequirement,
    ReadbackMismatch,
    BadMac,
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
            "bp2-root-job-count",
        )?;
        Ok(i.ActiveProcesses)
    }
    fn stop(&self) -> Result<()> {
        check(
            unsafe { TerminateJobObject(self.0.0, 3) },
            "bp2-root-job-stop",
        )?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.count()? != 0 {
            if Instant::now() >= deadline {
                return Err(invalid("bp2-root-job-drain"));
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
        "bp2-root-image",
    )?;
    Ok(PathBuf::from(
        String::from_utf16(&bytes[..n as usize]).map_err(|_| invalid("bp2-root-image"))?,
    ))
}
fn same_image(actual: &Path, expected: &Path) -> Result<bool> {
    fn file_id(path: &Path) -> Result<([u8; 16], u64)> {
        let _parents = pin_directories(path.parent().ok_or(invalid("bp2-image-parent"))?)?;
        let file = io(
            "bp2-image-pin",
            OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(local_path(path)?),
        )?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
        check(
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
            "bp2-image-info",
        )?;
        if info.dwFileAttributes & (BAD_ATTRIBUTES | FILE_ATTRIBUTE_DIRECTORY) != 0
            || info.nNumberOfLinks != 1
        {
            return Err(invalid("bp2-image-linked-or-wrong-kind"));
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
            "bp2-image-file-id",
        )?;
        Ok((id.FileId.Identifier, id.VolumeSerialNumber))
    }
    Ok(file_id(actual)? == file_id(expected)?)
}
fn append(
    store: &mut Store,
    e: &mut Bp2Event,
    stage: Stage,
    job: &Job,
    root: Option<&Process>,
    p: &Profile,
) -> Result<()> {
    if e.guardian != identity(unsafe { GetCurrentProcess() })? {
        return Err(invalid("bp2-root-guardian-generation"));
    }
    let count = job.count()?;
    if matches!(stage, Stage::Commit | Stage::Drained | Stage::Finish) {
        if count != 0 {
            return Err(invalid("bp2-root-record-not-drained"));
        }
    } else {
        if count != 1 {
            return Err(invalid("bp2-root-record-group"));
        }
        let root = root.ok_or(invalid("bp2-root-record-process"))?;
        let id = identity(root.0.0)?;
        if stage == Stage::RootBound {
            e.root = Some(id);
        } else if e.root != Some(id) {
            return Err(invalid("bp2-root-record-generation"));
        }
        let mut yes = 0;
        check(
            unsafe { IsProcessInJob(root.0.0, job.0.0, &mut yes) },
            "bp2-root-record-job",
        )?;
        let facts = token_facts(root.0.0, Some(p.sid))?;
        if yes == 0
            || facts["isAppContainer"] != true
            || facts["capabilityCount"] != 0
            || facts["expectedIdentity"] != true
        {
            return Err(invalid("bp2-root-record-token"));
        }
    }
    let mut next = e.clone();
    next.stage = stage;
    store.append_with_checked_context(
        store.state.actor,
        store.state.head,
        Action::Bp2LeafProbe {
            event: next.clone(),
        },
    )?;
    *e = next;
    Ok(())
}

fn medium_code(path: &Path) -> Result<()> {
    let mut sd = null_mut();
    check(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide("S:(ML;OICI;NW;;;ME)").as_ptr(),
                1,
                &mut sd,
                null_mut(),
            )
        },
        "bp2-code-label",
    )?;
    let _sd = Local(sd);
    let mut sacl = null_mut();
    let (mut present, mut defaulted) = (0, 0);
    check(
        unsafe { GetSecurityDescriptorSacl(sd, &mut present, &mut sacl, &mut defaulted) },
        "bp2-code-sacl",
    )?;
    let code = unsafe {
        SetNamedSecurityInfoW(
            wide(path).as_ptr(),
            SE_FILE_OBJECT,
            LABEL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null(),
            sacl,
        )
    };
    if code != 0 {
        return Err(Failure {
            stage: "bp2-code-medium-label",
            code: code.into(),
        });
    }
    Ok(())
}
pub(in super::super) fn run(executable: &Path, store: &mut Store, mode: Mode) -> Result<Value> {
    let actor = hash(&io("bp2-self", std::env::current_exe())?)?;
    store.c2_guard(actor, store.state.head)?;
    let _base = store.verify_c2(actor, store.state.head)?;
    let support = policy::support()?;
    let required = if mode == Mode::UnsupportedRequirement {
        let absent = (0..64)
            .find(|n| support[0] & (1u64 << n) == 0)
            .ok_or(invalid("bp2-no-negative-support-bit"))?;
        policy::REQUIRED | (1u64 << absent)
    } else {
        policy::REQUIRED
    };
    if policy::require_supported(required, support).is_err() {
        return Ok(
            json!({"rejectedBeforeCreate":true,"stage":"bp2-unsupported-requirement","recordsUnchanged":true,"nativeAuthority":false}),
        );
    }
    if MAX_RECORDS.saturating_sub(store.state.sequence + 1) < 6 {
        return Err(invalid("bp2-record-budget"));
    }
    let workspace = store.state.c2.as_ref().unwrap().id;
    let p = profile(workspace)?;
    let launch = Uuid::new_v4();
    let mut nonce = [0; 32];
    getrandom::fill(&mut nonce).map_err(|_| invalid("bp2-nonce"))?;
    let code_root = data_root(workspace).join(format!("bp2-leaf-{}", launch.simple()));
    secure_directory(&code_root)?;
    set_profile_acl(&code_root, true, p.sid, RX)?;
    medium_code(&code_root)?;
    let exe = code_root.join("leaf.exe");
    let (_source_dirs, mut source, code_hash) = binary_pin(executable)?;
    package::verify_persistent_records(&mut source)?;
    io("bp2-source-rewind", source.rewind())?;
    let mut bytes = Vec::new();
    io(
        "bp2-source-read",
        source.take(64 * 1024 * 1024 + 1).read_to_end(&mut bytes),
    )?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid("bp2-source-bound"));
    }
    write_new(&exe, &bytes)?;
    set_profile_acl(&exe, false, p.sid, RX)?;
    medium_code(&exe)?;
    let (_code_dirs, _code_pin, pinned_hash) = binary_pin(&exe)?;
    if pinned_hash != code_hash {
        return Err(invalid("bp2-code-copy"));
    }
    let package = Sha256::digest(
        [
            b"KRAQEL-BP2-PACKAGE\0\x01".as_slice(),
            &code_hash,
            &required.to_le_bytes(),
            &policy::DENY_CHILDREN.to_le_bytes(),
        ]
        .concat(),
    )
    .into();
    let raw = unsafe { CreateJobObjectW(null(), store.job_name().as_ptr()) };
    if raw.is_null() {
        return Err(fail("bp2-job"));
    }
    let job = Job(Handle(raw));
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Err(invalid("bp2-job-collision"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    // Diagnostic capacity for the attempted suspended child: denial must be 367, not quota.
    limits.BasicLimitInformation.ActiveProcessLimit = 2;
    check(
        unsafe {
            SetInformationJobObject(
                job.0.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&limits) as u32,
            )
        },
        "bp2-job-limit",
    )?;
    let guardian = identity(unsafe { GetCurrentProcess() })?;
    let mut event = Bp2Event {
        version: 1,
        stage: Stage::Commit,
        launch,
        workspace,
        package,
        guardian,
        nonce,
        root: None,
        receipt: [0; 32],
    };
    append(store, &mut event, Stage::Commit, &job, None, &p)?;
    let grant = Grant {
        version: 1,
        purpose: "bp2-synthetic-leaf-capability".into(),
        launch,
        workspace,
        guardian,
        nonce,
        package,
        code: code_hash,
        executable: exe.clone(),
        profile: name(workspace),
        required,
        child_policy: policy::DENY_CHILDREN,
    };
    let mut key = Zeroizing::new([0; 32]);
    getrandom::fill(key.as_mut()).map_err(|_| invalid("bp2-key"))?;
    let mut wire = encode(&grant, key.as_ref())?;
    if mode == Mode::BadMac {
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
    let child_policy = policy::DENY_CHILDREN;
    let jobs = [job.0.0];
    let attrs =
        Attributes::with_creation_policy(&caps, &handles, &jobs, Some((&required, &child_policy)))?;
    let mut si: STARTUPINFOEXW = unsafe { zeroed() };
    si.StartupInfo.cb = size_of_val(&si) as u32;
    si.lpAttributeList = attrs.ptr;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let mut cmd = wide(format!(
        "\"{}\" bp2-leaf-worker {} {} {} {}",
        exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize,
        handles[3] as usize
    ));
    let env = crate::windows::exchange::ui_restart::environment(&code_root);
    let ok = unsafe {
        CreateProcessW(
            wide(&exe).as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | DETACHED_PROCESS,
            env.as_ptr().cast(),
            wide(&code_root).as_ptr(),
            &si.StartupInfo,
            &mut pi,
        )
    };
    let create_error = (ok == 0).then(|| fail("bp2-root-create"));
    let root = (ok != 0).then(|| Process(Handle(pi.hProcess), Handle(pi.hThread)));
    inherit(&handles, false)?;
    drop(attrs);
    drop(boot);
    drop(secret);
    drop(write_events);
    drop(go);
    let (mut resumed, mut accepted, mut continued) = (false, false, false);
    let mut observation = Value::Null;
    let attempt = (|| {
        if let Some(e) = create_error {
            return Err(e);
        }
        let root = root.as_ref().ok_or(invalid("bp2-root-missing"))?;
        if !same_image(&image(root.0.0)?, &exe)? {
            return Err(invalid("bp2-root-image"));
        }
        verify_denials(root.0.0, &store.root)?;
        append(store, &mut event, Stage::RootBound, &job, Some(root), &p)?;
        let id = identity(root.0.0)?;
        let actual = policy::readback(root.0.0)?;
        observation = json!({"pid":id.pid,"created":id.created,"imageSha256":hex(&code_hash),"expectedImage":true,"token":token_facts(root.0.0,Some(p.sid))?,"imagePolicyBeforeResume":actual.0,"childPolicyBeforeResume":actual.1});
        policy::require_readback(actual, mode == Mode::ReadbackMismatch)?;
        if unsafe { ResumeThread(root.1.0) } == u32::MAX {
            return Err(fail("bp2-root-resume"));
        }
        resumed = true;
        let deadline = Instant::now() + Duration::from_secs(20);
        let ack: Ack = decode(&receive(&mut events, deadline)?, key.as_ref())?;
        if ack.version != 1
            || ack.sequence != 1
            || ack.grant != grant_hash(&grant)?
            || ack.process != id
        {
            return Err(invalid("bp2-root-ack"));
        }
        append(store, &mut event, Stage::Accepted, &job, Some(root), &p)?;
        accepted = true;
        append(store, &mut event, Stage::Running, &job, Some(root), &p)?;
        io(
            "bp2-root-continue",
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
        let result: Outcome = decode(&receive(&mut events, deadline)?, key.as_ref())?;
        if result.version != 1 || result.grant != grant_hash(&grant)? || result.process != id {
            return Err(invalid("bp2-outcome-binding"));
        }
        policy::require_readback((result.image_policy, result.child_policy), false)?;
        if result.child_created || result.child_error != ERROR_CHILD_PROCESS_BLOCKED {
            return Err(invalid("bp2-child-denial-not-proven"));
        }
        Ok(
            json!({"imagePolicyInLeaf":result.image_policy,"childPolicyInLeaf":result.child_policy,"childError":result.child_error,"childCreated":result.child_created}),
        )
    })();
    drop(write_go);
    key.zeroize();
    drop(key);
    job.stop()?;
    let (result, error) = match attempt {
        Ok(v) => (v, Value::Null),
        Err(e) => (Value::Null, json!({"stage":e.stage,"code":e.code})),
    };
    let mut root_exit = 0;
    if let Some(root) = &root {
        if unsafe { WaitForSingleObject(root.0.0, 5000) } != WAIT_OBJECT_0 {
            return Err(invalid("bp2-root-not-ended"));
        }
        check(
            unsafe { GetExitCodeProcess(root.0.0, &mut root_exit) },
            "bp2-root-exit",
        )?;
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
        "bp2-job-final",
    )?;
    let receipt = json!({"version":1,"mode":format!("{mode:?}"),"launch":launch,"guardian":guardian,"root":event.root,"rootCodeSha256":hex(&code_hash),"supportMask":support,"required":required,"childPolicyRequested":child_policy,"rootObservation":observation,"resumed":resumed,"accepted":accepted,"continued":continued,"result":result,"error":error,"rootExitCode":root_exit,"jobTotalProcesses":totals.TotalProcesses,"drained":true,"nativeRuntimeAuthority":false,"nativeDataAuthority":false});
    let receipt_bytes = serde_json::to_vec(&receipt).map_err(|_| invalid("bp2-receipt-json"))?;
    event.receipt = Sha256::digest(&receipt_bytes).into();
    append(store, &mut event, Stage::Drained, &job, None, &p)?;
    append(store, &mut event, Stage::Finish, &job, None, &p)?;
    drop(root);
    Ok(
        json!({"receipt":receipt,"receiptSha256":hex(&event.receipt),"controlFinished":!store.state.bp2_probe.pending()}),
    )
}
