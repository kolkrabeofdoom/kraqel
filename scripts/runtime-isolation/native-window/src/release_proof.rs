//! Guest-only release for a fresh v1 store. The live witness is the only caller of release().
use super::*;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    process::{Command, Stdio},
    sync::Mutex,
};
use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use zeroize::Zeroizing;

const INTENT: &[u8] = b"BP4b/proof-v1/intent";
const RUNNING: &[u8] = b"BP4b/proof-v1/running";
const END: &[u8] = b"BP4b/proof-v1/committed-revision-1/jobs-drained";
const SPENT: &[u8] = b"BP4b/proof-v1/read-consumed";

fn existing_lock() -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(Path::new(DIR).join("pipe.lock"))
}

fn matches_record(f: &mut File, state: &[u8], context: &[u8]) -> io::Result<bool> {
    let expected = [state, context].concat();
    if f.metadata()?.len() != expected.len() as u64 {
        return Ok(false);
    }
    let mut bytes = vec![0; expected.len()];
    f.read_exact(&mut bytes)?;
    Ok(bytes == expected)
}

fn context(anchor: &[u8], nonce: &[u8; 32], pins: &(File, File)) -> io::Result<[u8; 32]> {
    let mut hash = Sha256::new();
    hash.update(b"BP4b/proof-v1/A/store-v1/commit-1");
    hash.update(anchor);
    hash.update(nonce);
    let mut session = 0;
    unsafe {
        check(ProcessIdToSessionId(GetCurrentProcessId(), &mut session))?;
    }
    hash.update(session.to_le_bytes());
    for f in [&pins.0, &pins.1] {
        let mut info = BY_HANDLE_FILE_INFORMATION::default();
        unsafe {
            check(GetFileInformationByHandle(f.as_raw_handle(), &mut info))?;
        }
        require(
            info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0,
            "unexpected redirect",
        )?;
        hash.update(info.dwVolumeSerialNumber.to_le_bytes());
        hash.update(info.nFileIndexHigh.to_le_bytes());
        hash.update(info.nFileIndexLow.to_le_bytes());
    }
    Ok(hash.finalize().into())
}

pub(super) fn issue(anchor: &[u8], incomplete: bool) -> io::Result<()> {
    let mut nonce = Zeroizing::new([0u8; 32]);
    std::io::stdin().read_exact(&mut *nonce)?;
    let mut lock = existing_lock()?;
    // Check authorization before even pinning store metadata.
    let intent = Sha256::new()
        .chain_update(anchor)
        .chain_update(&nonce[..])
        .finalize();
    require(
        matches_record(&mut lock, INTENT, &intent)? && receipts_absent()?,
        "issuer intent",
    )?;
    let pins = scene_store::pin_existing_v1()?;
    let id = context(anchor, &nonce, &pins)?;
    record(&mut lock, RUNNING, &id)?;
    run(Trial::StoreEdit)?;
    require(receipts_absent()?, "old grants remain")?;
    if !incomplete {
        record(&mut lock, END, &id)?;
    }
    unsafe {
        check(TerminateProcess(GetCurrentProcess(), 101))?;
    }
    Err(io::Error::other("issuer termination returned"))
}

struct Witness {
    id: [u8; 32],
    spent: Mutex<bool>,
}

impl Witness {
    fn release(&self) -> io::Result<bool> {
        let mut spent = self
            .spent
            .lock()
            .map_err(|_| io::Error::other("witness unavailable"))?;
        if *spent {
            return Ok(false);
        }
        let mut lock = existing_lock()?;
        if !matches_record(&mut lock, END, &self.id)? || !receipts_absent()? {
            return Ok(false);
        }
        *spent = true;
        record(&mut lock, SPENT, &self.id)?;
        run(Trial::StoreRead)?;
        require(receipts_absent()?, "reader grants remain")?;
        Ok(true)
    }
}

pub(super) fn exercise(anchor: &[u8], incomplete: bool) -> io::Result<()> {
    let mut lock = begin(false, anchor)?.ok_or(io::Error::other("existing store blocked"))?;
    let pins = scene_store::prepare(None)?;
    let mut nonce = Zeroizing::new([0u8; 32]);
    getrandom::fill(&mut *nonce).map_err(|_| io::Error::other("generation unavailable"))?;
    let witness = Witness {
        id: context(anchor, &nonce, &pins)?,
        spent: Mutex::new(false),
    };
    let intent = Sha256::new()
        .chain_update(anchor)
        .chain_update(&nonce[..])
        .finalize();
    record(&mut lock, INTENT, &intent)?;
    drop(lock);
    let mut child = Command::new(std::env::current_exe()?)
        .arg(if incomplete {
            "proof-issuer-incomplete"
        } else {
            "proof-issuer"
        })
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or(io::Error::other("private bootstrap pipe"))?
        .write_all(&nonce[..])?;
    drop(nonce);
    require(child.wait()?.code() == Some(101), "issuer crash missing")?;
    if incomplete {
        require(!witness.release()?, "incomplete release")?;
        println!("No durable end: read blocked");
    } else {
        let mut foreign_nonce = Zeroizing::new([0u8; 32]);
        getrandom::fill(&mut *foreign_nonce)
            .map_err(|_| io::Error::other("generation unavailable"))?;
        let foreign = Witness {
            id: context(anchor, &foreign_nonce, &pins)?,
            spent: Mutex::new(false),
        };
        require(!foreign.release()?, "foreign generation accepted")?;
        let outcomes = std::thread::scope(|scope| -> io::Result<[bool; 2]> {
            let first = scope.spawn(|| witness.release());
            let second = witness.release()?;
            Ok([
                first
                    .join()
                    .map_err(|_| io::Error::other("reader panic"))??,
                second,
            ])
        })?;
        require(
            outcomes[0] != outcomes[1] && !witness.release()?,
            "single use",
        )?;
        println!(
            "Foreign generation blocked; one read confirmed revision 1; concurrent reuse and replay blocked"
        );
    }
    Ok(())
}
