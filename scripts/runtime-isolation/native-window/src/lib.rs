pub mod bootstrap;
mod frame_deadline;
pub mod scene_store;
pub mod suspended;
use std::io;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;
use windows_sys::Win32::System::SystemInformation::{ComputerNameNetBIOS, GetComputerNameExW};
use windows_sys::Win32::{
    Foundation::*,
    Security::*,
    System::{RemoteDesktop::*, Threading::*},
};

pub const DIR: &str = r"C:\Program Files\KRAQEL-Window-Probe";
pub const PROFILE: &str = "org.kraqel.bp3.window.v1";
pub const CHILD: &str = "native-leaf.exe";
pub fn wide(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}
pub fn check(ok: i32) -> io::Result<()> {
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
pub fn require(ok: bool, problem: &'static str) -> io::Result<()> {
    if ok {
        Ok(())
    } else {
        Err(io::Error::other(problem))
    }
}
pub struct Handle(pub HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
/// # Safety
/// The supplied handle must refer to a live process throughout the call.
pub unsafe fn ticket(proc: HANDLE) -> io::Result<String> {
    unsafe {
        let (mut c, mut e, mut k, mut u) = zeroed();
        check(GetProcessTimes(proc, &mut c, &mut e, &mut k, &mut u))?;
        let pid = GetProcessId(proc);
        let mut session = 0;
        check(ProcessIdToSessionId(pid, &mut session))?;
        require(
            pid != 0 && session != 0 && session == WTSGetActiveConsoleSessionId(),
            "console generation",
        )?;
        Ok(format!(
            "KRAQEL-NATIVE-EDIT-v1/{pid}/{:08x}{:08x}/{session}",
            c.dwHighDateTime, c.dwLowDateTime
        ))
    }
}
/// # Safety
/// The process handle and the expected SID must remain valid throughout the readback.
pub unsafe fn policy(proc: HANDLE, sid: PSID) -> io::Result<()> {
    unsafe {
        let mut token = null_mut();
        check(OpenProcessToken(proc, TOKEN_QUERY, &mut token))?;
        let token = Handle(token);
        let mut app = 0u32;
        let mut n = 0;
        check(GetTokenInformation(
            token.0,
            TokenIsAppContainer,
            (&raw mut app).cast(),
            4,
            &mut n,
        ))?;
        require(app == 1, "AppContainer")?;
        // Aligned storage; oversized token information fails closed.
        let mut data = [0usize; 256];
        check(GetTokenInformation(
            token.0,
            TokenCapabilities,
            data.as_mut_ptr().cast(),
            size_of_val(&data) as u32,
            &mut n,
        ))?;
        require(
            (*(data.as_ptr() as *const TOKEN_GROUPS)).GroupCount == 0,
            "capabilities",
        )?;
        check(GetTokenInformation(
            token.0,
            TokenAppContainerSid,
            data.as_mut_ptr().cast(),
            size_of_val(&data) as u32,
            &mut n,
        ))?;
        let actual = (*(data.as_ptr() as *const TOKEN_APPCONTAINER_INFORMATION)).TokenAppContainer;
        require(
            !actual.is_null() && EqualSid(actual, sid) != 0,
            "profile SID",
        )?;
        for (kind, name) in [
            (ProcessChildProcessPolicy, "child policy"),
            (ProcessDynamicCodePolicy, "dynamic code"),
        ] {
            let mut bits = 0u32;
            check(GetProcessMitigationPolicy(
                proc,
                kind,
                (&raw mut bits).cast(),
                size_of::<u32>(),
            ))?;
            require(bits == 1, name)?;
        }
        Ok(())
    }
}
pub fn guest_image(name: &str) -> io::Result<()> {
    require(
        std::env::current_exe()? == Path::new(DIR).join(name),
        "fixture location",
    )?;
    let mut buf = [0u16; 64];
    let mut n = buf.len() as u32;
    unsafe {
        check(GetComputerNameExW(
            ComputerNameNetBIOS,
            buf.as_mut_ptr(),
            &mut n,
        ))?;
    }
    require(
        String::from_utf16_lossy(&buf[..n as usize]).eq_ignore_ascii_case("KRAQEL-BP3"),
        "guest identity",
    )
}
pub fn verify_leaf() -> io::Result<()> {
    guest_image(CHILD)?;
    unsafe {
        use windows_sys::Win32::Security::Isolation::DeriveAppContainerSidFromAppContainerName;
        let mut sid = null_mut();
        let hr = DeriveAppContainerSidFromAppContainerName(wide(PROFILE).as_ptr(), &mut sid);
        require(hr >= 0, "profile")?;
        let proc = GetCurrentProcess();
        let result = policy(proc, sid);
        FreeSid(sid);
        result?;
        let mut assigned = 0;
        check(windows_sys::Win32::System::JobObjects::IsProcessInJob(
            proc,
            null_mut(),
            &mut assigned,
        ))?;
        require(assigned != 0, "job")?;
        let expected = ticket(proc)?;
        // The runner establishes a protected, local administrator/SYSTEM-owned directory.
        // Only the starter writes this one-generation receipt after the suspended readback.
        let actual = std::fs::read(Path::new(DIR).join("active.grant"))?;
        require(actual == expected.as_bytes(), "generation grant")
    }
}
