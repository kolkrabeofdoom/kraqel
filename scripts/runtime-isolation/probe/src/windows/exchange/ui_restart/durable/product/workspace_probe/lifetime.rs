//! Durable native identity for the single-process synthetic worker. No PID-only cleanup.
use super::*;
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct WorkerIdentity {
    pub(super) pid: u32,
    pub(super) creation_time: u64,
}
impl WorkerIdentity {
    pub(super) fn valid(self) -> bool {
        self.pid > 4 && self.creation_time != 0
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(in super::super) enum EndEvidence {
    Signaled,
    PidAbsent,
    PidReused { observed_creation_time: u64 },
}
impl EndEvidence {
    pub(super) fn valid_for(&self, worker: WorkerIdentity) -> bool {
        worker.valid()
            && match self {
                Self::PidReused {
                    observed_creation_time,
                } => {
                    *observed_creation_time != 0 && *observed_creation_time != worker.creation_time
                }
                _ => true,
            }
    }
}
pub(in super::super) fn creation_time(process: HANDLE) -> Result<u64> {
    let (mut created, mut exited, mut kernel, mut user): (FILETIME, FILETIME, FILETIME, FILETIME) =
        unsafe { zeroed() };
    check(
        unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) },
        "probe-worker-times",
    )?;
    let value = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
    if value == 0 {
        return Err(invalid("probe-worker-zero-time"));
    }
    Ok(value)
}
pub(super) fn capture_identity(process: HANDLE) -> Result<WorkerIdentity> {
    let identity = WorkerIdentity {
        pid: unsafe { GetProcessId(process) },
        creation_time: creation_time(process)?,
    };
    if !identity.valid() {
        return Err(invalid("probe-worker-bound-identity"));
    }
    Ok(identity)
}
pub(in super::super) fn verify_single_process_job(job: HANDLE) -> Result<()> {
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    check(
        unsafe {
            QueryInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                null_mut(),
            )
        },
        "probe-worker-job-limits",
    )?;
    let flags = limits.BasicLimitInformation.LimitFlags;
    let required = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    if limits.BasicLimitInformation.ActiveProcessLimit != 1
        || flags & required != required
        || flags & (JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK) != 0
    {
        return Err(invalid("probe-worker-job-policy"));
    }
    Ok(())
}
impl Store {
    // Called only before ResumeThread. Replaces idle with a narrow exact-job/token/lease guard;
    // all other control operations keep the ordinary actual-job-empty requirement.
    pub(super) fn bind_suspended_worker(
        &mut self,
        launch: LaunchIdentity,
        process: HANDLE,
    ) -> Result<WorkerIdentity> {
        let pending = self
            .state
            .probe
            .as_ref()
            .ok_or(invalid("probe-no-resource"))?;
        if pending.launch != launch || pending.execution.version != 3 {
            return Err(invalid("probe-worker-launch"));
        }
        let expected = profile(&pending.execution)?;
        let name = self.job_name();
        let job = Handle(unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, name.as_ptr()) });
        if job.0.is_null() {
            return Err(fail("probe-worker-bound-job"));
        }
        let mut assigned = 0;
        check(
            unsafe { IsProcessInJob(process, job.0, &mut assigned) },
            "probe-worker-bound-membership",
        )?;
        if assigned == 0 {
            return Err(invalid("probe-worker-bound-membership"));
        }
        verify_single_process_job(job.0)?;
        let facts = token_facts(process, Some(expected.sid))?;
        if facts["isAppContainer"] != true
            || facts["capabilityCount"] != 0
            || facts["expectedIdentity"] != true
        {
            return Err(invalid("probe-worker-bound-token"));
        }
        starter::verify_start_lease(&self.root, &name)?;
        let identity = capture_identity(process)?;
        if !identity.valid() || identity.pid == unsafe { GetCurrentProcessId() } {
            return Err(invalid("probe-worker-bound-identity"));
        }
        self.append_with_checked_context(
            self.state.actor,
            self.state.head,
            Action::WorkspaceProbeWorkerBound {
                launch,
                worker: identity,
                process_limit: 1,
            },
        )?;
        Ok(identity)
    }
}
fn pid_absent(pid: u32) -> Result<bool> {
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw == INVALID_HANDLE_VALUE {
        return Err(fail("probe-worker-snapshot"));
    }
    let snapshot = Handle(raw);
    let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut ok = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    while ok != 0 {
        if entry.th32ProcessID == pid {
            return Ok(false);
        }
        ok = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
        return Err(fail("probe-worker-snapshot-read"));
    }
    Ok(true)
}
pub(super) fn prove_end(worker: WorkerIdentity) -> Result<EndEvidence> {
    if !worker.valid() {
        return Err(invalid("probe-worker-invalid-identity"));
    }
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            worker.pid,
        )
    };
    if raw.is_null() {
        let error = unsafe { GetLastError() };
        if error == ERROR_INVALID_PARAMETER && pid_absent(worker.pid)? {
            return Ok(EndEvidence::PidAbsent);
        }
        return Err(Failure {
            stage: "probe-worker-end-open",
            code: error.into(),
        });
    }
    let process = Handle(raw);
    let observed = creation_time(process.0)?;
    if observed != worker.creation_time {
        return Ok(EndEvidence::PidReused {
            observed_creation_time: observed,
        });
    }
    match unsafe { WaitForSingleObject(process.0, 0) } {
        WAIT_OBJECT_0 => Ok(EndEvidence::Signaled),
        WAIT_TIMEOUT => Err(invalid("probe-worker-still-alive")),
        _ => Err(fail("probe-worker-end-wait")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn suspended_test_process() -> Worker {
        let exe = std::env::current_exe().unwrap();
        let mut command = wide(format!("\"{}\"", exe.display()));
        let mut startup: STARTUPINFOW = unsafe { zeroed() };
        startup.cb = size_of::<STARTUPINFOW>() as u32;
        let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                CreateProcessW(
                    wide(&exe).as_ptr(),
                    command.as_mut_ptr(),
                    null(),
                    null(),
                    0,
                    CREATE_SUSPENDED | DETACHED_PROCESS,
                    null(),
                    null(),
                    &startup,
                    &mut info,
                )
            },
            "test-worker-create-suspended",
        )
        .unwrap();
        Worker(Handle(info.hProcess), Handle(info.hThread))
    }
    #[test]
    fn native_identity_does_not_mistake_a_live_or_reused_pid_for_the_recorded_generation() {
        let current = WorkerIdentity {
            pid: unsafe { GetCurrentProcessId() },
            creation_time: creation_time(unsafe { GetCurrentProcess() }).unwrap(),
        };
        assert!(prove_end(current).is_err());
        let old = WorkerIdentity {
            creation_time: current.creation_time - 1,
            ..current
        };
        assert!(
            matches!(prove_end(old).unwrap(),EndEvidence::PidReused{observed_creation_time} if observed_creation_time==current.creation_time)
        );
        assert!(
            !EndEvidence::PidReused {
                observed_creation_time: current.creation_time
            }
            .valid_for(current)
        );
        assert!(
            prove_end(WorkerIdentity {
                pid: 0,
                creation_time: 1
            })
            .is_err()
        );
        let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
        assert!(!job.0.is_null());
        assert!(verify_single_process_job(job.0).is_err());
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        limits.BasicLimitInformation.ActiveProcessLimit = 1;
        check(
            unsafe {
                SetInformationJobObject(
                    job.0,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            },
            "test-worker-job-policy",
        )
        .unwrap();
        verify_single_process_job(job.0).unwrap();
        let first = suspended_test_process();
        check(
            unsafe { AssignProcessToJobObject(job.0, first.0.0) },
            "test-worker-assign-first",
        )
        .unwrap();
        let second = suspended_test_process();
        assert_eq!(unsafe { AssignProcessToJobObject(job.0, second.0.0) }, 0);
        assert_eq!(unsafe { GetLastError() }, ERROR_NOT_ENOUGH_QUOTA);
        let mut assigned = 1;
        check(
            unsafe { IsProcessInJob(second.0.0, job.0, &mut assigned) },
            "test-worker-second-membership",
        )
        .unwrap();
        assert_eq!(assigned, 0);
        // Both are our suspended fixture processes; neither executes Rust/test code.
        drop(second);
        drop(first);
    }
}
