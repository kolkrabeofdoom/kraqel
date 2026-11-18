//! Fail-closed sampled module admission. No signer exceptions or complete load attestation.
use super::*;
use std::collections::BTreeSet;
use windows_sys::Win32::System::SystemInformation::{GetSystemDirectoryW, GetWindowsDirectoryW};
type FileId = (u64, [u8; 16]);
fn id(file: &fs::File) -> Result<FileId> {
    let mut info: FILE_ID_INFO = unsafe { zeroed() };
    check(
        unsafe {
            GetFileInformationByHandleEx(
                file.as_raw_handle(),
                FileIdInfo,
                (&mut info as *mut FILE_ID_INFO).cast(),
                size_of_val(&info) as u32,
            )
        },
        "c4-module-file-id",
    )?;
    Ok((info.VolumeSerialNumber, info.FileId.Identifier))
}
fn open(path: &Path, directory: bool) -> Result<fs::File> {
    let _parents = pin_directories(path.parent().ok_or(invalid("c4-module-parent"))?)?;
    let file = io(
        "c4-module-open",
        OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(
                FILE_SHARE_READ | FILE_SHARE_WRITE | if directory { 0 } else { FILE_SHARE_DELETE },
            )
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
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
    check(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
        "c4-module-kind",
    )?;
    if info.dwFileAttributes & BAD_ATTRIBUTES != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != directory
    {
        return Err(invalid("c4-module-unsafe-kind"));
    }
    Ok(file)
}
pub(super) fn candidate_id(path: &Path) -> Result<FileId> {
    id(&open(path, false)?)
}
fn canonical(file: &fs::File) -> Result<PathBuf> {
    let mut buffer = vec![0; 32768];
    let n = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            0,
        )
    } as usize;
    if n == 0 || n >= buffer.len() {
        return Err(invalid("c4-module-canonical"));
    }
    Ok(PathBuf::from(
        String::from_utf16(&buffer[..n]).map_err(|_| invalid("c4-module-canonical-utf16"))?,
    ))
}
fn beneath(path: &Path, root: &Path) -> Result<bool> {
    let mut parts = path.components();
    for expected in root.components() {
        let Some(actual) = parts.next() else {
            return Ok(false);
        };
        if !filesystem::ordinal_equal(
            actual
                .as_os_str()
                .to_str()
                .ok_or(invalid("c4-module-component"))?,
            expected
                .as_os_str()
                .to_str()
                .ok_or(invalid("c4-module-component"))?,
        )? {
            return Ok(false);
        }
    }
    Ok(parts.next().is_some())
}
pub(super) struct Policy {
    package: BTreeSet<FileId>,
    platform: Vec<(fs::File, PathBuf, bool)>,
}
impl Policy {
    pub(super) fn new(package: BTreeSet<FileId>) -> Result<Self> {
        let mut buffer = vec![0; 32768];
        let n = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
        if n == 0 || n >= buffer.len() {
            return Err(invalid("c4-module-system-directory"));
        }
        let system = PathBuf::from(
            String::from_utf16(&buffer[..n]).map_err(|_| invalid("c4-module-system-utf16"))?,
        );
        let n = unsafe { GetWindowsDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) } as usize;
        if n == 0 || n >= buffer.len() {
            return Err(invalid("c4-module-windows-directory"));
        }
        let winsxs = PathBuf::from(
            String::from_utf16(&buffer[..n]).map_err(|_| invalid("c4-module-windows-utf16"))?,
        )
        .join("WinSxS");
        let mut platform = Vec::new();
        for (path, recursive) in [
            (system.clone(), false),
            (system.join("DriverStore/FileRepository"), true),
            (winsxs, true),
        ] {
            let file = open(&path, true)?;
            let path = canonical(&file)?;
            platform.push((file, path, recursive));
        }
        Ok(Self { package, platform })
    }
    pub(super) fn permits(&self, path: &Path) -> Result<bool> {
        let file = open(path, false)?;
        if self.package.contains(&id(&file)?) {
            let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
            check(
                unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
                "c4-module-link-count",
            )?;
            return Ok(info.nNumberOfLinks == 1);
        }
        let actual = canonical(&file)?;
        for (_, root, recursive) in &self.platform {
            if beneath(&actual, root)?
                && (*recursive || actual.components().count() == root.components().count() + 1)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c4_module_policy_binds_file_identity_and_rejects_equal_copy_and_prefix_alias() {
        let root = std::env::temp_dir().join(format!(
            "kraqel-isolation-controller-{}",
            Uuid::new_v4().simple()
        ));
        secure_directory(&root).unwrap();
        let a = root.join("allowed.dll");
        let b = root.join("copy.dll");
        write_new(&a, b"synthetic, never loaded").unwrap();
        write_new(&b, b"synthetic, never loaded").unwrap();
        let policy = Policy::new([candidate_id(&a).unwrap()].into()).unwrap();
        assert!(policy.permits(&a).unwrap());
        assert!(!policy.permits(&b).unwrap());
        assert!(policy.permits(&root.join("missing.dll")).is_err());
        assert!(
            !beneath(
                Path::new("C:/Windows/System32-extra/f.dll"),
                Path::new("C:/Windows/System32")
            )
            .unwrap()
        );
        assert!(
            beneath(
                Path::new("C:/WINDOWS/system32/f.dll"),
                Path::new("c:/Windows/System32")
            )
            .unwrap()
        );
        assert!(
            policy
                .permits(&policy.platform[0].1.join("ntdll.dll"))
                .unwrap()
        );
        let resolved = root.canonicalize().unwrap();
        assert_eq!(
            resolved.parent(),
            Some(std::env::temp_dir().canonicalize().unwrap().as_path())
        );
        assert!(crate::policy::valid_root(&resolved, true));
        drop(policy);
        fs::remove_dir_all(resolved).unwrap();
    }
}
