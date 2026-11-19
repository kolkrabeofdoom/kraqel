//! Pre-resume check for our own two suspended fixture generations, in the BP3 guest only.
#[path = "../package.rs"]
mod package;
#[path = "../release_proof.rs"]
mod release_proof;
use kraqel_window_probe::{
    suspended::{Suspended, suspended},
    *,
};
use std::{
    collections::BTreeMap,
    io,
    mem::{size_of, zeroed},
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, Isolation::*, *},
    Storage::FileSystem::*,
    System::{JobObjects::*, Pipes::*, Threading::*},
};

struct Profile {
    name: Vec<u16>,
    sid: PSID,
}
impl Profile {
    fn fresh(name: &str) -> io::Result<Self> {
        let name = wide(name);
        let mut sid = null_mut();
        require(
            unsafe {
                CreateAppContainerProfile(
                    name.as_ptr(),
                    name.as_ptr(),
                    name.as_ptr(),
                    null(),
                    0,
                    &mut sid,
                )
            } >= 0,
            "fresh profile",
        )?;
        Ok(Self { name, sid })
    }
}
impl Drop for Profile {
    fn drop(&mut self) {
        unsafe {
            FreeSid(self.sid);
            DeleteAppContainerProfile(self.name.as_ptr());
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Entry {
    handle: usize,
    refs: usize,
    pointers: usize,
    access: u32,
    kind: u32,
    attributes: u32,
    reserved: u32,
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtQueryInformationProcess(
        proc: HANDLE,
        class: u32,
        buf: *mut core::ffi::c_void,
        len: u32,
        returned: *mut u32,
    ) -> i32;
}
type Inventory = BTreeMap<usize, (u32, u32)>;
fn inventory(child: &Suspended) -> io::Result<Inventory> {
    // ProcessHandleInformation, scoped to the held suspended fixture. Fail closed on overflow.
    let mut buf = [0usize; 4096];
    let mut returned = 0;
    require(
        unsafe {
            NtQueryInformationProcess(
                child.proc.0,
                51,
                buf.as_mut_ptr().cast(),
                size_of_val(&buf) as u32,
                &mut returned,
            )
        } == 0,
        "handle inventory",
    )?;
    let header = 2 * size_of::<usize>();
    require(
        returned as usize <= size_of_val(&buf)
            && returned as usize >= header
            && buf[0] <= (returned as usize - header) / size_of::<Entry>(),
        "inventory bounds",
    )?;
    let entries =
        unsafe { std::slice::from_raw_parts(buf.as_ptr().add(2).cast::<Entry>(), buf[0]) };
    Ok(entries
        .iter()
        .map(|e| (e.handle, (e.access, e.attributes)))
        .collect())
}
fn pipe() -> io::Result<(Handle, Handle)> {
    unsafe {
        let mut sd = null_mut();
        check(ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide("D:P(A;;GA;;;SY)(A;;GA;;;BA)").as_ptr(),
            1,
            &mut sd,
            null_mut(),
        ))?;
        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd,
            bInheritHandle: 0,
        };
        let (mut read, mut write) = (null_mut(), null_mut());
        let result = check(CreatePipe(&mut read, &mut write, &sa, 0));
        LocalFree(sd);
        result?;
        Ok((Handle(read), Handle(write)))
    }
}
fn assign(
    child: &Suspended,
    source: &Handle,
    rights: u32,
    inventory: &mut Inventory,
) -> io::Result<usize> {
    unsafe {
        let mut remote = null_mut();
        check(DuplicateHandle(
            GetCurrentProcess(),
            source.0,
            child.proc.0,
            &mut remote,
            rights,
            0,
            0,
        ))?;
        let mut local = null_mut();
        check(DuplicateHandle(
            child.proc.0,
            remote,
            GetCurrentProcess(),
            &mut local,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        ))?;
        let copy = Handle(local);
        require(
            CompareObjectHandles(copy.0, source.0) != 0,
            "endpoint identity",
        )?;
        require(
            inventory.insert(remote as usize, (rights, 0)).is_none(),
            "unexpected existing handle",
        )?;
        Ok(remote as usize)
    }
}
fn check_pipe_acl(endpoint: &Handle, other: &Suspended) -> io::Result<()> {
    unsafe {
        let mut token = null_mut();
        check(OpenProcessToken(
            other.proc.0,
            TOKEN_DUPLICATE | TOKEN_QUERY,
            &mut token,
        ))?;
        let token = Handle(token);
        let mut client = null_mut();
        check(DuplicateToken(token.0, SecurityImpersonation, &mut client))?;
        let client = Handle(client);
        let mut sd = null_mut();
        let err = GetSecurityInfo(
            endpoint.0,
            SE_KERNEL_OBJECT,
            OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut sd,
        );
        if err != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(err as i32));
        }
        let result = (|| {
            let mapping = GENERIC_MAPPING {
                GenericRead: FILE_GENERIC_READ,
                GenericWrite: FILE_GENERIC_WRITE,
                GenericExecute: FILE_GENERIC_EXECUTE,
                GenericAll: FILE_ALL_ACCESS,
            };
            for right in [FILE_READ_DATA, FILE_WRITE_DATA, FILE_CREATE_PIPE_INSTANCE] {
                let mut privileges = [0u32; 128];
                let mut len = size_of_val(&privileges) as u32;
                let (mut granted, mut allowed) = (0, 0);
                check(AccessCheck(
                    sd,
                    client.0,
                    right,
                    &mapping,
                    privileges.as_mut_ptr().cast(),
                    &mut len,
                    &mut granted,
                    &mut allowed,
                ))?;
                require(
                    allowed == 0 && granted == 0,
                    "foreign pipe DACL grants access",
                )?;
            }
            Ok(())
        })();
        LocalFree(sd);
        result
    }
}
fn drain(child: &Suspended) -> io::Result<()> {
    unsafe {
        require(
            WaitForSingleObject(child.proc.0, 5000) == WAIT_OBJECT_0,
            "child end",
        )?;
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = zeroed();
        for _ in 0..100 {
            check(QueryInformationJobObject(
                child.job.0,
                JobObjectBasicAccountingInformation,
                (&raw mut info).cast(),
                size_of_val(&info) as u32,
                null_mut(),
            ))?;
            if info.ActiveProcesses == 0 && info.TotalProcesses == 1 {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    Err(io::Error::other("job drain"))
}
#[derive(Clone, Copy, PartialEq)]
enum CleanupPoint {
    BeforeCleanup,
    AfterWindowReceipt,
    AfterLastReceipt,
}
#[derive(Clone, Copy, PartialEq)]
enum Trial {
    Suspended,
    BadReady,
    ReadA,
    DeniedSave,
    StoreRead,
    HeldRead,
    ReadCrash(CleanupPoint),
    StoreEdit,
    StorePhase(scene_store::StorePhase),
    EditConflict,
    Valid,
}

struct Receipt(std::path::PathBuf);
impl Receipt {
    fn write(role: bootstrap::Role, content: &str) -> io::Result<Self> {
        use std::io::Write;
        let path = role.receipt();
        let mut file = std::fs::File::create_new(&path)?;
        let guard = Self(path);
        file.write_all(content.as_bytes())?;
        Ok(guard)
    }
}
impl Drop for Receipt {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn finish_pending_window(ui: &Suspended, data: &Suspended, crash_starter: bool) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let journal = Path::new(DIR)
        .join("scene-store-close")
        .join("synthetic.sqlite-journal");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    unsafe {
        loop {
            require(
                WaitForSingleObject(ui.proc.0, 0) == WAIT_TIMEOUT
                    && WaitForSingleObject(data.proc.0, 0) == WAIT_TIMEOUT,
                "expected live pair",
            )?;
            let hwnd = FindWindowW(
                wide("KRAQEL.Synthetic.Edit").as_ptr(),
                wide(CLOSE_PENDING_TITLE).as_ptr(),
            );
            let mut pid = 0;
            if !hwnd.is_null()
                && GetWindowThreadProcessId(hwnd, &mut pid) != 0
                && pid == GetProcessId(ui.proc.0)
                && IsWindowVisible(hwnd) != 0
                && journal.try_exists()?
            {
                if crash_starter {
                    check(TerminateProcess(GetCurrentProcess(), 101))?;
                    return Err(io::Error::other("starter termination returned"));
                }
                check(PostMessageW(hwnd, WM_CLOSE, 0, 0))?;
                require(
                    WaitForSingleObject(data.proc.0, 5000) == WAIT_OBJECT_0,
                    "service did not end on close",
                )?;
                return Ok(());
            }
            require(
                std::time::Instant::now() < deadline,
                "pending window and journal",
            )?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
fn run(trial: Trial) -> io::Result<()> {
    run_pair(trial, false)
}

fn run_pair(trial: Trial, crash_starter: bool) -> io::Result<()> {
    guest_image("pipe-check.exe")?;
    let ui_profile = Profile::fresh(PROFILE)?;
    let data_profile = Profile::fresh(
        if matches!(
            trial,
            Trial::StoreRead | Trial::HeldRead | Trial::ReadCrash(_)
        ) || matches!(trial,Trial::StorePhase(c) if c.reader())
        {
            scene_store::READER
        } else {
            "org.kraqel.bp4.data"
        },
    )?;
    let other_profile = Profile::fresh("org.kraqel.bp4.other")?;
    let ui = unsafe { suspended(ui_profile.sid, CHILD) }?;
    let data = unsafe { suspended(data_profile.sid, "pipe-service.exe") }?;
    let other = unsafe { suspended(other_profile.sid, "pipe-other.exe") }?;
    let mut ui_set = inventory(&ui)?;
    let mut data_set = inventory(&data)?;
    let other_set = inventory(&other)?;
    let (ar, aw) = pipe()?;
    let (br, bw) = pipe()?;
    let read = FILE_READ_DATA | SYNCHRONIZE;
    let write = FILE_WRITE_DATA | SYNCHRONIZE;
    let uw = assign(&ui, &aw, write, &mut ui_set)?;
    let ur = assign(&ui, &br, read, &mut ui_set)?;
    let dr = assign(&data, &ar, read, &mut data_set)?;
    let dw = assign(&data, &bw, write, &mut data_set)?;
    for end in [&ar, &aw, &br, &bw] {
        check_pipe_acl(end, &other)?;
    }
    println!("four endpoint ACLs deny read, write and create-instance for the foreign token");
    drop((ar, aw, br, bw));
    let mut receipts = Vec::new();
    if trial != Trial::Suspended {
        use bootstrap::Role;
        use windows_sys::Win32::System::Memory::SECTION_MAP_READ;
        let mut keys = zeroize::Zeroizing::new([0u8; 96]);
        getrandom::fill(keys.as_mut_slice()).map_err(|_| io::Error::other("random material"))?;
        for (role, child, ends, set) in [
            (Role::Window, &ui, [ur, uw], &mut ui_set),
            (Role::Data, &data, [dr, dw], &mut data_set),
        ] {
            if role == Role::Data && trial == Trial::BadReady {
                keys[95] ^= 1;
            }
            let purpose = if let Trial::StorePhase(c) = trial {
                bootstrap::Purpose::StorePhase(c)
            } else if trial == Trial::StoreEdit {
                bootstrap::Purpose::StoreEdit
            } else if trial == Trial::HeldRead {
                bootstrap::Purpose::HeldRead
            } else if matches!(trial, Trial::StoreRead | Trial::ReadCrash(_)) {
                bootstrap::Purpose::StoreRead
            } else if trial == Trial::EditConflict {
                bootstrap::Purpose::EditConflict
            } else if trial == Trial::DeniedSave {
                bootstrap::Purpose::DeniedSave
            } else if trial == Trial::ReadA {
                bootstrap::Purpose::ReadA
            } else {
                bootstrap::Purpose::Handshake
            };
            let section =
                bootstrap::section(purpose, role, &ui.generation, &data.generation, ends, &keys)?;
            let id = assign(child, &section, SECTION_MAP_READ, set)?;
            receipts.push(Receipt::write(
                role,
                &bootstrap::receipt(purpose, role, &ui.generation, &data.generation, id),
            )?);
        }
    }
    require(
        inventory(&ui)? == ui_set
            && inventory(&data)? == data_set
            && inventory(&other)? == other_set,
        "unexpected effective handle or flags",
    )?;
    println!(
        "both suspended: exact handle inventories; four identities and masks; inherit flags zero"
    );
    let ids = Path::new(DIR).join("pipe-targets.txt");
    let mut file = std::fs::File::create_new(&ids)?;
    use std::io::Write;
    writeln!(file, "{} {}", unsafe { GetProcessId(ui.proc.0) }, unsafe {
        GetProcessId(data.proc.0)
    })?;
    drop(file);
    let result = (|| unsafe {
        require(ResumeThread(other.thread.0) == 1, "countercheck resume")?;
        require(
            WaitForSingleObject(other.proc.0, 10000) == WAIT_OBJECT_0,
            "countercheck timeout",
        )?;
        let mut exit = 0;
        check(GetExitCodeProcess(other.proc.0, &mut exit))?;
        require(exit == 42, "countercheck denial")?;
        println!("foreign role: access denied for both held generations");
        if trial == Trial::Suspended {
            check(TerminateProcess(ui.proc.0, 75))?;
            check(TerminateProcess(data.proc.0, 75))?;
        } else {
            require(
                ResumeThread(data.thread.0) == 1 && ResumeThread(ui.thread.0) == 1,
                "pair resume",
            )?;
            if trial == Trial::StorePhase(scene_store::StorePhase::WindowClose) {
                finish_pending_window(&ui, &data, crash_starter)?;
            }
            let pair = [data.proc.0, ui.proc.0];
            require(
                WaitForMultipleObjects(
                    2,
                    pair.as_ptr(),
                    1,
                    if trial == Trial::StorePhase(scene_store::StorePhase::Timeout) {
                        45000
                    } else {
                        30000
                    },
                ) == WAIT_OBJECT_0,
                "pair deadline",
            )?;
            let (mut data_exit, mut ui_exit) = (0, 0);
            check(GetExitCodeProcess(data.proc.0, &mut data_exit))?;
            check(GetExitCodeProcess(ui.proc.0, &mut ui_exit))?;
            println!("data exit {data_exit}; UI exit {ui_exit}");
            require(
                (data_exit, ui_exit)
                    == match trial {
                        Trial::ReadA => (66, 67),
                        Trial::StoreEdit => (72, 73),
                        Trial::StorePhase(c) => c.exits(),
                        Trial::StoreRead | Trial::ReadCrash(_) => (74, 75),
                        Trial::EditConflict => (70, 71),
                        Trial::DeniedSave => (68, 69),
                        Trial::BadReady => (60, 63),
                        _ => (60, 62),
                    },
                "handshake outcome",
            )?;
        }
        drain(&ui)?;
        drain(&data)?;
        println!("UI and data jobs empty");
        if trial == Trial::ReadCrash(CleanupPoint::BeforeCleanup) {
            check(TerminateProcess(GetCurrentProcess(), 106))?;
            return Err(io::Error::other("pre-cleanup termination returned"));
        }
        Ok(())
    })();
    std::fs::remove_file(ids)?;
    result?;
    for receipt in receipts {
        let crash_here = trial == Trial::ReadCrash(CleanupPoint::AfterWindowReceipt)
            && receipt.0 == bootstrap::Role::Window.receipt();
        drop(receipt);
        if crash_here {
            require(
                !bootstrap::Role::Window.receipt().try_exists()?
                    && bootstrap::Role::Data.receipt().is_file(),
                "partial cleanup state",
            )?;
            unsafe {
                check(TerminateProcess(GetCurrentProcess(), 107))?;
            }
            return Err(io::Error::other("partial cleanup termination returned"));
        }
    }
    if trial == Trial::ReadCrash(CleanupPoint::AfterLastReceipt) {
        require(receipts_absent()?, "reader cleanup incomplete")?;
        unsafe {
            check(TerminateProcess(GetCurrentProcess(), 108))?;
        }
        return Err(io::Error::other("post-cleanup termination returned"));
    }
    Ok(())
}
// Only the elevated starter can modify this file in the protected package root.
const PENDING: &[u8] = b"BP4b/starter-v1/pending";
const DRAINED: &[u8] = b"BP4b/starter-v1/drained";

fn record(file: &mut std::fs::File, state: &[u8], anchor: &[u8]) -> io::Result<()> {
    use std::io::{Seek, Write};
    file.rewind()?;
    file.set_len(0)?;
    file.write_all(state)?;
    file.write_all(anchor)?;
    file.sync_all()
}

fn receipts_absent() -> io::Result<bool> {
    for name in ["window.start", "data.start", "pipe-targets.txt"] {
        if Path::new(DIR).join(name).try_exists()? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn begin(reopen: bool, anchor: &[u8]) -> io::Result<Option<std::fs::File>> {
    use std::{io::Read, os::windows::fs::OpenOptionsExt};
    let path = Path::new(DIR).join("pipe.lock");
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).share_mode(0);
    let mut lock = match options.create_new(true).open(&path) {
        Ok(f) => {
            // A missing record cannot authorize an existing store or stale generation.
            if reopen || !receipts_absent()? {
                return Ok(None);
            }
            for entry in std::fs::read_dir(DIR)? {
                if entry?
                    .file_name()
                    .to_string_lossy()
                    .starts_with("scene-store-")
                {
                    return Ok(None);
                }
            }
            f
        }
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            let mut f = options.create_new(false).open(path)?;
            let expected = [DRAINED, anchor].concat();
            if f.metadata()?.len() != expected.len() as u64 {
                return Ok(None);
            }
            let mut bytes = vec![0; expected.len()];
            f.read_exact(&mut bytes)?;
            if bytes != expected || !receipts_absent()? {
                return Ok(None);
            }
            f
        }
        Err(e) => return Err(e),
    };
    record(&mut lock, PENDING, anchor)?;
    Ok(Some(lock))
}

fn main() {
    let result = (|| {
        guest_image("pipe-check.exe")?;
        let trial = std::env::args().nth(1);
        let package = package::PinnedPackage::open(Path::new(DIR), None)?;
        let anchor = package.anchor_digest;
        use release_proof::ProofCase;
        match trial.as_deref() {
            Some("proof-post-cleanup-loss") => {
                return release_proof::exercise(
                    &anchor,
                    ProofCase::CleanupLoss(CleanupPoint::AfterLastReceipt),
                );
            }
            Some("proof-partial-cleanup-loss") => {
                return release_proof::exercise(
                    &anchor,
                    ProofCase::CleanupLoss(CleanupPoint::AfterWindowReceipt),
                );
            }
            Some("proof-pre-cleanup-loss") => {
                return release_proof::exercise(
                    &anchor,
                    ProofCase::CleanupLoss(CleanupPoint::BeforeCleanup),
                );
            }
            Some("proof-live-loss") => return release_proof::observe_live_loss(),
            Some("proof-live-witness") => {
                return release_proof::exercise(&anchor, ProofCase::LiveLoss);
            }
            Some("proof-consumed-loss") => {
                return release_proof::exercise(&anchor, ProofCase::LossAfterConsumption);
            }
            Some("proof-witness-loss") => {
                return release_proof::exercise(&anchor, ProofCase::WitnessLoss);
            }
            Some("proof-run") => return release_proof::exercise(&anchor, ProofCase::Complete),
            Some("proof-incomplete") => {
                return release_proof::exercise(&anchor, ProofCase::MissingEnd);
            }
            Some("proof-issuer") => return release_proof::issue(&anchor, false),
            Some("proof-issuer-incomplete") => return release_proof::issue(&anchor, true),
            _ => {}
        }
        let Some(mut lock) = begin(trial.as_deref() == Some("reopen-close-read"), &anchor)? else {
            println!("store locked: no verified end record");
            std::process::exit(102);
        };
        if trial.as_deref() == Some("starter-crash") {
            let phase = scene_store::StorePhase::WindowClose;
            let _pins = scene_store::prepare(Some(phase))?;
            run_pair(Trial::StorePhase(phase), true)
        } else if trial.as_deref() == Some("reopen-close-read") {
            run(Trial::StorePhase(scene_store::StorePhase::ReadClosed))
        } else if matches!(trial.as_deref(), Some("window-close" | "channel-ends")) {
            use scene_store::StorePhase::*;
            if trial.as_deref() == Some("channel-ends") {
                for (fault, readback) in [(Partial, ReadPartial), (Timeout, ReadTimeout)] {
                    let _pins = scene_store::prepare(Some(fault))?;
                    run(Trial::StorePhase(fault))?;
                    run(Trial::StorePhase(readback))?;
                }
            }
            let _pins = scene_store::prepare(Some(WindowClose))?;
            run(Trial::StorePhase(WindowClose))?;
            run(Trial::StorePhase(ReadClosed))
        } else if trial.as_deref() == Some("channel-interruptions") {
            use scene_store::StorePhase::*;
            for (fault, readback) in [(Partial, ReadPartial), (Timeout, ReadTimeout)] {
                let _pins = scene_store::prepare(Some(fault))?;
                run(Trial::StorePhase(fault))?;
                run(Trial::StorePhase(readback))?;
            }
            Ok(())
        } else if std::env::args().nth(1).as_deref() == Some("commit-crashes") {
            use scene_store::StorePhase::*;
            {
                let _pins = scene_store::prepare(Some(Before))?;
                run(Trial::StorePhase(Before))?;
                run(Trial::StorePhase(Recover))?;
                run(Trial::StorePhase(ReadOld))?;
            }
            let _pins = scene_store::prepare(Some(After))?;
            run(Trial::StorePhase(After))?;
            run(Trial::StorePhase(ReadNew))
        } else if std::env::args().nth(1).as_deref() == Some("durable-reopen") {
            let _pins = scene_store::prepare(None)?;
            run(Trial::StoreEdit)?;
            run(Trial::StoreRead)
        } else if std::env::args().nth(1).as_deref() == Some("edit-conflict") {
            run(Trial::EditConflict)
        } else if std::env::args().nth(1).as_deref() == Some("deny-save") {
            run(Trial::DeniedSave)
        } else if std::env::args().nth(1).as_deref() == Some("read-scene") {
            run(Trial::ReadA)
        } else if std::env::args().nth(1).as_deref() == Some("handshake") {
            run(Trial::BadReady)?;
            run(Trial::Valid)
        } else {
            run(Trial::Suspended)
        }?;
        require(receipts_absent()?, "generation receipts remain")?;
        record(&mut lock, DRAINED, &anchor)
    })();
    if let Err(e) = result {
        eprintln!("pipe check: {e}");
        std::process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::package::{IMAGES, PinnedPackage};
    use sha2::{Digest, Sha256};
    use std::fs::OpenOptions;

    #[test]
    fn image_cannot_be_replaced_until_package_release_and_changed_bytes_are_denied() {
        let root = std::env::temp_dir().join(format!("kraqel-package-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let mut files = Vec::new();
        for name in IMAGES {
            std::fs::write(root.join(name), name).unwrap();
            files.push(
                serde_json::json!({"name":name, "hash":format!("{:x}",Sha256::digest(name))}),
            );
        }
        std::fs::write(
            root.join("anchor.json"),
            serde_json::json!({"files":files}).to_string(),
        )
        .unwrap();
        let package = PinnedPackage::open(&root, None).unwrap();
        let target = root.join("pipe-service.exe");
        assert_eq!(
            std::fs::rename(&target, root.join("old.exe"))
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        assert_eq!(
            OpenOptions::new()
                .write(true)
                .open(&target)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
        drop(package);
        std::fs::write(target, b"changed").unwrap();
        assert!(PinnedPackage::open(&root, None).is_err());
        for name in IMAGES {
            std::fs::remove_file(root.join(name)).unwrap();
        }
        std::fs::remove_file(root.join("anchor.json")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
