use super::*;
use std::cell::Cell;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
use windows_sys::Win32::System::SystemServices::{
    SYSTEM_MANDATORY_LABEL_ACE_TYPE, SYSTEM_MANDATORY_LABEL_NO_WRITE_UP,
};

pub(super) fn identity(process: HANDLE) -> Result<Identity> {
    let (mut c, mut e, mut k, mut u) = unsafe { zeroed() };
    check(
        unsafe { GetProcessTimes(process, &mut c, &mut e, &mut k, &mut u) },
        "c3-process-times",
    )?;
    let id = Identity {
        pid: unsafe { GetProcessId(process) },
        created: (u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime),
    };
    if !id.valid() {
        return Err(invalid("c3-process-identity"));
    }
    Ok(id)
}
pub(super) struct GroupJob {
    job: Handle,
    launch: crate::product_start_p2c::LaunchIdentity,
    epoch: Uuid,
    guardian: Identity,
    outside_root: Cell<bool>,
}
pub(super) fn fresh_guardian(p: &Pending) -> Result<()> {
    if p.phase != Phase::Committed || p.guardian != identity(unsafe { GetCurrentProcess() })? {
        return Err(invalid("c3-not-fresh-guardian"));
    }
    Ok(())
}
impl GroupJob {
    pub fn create(store: &Store) -> Result<Self> {
        let p = store
            .state
            .c3
            .pending
            .as_ref()
            .ok_or(invalid("c3-no-pending"))?;
        fresh_guardian(p)?;
        let handle = unsafe { CreateJobObjectW(null(), store.job_name().as_ptr()) };
        if handle.is_null() {
            return Err(fail("c3-job-create"));
        }
        let job = Handle(handle);
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return Err(invalid("c3-job-collision"));
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        limits.BasicLimitInformation.ActiveProcessLimit = 2;
        check(
            unsafe {
                SetInformationJobObject(
                    job.0,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            },
            "c3-job-policy",
        )?;
        Ok(Self {
            job,
            launch: p.launch,
            epoch: p.epoch,
            guardian: p.guardian,
            outside_root: Cell::new(false),
        })
    }
    pub fn count(&self) -> Result<u32> {
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    self.job.0,
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            "c3-job-count",
        )?;
        Ok(info.ActiveProcesses)
    }
    fn policy(&self) -> Result<()> {
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    self.job.0,
                    JobObjectExtendedLimitInformation,
                    (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            "c3-job-check-policy",
        )?;
        if limits.BasicLimitInformation.ActiveProcessLimit != 2
            || limits.BasicLimitInformation.LimitFlags
                != (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS)
        {
            return Err(invalid("c3-job-policy-changed"));
        }
        Ok(())
    }
    pub fn append(&self, store: &mut Store, event: GroupEvent) -> Result<()> {
        let pending = store
            .state
            .c3
            .pending
            .as_ref()
            .ok_or(invalid("c3-no-pending"))?;
        if pending.launch != self.launch
            || pending.epoch != self.epoch
            || pending.guardian != self.guardian
            || identity(unsafe { GetCurrentProcess() })? != self.guardian
        {
            return Err(invalid("c3-guardian-binding"));
        }
        // Only this continuously held job object grants the active record path. Never reconstruct it.
        self.policy()?;
        match open_file(&store.root.join("control/owner.lock"), true, false) {
            Err(e) if e.code == i64::from(ERROR_SHARING_VIOLATION) => {}
            _ => return Err(invalid("c3-lease-not-held")),
        }
        let count = self.count()?;
        if matches!(event, GroupEvent::Drained | GroupEvent::Finish { .. }) {
            if count != 0 {
                return Err(invalid("c3-group-still-active"));
            }
        } else if matches!(event, GroupEvent::Staged { .. }) {
            if count != 0 {
                return Err(invalid("c3-stage-not-empty"));
            }
        } else if matches!(
            event,
            GroupEvent::RootBound { .. } | GroupEvent::Accepted | GroupEvent::Running
        ) {
            if count != 1 {
                return Err(invalid("c3-root-only-group-count"));
            }
        } else if count == 0 || count > 2 {
            return Err(invalid("c3-active-group-count"));
        }
        if let GroupEvent::Staged { root } = &event {
            let id = store.state.c2.as_ref().unwrap().id;
            let path = launch_root(id, self.launch.id);
            if &root_identity(&path)? != root {
                return Err(invalid("c3-stage-identity"));
            }
            check_low_label(&path)?;
        }
        let current = match &event {
            GroupEvent::RootBound { identity } | GroupEvent::ChildBound { identity } => {
                Some(*identity)
            }
            GroupEvent::Accepted | GroupEvent::Running => pending.root,
            _ => None,
        };
        if let Some(expected) = current {
            let handle = Handle(unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    expected.pid,
                )
            });
            if handle.0.is_null() || identity(handle.0)? != expected {
                return Err(invalid("c3-record-process-identity"));
            }
            let id = store.state.c2.as_ref().unwrap().id;
            self.check_process(
                handle.0,
                &profile(id)?,
                store
                    .state
                    .c3
                    .code
                    .as_ref()
                    .ok_or(invalid("c3-code-unbound"))?,
                id,
            )?;
            self.members(expected.pid, pending.root.map(|r| r.pid))?;
        }
        store.append_with_checked_context(store.state.actor, store.state.head, Action::C3 { event })
    }
    pub fn stop_and_drain(&self) -> Result<()> {
        if self.outside_root.get() {
            return Err(invalid("c3-uncontained-root-end-unproved"));
        }
        check(unsafe { TerminateJobObject(self.job.0, 3) }, "c3-job-stop")?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.count()? != 0 {
            if Instant::now() >= deadline {
                return Err(invalid("c3-drain-deadline"));
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
    fn members(&self, required: u32, root: Option<u32>) -> Result<()> {
        #[repr(C)]
        struct List {
            assigned: u32,
            listed: u32,
            ids: [usize; 2],
        }
        let mut list: List = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    self.job.0,
                    JobObjectBasicProcessIdList,
                    (&mut list as *mut List).cast(),
                    size_of::<List>() as u32,
                    null_mut(),
                )
            },
            "c3-job-members",
        )?;
        if list.listed == 0
            || list.listed > 2
            || !list.ids[..list.listed as usize].contains(&(required as usize))
            || list.ids[..list.listed as usize]
                .iter()
                .any(|pid| *pid != required as usize && Some(*pid as u32) != root)
        {
            return Err(invalid("c3-unexpected-job-member"));
        }
        Ok(())
    }
    pub fn check_process(
        &self,
        process: HANDLE,
        p: &Profile,
        code: &Code,
        id: Uuid,
    ) -> Result<Value> {
        self.policy()?;
        let mut yes = 0;
        check(
            unsafe { IsProcessInJob(process, self.job.0, &mut yes) },
            "c3-exact-job",
        )?;
        let facts = token_facts(process, Some(p.sid))?;
        if yes == 0
            || facts["isAppContainer"] != true
            || facts["capabilityCount"] != 0
            || facts["expectedIdentity"] != true
        {
            return Err(invalid("c3-job-or-token"));
        }
        let mut path = [0u16; 32768];
        let mut len = path.len() as u32;
        check(
            unsafe { QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut len) },
            "c3-process-image",
        )?;
        let actual = PathBuf::from(
            String::from_utf16(&path[..len as usize]).map_err(|_| invalid("c3-process-image"))?,
        );
        if local_path(&actual)? != local_path(&code_root(id).join("worker.exe"))?
            || hash(&actual)? != code.sha256
        {
            return Err(invalid("c3-process-package"));
        }
        Ok(facts)
    }
}
fn parent_is(child: u32, parent: u32) -> Result<()> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(fail("c3-snapshot"));
    }
    let snapshot = Handle(raw);
    let mut p: PROCESSENTRY32W = unsafe { zeroed() };
    p.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut p) };
    while more != 0 {
        if p.th32ProcessID == child {
            return if p.th32ParentProcessID == parent {
                Ok(())
            } else {
                Err(invalid("c3-child-parent"))
            };
        }
        more = unsafe { Process32NextW(snapshot.0, &mut p) };
    }
    Err(invalid("c3-child-not-found"))
}
fn wait_exit(process: HANDLE) -> Result<u32> {
    if unsafe { WaitForSingleObject(process, 10000) } != WAIT_OBJECT_0 {
        return Err(invalid("c3-process-timeout"));
    }
    let mut code = 0;
    check(
        unsafe { GetExitCodeProcess(process, &mut code) },
        "c3-process-exit",
    )?;
    Ok(code)
}

