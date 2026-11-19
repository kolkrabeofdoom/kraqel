use kraqel_window_probe::*;
use std::{
    fs::OpenOptions,
    io::{self, Write},
    mem::{size_of, zeroed},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Isolation::*, *},
    System::{JobObjects::*, Threading::*},
};

struct Profile(PSID);
impl Drop for Profile {
    fn drop(&mut self) {
        unsafe {
            FreeSid(self.0);
        }
    }
}
struct Attributes(Vec<usize>);
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast());
        }
    }
}
fn run_leaf(profile: &Profile, negative: bool) -> io::Result<()> {
    let dir = Path::new(DIR);
    let exe = dir.join(CHILD);
    let _pin = OpenOptions::new().read(true).share_mode(1).open(&exe)?;
    let image = wide(&exe);
    let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
    require(!job.0.is_null(), "job creation")?;
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.0,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let child =
        windows_sys::Win32::System::WindowsProgramming::PROCESS_CREATION_CHILD_PROCESS_RESTRICTED;
    let dynamic: u64 = 1 << 36; // Windows SDK: PROCESS_CREATION_MITIGATION_POLICY_PROHIBIT_DYNAMIC_CODE_ALWAYS_ON
    let mut size = 0;
    unsafe {
        check(SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            (&raw const limits).cast(),
            size_of_val(&limits) as u32,
        ))?;
        InitializeProcThreadAttributeList(null_mut(), 4, 0, &mut size);
    }
    require(size > 0, "attribute list size")?;
    let mut storage = vec![0usize; size.div_ceil(size_of::<usize>())];
    let list = storage.as_mut_ptr().cast();
    unsafe {
        check(InitializeProcThreadAttributeList(list, 4, 0, &mut size))?;
    }
    let _attrs = Attributes(storage);
    let mut si: STARTUPINFOEXW = unsafe { zeroed() };
    si.StartupInfo.cb = size_of_val(&si) as u32;
    si.lpAttributeList = list;
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    unsafe {
        for (key, ptr, len) in [
            (
                PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES,
                (&raw const caps).cast(),
                size_of_val(&caps),
            ),
            (
                PROC_THREAD_ATTRIBUTE_JOB_LIST,
                (&raw const job.0).cast(),
                size_of::<HANDLE>(),
            ),
            (
                PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY,
                (&raw const child).cast(),
                size_of_val(&child),
            ),
            (
                PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY,
                (&raw const dynamic).cast(),
                size_of_val(&dynamic),
            ),
        ] {
            check(UpdateProcThreadAttribute(
                list,
                0,
                key as usize,
                ptr,
                len,
                null_mut(),
                null(),
            ))?;
        }
        // FALSE disables inheritance of handles, including the job and the starter's output.
        check(CreateProcessW(
            image.as_ptr(),
            null_mut(),
            null(),
            null(),
            0,
            CREATE_SUSPENDED | EXTENDED_STARTUPINFO_PRESENT,
            null(),
            wide(dir).as_ptr(),
            &si.StartupInfo,
            &mut pi,
        ))?;
    }
    let proc = Handle(pi.hProcess);
    let thread = Handle(pi.hThread);
    let generation = unsafe { ticket(proc.0) }?;
    unsafe { policy(proc.0, profile.0) }?;
    let mut actual = [0u16; 512];
    let mut n = actual.len() as u32;
    let mut member = 0;
    unsafe {
        check(QueryFullProcessImageNameW(
            proc.0,
            0,
            actual.as_mut_ptr(),
            &mut n,
        ))?;
        check(IsProcessInJob(proc.0, job.0, &mut member))?;
    }
    require(
        member != 0
            && String::from_utf16_lossy(&actual[..n as usize])
                .eq_ignore_ascii_case(&exe.to_string_lossy()),
        "image or job",
    )?;
    let mut effective: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    unsafe {
        check(QueryInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            (&raw mut effective).cast(),
            size_of_val(&effective) as u32,
            null_mut(),
        ))?;
    }
    require(
        effective.BasicLimitInformation.ActiveProcessLimit == 1
            && effective.BasicLimitInformation.LimitFlags & limits.BasicLimitInformation.LimitFlags
                == limits.BasicLimitInformation.LimitFlags,
        "job limits",
    )?;
    let grant_path = dir.join("active.grant");
    let requested_role = if negative { "writer" } else { "synthetic-edit" };
    let exit = if requested_role != "synthetic-edit" {
        unsafe {
            check(TerminateProcess(proc.0, 75))?;
        }
        75
    } else {
        let mut grant = OpenOptions::new()
            .create_new(true)
            .write(true)
            .share_mode(1)
            .open(&grant_path)?;
        let result = (|| {
            grant.write_all(generation.as_bytes())?;
            grant.sync_all()?;
            require(unsafe { ResumeThread(thread.0) } == 1, "resume")?;
            require(
                unsafe { WaitForSingleObject(proc.0, 300000) } == WAIT_OBJECT_0,
                "window wait",
            )
        })();
        drop(grant);
        std::fs::remove_file(&grant_path)?;
        result?;
        40
    };
    unsafe {
        require(
            WaitForSingleObject(proc.0, 5000) == WAIT_OBJECT_0,
            "leaf end",
        )?;
        let mut code = 0;
        check(GetExitCodeProcess(proc.0, &mut code))?;
        if code != exit {
            eprintln!("leaf exit {code}");
            return Err(io::Error::other("leaf result"));
        }
        drop(thread);
        drop(proc);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = zeroed();
            check(QueryInformationJobObject(
                job.0,
                JobObjectBasicAccountingInformation,
                (&raw mut info).cast(),
                size_of_val(&info) as u32,
                null_mut(),
            ))?;
            if info.ActiveProcesses == 0 && info.TotalProcesses == 1 {
                break;
            }
            if std::time::Instant::now() >= deadline {
                eprintln!(
                    "job active {} total {}",
                    info.ActiveProcesses, info.TotalProcesses
                );
                return Err(io::Error::other("drain"));
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
    }
    println!("role {requested_role} exit {exit} drained 1");
    Ok(())
}
fn run() -> io::Result<()> {
    guest_image("native-starter.exe")?;
    unsafe { ticket(GetCurrentProcess()) }?;
    let profile = wide(PROFILE);
    let mut sid = null_mut();
    let hr = unsafe {
        CreateAppContainerProfile(
            profile.as_ptr(),
            profile.as_ptr(),
            profile.as_ptr(),
            null(),
            0,
            &mut sid,
        )
    };
    require(hr >= 0, "fresh probe profile")?;
    let p = Profile(sid);
    let result = run_leaf(&p, true).and_then(|_| run_leaf(&p, false));
    drop(p);
    let hr = unsafe { DeleteAppContainerProfile(profile.as_ptr()) };
    require(hr >= 0, "profile cleanup")?;
    result
}
fn main() {
    if let Err(e) = run() {
        eprintln!("probe: {e}");
        std::process::exit(2);
    }
}
