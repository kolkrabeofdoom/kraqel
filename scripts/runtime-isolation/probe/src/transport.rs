//! Research-only receive boundary. No named endpoint, network socket or renderer handle input.
#![deny(unsafe_op_in_unsafe_fn)]
use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

const MAX_FRAME: usize = 2048;

#[derive(Debug, Clone, Copy)]
pub enum ChannelError {
    InvalidFrame,
    InvalidIdentity,
    InvalidHandle,
    InvalidRoot,
    Io,
}

pub fn write_frame(writer: &mut impl Write, value: &impl Serialize) -> Result<(), ChannelError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ChannelError::InvalidFrame)?;
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(ChannelError::InvalidFrame);
    }
    writer
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .map_err(|_| ChannelError::Io)?;
    writer.write_all(&bytes).map_err(|_| ChannelError::Io)
}
pub fn read_frame<T: DeserializeOwned>(reader: &mut impl Read) -> Result<T, ChannelError> {
    let mut prefix = [0u8; 4];
    reader
        .read_exact(&mut prefix)
        .map_err(|_| ChannelError::InvalidFrame)?;
    let size = u32::from_le_bytes(prefix) as usize;
    if size == 0 || size > MAX_FRAME {
        return Err(ChannelError::InvalidFrame);
    }
    let mut bytes = vec![0u8; size];
    reader
        .read_exact(&mut bytes)
        .map_err(|_| ChannelError::InvalidFrame)?;
    let mut extra = [0u8; 1];
    if reader
        .read(&mut extra)
        .map_err(|_| ChannelError::InvalidFrame)?
        != 0
    {
        return Err(ChannelError::InvalidFrame);
    }
    serde_json::from_slice(&bytes).map_err(|_| ChannelError::InvalidFrame)
}

#[cfg(windows)]
mod native {
    use super::*;
    use kraqel_isolation_test_adapter::FileTask;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::{fs::File, os::windows::io::FromRawHandle, path::PathBuf, ptr::null_mut};
    use uuid::Uuid;
    pub(super) static CLAIMED: AtomicBool = AtomicBool::new(false);

