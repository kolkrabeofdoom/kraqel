//! Narrow Win32 bindings used while capturing an explicitly selected local project.
#![cfg(windows)]
use std::{io, os::windows::ffi::OsStrExt, path::Path};
use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT},
    Storage::FileSystem::{
        FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME, FindCloseChangeNotification,
        FindFirstChangeNotificationW, GetDriveTypeW,
    },
    System::Threading::WaitForSingleObject,
};

/// Check before any filesystem access to a picker-provided drive.
pub fn is_local_drive(letter: u8) -> bool {
    let root = [u16::from(letter), 58, 92, 0];
    // SAFETY: root is a live NUL-terminated UTF-16 drive designator.
    matches!(unsafe { GetDriveTypeW(root.as_ptr()) }, 2 | 3)
}

/// One-shot recursive signal. Reads and last-access timestamps do not signal it.
pub struct TreeChange(HANDLE);

// SAFETY: a change-notification handle has no thread affinity. Moving the sole
// owner between the native picker and its confirmation worker preserves single close.
unsafe impl Send for TreeChange {}

impl TreeChange {
    /// Start before enumeration, while the path and its ancestors are pinned.
    pub fn watch(path: &Path) -> io::Result<Self> {
        let mut name: Vec<u16> = path.as_os_str().encode_wide().collect();
        if name.contains(&0) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        name.push(0);
        let filter = FILE_NOTIFY_CHANGE_DIR_NAME | FILE_NOTIFY_CHANGE_FILE_NAME;
        // SAFETY: name remains valid through the synchronous call; we own its result.
        let handle = unsafe { FindFirstChangeNotificationW(name.as_ptr(), 1, filter) };
        if handle == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }

    pub fn is_unchanged(&self) -> bool {
        // SAFETY: the notification handle stays live for the borrowed lifetime.
        unsafe { WaitForSingleObject(self.0, 0) == WAIT_TIMEOUT }
    }
}

impl Drop for TreeChange {
    fn drop(&mut self) {
        // SAFETY: handle came from FindFirstChangeNotificationW and is closed once.
        unsafe { FindCloseChangeNotification(self.0) };
    }
}
