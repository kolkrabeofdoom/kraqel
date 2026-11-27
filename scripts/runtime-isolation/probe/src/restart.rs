//! Capability-bound research restart ticket. No named endpoint or renderer-supplied authority.
use super::*;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use std::{
    fs::File,
    os::windows::{
        fs::MetadataExt as _,
        io::{AsRawHandle, FromRawHandle},
    },
    path::PathBuf,
    ptr::null_mut,
};
use uuid::Uuid;
use windows_sys::Win32::{
    Foundation::*,
    Security::{Isolation::*, *},
    Storage::FileSystem::*,
    System::Threading::*,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CrashPoint {
    Claimed,
    Reading,
    Prepared,
    CommitIntent,
    Published,
    Completed,
}
impl CrashPoint {
    pub const ALL: [Self; 6] = [
        Self::Claimed,
        Self::Reading,
        Self::Prepared,
        Self::CommitIntent,
        Self::Published,
        Self::Completed,
    ];
    pub fn publishes(self) -> bool {
        matches!(self, Self::Published | Self::Completed)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestartTicket {
    pub version: u16,
    pub session_id: Uuid,
    pub task_id: Uuid,
    pub launch_id: Uuid,
    pub recovery: bool,
    pub automatic: bool,
    pub source: RestartSource,
    pub point: CrashPoint,
    pub app_sha256: [u8; 32],
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RestartSource {
    Synthetic,
    Live,
}
impl RestartTicket {
    pub fn live_channel(&self) -> bool {
        self.source == RestartSource::Live && !self.recovery
    }
    pub const VERSION: u16 = 3;
    pub fn valid_for(&self, launch: Uuid) -> bool {
        self.version == Self::VERSION
            && !self.session_id.is_nil()
            && !self.task_id.is_nil()
            && !launch.is_nil()
            && self.launch_id == launch
            && (self.automatic || self.recovery || self.source == RestartSource::Live)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoverySummary {
    pub published: bool,
    pub document_count: u32,
    pub project_count: u32,
    pub staging_empty: bool,
    pub replay_blocked: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RestartEventKind {
    Renderer,
    Checkpoint,
    Recovered,
    Replay,
    ReuseRejected,
    Finished,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestartEvent {
    pub launch_id: Uuid,
    pub kind: RestartEventKind,
    pub summary: Option<RecoverySummary>,
}
pub struct RestartGrant {
    pub ticket: RestartTicket,
    pub data_root: PathBuf,
    pub live: Option<super::native::LiveSession>,
    events: File,
}
impl RestartGrant {
    pub fn event(
        &mut self,
        kind: RestartEventKind,
        summary: Option<RecoverySummary>,
    ) -> Result<(), ChannelError> {
        write_frame(
            &mut self.events,
            &RestartEvent {
                launch_id: self.ticket.launch_id,
                kind,
                summary,
            },
        )
    }
}

fn identity_matches(session: Uuid) -> bool {
    let name = format!("org.kraqel.probe.ui-restart-{}", session.simple())
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut expected = null_mut();
    if unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut expected) } < 0 {
        return false;
    }
    let mut token = null_mut();
    let result = if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        false
    } else {
        let mut size = 0;
        unsafe { GetTokenInformation(token, TokenAppContainerSid, null_mut(), 0, &mut size) };
        let matches = if (std::mem::size_of::<TOKEN_APPCONTAINER_INFORMATION>()..=65536)
            .contains(&(size as usize))
        {
            let mut buf = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
            let ok = unsafe {
                GetTokenInformation(
                    token,
                    TokenAppContainerSid,
                    buf.as_mut_ptr().cast(),
                    size,
                    &mut size,
                )
            };
            ok != 0
                && unsafe {
                    EqualSid(
                        (*buf.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>()).TokenAppContainer,
                        expected,
                    )
                } != 0
        } else {
            false
        };
        unsafe { CloseHandle(token) };
        matches
    };
    unsafe { FreeSid(expected) };
    result
}

/// Receives a fresh EOF-terminated ticket through the single inherited startup pipe.
/// The random argument is only a binding; the pipe and exact OS identity supply authority.
pub fn receive_restart() -> Result<RestartGrant, ChannelError> {
    if !super::native::contained() || !super::native::claim(&super::native::CLAIMED) {
        return Err(ChannelError::InvalidIdentity);
    }
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !matches!(args.len(), 4 | 6) || args[0] != "ui-restart" {
        return Err(ChannelError::InvalidFrame);
    }
    let parse = |n: usize| {
        args[n]
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
    let mut input = unsafe { File::from_raw_handle(a) };
    let events = unsafe { File::from_raw_handle(b) };
    for handle in [input.as_raw_handle(), events.as_raw_handle()] {
        let mut flags = 0;
        if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0
            || unsafe { GetHandleInformation(handle, &mut flags) } == 0
            || flags & HANDLE_FLAG_INHERIT != 0
        {
            return Err(ChannelError::InvalidHandle);
        }
    }
    let ticket: RestartTicket = read_frame(&mut input)?;
    let launch = Uuid::parse_str(&args[3]).map_err(|_| ChannelError::InvalidIdentity)?;
    if !ticket.valid_for(launch) || !identity_matches(ticket.session_id) {
        return Err(ChannelError::InvalidIdentity);
    }
    let exe = std::env::current_exe().map_err(|_| ChannelError::InvalidRoot)?;
    let root = exe
        .parent()
        .and_then(|p| p.parent())
        .ok_or(ChannelError::InvalidRoot)?;
    let name = format!("kraqel-isolation-ui-restart-{}", ticket.session_id.simple());
    if root.file_name() != Some(std::ffi::OsStr::new(&name))
        || exe != root.join("bin/kraqel-desktop.exe")
    {
        return Err(ChannelError::InvalidRoot);
    }
    for path in [root.to_path_buf(), root.join("bin"), exe.clone()] {
        let info = std::fs::symlink_metadata(path).map_err(|_| ChannelError::InvalidRoot)?;
        if info.file_attributes() & 0x400 != 0 {
            return Err(ChannelError::InvalidRoot);
        }
    }
    let mut hash = Sha256::new();
    let mut file = File::open(&exe).map_err(|_| ChannelError::Io)?;
    let mut block = [0u8; 16384];
    loop {
        let n = file.read(&mut block).map_err(|_| ChannelError::Io)?;
        if n == 0 {
            break;
        }
        hash.update(&block[..n]);
    }
    let actual: [u8; 32] = hash.finalize().into();
    if actual != ticket.app_sha256 {
        return Err(ChannelError::InvalidIdentity);
    }
    let needs_live = ticket.live_channel();
    if needs_live != (args.len() == 6) {
        return Err(ChannelError::InvalidFrame);
    }
    let live = if needs_live {
        let c = parse(4)? as HANDLE;
        let d = parse(5)? as HANDLE;
        if [a, b].contains(&c) || [a, b, c].contains(&d) {
            return Err(ChannelError::InvalidHandle);
        }
        Some(super::native::live_from_restart(
            c,
            d,
            ticket.task_id,
            root.join("data"),
        )?)
    } else {
        None
    };
    Ok(RestartGrant {
        ticket,
        data_root: root.join("data"),
        live,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_is_mandatory_and_only_initial_live_tasks_get_selection_handles() {
        let mut ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: false,
            automatic: false,
            source: RestartSource::Live,
            point: CrashPoint::Prepared,
            app_sha256: [7; 32],
        };
        assert!(ticket.valid_for(ticket.launch_id) && ticket.live_channel());
        ticket.recovery = true;
        assert!(ticket.valid_for(ticket.launch_id) && !ticket.live_channel());
        ticket.recovery = false;
        ticket.source = RestartSource::Synthetic;
        assert!(!ticket.valid_for(ticket.launch_id) && !ticket.live_channel());
        let mut value = serde_json::to_value(&ticket).unwrap();
        value.as_object_mut().unwrap().remove("source");
        assert!(serde_json::from_value::<RestartTicket>(value.clone()).is_err());
        value["source"] = serde_json::json!("arbitraryPath");
        assert!(serde_json::from_value::<RestartTicket>(value).is_err());
    }
    #[test]
    fn tickets_reject_unknown_authority_and_wrong_launch_binding() {
        let ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: true,
            automatic: true,
            source: RestartSource::Synthetic,
            point: CrashPoint::Prepared,
            app_sha256: [7; 32],
        };
        assert!(ticket.valid_for(ticket.launch_id));
        assert!(!ticket.valid_for(Uuid::new_v4()));
        let mut value = serde_json::to_value(&ticket).unwrap();
        value["path"] = serde_json::json!("C:\\private");
        assert!(serde_json::from_value::<RestartTicket>(value).is_err());
        for version in [0, 1, 2, 4] {
            let mut t = ticket.clone();
            t.version = version;
            assert!(!t.valid_for(t.launch_id));
        }
        let mut t = ticket.clone();
        t.session_id = Uuid::nil();
        assert!(!t.valid_for(t.launch_id));
        let mut manual = ticket.clone();
        manual.automatic = false;
        assert!(manual.valid_for(manual.launch_id));
        manual.recovery = false;
        assert!(!manual.valid_for(manual.launch_id));
        let mut t = ticket.clone();
        t.task_id = Uuid::nil();
        assert!(!t.valid_for(t.launch_id));
        let mut t = ticket.clone();
        t.launch_id = Uuid::nil();
        assert!(!t.valid_for(t.launch_id));
        let event = RestartEvent {
            launch_id: ticket.launch_id,
            kind: RestartEventKind::Replay,
            summary: None,
        };
        let mut value = serde_json::to_value(event).unwrap();
        value["path"] = serde_json::json!("private");
        assert!(serde_json::from_value::<RestartEvent>(value).is_err());
    }
}
