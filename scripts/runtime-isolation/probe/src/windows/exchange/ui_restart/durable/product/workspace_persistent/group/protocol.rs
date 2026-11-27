//! Bounded synthetic-group MAC frames; shared group key is not a proof of writer PID.
use super::*;
use windows_sys::Win32::System::Pipes::{CreatePipe, GetNamedPipeServerProcessId, PeekNamedPipe};
const DOMAIN: &[u8] = b"KRAQEL-C3-SYNTHETIC-GROUP\0\x01";
const LIMIT: usize = 8192;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Grant {
    pub version: u16,
    pub purpose: String,
    pub role: String,
    pub parent_pid: u32,
    pub guardian: Identity,
    pub launch: crate::product_start_p2c::LaunchIdentity,
    pub epoch: Uuid,
    pub nonce: [u8; 32],
    pub workspace: Uuid,
    pub root: PathBuf,
    pub profile_name: String,
    pub code: Code,
    pub binding_sha256: [u8; 32],
    pub data_sha256: [u8; 32],
    pub bad_leaf: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Message {
    pub version: u16,
    pub sequence: u8,
    pub role: String,
    pub grant_sha256: [u8; 32],
    pub process: Identity,
    pub thread_id: u32,
}
pub(super) fn grant_digest(grant: &Grant) -> Result<[u8; 32]> {
    Ok(Sha256::digest(serde_json::to_vec(grant).map_err(|_| invalid("c3-grant-encode"))?).into())
}
pub(super) fn encode<T: Serialize>(value: &T, key: &[u8; 32]) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid("c3-frame-json"))?;
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err(invalid("c3-frame-size"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("c3-frame-key"))?;
    mac.update(DOMAIN);
    mac.update(&bytes);
    let mut out = (bytes.len() as u32).to_le_bytes().to_vec();
    out.extend(mac.finalize().into_bytes());
    out.extend(bytes);
    Ok(out)
}
pub(super) fn decode<T: serde::de::DeserializeOwned>(wire: &[u8], key: &[u8; 32]) -> Result<T> {
    if !(37..=LIMIT + 36).contains(&wire.len()) {
        return Err(invalid("c3-frame-size"));
    }
    let size = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    if size == 0 || size > LIMIT || size + 36 != wire.len() {
        return Err(invalid("c3-frame-length"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("c3-frame-key"))?;
    mac.update(DOMAIN);
    mac.update(&wire[36..]);
    mac.verify_slice(&wire[4..36])
        .map_err(|_| invalid("c3-frame-mac"))?;
    serde_json::from_slice(&wire[36..]).map_err(|_| invalid("c3-frame-json"))
}
pub(super) fn pipe() -> Result<(fs::File, fs::File)> {
    let (mut r, mut w) = (null_mut(), null_mut());
    check(
        unsafe { CreatePipe(&mut r, &mut w, null(), 16384) },
        "c3-pipe",
    )?;
    Ok(unsafe { (fs::File::from_raw_handle(r), fs::File::from_raw_handle(w)) })
}
pub(super) fn prepaid(bytes: &[u8]) -> Result<fs::File> {
    let (r, mut w) = pipe()?;
    io("c3-prepaid-write", w.write_all(bytes))?;
    drop(w);
    Ok(r)
}
pub(super) fn read_frame(file: &mut fs::File, deadline: Instant) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut goal = 4;
    while out.len() < goal {
        if Instant::now() >= deadline {
            return Err(invalid("c3-frame-deadline"));
        }
        let mut available = 0;
        if unsafe {
            PeekNamedPipe(
                file.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        } == 0
        {
            return Err(fail("c3-frame-peek"));
        }
        if available == 0 {
            thread::sleep(Duration::from_millis(5));
            continue;
        }
        let n = (available as usize).min(goal - out.len());
        let mut bytes = vec![0; n];
        let n = io("c3-frame-read", file.read(&mut bytes))?;
        if n == 0 {
            return Err(invalid("c3-frame-eof"));
        }
        out.extend(&bytes[..n]);
        if out.len() == 4 {
            let size = u32::from_le_bytes(out[..4].try_into().unwrap()) as usize;
            if size == 0 || size > LIMIT {
                return Err(invalid("c3-frame-length"));
            }
            goal = size + 36;
        }
    }
    Ok(out)
}
pub(super) fn peer(file: &fs::File) -> Result<u32> {
    let mut pid = 0;
    check(
        unsafe { GetNamedPipeServerProcessId(file.as_raw_handle(), &mut pid) },
        "c3-pipe-peer",
    )?;
    Ok(pid)
}
pub(super) fn inherited(handles: &[HANDLE], yes: bool) -> Result<()> {
    for h in handles {
        check(
            unsafe {
                SetHandleInformation(
                    *h,
                    HANDLE_FLAG_INHERIT,
                    if yes { HANDLE_FLAG_INHERIT } else { 0 },
                )
            },
            "c3-handle-inherit",
        )?;
    }
    Ok(())
}
pub(super) fn message(
    grant: &Grant,
    role: &str,
    sequence: u8,
    process: Identity,
    thread_id: u32,
) -> Result<Message> {
    Ok(Message {
        version: 1,
        sequence,
        role: role.into(),
        grant_sha256: grant_digest(grant)?,
        process,
        thread_id,
    })
}
pub(super) fn verify_message(m: &Message, grant: &Grant, role: &str, sequence: u8) -> Result<()> {
    if m.version != 1
        || m.role != role
        || m.sequence != sequence
        || m.grant_sha256 != grant_digest(grant)?
        || !m.process.valid()
    {
        return Err(invalid("c3-message-binding"));
    }
    Ok(())
}