pub(super) fn run(
    store: &mut Store,
    job: &GroupJob,
    p: &Profile,
    id: Uuid,
    code: &Code,
    mode: Mode,
) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let pending = store.state.c3.pending.as_ref().unwrap().clone();
    let data = data_root(id);
    let exe = code_root(id).join("worker.exe");
    let _exe = binary_pin(&exe)?;
    let binding = store.state.c2.as_ref().unwrap().binding.as_ref().unwrap();
    let grant = Grant {
        version: 1,
        purpose: "c3-synthetic-group".into(),
        role: "root".into(),
        parent_pid: pending.guardian.pid,
        guardian: pending.guardian,
        launch: pending.launch,
        epoch: pending.epoch,
        nonce: pending.nonce,
        workspace: id,
        root: data.clone(),
        profile_name: name(id),
        code: code.clone(),
        binding_sha256: Sha256::digest(
            serde_json::to_vec(binding).map_err(|_| invalid("c3-binding-json"))?,
        )
        .into(),
        data_sha256: Sha256::digest(read_data(&data.join("projects/synthetic.txt"))?).into(),
        bad_leaf: mode == Mode::BadLeaf,
    };
    let mut key = Zeroizing::new([0; 32]);
    getrandom::fill(key.as_mut()).map_err(|_| invalid("c3-key"))?;
    let mut wire = encode(&grant, &key)?;
    if mode == Mode::BadMac {
        wire[4] ^= 1;
    }
    let boot = prepaid(&wire)?;
    let secret = prepaid(key.as_ref())?;
    let (mut events, event_write) = pipe()?;
    let (go, mut send_go) = pipe()?;
    let handles = [
        boot.as_raw_handle(),
        secret.as_raw_handle(),
        event_write.as_raw_handle(),
        go.as_raw_handle(),
    ];
    inherited(&handles, true)?;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: p.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let jobs = [job.job.0];
    let attrs = Attributes::with_handles_and_jobs(
        &caps,
        &handles,
        if mode == Mode::Unassigned { &[] } else { &jobs },
    )?;
    let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.lpAttributeList = attrs.ptr;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let mut cmd = wide(format!(
        "\"{}\" c3-synthetic-worker root {} {} {} {} {}",
        exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize,
        handles[3] as usize,
        Uuid::from_bytes(pending.launch.id)
    ));
    let env = super::super::super::super::super::environment(&launch_root(id, pending.launch.id));
    let ok = unsafe {
        CreateProcessW(
            wide(&exe).as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_UNICODE_ENVIRONMENT
                | CREATE_SUSPENDED
                | DETACHED_PROCESS,
            env.as_ptr().cast(),
            wide(exe.parent().ok_or(invalid("c3-code-parent"))?).as_ptr(),
            &startup.StartupInfo,
            &mut pi,
        )
    };
    let error = (ok == 0).then(|| fail("c3-create-root"));
    let root = (ok != 0).then(|| Worker(Handle(pi.hProcess), Handle(pi.hThread)));
    if root.is_some() {
        job.outside_root.set(true);
    }
    inherited(&handles, false)?;
    drop(attrs);
    drop(boot);
    drop(secret);
    drop(event_write);
    drop(go);
    if let Some(e) = error {
        return Err(e);
    }
    let root = root.ok_or(invalid("c3-root-missing"))?;
    let mut exact = 0;
    if unsafe { IsProcessInJob(root.0.0, job.job.0, &mut exact) } == 0 || exact == 0 {
        check(
            unsafe { TerminateProcess(root.0.0, 1) },
            "c3-stop-unassigned-root",
        )?;
        wait_exit(root.0.0)?;
        job.outside_root.set(false);
        return Err(invalid("c3-root-not-assigned-before-resume"));
    }
    job.outside_root.set(false);
    let facts = job.check_process(root.0.0, p, code, id)?;
    verify_denials(root.0.0, &store.root)?;
    let root_id = identity(root.0.0)?;
    job.append(store, GroupEvent::RootBound { identity: root_id })?;
    if unsafe { ResumeThread(root.1.0) } == u32::MAX {
        return Err(fail("c3-resume-root"));
    }
    let ack: Message = decode(&read_frame(&mut events, deadline)?, &key)?;
    verify_message(&ack, &grant, "accepted", 1)?;
    if ack.process != root_id {
        return Err(invalid("c3-root-receipt-pid"));
    }
    job.append(store, GroupEvent::Accepted)?;
    job.append(store, GroupEvent::Running)?;
    io(
        "c3-continue",
        send_go.write_all(&encode(
            &message(&grant, "continue", 2, pending.guardian, 0)?,
            &key,
        )?),
    )?;
    drop(send_go);
    let child: Message = decode(&read_frame(&mut events, deadline)?, &key)?;
    verify_message(&child, &grant, "child-prepared", 3)?;
    if child.process.pid == root_id.pid || child.thread_id == 0 {
        return Err(invalid("c3-child-alias"));
    }
    let handle = Handle(unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            child.process.pid,
        )
    });
    if handle.0.is_null() {
        return Err(fail("c3-open-child"));
    }
    if identity(handle.0)? != child.process {
        return Err(invalid("c3-child-identity"));
    }
    parent_is(child.process.pid, root_id.pid)?;
    let child_facts = job.check_process(handle.0, p, code, id)?;
    verify_denials(handle.0, &store.root)?;
    let thread = Handle(unsafe {
        OpenThread(
            THREAD_SUSPEND_RESUME | THREAD_QUERY_LIMITED_INFORMATION,
            0,
            child.thread_id,
        )
    });
    if thread.0.is_null() || unsafe { GetProcessIdOfThread(thread.0) } != child.process.pid {
        return Err(invalid("c3-child-thread"));
    }
    job.append(
        store,
        GroupEvent::ChildBound {
            identity: child.process,
        },
    )?;
    if wait_exit(root.0.0)? != 0
        || unsafe { WaitForSingleObject(handle.0, 0) } != WAIT_TIMEOUT
        || job.count()? != 1
    {
        return Err(invalid("c3-root-end-is-not-group-end"));
    }
    // Root is gone, child still suspended: both a second writer and profile retirement must stay blocked.
    if Store::open(&store.root).is_ok()
        || store.retire_c2(store.state.actor, store.state.head).is_ok()
    {
        return Err(invalid("c3-lease-or-retirement-escaped"));
    }
    if mode == Mode::CrashGuardian {
        write_json(
            &store.root.parent().unwrap().join("c3-crash.json"),
            &json!({"root":root_id,"child":child.process,
            "rootExited":true,"childSuspended":true,"activeProcesses":1,"controlHead":hex(&store.state.head)}),
        )?;
        crash::hit(crash::Point::C3RootEnded);
        return Err(invalid("c3-crash-not-armed"));
    }
    if unsafe { ResumeThread(thread.0) } == u32::MAX {
        return Err(fail("c3-resume-child"));
    }
    let mut delegated = grant.clone();
    delegated.role = "leaf".into();
    delegated.parent_pid = root_id.pid;
    let done: Message = decode(&read_frame(&mut events, deadline)?, &key)?;
    verify_message(&done, &delegated, "leaf-done", 4)?;
    if done.process != child.process || wait_exit(handle.0)? != 0 {
        return Err(invalid("c3-child-not-completed"));
    }
    if read_data(&launch_root(id, pending.launch.id).join("leaf.txt"))? != worker::LEAF_TEXT {
        return Err(invalid("c3-leaf-data"));
    }
    Ok(
        json!({"rootExitedWhileChildAlive":true,"activeAfterRootExit":1,"secondWriterAndRetirementBlocked":true,
        "rootToken":facts,"childToken":child_facts,"privateDenials":12,"rootIdentity":root_id,"childIdentity":child.process,
        "childParentMatched":true,"rootAndChildExit":0,"dataReadAndWriteVerified":true}),
    )
}

