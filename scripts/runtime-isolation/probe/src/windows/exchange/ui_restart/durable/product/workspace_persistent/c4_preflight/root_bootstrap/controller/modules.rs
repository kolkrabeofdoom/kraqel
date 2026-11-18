//! Bounded sampling with fail-closed admission; snapshots do not attest every load or grant data.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use windows_sys::Win32::Foundation::{ERROR_BAD_LENGTH, ERROR_NO_MORE_FILES};

#[derive(Default)]
pub(super) struct Observation {
    attempts: u32,
    successes: u32,
    paths: BTreeSet<String>,
    errors: BTreeMap<String, u32>,
    blocked: BTreeSet<String>,
}
impl Observation {
    #[cfg(test)]
    pub(super) fn sample(&mut self, process: HANDLE) {
        // 30-second controller deadline remains authoritative; diagnostics have a separate cap.
        if self.attempts >= 256 {
            return;
        }
        self.attempts += 1;
        match snapshot(process) {
            Ok(paths) if self.paths.union(&paths).count() <= 512 => {
                self.paths.extend(paths);
                self.successes += 1;
            }
            result => {
                let error = result
                    .err()
                    .unwrap_or_else(|| invalid("c4-module-union-bound"));
                *self
                    .errors
                    .entry(format!("{}:{}", error.stage, error.code))
                    .or_default() += 1;
            }
        }
    }
    pub(super) fn value(&self) -> Value {
        json!({"attempts":self.attempts,"successfulSnapshots":self.successes,"paths":self.paths,"errors":self.errors,"blockedPaths":self.blocked,"completeLoadHistory":false,"packageBindingVerified":false})
    }
    pub(super) fn enforce(
        &mut self,
        process: HANDLE,
        policy: &super::super::super::module_policy::Policy,
    ) -> Result<()> {
        self.attempts += 1;
        let attempt = (|| {
            if self.attempts > 256 {
                return Err(invalid("c4-module-attempt-bound"));
            }
            let paths = snapshot(process)?;
            if self.paths.union(&paths).count() > 512 {
                return Err(invalid("c4-module-union-bound"));
            }
            for path in &paths {
                if !policy.permits(Path::new(path))? {
                    self.blocked.insert(path.clone());
                }
            }
            self.paths.extend(paths);
            self.successes += 1;
            if !self.blocked.is_empty() {
                return Err(invalid("c4-module-outside-policy"));
            }
            Ok(())
        })();
        if let Err(error) = &attempt {
            *self
                .errors
                .entry(format!("{}:{}", error.stage, error.code))
                .or_default() += 1;
        }
        attempt
    }
}
fn snapshot(process: HANDLE) -> Result<BTreeSet<String>> {
    let id = identity(process)?;
    if unsafe { WaitForSingleObject(process, 0) } != WAIT_TIMEOUT {
        return Err(invalid("c4-module-process-ended"));
    }
    // Loader churn may report BAD_LENGTH. Bound retries; do not silently treat failure as empty.
    let mut raw = INVALID_HANDLE_VALUE;
    for _ in 0..3 {
        raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, id.pid) };
        if raw != INVALID_HANDLE_VALUE {
            break;
        }
        if unsafe { GetLastError() } != ERROR_BAD_LENGTH {
            return Err(fail("c4-module-snapshot"));
        }
    }
    if raw == INVALID_HANDLE_VALUE {
        return Err(fail("c4-module-snapshot"));
    }
    let held = Handle(raw);
    let mut entry: MODULEENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of_val(&entry) as u32;
    if unsafe { Module32FirstW(held.0, &mut entry) } == 0 {
        return Err(fail("c4-module-first"));
    }
    let mut paths = BTreeSet::new();
    for _ in 0..512 {
        let end = entry
            .szExePath
            .iter()
            .position(|c| *c == 0)
            .ok_or(invalid("c4-module-path-truncated"))?;
        if end == 0 {
            return Err(invalid("c4-module-path-empty"));
        }
        paths.insert(
            String::from_utf16(&entry.szExePath[..end])
                .map_err(|_| invalid("c4-module-path-utf16"))?,
        );
        if unsafe { Module32NextW(held.0, &mut entry) } == 0 {
            if unsafe { GetLastError() } != ERROR_NO_MORE_FILES {
                return Err(fail("c4-module-next"));
            }
            // The held generation must still be alive after the PID-addressed snapshot.
            if identity(process)? != id
                || unsafe { WaitForSingleObject(process, 0) } != WAIT_TIMEOUT
            {
                return Err(invalid("c4-module-generation-ended"));
            }
            return Ok(paths);
        }
    }
    Err(invalid("c4-module-count-bound"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c4_module_diagnostics_snapshot_current_generation_and_keep_failures_explicit() {
        let paths = snapshot(unsafe { GetCurrentProcess() }).unwrap();
        assert!(
            paths
                .iter()
                .any(|p| p.to_ascii_lowercase().ends_with(".exe"))
        );
        assert!(snapshot(null_mut()).is_err());
        let mut observation = Observation::default();
        observation.sample(null_mut());
        assert_eq!(observation.value()["successfulSnapshots"], 0);
        assert!(!observation.errors.is_empty());
        assert_eq!(observation.value()["packageBindingVerified"], false);
    }
}
