//! BP3 synthetic debugger experiment. No store, WebView, project commands or native grants.
//! Pipe reports are observations, not authority. Only the ignored local controller can launch it.
use super::*;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    fs::File,
    os::windows::io::{AsRawHandle, FromRawHandle},
};
use windows_sys::Win32::{
    Storage::FileSystem::*,
    System::{Diagnostics::Debug::*, Pipes::*},
};
#[cfg(test)]
mod controller;
mod worker;
pub(super) use worker::run as worker;

const PURPOSE: &str = "bp3-synthetic-debug-chain-v1";
const LIMIT: usize = 2048;
fn invalid(stage: &'static str) -> Failure {
    Failure { stage, code: -1 }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    pid: u32,
    created: u64,
}
fn identity(process: HANDLE) -> Result<Identity> {
    let (mut c, mut e, mut k, mut u) = unsafe { zeroed() };
    check(
        unsafe { GetProcessTimes(process, &mut c, &mut e, &mut k, &mut u) },
        "bp3-generation",
    )?;
    let value = Identity {
        pid: unsafe { GetProcessId(process) },
        created: (u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime),
    };
    if value.pid == 0 || value.created == 0 {
        return Err(invalid("bp3-generation"));
    }
    Ok(value)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Mode {
    Inherit,
    NestedDebug,
    Leaf,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Boot {
    purpose: String,
    nonce: [u8; 32],
    parent: u32,
    mode: Mode,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    nonce: [u8; 32],
    kind: Kind,
    process: Identity,
    child: Option<Identity>,
    code: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    RootStarted,
    ChildCreated,
    InnerCreate,
    InnerConsole,
    LeafMarker,
    RootFinished,
    ChildFailed,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Go {
    nonce: [u8; 32],
    child: Identity,
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid("bp3-frame-encode"))?;
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err(invalid("bp3-frame-limit"));
    }
    let mut out = (bytes.len() as u32).to_le_bytes().to_vec();
    out.extend(bytes);
    Ok(out)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() < 4 {
        return Err(invalid("bp3-frame-short"));
    }
    let n = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
    if n == 0 || n > LIMIT || bytes.len() != n + 4 {
        return Err(invalid("bp3-frame-limit"));
    }
    serde_json::from_slice(&bytes[4..]).map_err(|_| invalid("bp3-frame-shape"))
}
fn available(file: &File) -> Result<u32> {
    let mut n = 0;
    let ok = unsafe {
        PeekNamedPipe(
            file.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            &mut n,
            null_mut(),
        )
    };
    if ok == 0 && unsafe { GetLastError() } == ERROR_BROKEN_PIPE {
        return Ok(0);
    }
    check(ok, "bp3-pipe-peek")?;
    Ok(n)
}
fn receive<T: serde::de::DeserializeOwned>(file: &mut File, deadline: Instant) -> Result<T> {
    let mut bytes = Vec::new();
    let mut goal = 4;
    while bytes.len() < goal {
        if Instant::now() >= deadline {
            return Err(invalid("bp3-pipe-deadline"));
        }
        let n = available(file)? as usize;
        if n == 0 {
            thread::sleep(Duration::from_millis(2));
            continue;
        }
        let mut part = vec![0; n.min(goal - bytes.len())];
        io("bp3-pipe-read", file.read_exact(&mut part))?;
        bytes.extend(part);
        if bytes.len() == 4 && goal == 4 {
            let size = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            if size == 0 || size > LIMIT {
                return Err(invalid("bp3-frame-limit"));
            }
            goal = size + 4;
        }
    }
    decode(&bytes)
}
fn send(file: &mut File, value: &impl Serialize) -> Result<()> {
    io("bp3-pipe-write", file.write_all(&encode(value)?))
}
fn inheritable(handles: &[HANDLE], inherit: bool) -> Result<()> {
    for handle in handles {
        check(
            unsafe {
                SetHandleInformation(
                    *handle,
                    HANDLE_FLAG_INHERIT,
                    if inherit { HANDLE_FLAG_INHERIT } else { 0 },
                )
            },
            "bp3-inherit",
        )?;
    }
    Ok(())
}
fn pipe() -> Result<(File, File)> {
    let (mut read, mut write) = (null_mut(), null_mut());
    check(
        unsafe { CreatePipe(&mut read, &mut write, null(), 4096) },
        "bp3-pipe",
    )?;
    Ok(unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) })
}
fn prepaid(value: &impl Serialize) -> Result<File> {
    let (read, mut write) = pipe()?;
    send(&mut write, value)?;
    drop(write);
    Ok(read)
}
fn own_token() -> Result<()> {
    let facts = token_facts(unsafe { GetCurrentProcess() }, None)?;
    let mut member = 0;
    check(
        unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut member) },
        "bp3-worker-job",
    )?;
    if facts["isAppContainer"] != true || facts["capabilityCount"] != 0 || member == 0 {
        return Err(invalid("bp3-worker-token"));
    }
    Ok(())
}
fn image_path(process: HANDLE) -> Result<PathBuf> {
    let mut name = [0u16; 32768];
    let mut n = name.len() as u32;
    check(
        unsafe { QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut n) },
        "bp3-image-query",
    )?;
    Ok(PathBuf::from(
        String::from_utf16(&name[..n as usize]).map_err(|_| invalid("bp3-image-query"))?,
    ))
}
fn console_path() -> Result<PathBuf> {
    let mut name = [0u16; 32768];
    let n = unsafe {
        windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(
            name.as_mut_ptr(),
            name.len() as u32,
        )
    } as usize;
    if n == 0 || n >= name.len() {
        return Err(invalid("bp3-system-directory"));
    }
    Ok(
        PathBuf::from(String::from_utf16(&name[..n]).map_err(|_| invalid("bp3-system-directory"))?)
            .join("conhost.exe"),
    )
}
fn inner_console(process: HANDLE) -> Result<bool> {
    if io("bp3-console-image", image_path(process)?.canonicalize())?
        != io("bp3-console-image", console_path()?.canonicalize())?
    {
        return Ok(false);
    }
    let mut token = null_mut();
    check(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) },
        "bp3-self-token",
    )?;
    let token = Handle(token);
    let raw = token_bytes(token.0, TokenAppContainerSid)?;
    let sid = unsafe { (*raw.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>()).TokenAppContainer };
    let facts = token_facts(process, Some(sid))?;
    let mut member = 0;
    check(
        unsafe { IsProcessInJob(process, null_mut(), &mut member) },
        "bp3-console-job",
    )?;
    Ok(member != 0
        && facts["isAppContainer"] == true
        && facts["capabilityCount"] == 0
        && facts["expectedIdentity"] == true)
}
fn debug_continue(event: &DEBUG_EVENT) -> Result<()> {
    let status = if event.dwDebugEventCode == EXCEPTION_DEBUG_EVENT
        && unsafe { event.u.Exception.ExceptionRecord.ExceptionCode } != EXCEPTION_BREAKPOINT
    {
        DBG_EXCEPTION_NOT_HANDLED
    } else {
        DBG_CONTINUE
    };
    check(
        unsafe { ContinueDebugEvent(event.dwProcessId, event.dwThreadId, status) },
        "bp3-debug-continue",
    )
}
fn close_debug_file(event: &DEBUG_EVENT) {
    let file = match event.dwDebugEventCode {
        CREATE_PROCESS_DEBUG_EVENT => unsafe { event.u.CreateProcessInfo.hFile },
        LOAD_DLL_DEBUG_EVENT => unsafe { event.u.LoadDll.hFile },
        _ => null_mut(),
    };
    if !file.is_null() && file != INVALID_HANDLE_VALUE {
        unsafe {
            CloseHandle(file);
        }
    }
    // Event process/thread handles belong to Windows until the corresponding exit Continue.
}
struct Process(Handle, Handle);
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0.0, 3);
        }
    }
}

