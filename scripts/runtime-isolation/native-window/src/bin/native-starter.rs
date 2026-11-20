use kraqel_window_probe::*;
use std::{
    fs::OpenOptions,
    io::{self, Write},
    mem::zeroed,
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
fn run_leaf(profile: &Profile, negative: bool) -> io::Result<()> {
    let dir = Path::new(DIR);
    let suspended::Suspended {
        job,
        proc,
        thread,
        generation,
        pin: _pin,
    } = unsafe { suspended::suspended(profile.0, CHILD) }?;
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
