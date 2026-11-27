//! Handle-pinned, bounded local fixture inventory. Does not execute any package content.
use super::*;
use windows_sys::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};

fn native_info(file: &fs::File) -> Result<BY_HANDLE_FILE_INFORMATION> {
    let mut info = unsafe { zeroed() };
    check(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
        "c2-file-info",
    )?;
    Ok(info) // Callers explicitly distinguish files/directories before use.
}

fn acl_for(sid: PSID, directory: bool, rights: u32) -> Result<Local> {
    let descriptor = private_descriptor()?;
    let mut base = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    check(
        unsafe { GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut base, &mut defaulted) },
        "c2-base-acl",
    )?;
    // Windows removes directory inheritance flags when installing this DACL on a file.
    if !directory {
        for i in 0..2 {
            let mut ace = null_mut();
            check(unsafe { GetAce(base, i, &mut ace) }, "c2-base-ace")?;
            unsafe {
                (*ace.cast::<ACCESS_ALLOWED_ACE>()).Header.AceFlags = 0;
            }
        }
    }
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: rights,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: if directory {
            OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
        } else {
            0
        },
        Trustee: TRUSTEE_W {
            pMultipleTrustee: null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: sid.cast(),
        },
    };
    let mut acl = null_mut();
    let status = unsafe { SetEntriesInAclW(1, &access, base, &mut acl) };
    if status != 0 {
        return Err(Failure {
            stage: "c2-build-acl",
            code: status.into(),
        });
    }
    Ok(Local(acl.cast()))
}
fn acl_handle(path: &Path, directory: bool, write: bool) -> Result<fs::File> {
    let file = io(
        "c2-acl-handle",
        OpenOptions::new()
            .access_mode(0x0002_0000 | FILE_READ_ATTRIBUTES | if write { WRITE_DAC } else { 0 })
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(
                FILE_FLAG_OPEN_REPARSE_POINT
                    | if directory {
                        FILE_FLAG_BACKUP_SEMANTICS
                    } else {
                        0
                    },
            )
            .open(local_path(path)?),
    )?;
    let info = native_info(&file)?;
    if info.dwFileAttributes & BAD_ATTRIBUTES != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
        || (!directory && info.nNumberOfLinks != 1)
    {
        return Err(invalid("c2-acl-file-kind"));
    }
    Ok(file)
}
pub(super) fn set_profile_acl(path: &Path, directory: bool, sid: PSID, rights: u32) -> Result<()> {
    let _parents = pin_directories(path.parent().ok_or(invalid("c2-acl-parent"))?)?;
    let f = acl_handle(path, directory, true)?;
    let acl = acl_for(sid, directory, rights)?;
    let code = unsafe {
        SetSecurityInfo(
            f.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            acl.0.cast(),
            null_mut(),
        )
    };
    if code != 0 {
        return Err(Failure {
            stage: "c2-set-acl",
            code: code.into(),
        });
    }
    drop(f);
    check_profile_acl(path, directory, sid, rights)
}
fn entries(acl: *mut ACL) -> Result<Vec<Vec<u8>>> {
    let mut info: ACL_SIZE_INFORMATION = unsafe { zeroed() };
    check(
        unsafe {
            GetAclInformation(
                acl,
                (&mut info as *mut ACL_SIZE_INFORMATION).cast(),
                size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            )
        },
        "c2-acl-info",
    )?;
    if info.AceCount != 3 {
        return Err(invalid("c2-acl-count"));
    }
    let mut out = Vec::new();
    for i in 0..3 {
        let mut ptr = null_mut();
        check(unsafe { GetAce(acl, i, &mut ptr) }, "c2-acl-ace")?;
        let ace = unsafe { &*ptr.cast::<ACCESS_ALLOWED_ACE>() };
        if ace.Header.AceType != 0 || ace.Header.AceSize < 20 {
            return Err(invalid("c2-acl-type"));
        }
        let sid = (&raw const ace.SidStart).cast_mut().cast();
        if unsafe { IsValidSid(sid) } == 0 {
            return Err(invalid("c2-acl-sid"));
        }
        let length = unsafe { GetLengthSid(sid) } as usize;
        if length + 8 > usize::from(ace.Header.AceSize) {
            return Err(invalid("c2-acl-size"));
        }
        let mut value = vec![ace.Header.AceFlags];
        value.extend(ace.Mask.to_le_bytes());
        value.extend(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) });
        out.push(value);
    }
    out.sort();
    Ok(out)
}
pub(super) fn check_profile_acl(
    path: &Path,
    directory: bool,
    sid: PSID,
    rights: u32,
) -> Result<()> {
    let expected = acl_for(sid, directory, rights)?;
    let f = acl_handle(path, directory, false)?;
    let mut sd = null_mut();
    let mut acl = null_mut();
    let code = unsafe {
        GetSecurityInfo(
            f.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut acl,
            null_mut(),
            &mut sd,
        )
    };
    if code != 0 {
        return Err(Failure {
            stage: "c2-read-acl",
            code: code.into(),
        });
    }
    let _sd = Local(sd);
    let mut control = 0;
    let mut revision = 0;
    check(
        unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) },
        "c2-acl-control",
    )?;
    let actual_entries = if acl.is_null() {
        Vec::new()
    } else {
        entries(acl)?
    };
    let expected_entries = entries(expected.0.cast())?;
    if acl.is_null() || control & SE_DACL_PROTECTED == 0 || actual_entries != expected_entries {
        return Err(invalid("c2-wrong-acl"));
    }
    Ok(())
}
pub(super) fn root_identity(path: &Path) -> Result<RootIdentityV1> {
    let _pins = pin_directories(path)?;
    let file = acl_handle(path, true, false)?;
    let mut info: FILE_ID_INFO = unsafe { zeroed() };
    check(
        unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileIdInfo,
                (&mut info as *mut FILE_ID_INFO).cast(),
                size_of::<FILE_ID_INFO>() as u32,
            )
        },
        "c2-root-file-id",
    )?;
    Ok(RootIdentityV1 {
        canonical_path_sha256: root_hash(&io("c2-root-canonical", path.canonicalize())?),
        volume_id: info.VolumeSerialNumber,
        file_id: info.FileId.Identifier,
    })
}
pub(super) fn ordinal_equal(a: &str, b: &str) -> Result<bool> {
    let a: Vec<_> = a.encode_utf16().collect();
    let b: Vec<_> = b.encode_utf16().collect();
    if a.len() > 1024 || b.len() > 1024 {
        return Err(invalid("c2-name-bound"));
    }
    let result =
        unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1) };
    if result == 0 {
        return Err(fail("c2-name-ordinal"));
    }
    Ok(result == CSTR_EQUAL)
}
fn streams(path: &Path, directory: bool) -> Result<()> {
    let mut data: WIN32_FIND_STREAM_DATA = unsafe { zeroed() };
    let raw = unsafe {
        FindFirstStreamW(
            path_wide(path)?.as_ptr(),
            FindStreamInfoStandard,
            (&mut data as *mut WIN32_FIND_STREAM_DATA).cast(),
            0,
        )
    };
    if raw == INVALID_HANDLE_VALUE {
        if directory && unsafe { GetLastError() } == ERROR_HANDLE_EOF {
            return Ok(());
        }
        return Err(fail("c2-stream-enumeration"));
    }
    struct Search(HANDLE);
    impl Drop for Search {
        fn drop(&mut self) {
            unsafe {
                FindClose(self.0);
            }
        }
    }
    let search = Search(raw);
    let length = data
        .cStreamName
        .iter()
        .position(|c| *c == 0)
        .ok_or(invalid("c2-stream-name"))?;
    if directory
        || String::from_utf16(&data.cStreamName[..length]).map_err(|_| invalid("c2-stream-name"))?
            != "::$DATA"
    {
        return Err(invalid("c2-extra-stream"));
    }
    if unsafe { FindNextStreamW(search.0, (&mut data as *mut WIN32_FIND_STREAM_DATA).cast()) } != 0
    {
        return Err(invalid("c2-extra-stream"));
    }
    if unsafe { GetLastError() } != ERROR_HANDLE_EOF {
        return Err(fail("c2-stream-enumeration"));
    }
    Ok(())
}
pub(super) fn verify_package(
    root: &Path,
    expected: &[InventoryEntry],
    profile: Option<PSID>,
) -> Result<Vec<fs::File>> {
    verify_package_bounded(root, expected, profile, false)
}

