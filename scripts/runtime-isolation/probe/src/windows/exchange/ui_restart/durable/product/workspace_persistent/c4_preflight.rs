//! C4b read-only candidate preflight. NOT a UI grant or a runtime process launcher.
use super::*;
use crate::product_start_p2c::c4::{LoaderV1, Role, UiRuntimePackageV1};
use windows_sys::Win32::System::Registry::*;
mod creation_probe;

/// Only lifetime-bound pinned files; no persistable success token or native start capability.
#[allow(dead_code)] // C4 UI/record integration deliberately remains unavailable.
pub(super) struct CandidatePins {
    _files: Vec<fs::File>,
    _package: [u8; 32],
    _host_root: RootIdentityV1,
    _ui_root: RootIdentityV1,
    exe: PathBuf,
    exe_sha256: [u8; 32],
}

fn environment_clear(values: impl IntoIterator<Item = (OsString, OsString)>) -> Result<()> {
    for (name, value) in values {
        let key = name
            .to_str()
            .ok_or(invalid("c4-environment-name"))?
            .to_ascii_uppercase();
        // Reject even empty known overrides and future WEBVIEW2_* variables. Do not silently strip.
        if key.starts_with("WEBVIEW2_")
            || key == "COREWEBVIEW2_MAX_INSTANCES"
            || (key == "__COMPAT_LAYER" && !value.is_empty())
        {
            return Err(invalid("c4-runtime-environment-override"));
        }
    }
    Ok(())
}

fn registry_clear() -> Result<()> {
    // Conservatively reject the entire WebView2 policy key when nonempty, including policies
    // for other app IDs. Read both views and hives. Errors are not equivalent to absence.
    let path = wide("Software\\Policies\\Microsoft\\Edge\\WebView2");
    for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        for view in [KEY_WOW64_32KEY, KEY_WOW64_64KEY] {
            let mut key = null_mut();
            let result = unsafe {
                RegOpenKeyExW(
                    hive,
                    path.as_ptr(),
                    0,
                    KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS | view,
                    &mut key,
                )
            };
            if result == ERROR_FILE_NOT_FOUND || result == ERROR_PATH_NOT_FOUND {
                continue;
            }
            if result != 0 {
                return Err(Failure {
                    stage: "c4-runtime-registry-open",
                    code: result.into(),
                });
            }
            struct Key(HKEY);
            impl Drop for Key {
                fn drop(&mut self) {
                    unsafe {
                        RegCloseKey(self.0);
                    }
                }
            }
            let key = Key(key);
            let (mut subkeys, mut values) = (0, 0);
            let result = unsafe {
                RegQueryInfoKeyW(
                    key.0,
                    null_mut(),
                    null_mut(),
                    null(),
                    &mut subkeys,
                    null_mut(),
                    null_mut(),
                    &mut values,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                )
            };
            if result != 0 {
                return Err(Failure {
                    stage: "c4-runtime-registry-query",
                    code: result.into(),
                });
            }
            if subkeys != 0 || values != 0 {
                return Err(invalid("c4-runtime-registry-override"));
            }
        }
    }
    Ok(())
}

fn reference<'a>(
    entries: &'a [InventoryEntry],
    path: &str,
    role: FileRole,
) -> Result<&'a InventoryEntry> {
    entries
        .iter()
        .find(|e| e.path == path && e.role == role)
        .ok_or(invalid("c4-package-reference"))
}

impl Store {
    #[allow(dead_code)] // Preflight tested before adding any executable C4 route.
    pub(super) fn verify_c4_candidate(
        &self,
        actor: [u8; 32],
        head: [u8; 32],
        descriptor: &[u8],
        host_inventory: &[u8],
        ui_inventory: &[u8],
    ) -> Result<CandidatePins> {
        self.c2_guard(actor, head)?;
        scope(&self.root)?;
        let candidate =
            UiRuntimePackageV1::decode(descriptor).map_err(|_| invalid("c4-descriptor"))?;
        let (host_role, host) =
            contract::decode_inventory(host_inventory).map_err(|_| invalid("c4-host-inventory"))?;
        let (ui_role, ui) =
            contract::decode_inventory(ui_inventory).map_err(|_| invalid("c4-ui-inventory"))?;
        if host_role != PackageRole::Host
            || ui_role != PackageRole::Ui
            || contract::inventory_digest(host_role, &host)
                .map_err(|_| invalid("c4-host-digest"))?
                != candidate.host_inventory_sha256
            || contract::inventory_digest(ui_role, &ui).map_err(|_| invalid("c4-ui-digest"))?
                != candidate.ui_inventory_sha256
        {
            return Err(invalid("c4-inventory-binding"));
        }
        let state = self.state.c2.as_ref().ok_or(invalid("c4-no-c2-binding"))?;
        let binding = state.binding.as_ref().ok_or(invalid("c4-no-c2-binding"))?;
        if !state.profile_ready || state.retiring || state.retired {
            return Err(invalid("c4-candidate-lifecycle"));
        }
        if candidate.binding_revision != binding.revision {
            return Err(invalid("c4-candidate-revision"));
        }
        environment_clear(std::env::vars_os())?;
        registry_clear()?;
        let mut files = self.verify_c2(actor, head)?;
        // Never reuse/relabel the old C2 package. Only an explicit new private candidate subtree.
        let root = data_root(state.id).join("c4-candidate");
        files.extend(pin_directories(&root)?);
        private_acl(&root, true)?;
        let host_root = root.join("host");
        let ui_root = root.join("ui");
        files.extend(filesystem::verify_c4_package(&host_root, &host)?);
        files.extend(filesystem::verify_c4_package(&ui_root, &ui)?);
        let root_program = reference(&ui, &candidate.ui_exe, FileRole::Program)?;
        for role in &candidate.policy.roles {
            reference(&ui, &role.image, FileRole::Program)?;
        }
        if let LoaderV1::Dynamic { path, sha256 } = &candidate.loader
            && reference(&ui, path, FileRole::Library)?.sha256 != *sha256
        {
            return Err(invalid("c4-loader-binding"));
        }
        // This checked candidate remains no-execution. Native launch requires separate durable
        // UI records, token/job/peer enforcement and runtime image/module witnesses.
        let _root_role = candidate
            .policy
            .roles
            .iter()
            .find(|r| r.role == Role::Root)
            .ok_or(invalid("c4-root-role"))?;
        Ok(CandidatePins {
            _files: files,
            _package: candidate
                .digest()
                .map_err(|_| invalid("c4-package-digest"))?,
            _host_root: root_identity(&host_root)?,
            _ui_root: root_identity(&ui_root)?,
            exe: ui_root.join(&candidate.ui_exe),
            exe_sha256: root_program.sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c4_override_names_are_conservative_and_never_mutate_environment() {
        for name in [
            "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER",
            "webview2_user_data_folder",
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER",
            "WEBVIEW2_FUTURE_OVERRIDE",
            "COREWEBVIEW2_MAX_INSTANCES",
        ] {
            assert!(environment_clear([(name.into(), "".into())]).is_err());
        }
        assert!(environment_clear([("__COMPAT_LAYER".into(), "RUNASADMIN".into())]).is_err());
        environment_clear([("TEMP".into(), "synthetic".into())]).unwrap();
        let before = std::env::vars_os().collect::<Vec<_>>();
        let _ = registry_clear();
        assert_eq!(std::env::vars_os().collect::<Vec<_>>(), before);
    }
}
