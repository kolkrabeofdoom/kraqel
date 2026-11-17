//! Local-only, pinned Windows filesystem and account/SYSTEM ACL boundary.
use super::*;
// Pin every ancestor without delete sharing. Reject remote namespaces before touching paths.
pub(super) fn pin_directories(path: &Path) -> Result<Vec<fs::File>> {
    let normalized = local_path(path)?;
    let path = normalized.as_path();
    use std::path::{Component, Prefix};
    let mut current = PathBuf::new();
    let mut handles = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(p) => match p.kind() {
                Prefix::Disk(_) | Prefix::VerbatimDisk(_) => {
                    current.push(p.as_os_str());
                    continue;
                }
                _ => return Err(invalid("product-remote-or-namespace-path")),
            },
            Component::RootDir => {
                current.push(component.as_os_str());
                if unsafe { GetDriveTypeW(wide(&current).as_ptr()) } != FIXED_DRIVE {
                    return Err(invalid("product-not-fixed-drive"));
                }
            }
            Component::Normal(p) => current.push(p),
            _ => return Err(invalid("product-noncanonical-component")),
        }
        if current.has_root() {
            let file = io(
                "product-pin-directory",
                OpenOptions::new()
                    .access_mode(FILE_READ_ATTRIBUTES)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                    .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(&current),
            )?;
            let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
            check(
                unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
                "product-directory-info",
            )?;
            if info.dwFileAttributes & BAD_ATTRIBUTES != 0
                || info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            {
                return Err(invalid("product-unsafe-directory"));
            }
            handles.push(file);
        }
    }
    if !path.is_absolute() || handles.is_empty() {
        return Err(invalid("product-relative-directory"));
    }
    Ok(handles)
}
// Use one explicit local namespace before I/O. In a packaged host, plain AppData paths can be
// redirected to a different volume; also normalize separators for verbatim Win32 API paths.
pub(super) fn local_path(path: &Path) -> Result<PathBuf> {
    use std::path::{Component, Prefix};
    let mut components = path.components();
    let drive = match components.next() {
        Some(Component::Prefix(p)) => match p.kind() {
            Prefix::Disk(d) | Prefix::VerbatimDisk(d) => d,
            _ => return Err(invalid("product-remote-or-namespace-path")),
        },
        _ => return Err(invalid("product-relative-directory")),
    };
    if !matches!(components.next(), Some(Component::RootDir)) {
        return Err(invalid("product-relative-directory"));
    }
    let mut out = PathBuf::from(format!("\\\\?\\{}:\\", char::from(drive)));
    for part in components {
        match part {
            Component::Normal(name) => out.push(name),
            _ => return Err(invalid("product-noncanonical-component")),
        }
    }
    Ok(out)
}
pub(super) fn path_wide(path: &Path) -> Result<Vec<u16>> {
    Ok(wide(local_path(path)?))
}
pub(super) fn secure_directory(path: &Path) -> Result<()> {
    let descriptor = private_descriptor()?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    check(
        unsafe { CreateDirectoryW(path_wide(path)?.as_ptr(), &attributes) },
        "product-create-private-directory",
    )?;
    // Verify ACL support before any secret is created, including on unusual local filesystems.
    private_acl(path, true)
}
// Exactly the two full-control SID grants from the native account/SYSTEM descriptor.
// Inherited flags on files differ from directory flags; no additional ACE is permitted.
pub(super) fn private_acl(path: &Path, directory: bool) -> Result<()> {
    let expected = private_descriptor()?;
    let mut expected_acl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    check(
        unsafe {
            GetSecurityDescriptorDacl(expected.0, &mut present, &mut expected_acl, &mut defaulted)
        },
        "product-expected-acl",
    )?;
    let mut sd = null_mut();
    let mut actual = null_mut();
    let code = unsafe {
        GetNamedSecurityInfoW(
            path_wide(path)?.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut actual,
            null_mut(),
            &mut sd,
        )
    };
    if code != 0 {
        return Err(Failure {
            stage: "product-read-acl",
            code: code.into(),
        });
    }
    let _sd = Local(sd);
    let mut control = 0;
    let mut revision = 0;
    check(
        unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) },
        "product-acl-control",
    )?;
    if actual.is_null() || (directory && control & SE_DACL_PROTECTED == 0) {
        return Err(invalid("product-unprotected-acl"));
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
            "product-acl-info",
        )?;
        if info.AceCount != 2 {
            return Err(invalid("product-extra-acl-grant"));
        }
        let mut entries = Vec::new();
        for i in 0..2 {
            let mut raw = null_mut();
            check(unsafe { GetAce(acl, i, &mut raw) }, "product-acl-entry")?;
            let ace = unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() };
            if ace.Header.AceType != 0 || ace.Mask != FILE_ALL_ACCESS || ace.Header.AceSize < 20 {
                return Err(invalid("product-acl-rights"));
            }
            let sid = (&raw const ace.SidStart).cast_mut().cast();
            if unsafe { IsValidSid(sid) } == 0 {
                return Err(invalid("product-acl-sid"));
            }
            let length = unsafe { GetLengthSid(sid) } as usize;
            if length > usize::from(ace.Header.AceSize) - 8 {
                return Err(invalid("product-acl-sid-size"));
            }
            entries.push(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }.to_vec());
        }
        entries.sort();
        Ok(entries)
    }
    if entries(actual)? != entries(expected_acl)? {
        return Err(invalid("product-wrong-acl-principal"));
    }
    Ok(())
}
pub(super) fn protected_file(path: &Path) -> Result<fs::File> {
    let file = open_file(path, false, false)?;
    if file_info(&file)?.dwFileAttributes & BAD_ATTRIBUTES != 0 {
        return Err(invalid("product-unsafe-file"));
    }
    private_acl(path, false)?;
    Ok(file)
}
// Caller has authenticated an Abort for this exact hash, excluded all activated hashes,
// checked head/job/transition, and holds the store/ancestor leases. Never follow or sweep paths.
pub(super) fn remove_aborted_package(path: &Path) -> Result<()> {
    let file = match OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | DELETE)
        .share_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(Failure {
                stage: "product-aborted-package-open",
                code: error.raw_os_error().unwrap_or(0).into(),
            });
        }
    };
    let info = file_info(&file)?;
    if info.dwFileAttributes & (BAD_ATTRIBUTES | FILE_ATTRIBUTE_DIRECTORY) != 0
        || info.nNumberOfLinks != 1
    {
        return Err(invalid("product-unsafe-aborted-package"));
    }
    private_acl(path, false)?;
    let disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
    check(
        unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle(),
                FileDispositionInfo,
                (&raw const disposition).cast(),
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        },
        "product-aborted-package-delete",
    )?;
    // Delete disposition addresses the verified handle, not a newly resolved path.
    drop(file);
    Ok(())
}
pub(super) fn binary_pin(path: &Path) -> Result<(Vec<fs::File>, fs::File, [u8; 32])> {
    let normalized = local_path(path)?;
    let path = normalized.as_path();
    let dirs = pin_directories(path.parent().ok_or(invalid("product-binary-parent"))?)?;
    let mut file = open_file(path, false, false)?;
    let info = file_info(&file)?;
    if info.dwFileAttributes & BAD_ATTRIBUTES != 0 {
        return Err(invalid("product-unsafe-binary"));
    }
    let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
    if size == 0 || size > 128 * 1024 * 1024 {
        return Err(invalid("product-binary-budget"));
    }
    let mut digest = Sha256::new();
    let mut block = [0; 16384];
    loop {
        let n = io("product-binary-read", file.read(&mut block))?;
        if n == 0 {
            break;
        }
        digest.update(&block[..n]);
    }
    Ok((dirs, file, digest.finalize().into()))
}