    pub(super) fn claim(flag: &AtomicBool) -> bool {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    fn valid_task(task: &FileTask, expected: Uuid, size: u64, now: u64) -> bool {
        task.task_id == expected
            && !expected.is_nil()
            && task.contract_version == 1
            && task.byte_count == size
            && size > 0
            && size <= kraqel_isolation_test_adapter::MAX_TASK_BYTES
            && task
                .expires_unix_ms
                .checked_sub(task.issued_unix_ms)
                .is_some_and(|d| d > 0 && d <= 60000)
            && now >= task.issued_unix_ms
            && now < task.expires_unix_ms
    }
    use windows_sys::Win32::{
        Foundation::*, Security::*, Storage::FileSystem::*, System::Threading::*,
    };

    pub struct ReceivedGrant {
        pub task: FileTask,
        pub input: File,
        pub data_root: PathBuf,
    }

    type LiveReply = Option<(FileTask, std::io::Cursor<Vec<u8>>)>;

    pub struct LiveSession {
        request: File,
        response: File,
        expected: Uuid,
        pub data_root: PathBuf,
    }
    impl LiveSession {
        /// Native controller binding; never supplied by the renderer.
        pub fn task_id(&self) -> Uuid {
            self.expected
        }

        pub fn select(self, cancelled: impl Fn() -> bool) -> Result<LiveReply, ChannelError> {
            self.select_observed(cancelled, |_| {})
        }
        pub fn select_observed(
            mut self,
            cancelled: impl Fn() -> bool,
            observe: impl Fn(usize),
        ) -> Result<LiveReply, ChannelError> {
            self.request.write_all(&[1]).map_err(|_| ChannelError::Io)?;
            drop(self.request);
            let mut bytes = Vec::new();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
            loop {
                if cancelled() {
                    return Err(ChannelError::Io);
                }
                let mut available = 0;
                use std::os::windows::io::AsRawHandle;
                let ok = unsafe {
                    windows_sys::Win32::System::Pipes::PeekNamedPipe(
                        self.response.as_raw_handle(),
                        std::ptr::null_mut(),
                        0,
                        std::ptr::null_mut(),
                        &mut available,
                        std::ptr::null_mut(),
                    )
                };
                if ok == 0 {
                    break;
                }
                if std::time::Instant::now() >= deadline {
                    return Err(ChannelError::Io);
                }
                if available == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    continue;
                }
                if bytes.len() + available as usize
                    > kraqel_isolation_test_adapter::MAX_TASK_BYTES as usize + 2053
                {
                    return Err(ChannelError::InvalidFrame);
                }
                let old = bytes.len();
                bytes.resize(old + available as usize, 0);
                self.response
                    .read_exact(&mut bytes[old..])
                    .map_err(|_| ChannelError::Io)?;
                if bytes.len() >= 5 && bytes[0] == 1 {
                    let header = u32::from_le_bytes(
                        bytes[1..5]
                            .try_into()
                            .map_err(|_| ChannelError::InvalidFrame)?,
                    ) as usize;
                    if header > MAX_FRAME {
                        return Err(ChannelError::InvalidFrame);
                    }
                    let payload = bytes.len().saturating_sub(5 + header);
                    if payload > 0 {
                        let progress = serde_json::to_vec(
                            &serde_json::json!({"payloadBytesReceived":payload}),
                        )
                        .map_err(|_| ChannelError::Io)?;
                        std::fs::write(self.data_root.join("transfer-received.json"), progress)
                            .map_err(|_| ChannelError::Io)?;
                        observe(payload);
                    }
                }
            }
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| ChannelError::InvalidIdentity)?
                .as_millis() as u64;
            decode_live_reply(bytes, self.expected, now)
        }
    }

    pub(super) fn live_from_restart(
        a: HANDLE,
        b: HANDLE,
        expected: Uuid,
        data_root: PathBuf,
    ) -> Result<LiveSession, ChannelError> {
        if a == b
            || expected.is_nil()
            || unsafe { GetFileType(a) } != FILE_TYPE_PIPE
            || unsafe { GetFileType(b) } != FILE_TYPE_PIPE
        {
            return Err(ChannelError::InvalidHandle);
        }
        let request = unsafe { File::from_raw_handle(a) };
        let response = unsafe { File::from_raw_handle(b) };
        for handle in [a, b] {
            let mut flags = 0;
            if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0
                || unsafe { GetHandleInformation(handle, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT != 0
            {
                return Err(ChannelError::InvalidHandle);
            }
        }
        Ok(LiveSession {
            request,
            response,
            expected,
            data_root,
        })
    }
    fn decode_live_reply(
        mut bytes: Vec<u8>,
        expected: Uuid,
        now: u64,
    ) -> Result<LiveReply, ChannelError> {
        if bytes == [0] {
            return Ok(None);
        }
        if bytes.len() < 5 || bytes[0] != 1 {
            return Err(ChannelError::InvalidFrame);
        }
        let size = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as usize;
        if size == 0 || size > MAX_FRAME || bytes.len() < size + 5 {
            return Err(ChannelError::InvalidFrame);
        }
        let task: FileTask =
            serde_json::from_slice(&bytes[5..5 + size]).map_err(|_| ChannelError::InvalidFrame)?;
        let payload = bytes.split_off(5 + size);
        if !valid_task(&task, expected, payload.len() as u64, now) {
            return Err(ChannelError::InvalidIdentity);
        }
        Ok(Some((task, std::io::Cursor::new(payload))))
    }

    pub fn receive_live_session() -> Result<LiveSession, ChannelError> {
        if !contained() || !claim(&CLAIMED) {
            return Err(ChannelError::InvalidIdentity);
        }
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        if args.len() != 4 || args[0] != "ui-live" {
            return Err(ChannelError::InvalidFrame);
        }
        let parse = |i: usize| {
            args[i]
                .parse::<usize>()
                .ok()
                .filter(|v| *v != 0 && *v != usize::MAX)
                .ok_or(ChannelError::InvalidHandle)
        };
        let a = parse(1)? as HANDLE;
        let b = parse(2)? as HANDLE;
        if a == b
            || unsafe { GetFileType(a) } != FILE_TYPE_PIPE
            || unsafe { GetFileType(b) } != FILE_TYPE_PIPE
        {
            return Err(ChannelError::InvalidHandle);
        }
        let request = unsafe { File::from_raw_handle(a) };
        let response = unsafe { File::from_raw_handle(b) };
        for handle in [a, b] {
            let mut flags = 0;
            if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0
                || unsafe { GetHandleInformation(handle, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT != 0
            {
                return Err(ChannelError::InvalidHandle);
            }
        }
        let expected = Uuid::parse_str(&args[3]).map_err(|_| ChannelError::InvalidIdentity)?;
        if expected.is_nil() {
            return Err(ChannelError::InvalidIdentity);
        }
        let exe = std::env::current_exe().map_err(|_| ChannelError::InvalidRoot)?;
        let root = exe
            .parent()
            .and_then(|p| p.parent())
            .ok_or(ChannelError::InvalidRoot)?;
        let name = root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(ChannelError::InvalidRoot)?;
        if !name.starts_with("kraqel-isolation-ui-")
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || root.parent().and_then(|p| p.canonicalize().ok())
                != std::env::temp_dir().canonicalize().ok()
            || exe != root.join("bin/kraqel-desktop.exe")
        {
            return Err(ChannelError::InvalidRoot);
        }
        Ok(LiveSession {
            request,
            response,
            expected,
            data_root: root.join("data"),
        })
    }

    pub(super) fn contained() -> bool {
        let mut token = null_mut();
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return false;
        }
        let result = (|| {
            let mut yes = 0u32;
            let mut returned = 0;
            if unsafe {
                GetTokenInformation(
                    token,
                    TokenIsAppContainer,
                    (&mut yes as *mut u32).cast(),
                    4,
                    &mut returned,
                )
            } == 0
                || yes != 1
            {
                return false;
            }
            let mut size = 0;
            unsafe {
                GetTokenInformation(token, TokenCapabilities, null_mut(), 0, &mut size);
            }
            if !(4..=65536).contains(&size) {
                return false;
            }
            let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
            if unsafe {
                GetTokenInformation(
                    token,
                    TokenCapabilities,
                    buffer.as_mut_ptr().cast(),
                    size,
                    &mut returned,
                )
            } == 0
            {
                return false;
            }
            unsafe { buffer.as_ptr().cast::<u32>().read() == 0 }
        })();
        unsafe {
            CloseHandle(token);
        }
        result
    }

    /// Only the trusted launcher supplies these handles. Clears inheritance before WebView starts.
    pub fn receive_from_launcher() -> Result<ReceivedGrant, ChannelError> {
        if !contained() {
            return Err(ChannelError::InvalidIdentity);
        }
        if !claim(&CLAIMED) {
            return Err(ChannelError::InvalidIdentity);
        }
        let args = std::env::args_os().skip(1).collect::<Vec<_>>();
        if args.len() != 4 || args[0] != "ui-grant" {
            return Err(ChannelError::InvalidFrame);
        }
        let parse = |i: usize| {
            args[i]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|v| *v != 0 && *v != usize::MAX)
                .ok_or(ChannelError::InvalidHandle)
        };
        let first = parse(1)?;
        let second = parse(2)?;
        if first == second {
            return Err(ChannelError::InvalidHandle);
        }
        let expected = Uuid::parse_str(&args[3].to_string_lossy())
            .map_err(|_| ChannelError::InvalidIdentity)?;
        let raw_file = first as HANDLE;
        let raw_pipe = second as HANDLE;
        if unsafe { GetFileType(raw_file) } != FILE_TYPE_DISK
            || unsafe { GetFileType(raw_pipe) } != FILE_TYPE_PIPE
        {
            return Err(ChannelError::InvalidHandle);
        }
        let input = unsafe { File::from_raw_handle(raw_file) };
        let mut pipe = unsafe { File::from_raw_handle(raw_pipe) };
        if unsafe { SetHandleInformation(raw_file, HANDLE_FLAG_INHERIT, 0) } == 0
            || unsafe { SetHandleInformation(raw_pipe, HANDLE_FLAG_INHERIT, 0) } == 0
        {
            return Err(ChannelError::InvalidHandle);
        }
        let task: FileTask = read_frame(&mut pipe)?;
        for handle in [raw_file, raw_pipe] {
            let mut flags = 0;
            if unsafe { GetHandleInformation(handle, &mut flags) } == 0
                || flags & HANDLE_FLAG_INHERIT != 0
            {
                return Err(ChannelError::InvalidHandle);
            }
        }
        drop(pipe);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|v| u64::try_from(v.as_millis()).ok())
            .ok_or(ChannelError::InvalidIdentity)?;
        if !valid_task(
            &task,
            expected,
            input.metadata().map_err(|_| ChannelError::Io)?.len(),
            now,
        ) {
            return Err(ChannelError::InvalidIdentity);
        }
        let mut info = unsafe { std::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>() };
        if unsafe { GetFileInformationByHandle(raw_file, &mut info) } == 0
            || info.nNumberOfLinks != 1
            || info.dwFileAttributes
                & (FILE_ATTRIBUTE_REPARSE_POINT
                    | FILE_ATTRIBUTE_DIRECTORY
                    | FILE_ATTRIBUTE_OFFLINE
                    | FILE_ATTRIBUTE_RECALL_ON_OPEN
                    | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
                != 0
            || task.byte_count
                != ((u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow))
        {
            return Err(ChannelError::InvalidHandle);
        }
        let exe = std::env::current_exe().map_err(|_| ChannelError::InvalidRoot)?;
        let root = exe
            .parent()
            .and_then(|p| p.parent())
            .ok_or(ChannelError::InvalidRoot)?;
        let name = root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(ChannelError::InvalidRoot)?;
        if !name.starts_with("kraqel-isolation-ui-")
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || root.parent().and_then(|p| p.canonicalize().ok())
                != std::env::temp_dir().canonicalize().ok()
            || exe != root.join("bin/kraqel-desktop.exe")
        {
            return Err(ChannelError::InvalidRoot);
        }
        Ok(ReceivedGrant {
            task,
            input,
            data_root: root.join("data"),
        })
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn live_reply_rejects_truncation_replay_binding_expiry_and_trailing_bytes() {
            let id = Uuid::new_v4();
            let task = FileTask {
                contract_version: 1,
                task_id: id,
                operation: kraqel_isolation_test_adapter::FileOperation::ImportSnapshotV3,
                byte_count: 3,
                issued_unix_ms: 1000,
                expires_unix_ms: 61000,
            };
            let json = serde_json::to_vec(&task).unwrap();
            let mut bytes = vec![1];
            bytes.extend_from_slice(&(json.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&json);
            bytes.extend_from_slice(b"abc");
            assert!(
                decode_live_reply(bytes.clone(), id, 1001)
                    .unwrap()
                    .is_some()
            );
            for length in 0..bytes.len() {
                assert!(decode_live_reply(bytes[..length].to_vec(), id, 1001).is_err());
            }
            assert!(decode_live_reply(bytes.clone(), Uuid::new_v4(), 1001).is_err());
            assert!(decode_live_reply(bytes.clone(), id, 61000).is_err());
            bytes.push(0);
            assert!(decode_live_reply(bytes, id, 1001).is_err());
            assert!(decode_live_reply(vec![0], id, 1001).unwrap().is_none());
            assert!(decode_live_reply(vec![0, 0], id, 1001).is_err());
            assert!(decode_live_reply(vec![1, 255, 255, 255, 255], id, 1001).is_err());
        }
        #[test]
        fn binding_budget_and_expiry_are_checked_before_ui_start() {
            let id = Uuid::new_v4();
            let mut task = FileTask {
                contract_version: 1,
                task_id: id,
                operation: kraqel_isolation_test_adapter::FileOperation::ImportSnapshotV3,
                byte_count: 7,
                issued_unix_ms: 1000,
                expires_unix_ms: 61000,
            };
            assert!(valid_task(&task, id, 7, 1001));
            assert!(!valid_task(&task, Uuid::new_v4(), 7, 1001));
            assert!(!valid_task(&task, id, 8, 1001));
            assert!(!valid_task(&task, id, 7, 61000));
            task.contract_version = 2;
            assert!(!valid_task(&task, id, 7, 1001));
        }
        #[test]
        fn channel_can_only_be_claimed_once() {
            let flag = AtomicBool::new(false);
            assert!(claim(&flag));
            assert!(!claim(&flag));
        }
    }
}
#[cfg(windows)]
pub use native::{LiveSession, ReceivedGrant, receive_from_launcher, receive_live_session};

#[cfg(windows)]
mod restart;
#[cfg(windows)]
pub use restart::{
    CrashPoint, RecoverySummary, RestartEvent, RestartEventKind, RestartGrant, RestartSource,
    RestartTicket, receive_restart,
};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_frame_requires_eof_and_rejects_truncation_and_oversize() {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &serde_json::json!({"v":1})).unwrap();
        assert!(read_frame::<serde_json::Value>(&mut &bytes[..]).is_ok());
        for size in 0..bytes.len() {
            assert!(read_frame::<serde_json::Value>(&mut &bytes[..size]).is_err());
        }
        bytes.push(0);
        assert!(read_frame::<serde_json::Value>(&mut &bytes[..]).is_err());
        assert!(read_frame::<serde_json::Value>(&mut &u32::MAX.to_le_bytes()[..]).is_err());
        assert!(write_frame(&mut Vec::new(), &"x".repeat(MAX_FRAME)).is_err());
    }
    #[test]
    fn unknown_fields_and_operations_remain_typed_errors() {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Envelope {
            #[serde(rename = "version")]
            _version: u16,
        }
        let mut bytes = Vec::new();
        write_frame(
            &mut bytes,
            &serde_json::json!({"version":1,"path":"C:\\private"}),
        )
        .unwrap();
        assert!(read_frame::<Envelope>(&mut &bytes[..]).is_err());
    }
}
