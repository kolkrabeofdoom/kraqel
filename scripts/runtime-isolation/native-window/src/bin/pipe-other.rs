//! Fixed guest countercheck. Only test whether the protected fixture processes grant access.
use kraqel_window_probe::*;
use std::{io, path::Path, ptr::null_mut};
use windows_sys::Win32::{
    Foundation::*,
    Security::{FreeSid, Isolation::*},
    System::Threading::*,
};
fn run() -> io::Result<()> {
    guest_image("pipe-other.exe")?;
    unsafe {
        let mut sid = null_mut();
        require(
            DeriveAppContainerSidFromAppContainerName(
                wide("org.kraqel.bp4.other").as_ptr(),
                &mut sid,
            ) >= 0,
            "profile",
        )?;
        let verified = policy(GetCurrentProcess(), sid);
        FreeSid(sid);
        verified?;
        let me = Handle(OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            GetCurrentProcessId(),
        ));
        require(
            !me.0.is_null() && GetProcessId(me.0) == GetCurrentProcessId(),
            "local control",
        )?;
        let ids = std::fs::read_to_string(Path::new(DIR).join("pipe-targets.txt"))?;
        let ids = ids
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| io::Error::other("targets"))?;
        require(ids.len() == 2 && ids[0] != ids[1], "target pair")?;
        for id in ids {
            for right in [PROCESS_DUP_HANDLE, PROCESS_VM_READ, PROCESS_VM_WRITE] {
                let handle = OpenProcess(right, 0, id);
                if !handle.is_null() {
                    drop(Handle(handle));
                    return Err(io::Error::other("foreign right granted"));
                }
                require(
                    GetLastError() == ERROR_ACCESS_DENIED,
                    "unexpected negative result",
                )?;
            }
        }
    }
    Ok(())
}
fn main() {
    std::process::exit(if run().is_ok() { 42 } else { 1 })
}
