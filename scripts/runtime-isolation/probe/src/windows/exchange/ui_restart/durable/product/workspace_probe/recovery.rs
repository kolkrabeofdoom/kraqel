//! Reconcile only resource states proved by authenticated records; no inferred crash ownership.
use super::*;
use windows_sys::Win32::System::Registry::*;

struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

fn present(execution: &Execution) -> Result<bool> {
    let profile = profile(execution)?;
    profile_present(&profile)
}
pub(in super::super) fn profile_present(profile: &Profile) -> Result<bool> {
    let mut text = null_mut();
    check(
        unsafe { ConvertSidToStringSidW(profile.sid, &mut text) },
        "probe-profile-sid-text",
    )?;
    let _text = Local(text.cast());
    let mut len = 0;
    while len < 256 && unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    if len == 256 {
        return Err(invalid("probe-profile-sid-bound"));
    }
    let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) })
        .map_err(|_| invalid("probe-profile-sid-text"))?;
    let path = wide(format!(
        "Software\\Classes\\Local Settings\\Software\\Microsoft\\Windows\\CurrentVersion\\AppContainer\\Mappings\\{sid}"
    ));
    let mut raw = null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut raw,
        )
    };
    if status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND {
        return Ok(false);
    }
    if status != 0 {
        return Err(Failure {
            stage: "probe-profile-query",
            code: status.into(),
        });
    }
    let key = Key(raw);
    let mut bytes = [0u16; 256];
    let mut size = (bytes.len() * 2) as u32;
    let status = unsafe {
        RegGetValueW(
            key.0,
            null(),
            wide("Moniker").as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            bytes.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != 0 {
        return Err(Failure {
            stage: "probe-profile-moniker",
            code: status.into(),
        });
    }
    if !size.is_multiple_of(2)
        || size as usize != profile.name.len() * 2
        || bytes[..size as usize / 2] != profile.name
    {
        return Err(invalid("probe-profile-identity-mismatch"));
    }
    Ok(true)
}

impl Store {
    pub(super) fn reconcile_probe_resources(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
    ) -> Result<Value> {
        self.append_guard(actor, head)?;
        self.verify_workspace_files()?;
        let pending = self
            .state
            .probe
            .clone()
            .ok_or(invalid("probe-no-resource"))?;
        if pending.execution.version != 3 {
            return Err(invalid("probe-resource-lifetime-unproved"));
        }
        let exists = present(&pending.execution)?;
        if !pending.ready && exists {
            return Err(invalid("probe-profile-ownership-unconfirmed"));
        }
        let head = if pending.run_intent && !pending.drained {
            let worker = pending
                .worker
                .ok_or(invalid("probe-worker-binding-missing"))?;
            let evidence = lifetime::prove_end(worker)?;
            if MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 2 {
                return Err(invalid("probe-worker-end-budget"));
            }
            self.append(
                actor,
                head,
                Action::WorkspaceProbeRecoveredDrain {
                    launch: pending.launch,
                    worker,
                    evidence,
                },
            )?;
            self.state.head
        } else {
            head
        };
        let action = if pending.ready {
            Action::ReconcileWorkspaceProbe {
                launch: pending.launch,
            }
        } else {
            Action::AbortUncreatedWorkspaceProbe {
                launch: pending.launch,
            }
        };
        // Complete transition/budget preflight before a potentially destructive native call.
        let mut next = head;
        next[0] ^= 1;
        if next == [0; 32] {
            next[1] = 1;
        }
        self.state.clone().apply_record(actor, &action, next)?;
        if exists {
            remove_profile(&pending.execution)?;
        }
        if present(&pending.execution)? {
            return Err(invalid("probe-profile-still-present"));
        }
        self.append(actor, head, action)?;
        Ok(
            json!({"resourceReconciled":true,"profileRemoved":exists,"profileAlreadyAbsent":!exists,
            "launchConsumed":true,"workerStarted":false,"uiStarted":false,"importTaskCreated":false}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_profile_presence_distinguishes_missing_mapping_and_matching_identity() {
        let execution = Execution {
            version: 3,
            probe_id: Uuid::new_v4(),
            workspace_id: [1; 16],
            nonce: [2; 32],
            worker_sha256: [3; 32],
            root_sha256: [4; 32],
            policy_sha256: [5; 32],
            sentinel_sha256: [6; 32],
            live_hold_ms: 0,
        };
        assert!(!present(&execution).unwrap());
        let mut owned = Profile::create(&format!("boot-{}", execution.probe_id.simple())).unwrap();
        assert!(present(&execution).unwrap());
        assert!(owned.remove());
        assert!(!present(&execution).unwrap());
    }
}
