//! Pre-resume check for our own two suspended fixture generations, in the BP3 guest only.
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
enum Trial {
    Suspended,
    BadReady,
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

fn run(trial: Trial) -> io::Result<()> {
    guest_image("pipe-check.exe")?;
    let ui_profile = Profile::fresh(PROFILE)?;
    let data_profile = Profile::fresh("org.kraqel.bp4.data")?;
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
            let section = bootstrap::section(role, &ui.generation, &data.generation, ends, &keys)?;
            let id = assign(child, &section, SECTION_MAP_READ, set)?;
            receipts.push(Receipt::write(
                role,
                &bootstrap::receipt(role, &ui.generation, &data.generation, id),
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
            let pair = [data.proc.0, ui.proc.0];
            require(
                WaitForMultipleObjects(2, pair.as_ptr(), 1, 30000) == WAIT_OBJECT_0,
                "pair deadline",
            )?;
            let (mut data_exit, mut ui_exit) = (0, 0);
            check(GetExitCodeProcess(data.proc.0, &mut data_exit))?;
            check(GetExitCodeProcess(ui.proc.0, &mut ui_exit))?;
            println!("data exit {data_exit}; UI exit {ui_exit}");
            require(
                data_exit == 60 && ui_exit == if trial == Trial::BadReady { 63 } else { 62 },
                "handshake outcome",
            )?;
        }
        drain(&ui)?;
        drain(&data)?;
        println!("UI and data jobs empty");
        Ok(())
    })();
    std::fs::remove_file(ids)?;
    result
}
fn main() {
    let result = (|| {
        guest_image("pipe-check.exe")?;
        use std::os::windows::fs::OpenOptionsExt;
        let _lock = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .open(Path::new(DIR).join("pipe.lock"))?;
        if std::env::args().nth(1).as_deref() == Some("handshake") {
            run(Trial::BadReady)?;
            run(Trial::Valid)
        } else {
            run(Trial::Suspended)
        }
    })();
    if let Err(e) = result {
        eprintln!("pipe check: {e}");
        std::process::exit(1)
    }
}
