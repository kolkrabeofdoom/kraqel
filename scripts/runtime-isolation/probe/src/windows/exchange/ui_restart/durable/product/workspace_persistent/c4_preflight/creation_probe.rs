//! Bounded native creation experiment only: two suspended sibling processes, never resumed.
//! This is not WebView child interception, a C4 UI bootstrap or a durable launch operation.
use super::*;

struct Suspended(Handle, Handle);
impl Drop for Suspended {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0.0, 3);
            WaitForSingleObject(self.0.0, 5000);
        }
    }
}
fn count(job: HANDLE) -> Result<u32> {
    let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
    check(
        unsafe {
            QueryInformationJobObject(
                job,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of_val(&info) as u32,
                null_mut(),
            )
        },
        "c4-probe-job-count",
    )?;
    Ok(info.ActiveProcesses)
}

impl CandidatePins {
    #[allow(dead_code)] // No production/UI route; invoked only by the explicit synthetic native test.
    pub(in super::super) fn probe_suspended_pair(&self, store: &Store) -> Result<()> {
        let actor = hash(&std::env::current_exe().map_err(|_| invalid("c4-probe-self"))?)?;
        store.c2_guard(actor, store.state.head)?;
        environment_clear(std::env::vars_os())?;
        registry_clear()?;
        let _workspace = store.verify_c2(actor, store.state.head)?;
        let state = store
            .state
            .c2
            .as_ref()
            .ok_or(invalid("c4-probe-workspace"))?;
        let expected = data_root(state.id).join("c4-candidate/ui");
        if root_identity(&expected)? != self._ui_root || hash(&self.exe)? != self.exe_sha256 {
            return Err(invalid("c4-probe-package"));
        }
        let p = profile(state.id)?;
        let raw = unsafe { CreateJobObjectW(null(), store.job_name().as_ptr()) };
        if raw.is_null() {
            return Err(fail("c4-probe-job"));
        }
        let job = Handle(raw);
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return Err(invalid("c4-probe-job-collision"));
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
                    size_of_val(&limits) as u32,
                )
            },
            "c4-probe-job-policy",
        )?;
        let caps = SECURITY_CAPABILITIES {
            AppContainerSid: p.sid,
            Capabilities: null_mut(),
            CapabilityCount: 0,
            Reserved: 0,
        };
        let attrs = Attributes::with_handles_and_jobs(&caps, &[], &[job.0])?;
        let mut members = Vec::new();
        for _ in 0..2 {
            let mut si: STARTUPINFOEXW = unsafe { zeroed() };
            si.StartupInfo.cb = size_of_val(&si) as u32;
            si.lpAttributeList = attrs.ptr;
            let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
            let mut args = wide(format!("\"{}\" c4-never-resumed-probe", self.exe.display()));
            let env = crate::windows::exchange::ui_restart::environment(&expected);
            check(
                unsafe {
                    CreateProcessW(
                        wide(&self.exe).as_ptr(),
                        args.as_mut_ptr(),
                        null(),
                        null(),
                        0,
                        EXTENDED_STARTUPINFO_PRESENT
                            | CREATE_SUSPENDED
                            | CREATE_UNICODE_ENVIRONMENT
                            | DETACHED_PROCESS,
                        env.as_ptr().cast(),
                        wide(&expected).as_ptr(),
                        &si.StartupInfo,
                        &mut pi,
                    )
                },
                "c4-probe-create-suspended",
            )?;
            let process = Suspended(Handle(pi.hProcess), Handle(pi.hThread));
            let mut image = [0u16; 32768];
            let mut length = image.len() as u32;
            check(
                unsafe {
                    QueryFullProcessImageNameW(process.0.0, 0, image.as_mut_ptr(), &mut length)
                },
                "c4-probe-process-image",
            )?;
            let actual = PathBuf::from(
                String::from_utf16(&image[..length as usize])
                    .map_err(|_| invalid("c4-probe-process-image"))?,
            );
            if local_path(&actual)? != local_path(&self.exe)? || hash(&actual)? != self.exe_sha256 {
                return Err(invalid("c4-probe-image-mismatch"));
            }
            let mut yes = 0;
            check(
                unsafe { IsProcessInJob(process.0.0, job.0, &mut yes) },
                "c4-probe-job-membership",
            )?;
            let facts = token_facts(process.0.0, Some(p.sid))?;
            if yes == 0
                || facts["isAppContainer"] != true
                || facts["capabilityCount"] != 0
                || facts["expectedIdentity"] != true
            {
                return Err(invalid("c4-probe-token-or-job"));
            }
            let mut creation = unsafe { zeroed() };
            let mut exit = unsafe { zeroed() };
            let mut kernel = unsafe { zeroed() };
            let mut user = unsafe { zeroed() };
            check(
                unsafe {
                    GetProcessTimes(
                        process.0.0,
                        &mut creation,
                        &mut exit,
                        &mut kernel,
                        &mut user,
                    )
                },
                "c4-probe-creation-time",
            )?;
            if pi.dwProcessId == 0 || (creation.dwHighDateTime == 0 && creation.dwLowDateTime == 0)
            {
                return Err(invalid("c4-probe-process-generation"));
            }
            // A thread handle is held only for cleanup; no ResumeThread API is called.
            let _thread = &process.1;
            members.push(process);
        }
        if count(job.0)? != 2 {
            return Err(invalid("c4-probe-two-members"));
        }
        if store.idle().is_ok() {
            return Err(invalid("c4-probe-idle-guard"));
        }
        if Store::open(&store.root).is_ok() {
            return Err(invalid("c4-probe-control-lease"));
        }
        drop(members);
        let deadline = Instant::now() + Duration::from_secs(5);
        while count(job.0)? != 0 {
            if Instant::now() >= deadline {
                return Err(invalid("c4-probe-drain"));
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}