// Separate C4 preflight budget. C2 retains its original smaller limits.
pub(super) fn verify_c4_package(root: &Path, expected: &[InventoryEntry]) -> Result<Vec<fs::File>> {
    verify_package_bounded(root, expected, None, true)
}

fn verify_package_bounded(
    root: &Path,
    expected: &[InventoryEntry],
    profile: Option<PSID>,
    c4: bool,
) -> Result<Vec<fs::File>> {
    let (files, tree, depth_limit, file_limit, total_limit) = if c4 {
        (1024, 2048, 32, 512 * 1024 * 1024, 2 * 1024 * 1024 * 1024u64)
    } else {
        (64, 128, 8, 128 * 1024 * 1024, 256 * 1024 * 1024u64)
    };
    if expected.is_empty() || expected.len() > files {
        return Err(invalid("c2-native-inventory-count"));
    }
    let mut pins = pin_directories(root)?;
    let mut dirs = vec![(root.to_path_buf(), String::new(), 0)];
    let mut actual = Vec::new();
    let mut names = Vec::<String>::new();
    let mut total = 0u64;
    while let Some((directory, prefix, depth)) = dirs.pop() {
        pins.extend(pin_directories(&directory)?);
        if let Some(sid) = profile {
            check_profile_acl(&directory, true, sid, RX)?;
        } else {
            private_acl(&directory, true)?;
        }
        streams(&directory, true)?;
        let children = io("c2-list-package", fs::read_dir(&directory))?;
        for entry in children {
            if names.len() >= tree {
                return Err(invalid("c2-native-tree-count"));
            }
            let entry = io("c2-package-entry", entry)?;
            let file_name = entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("c2-package-name"))?;
            let relative = if prefix.is_empty() {
                file_name
            } else {
                format!("{prefix}/{file_name}")
            };
            for previous in &names {
                if ordinal_equal(previous, &relative)? {
                    return Err(invalid("c2-native-name-alias"));
                }
            }
            names.push(relative.clone());
            let path = entry.path();
            // Open as reparse point before recursion/read; no metadata-following traversal.
            let f = io(
                "c2-package-pin",
                OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
                    .open(&path),
            )?;
            let info = native_info(&f)?;
            if info.dwFileAttributes & BAD_ATTRIBUTES != 0 {
                return Err(invalid("c2-unsafe-package-entry"));
            }
            if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                if depth >= depth_limit
                    || !expected
                        .iter()
                        .any(|e| e.path.starts_with(&format!("{relative}/")))
                {
                    return Err(invalid("c2-extra-directory"));
                }
                pins.push(f);
                dirs.push((path, relative, depth + 1));
                continue;
            }
            if info.nNumberOfLinks != 1 {
                return Err(invalid("c2-package-hardlink"));
            }
            let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
            total = total.checked_add(size).ok_or(invalid("c2-package-total"))?;
            if size > file_limit || total > total_limit {
                return Err(invalid("c2-native-package-budget"));
            }
            if let Some(sid) = profile {
                check_profile_acl(&path, false, sid, RX)?;
            } else {
                private_acl(&path, false)?;
            }
            streams(&path, false)?;
            let declared = expected
                .iter()
                .find(|e| e.path == relative)
                .ok_or(invalid("c2-undeclared-file"))?;
            let mut file = f;
            let mut digest = Sha256::new();
            let mut block = [0; 16384];
            let mut read = 0u64;
            loop {
                let n = io("c2-package-read", file.read(&mut block))?;
                if n == 0 {
                    break;
                }
                read += n as u64;
                if read > size {
                    return Err(invalid("c2-package-size-changed"));
                }
                digest.update(&block[..n]);
            }
            let sha256 = digest.finalize().into();
            if read != size || declared.size != size || declared.sha256 != sha256 {
                return Err(invalid("c2-package-content"));
            }
            actual.push(InventoryEntry {
                path: relative,
                size,
                sha256,
                role: declared.role,
            });
            pins.push(file);
        }
    }
    if actual.len() != expected.len()
        || contract::encode_inventory(PackageRole::Ui, &actual)
            .map_err(|_| invalid("c2-native-path-shape"))?
            != contract::encode_inventory(PackageRole::Ui, expected)
                .map_err(|_| invalid("c2-native-path-shape"))?
    {
        return Err(invalid("c2-inventory-mismatch"));
    }
    Ok(pins)
}
