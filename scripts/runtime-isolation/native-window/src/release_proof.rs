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
        self.release_with(Trial::StoreRead, || Ok(()))
    }

    fn release_with(
        &self,
        trial: Trial,
        after_consumption: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<bool> {
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
        after_consumption()?;
        run(trial)?;
        require(receipts_absent()?, "reader grants remain")?;
        Ok(true)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ProofCase {
    Complete,
    MissingEnd,
    WitnessLoss,
    LossAfterConsumption,
    LiveLoss,
    BeforeCleanup,
}

pub(super) fn exercise(anchor: &[u8], case: ProofCase) -> io::Result<()> {
    let Some(mut lock) = begin(false, anchor)? else {
        println!("fresh witness blocked before store preparation");
        std::process::exit(102);
    };
    let incomplete = case == ProofCase::MissingEnd;
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
    if case == ProofCase::BeforeCleanup {
        witness.release_with(Trial::ReadCrashBeforeCleanup, || Ok(()))?;
        return Err(io::Error::other("pre-cleanup crash missing"));
    }
    if case == ProofCase::LiveLoss {
        witness.release_with(Trial::HeldRead, || Ok(()))?;
        return Err(io::Error::other("live witness crash missing"));
    }
    if case == ProofCase::LossAfterConsumption {
        witness.release_with(Trial::StoreRead, || {
            unsafe {
                check(TerminateProcess(GetCurrentProcess(), 104))?;
            }
            Err(io::Error::other("post-consumption termination returned"))
        })?;
        return Err(io::Error::other("post-consumption crash missing"));
    }
    if case == ProofCase::WitnessLoss {
        let mut lock = existing_lock()?;
        require(
            matches_record(&mut lock, END, &witness.id)? && receipts_absent()?,
            "end not ready",
        )?;
        // No call to release: the generation and the store pins die with this process.
        unsafe {
            check(TerminateProcess(GetCurrentProcess(), 103))?;
        }
        return Err(io::Error::other("witness termination returned"));
    }
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

// Keep process handles, never job handles: the witness must remain the last job owner.
pub(super) fn observe_live_loss() -> io::Result<()> {
    use bootstrap::{Purpose, Role};
    use std::time::{Duration, Instant};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let mut witness = Command::new(std::env::current_exe()?)
        .arg("proof-live-witness")
        .spawn()?;
    let result = (|| unsafe {
        let deadline = Instant::now() + Duration::from_secs(25);
        let window_pid = loop {
            require(
                witness.try_wait()?.is_none() && Instant::now() < deadline,
                "live read deadline",
            )?;
            let hwnd = FindWindowW(
                wide("KRAQEL.Synthetic.Edit").as_ptr(),
                wide(LIVE_READ_TITLE).as_ptr(),
            );
            let mut pid = 0;
            if !hwnd.is_null()
                && IsWindowVisible(hwnd) != 0
                && GetWindowThreadProcessId(hwnd, &mut pid) != 0
            {
                break pid;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let targets = std::fs::read_to_string(Path::new(DIR).join("pipe-targets.txt"))?;
        let ids = targets
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| io::Error::other("invalid target"))?;
        require(
            ids.len() == 2 && ids[0] == window_pid && ids[0] != ids[1],
            "reader identities",
        )?;
        let open = |id| -> io::Result<Handle> {
            let handle = Handle(OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                id,
            ));
            require(!handle.0.is_null(), "observe reader")?;
            Ok(handle)
        };
        let ui = open(ids[0])?;
        let data = open(ids[1])?;
        let (u, d) = (ticket(ui.0)?, ticket(data.0)?);
        for role in [Role::Window, Role::Data] {
            let record = std::fs::read_to_string(role.receipt())?;
            let section = record
                .split_once('\n')
                .and_then(|(_, s)| s.parse().ok())
                .ok_or(io::Error::other("reader receipt"))?;
            require(
                record == bootstrap::receipt(Purpose::HeldRead, role, &u, &d, section),
                "reader generation",
            )?;
        }
        require(
            WaitForSingleObject(ui.0, 0) == WAIT_TIMEOUT
                && WaitForSingleObject(data.0, 0) == WAIT_TIMEOUT,
            "read pair inactive",
        )?;
        check(TerminateProcess(witness.as_raw_handle(), 105))?;
        require(
            witness.wait()?.code() == Some(105),
            "live witness termination",
        )?;
        require(
            WaitForMultipleObjects(2, [ui.0, data.0].as_ptr(), 1, 5000) == WAIT_OBJECT_0,
            "read roles survived",
        )?;
        println!(
            "Live authenticated read pair observed; witness terminated; both original processes ended"
        );
        Ok(())
    })();
    if result.is_err() {
        let _ = witness.kill();
        let _ = witness.wait();
    }
    result
}