#[cfg(test)]
fn validate_reports(
    reports: &[Report],
    nonce: [u8; 32],
    root: Identity,
    child: Option<Identity>,
    mode: Mode,
    rejected: bool,
) -> Result<()> {
    if rejected {
        return if reports.is_empty() {
            Ok(())
        } else {
            Err(invalid("bp3-denied-report"))
        };
    }
    if reports.len() > 6 || reports.first().map(|r| r.kind) != Some(Kind::RootStarted) {
        return Err(invalid("bp3-report-order"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for r in reports {
        if r.nonce != nonce
            || !seen.insert(r.kind as u8)
            || r.process.pid == 0
            || r.process.created == 0
        {
            return Err(invalid("bp3-report-context"));
        }
        let leaf = r.kind == Kind::LeafMarker;
        if (leaf && Some(r.process) != child) || (!leaf && r.process != root) {
            return Err(invalid("bp3-report-actor"));
        }
        if r.kind != Kind::ChildFailed && r.code != 0 {
            return Err(invalid("bp3-report-code"));
        }
        match r.kind {
            Kind::RootStarted | Kind::LeafMarker if r.child.is_none() => {}
            Kind::ChildFailed if child.is_none() && r.child.is_none() && r.code != 0 => {}
            Kind::ChildCreated | Kind::RootFinished if child.is_some() && r.child == child => {}
            Kind::InnerCreate
                if mode == Mode::NestedDebug && child.is_some() && r.child == child => {}
            Kind::InnerConsole
                if mode == Mode::NestedDebug
                    && r.child.is_some_and(|id| {
                        id.pid > 0 && id.created > 0 && id != root && Some(id) != child
                    }) => {}
            _ => return Err(invalid("bp3-report-shape")),
        }
    }
    let expected = if child.is_some() {
        Kind::RootFinished
    } else {
        Kind::ChildFailed
    };
    if reports.last().map(|r| r.kind) != Some(expected) {
        return Err(invalid("bp3-report-order"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bp3_debug_frames_reject_truncation_extra_fields_and_oversize() {
        let boot = Boot {
            purpose: PURPOSE.into(),
            nonce: [1; 32],
            parent: 42,
            mode: Mode::Inherit,
        };
        let wire = encode(&boot).unwrap();
        let decoded: Boot = decode(&wire).unwrap();
        assert_eq!(decoded.mode, Mode::Inherit);
        for n in 0..wire.len() {
            assert!(decode::<Boot>(&wire[..n]).is_err());
        }
        let mut extra = wire.clone();
        extra.push(0);
        assert!(decode::<Boot>(&extra).is_err());
        assert!(decode::<Boot>(&((LIMIT as u32 + 1).to_le_bytes())).is_err());
        let mut value = serde_json::to_value(boot).unwrap();
        value["grant"] = json!(true);
        assert!(decode::<Boot>(&encode(&value).unwrap()).is_err());
        value.as_object_mut().unwrap().remove("grant");
        value["mode"] = json!("Runtime");
        assert!(decode::<Boot>(&encode(&value).unwrap()).is_err());
    }
    #[test]
    fn bp3_debug_direct_worker_does_not_accept_cli_as_authority() {
        assert!(worker(&[]).is_err());
        assert!(worker(&["1".into(), "2".into(), "3".into()]).is_err());
    }
    #[test]
    fn bp3_debug_reports_cannot_relabel_generation_or_add_a_second_marker() {
        let root = Identity {
            pid: 42,
            created: 100,
        };
        let child = Identity {
            pid: 43,
            created: 101,
        };
        let nonce = [1; 32];
        let mut reports = vec![
            Report {
                nonce,
                kind: Kind::RootStarted,
                process: root,
                child: None,
                code: 0,
            },
            Report {
                nonce,
                kind: Kind::ChildCreated,
                process: root,
                child: Some(child),
                code: 0,
            },
            Report {
                nonce,
                kind: Kind::LeafMarker,
                process: child,
                child: None,
                code: 0,
            },
            Report {
                nonce,
                kind: Kind::RootFinished,
                process: root,
                child: Some(child),
                code: 0,
            },
        ];
        assert!(validate_reports(&reports, nonce, root, Some(child), Mode::Inherit, false).is_ok());
        reports[2].process.created += 1;
        assert!(
            validate_reports(&reports, nonce, root, Some(child), Mode::Inherit, false).is_err()
        );
        reports[2].process = child;
        reports[2].nonce = [2; 32];
        assert!(
            validate_reports(&reports, nonce, root, Some(child), Mode::Inherit, false).is_err()
        );
        reports[2].nonce = nonce;
        let extra =
            serde_json::from_slice::<Report>(&serde_json::to_vec(&reports[2]).unwrap()).unwrap();
        reports.insert(3, extra);
        assert!(
            validate_reports(&reports, nonce, root, Some(child), Mode::Inherit, false).is_err()
        );
        assert!(validate_reports(&reports, nonce, root, Some(child), Mode::Inherit, true).is_err());
    }
}
