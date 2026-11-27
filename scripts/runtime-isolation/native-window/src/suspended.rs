use crate::*;
use std::{
    fs::OpenOptions,
    io,
    mem::{size_of, zeroed},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::*,
    System::{JobObjects::*, Threading::*},
};

pub struct Suspended {
    pub job: Handle,
    pub proc: Handle,
    pub thread: Handle,
    pub generation: String,
    pub pin: std::fs::File,
}
struct Attributes(Vec<usize>);
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast());
        }
    }
}
/// Starts one of the caller's pinned guest fixtures in a one-process job.
///
/// # Safety
/// `sid` must be a valid AppContainer SID for the full call.
pub unsafe fn suspended(sid: PSID, name: &str) -> io::Result<Suspended> {
    let dir = Path::new(DIR);
    let exe = dir.join(name);
    let pin = OpenOptions::new().read(true).share_mode(1).open(&exe)?;
    let image = wide(&exe);
    let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
    require(!job.0.is_null(), "job creation")?;
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: sid,
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
    unsafe { policy(proc.0, sid) }?;
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
    Ok(Suspended {
        job,
        proc,
        thread,
        generation,
        pin,
    })
}
