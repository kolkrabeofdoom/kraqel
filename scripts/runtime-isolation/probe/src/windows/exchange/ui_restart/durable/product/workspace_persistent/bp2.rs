//! BP2 synthetic leaf capability probe. No UI, database, runtime or product authority.
use super::*;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use windows_sys::Win32::System::Pipes::{CreatePipe, GetNamedPipeServerProcessId, PeekNamedPipe};
use zeroize::{Zeroize, Zeroizing};
mod controller;
mod policy;
#[cfg(test)]
pub(super) use controller::{Mode, run};

const DOMAIN: &[u8] = b"KRAQEL-BP2-LEAF\0\x01";
const LIMIT: usize = 8192;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Grant {
    version: u16,
    purpose: String,
    launch: Uuid,
    workspace: Uuid,
    guardian: Identity,
    nonce: [u8; 32],
    package: [u8; 32],
    code: [u8; 32],
    executable: PathBuf,
    profile: String,
    required: u64,
    child_policy: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Outcome {
    version: u16,
    grant: [u8; 32],
    process: Identity,
    image_policy: u32,
    child_policy: u32,
    child_error: u32,
    child_created: bool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct Identity {
    pub pid: u32,
    pub created: u64,
}
fn identity(p: HANDLE) -> Result<Identity> {
    let (mut c, mut e, mut k, mut u) = unsafe { zeroed() };
    check(
        unsafe { GetProcessTimes(p, &mut c, &mut e, &mut k, &mut u) },
        "c4-root-identity",
    )?;
    let id = Identity {
        pid: unsafe { GetProcessId(p) },
        created: (u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime),
    };
    if id.pid == 0 || id.created == 0 {
        return Err(invalid("c4-root-identity"));
    }
    Ok(id)
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(in super::super) enum Stage {
    Commit,
    RootBound,
    Accepted,
    Running,
    Drained,
    Finish,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct Bp2Event {
    pub version: u16,
    pub stage: Stage,
    pub launch: Uuid,
    pub workspace: Uuid,
    pub package: [u8; 32],
    pub guardian: Identity,
    pub nonce: [u8; 32],
    pub root: Option<Identity>,
    pub receipt: [u8; 32],
}
#[derive(Clone, Default)]
pub(in super::super) struct Bp2State {
    used: std::collections::BTreeSet<Uuid>,
    active: Option<Bp2Event>,
}
impl Bp2State {
    pub(in super::super) fn pending(&self) -> bool {
        self.active.is_some()
    }
    #[cfg(test)]
    pub(in super::super) fn used_count_for_test(&self) -> usize {
        self.used.len()
    }
}
pub(in super::super) fn apply_bp2(state: &mut State, actor: [u8; 32], e: &Bp2Event) -> Result<()> {
    if state.schema != 6
        || actor != state.actor
        || state.pending.is_some()
        || state.session.is_some()
        || state.workspace.is_some()
        || state.probe.is_some()
        || state.c3.pending()
        || state.c4_probe.pending()
        || e.version != 1
        || e.launch.is_nil()
        || e.nonce == [0; 32]
        || e.package == [0; 32]
        || e.guardian.pid == 0
        || e.guardian.created == 0
    {
        return Err(invalid("bp2-record-record-context"));
    }
    let ws = state
        .c2
        .as_ref()
        .ok_or(invalid("bp2-record-record-workspace"))?;
    if ws.id != e.workspace
        || ws.retiring
        || ws.retired
        || !ws.profile_ready
        || ws.binding.is_none()
    {
        return Err(invalid("bp2-record-record-workspace"));
    }
    if e.stage == Stage::Commit {
        if state.bp2_probe.active.is_some()
            || state.bp2_probe.used.contains(&e.launch)
            || e.root.is_some()
            || e.receipt != [0; 32]
        {
            return Err(invalid("bp2-record-commit"));
        }
        state.bp2_probe.used.insert(e.launch);
        state.bp2_probe.active = Some(e.clone());
        return Ok(());
    }
    let previous = state
        .bp2_probe
        .active
        .as_ref()
        .ok_or(invalid("bp2-record-record-order"))?;
    let mut expected = previous.clone();
    expected.stage = e.stage;
    expected.root = e.root;
    expected.receipt = e.receipt;
    if &expected != e {
        return Err(invalid("bp2-record-record-binding"));
    }
    let valid = match (previous.stage, e.stage) {
        (Stage::Commit, Stage::RootBound) => {
            e.root.is_some_and(|r| r.pid > 0 && r.created > 0) && e.receipt == [0; 32]
        }
        (Stage::RootBound, Stage::Accepted) | (Stage::Accepted, Stage::Running) => {
            e.root == previous.root && e.receipt == [0; 32]
        }
        (Stage::Commit | Stage::RootBound | Stage::Accepted | Stage::Running, Stage::Drained) => {
            e.root == previous.root && e.receipt != [0; 32]
        }
        (Stage::Drained, Stage::Finish) => e.root == previous.root && e.receipt == previous.receipt,
        _ => false,
    };
    if !valid {
        return Err(invalid("bp2-record-record-order"));
    }
    state.bp2_probe.active = if e.stage == Stage::Finish {
        None
    } else {
        Some(e.clone())
    };
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Ack {
    version: u16,
    sequence: u8,
    grant: [u8; 32],
    process: Identity,
}
fn grant_hash(g: &Grant) -> Result<[u8; 32]> {
    Ok(Sha256::digest(serde_json::to_vec(g).map_err(|_| invalid("bp2-grant-json"))?).into())
}
fn encode<T: Serialize>(v: &T, key: &[u8]) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(v).map_err(|_| invalid("bp2-frame-json"))?;
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err(invalid("bp2-frame-bound"));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| invalid("bp2-frame-key"))?;
    mac.update(DOMAIN);
    mac.update(&bytes);
    let mut out = (bytes.len() as u32).to_le_bytes().to_vec();
    out.extend(mac.finalize().into_bytes());
    out.extend(bytes);
    Ok(out)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8], key: &[u8]) -> Result<T> {
    if !(37..=LIMIT + 36).contains(&bytes.len())
        || u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize != bytes.len() - 36
    {
        return Err(invalid("bp2-frame-bound"));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| invalid("bp2-frame-key"))?;
    mac.update(DOMAIN);
    mac.update(&bytes[36..]);
    mac.verify_slice(&bytes[4..36])
        .map_err(|_| invalid("bp2-frame-mac"))?;
    serde_json::from_slice(&bytes[36..]).map_err(|_| invalid("bp2-frame-json"))
}
fn pipe() -> Result<(fs::File, fs::File)> {
    let (mut r, mut w) = (null_mut(), null_mut());
    check(
        unsafe { CreatePipe(&mut r, &mut w, null(), 16384) },
        "bp2-pipe",
    )?;
    Ok(unsafe { (fs::File::from_raw_handle(r), fs::File::from_raw_handle(w)) })
}
fn prepaid(bytes: &[u8]) -> Result<fs::File> {
    let (r, mut w) = pipe()?;
    io("bp2-prepaid", w.write_all(bytes))?;
    drop(w);
    Ok(r)
}
fn inherit(handles: &[HANDLE], yes: bool) -> Result<()> {
    for h in handles {
        check(
            unsafe {
                SetHandleInformation(
                    *h,
                    HANDLE_FLAG_INHERIT,
                    if yes { HANDLE_FLAG_INHERIT } else { 0 },
                )
            },
            "bp2-handle-inherit",
        )?;
        let mut flags = 0;
        check(
            unsafe { GetHandleInformation(*h, &mut flags) },
            "bp2-handle-flags",
        )?;
        if (flags & HANDLE_FLAG_INHERIT != 0) != yes {
            return Err(invalid("bp2-handle-flags"));
        }
    }
    Ok(())
}
fn peer(f: &fs::File) -> Result<u32> {
    let mut pid = 0;
    check(
        unsafe { GetNamedPipeServerProcessId(f.as_raw_handle(), &mut pid) },
        "bp2-pipe-peer",
    )?;
    Ok(pid)
}
fn receive(f: &mut fs::File, deadline: Instant) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut goal = 4;
    while out.len() < goal {
        if Instant::now() >= deadline {
            return Err(invalid("bp2-frame-timeout"));
        }
        let mut count = 0;
        check(
            unsafe {
                PeekNamedPipe(
                    f.as_raw_handle(),
                    null_mut(),
                    0,
                    null_mut(),
                    &mut count,
                    null_mut(),
                )
            },
            "bp2-frame-peek",
        )?;
        if count == 0 {
            thread::sleep(Duration::from_millis(5));
            continue;
        }
        let mut part = vec![0; (count as usize).min(goal - out.len())];
        let n = io("bp2-frame-read", f.read(&mut part))?;
        if n == 0 {
            return Err(invalid("bp2-frame-eof"));
        }
        out.extend(&part[..n]);
        if out.len() == 4 {
            let n = u32::from_le_bytes(out[..4].try_into().unwrap()) as usize;
            if n == 0 || n > LIMIT {
                return Err(invalid("bp2-frame-bound"));
            }
            goal = n + 36;
        }
    }
    Ok(out)
}

pub(in crate::windows) fn worker(args: &[OsString]) -> Result<()> {
    if args.len() != 4 {
        return Err(invalid("bp2-worker-arguments"));
    }
    let mut raw = Vec::new();
    for arg in args {
        let n = arg
            .to_str()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n > 0 && *n != usize::MAX)
            .ok_or(invalid("bp2-worker-handle"))?;
        if raw.contains(&(n as HANDLE)) {
            return Err(invalid("bp2-worker-handle"));
        }
        raw.push(n as HANDLE);
    }
    inherit(&raw, false)?;
    let mut boot = unsafe { fs::File::from_raw_handle(raw[0]) };
    let mut secret = unsafe { fs::File::from_raw_handle(raw[1]) };
    let mut events = unsafe { fs::File::from_raw_handle(raw[2]) };
    let mut go = unsafe { fs::File::from_raw_handle(raw[3]) };
    let peers = [peer(&boot)?, peer(&secret)?, peer(&go)?];
    let mut key = Zeroizing::new(Vec::new());
    io(
        "bp2-worker-secret",
        (&mut secret).take(33).read_to_end(&mut key),
    )?;
    if key.len() != 32 {
        return Err(invalid("bp2-worker-key"));
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    let grant: Grant = decode(&receive(&mut boot, deadline)?, &key)?;
    drop(boot);
    drop(secret);
    if grant.version != 1
        || grant.purpose != "bp2-synthetic-leaf-capability"
        || grant.launch.is_nil()
        || grant.workspace.is_nil()
        || grant.nonce == [0; 32]
        || grant.package == [0; 32]
        || grant.guardian.pid == unsafe { GetCurrentProcessId() }
        || grant.guardian.created == 0
        || peers != [grant.guardian.pid; 3]
        || grant.profile != name(grant.workspace)
        || grant.required != policy::REQUIRED
        || grant.child_policy != policy::DENY_CHILDREN
    {
        return Err(invalid("bp2-worker-binding"));
    }
    if hash(&io("bp2-worker-exe", std::env::current_exe())?)? != grant.code
        || hash(&grant.executable)? != grant.code
        || grant.executable.file_name() != Some(std::ffi::OsStr::new("leaf.exe"))
    {
        return Err(invalid("bp2-worker-image"));
    }
    let p = profile(grant.workspace)?;
    let facts = token_facts(unsafe { GetCurrentProcess() }, Some(p.sid))?;
    let mut assigned = 0;
    check(
        unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut assigned) },
        "bp2-worker-job",
    )?;
    if assigned == 0
        || facts["isAppContainer"] != true
        || facts["capabilityCount"] != 0
        || facts["expectedIdentity"] != true
    {
        return Err(invalid("bp2-worker-token"));
    }
    policy::require_supported(policy::REQUIRED, policy::support()?)?;
    let actual = policy::readback(unsafe { GetCurrentProcess() })?;
    policy::require_readback(actual, false)?;
    let me = identity(unsafe { GetCurrentProcess() })?;
    let digest = grant_hash(&grant)?;
    io(
        "bp2-worker-accepted",
        events.write_all(&encode(
            &Ack {
                version: 1,
                sequence: 1,
                grant: digest,
                process: me,
            },
            &key,
        )?),
    )?;
    let ack: Ack = decode(&receive(&mut go, deadline)?, &key)?;
    if ack.version != 1 || ack.sequence != 2 || ack.grant != digest || ack.process != grant.guardian
    {
        return Err(invalid("bp2-worker-continue"));
    }
    drop(go);
    inherit(&[events.as_raw_handle()], false)?;
    let mut si: STARTUPINFOW = unsafe { zeroed() };
    si.cb = size_of_val(&si) as u32;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let mut cmd = wide(format!(
        "\"{}\" bp2-must-not-start",
        grant.executable.display()
    ));
    let env = crate::windows::exchange::ui_restart::environment(
        grant
            .executable
            .parent()
            .ok_or(invalid("bp2-worker-parent"))?,
    );
    let ok = unsafe {
        CreateProcessW(
            wide(&grant.executable).as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            0,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS,
            env.as_ptr().cast(),
            wide(grant.executable.parent().unwrap()).as_ptr(),
            &si,
            &mut pi,
        )
    };
    let child_error = if ok == 0 {
        unsafe { GetLastError() }
    } else {
        0
    };
    if ok != 0 {
        let child = Handle(pi.hProcess);
        let _thread = Handle(pi.hThread);
        check(
            unsafe { TerminateProcess(child.0, 3) },
            "bp2-unexpected-child-stop",
        )?;
        if unsafe { WaitForSingleObject(child.0, 5000) } != WAIT_OBJECT_0 {
            return Err(invalid("bp2-unexpected-child-end"));
        }
    }
    io(
        "bp2-worker-outcome",
        events.write_all(&encode(
            &Outcome {
                version: 1,
                grant: digest,
                process: me,
                image_policy: actual.0,
                child_policy: actual.1,
                child_error,
                child_created: ok != 0,
            },
            &key,
        )?),
    )?;
    key.zeroize();
    drop(key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bp2_frames_are_separate_bounded_and_tamper_evident() {
        let a = Ack {
            version: 1,
            sequence: 1,
            grant: [1; 32],
            process: Identity {
                pid: 42,
                created: 1,
            },
        };
        let wire = encode(&a, &[2; 32]).unwrap();
        let _: Ack = decode(&wire, &[2; 32]).unwrap();
        assert!(decode::<Ack>(&wire, &[3; 32]).is_err());
        for n in [0, 4, 36] {
            let mut bad = wire.clone();
            bad[n] ^= 1;
            assert!(decode::<Ack>(&bad, &[2; 32]).is_err());
        }
        let mut bad = wire;
        bad.push(0);
        assert!(decode::<Ack>(&bad, &[2; 32]).is_err());
    }
}