pub(super) fn check_low_label(path: &Path) -> Result<()> {
    let mut sd = null_mut();
    let mut acl = null_mut();
    let code = unsafe {
        GetNamedSecurityInfoW(
            path_wide(path)?.as_ptr(),
            SE_FILE_OBJECT,
            LABEL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null_mut(),
            &mut acl,
            &mut sd,
        )
    };
    if code != 0 {
        return Err(Failure {
            stage: "c3-read-label",
            code: code.into(),
        });
    }
    let _sd = Local(sd);
    if acl.is_null() {
        return Err(invalid("c3-missing-label"));
    }
    let mut ptr = null_mut();
    check(unsafe { GetAce(acl, 0, &mut ptr) }, "c3-label-ace")?;
    let ace = unsafe { &*ptr.cast::<SYSTEM_MANDATORY_LABEL_ACE>() };
    let mut low = null_mut();
    check(
        unsafe { ConvertStringSidToSidW(wide("S-1-16-4096").as_ptr(), &mut low) },
        "c3-low-sid",
    )?;
    let _low = Local(low);
    if u32::from(ace.Header.AceType) != SYSTEM_MANDATORY_LABEL_ACE_TYPE
        || ace.Mask != SYSTEM_MANDATORY_LABEL_NO_WRITE_UP
        || unsafe { EqualSid((&raw const ace.SidStart).cast_mut().cast(), low) } == 0
    {
        return Err(invalid("c3-wrong-label"));
    }
    Ok(())
}
