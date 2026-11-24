//! Experimental Win32 FFI boundary, deliberately outside the product Cargo workspace.
//!
//! All SID/ACL/attribute pointers stay owned and alive until the corresponding call finishes.
//! Token buffers are aligned, size-bounded allocations. Only exchange's two selected file
//! handles may be inherited via an explicit handle list; other modes inherit none. The
//! suspended root enters the kill-on-close job before ResumeThread. Only the fresh validated
//! TEMP root/data receive additional ACL entries. No firewall or loopback-exemption policy is set.
#![deny(unsafe_op_in_unsafe_fn)]
mod debug_guardian;
mod exchange;
mod p3_picker;
use serde_json::{Value, json};
use std::{
    ffi::c_void,
    fs,
    io::{Read, Write},
    mem::{size_of, zeroed},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    os::windows::{ffi::OsStrExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
    ptr::{null, null_mut},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, Isolation::*, *},
    Storage::FileSystem::{
        DELETE, FILE_DELETE_CHILD, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    },
    System::{JobObjects::*, Threading::*},
};

#[derive(Debug)]
struct Failure {
    stage: &'static str,
    code: i64,
}
type Result<T> = std::result::Result<T, Failure>;
fn fail(stage: &'static str) -> Failure {
    Failure {
        stage,
        code: std::io::Error::last_os_error()
            .raw_os_error()
            .unwrap_or(-1)
            .into(),
    }
}
fn io<T>(stage: &'static str, result: std::io::Result<T>) -> Result<T> {
    result.map_err(|e| Failure {
        stage,
        code: e.raw_os_error().unwrap_or(-1).into(),
    })
}
fn check(value: i32, stage: &'static str) -> Result<()> {
    if value == 0 { Err(fail(stage)) } else { Ok(()) }
}
fn wide(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct Local(*mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
struct Profile {
    name: Vec<u16>,
    sid: PSID,
    active: bool,
}
impl Profile {
    fn create(identity: &str) -> Result<Self> {
        let identity = format!("org.kraqel.probe.{identity}");
        if identity.len() > 64 {
            return Err(Failure {
                stage: "profile-name-too-long",
                code: -1,
            });
        }
        let name = wide(identity);
        let mut sid = null_mut();
        let hr = unsafe {
            CreateAppContainerProfile(
                name.as_ptr(),
                name.as_ptr(),
                wide("Synthetic no-network capability probe").as_ptr(),
                null(),
                0,
                &mut sid,
            )
        };
        if hr < 0 {
            return Err(Failure {
                stage: "create-profile",
                code: hr.into(),
            });
        }
        Ok(Self {
            name,
            sid,
            active: true,
        })
    }
    fn remove(&mut self) -> bool {
        let hr = unsafe { DeleteAppContainerProfile(self.name.as_ptr()) };
        if hr >= 0 {
            self.active = false;
        }
        hr >= 0
    }
}
impl Drop for Profile {
    fn drop(&mut self) {
        if self.active {
            let _ = self.remove();
        }
        unsafe {
            FreeSid(self.sid);
        }
    }
}

// Only fresh, task-owned directories are passed here. Preserve the inherited DACL.
fn grant(path: &Path, sid: PSID, rights: u32) -> Result<()> {
    let path = wide(path);
    let mut old = null_mut();
    let mut sd = null_mut();
    let status = unsafe {
        GetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut old,
            null_mut(),
            &mut sd,
        )
    };
    if status != 0 {
        return Err(Failure {
            stage: "read-owned-acl",
            code: status.into(),
        });
    }
    let _sd = Local(sd);
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: rights,
        grfAccessMode: GRANT_ACCESS,
        grfInheritance: OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_UNKNOWN,
            ptstrName: sid.cast(),
        },
    };
    let mut acl = null_mut();
    let status = unsafe { SetEntriesInAclW(1, &access, old, &mut acl) };
    if status != 0 {
        return Err(Failure {
            stage: "merge-owned-acl",
            code: status.into(),
        });
    }
    let _acl = Local(acl.cast());
    let status = unsafe {
        SetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            acl,
            null(),
        )
    };
    if status != 0 {
        return Err(Failure {
            stage: "write-owned-acl",
            code: status.into(),
        });
    }
    Ok(())
}
fn low_label(path: &Path) -> Result<()> {
    let mut sd = null_mut();
    check(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide("S:(ML;OICI;NW;;;LW)").as_ptr(),
                1,
                &mut sd,
                null_mut(),
            )
        },
        "make-owned-low-label",
    )?;
    let _sd = Local(sd);
    let mut sacl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    check(
        unsafe { GetSecurityDescriptorSacl(sd, &mut present, &mut sacl, &mut defaulted) },
        "read-owned-low-label",
    )?;
    let status = unsafe {
        SetNamedSecurityInfoW(
            wide(path).as_ptr(),
            SE_FILE_OBJECT,
            LABEL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null(),
            sacl,
        )
    };
    if status != 0 {
        return Err(Failure {
            stage: "set-owned-low-label",
            code: status.into(),
        });
    }
    Ok(())
}
fn token_bytes(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> Result<Vec<usize>> {
    let mut bytes = 0;
    unsafe {
        GetTokenInformation(token, class, null_mut(), 0, &mut bytes);
    }
    if bytes == 0 || bytes > 65536 {
        return Err(fail("token-size"));
    }
    let mut buffer = vec![0usize; (bytes as usize).div_ceil(size_of::<usize>())];
    check(
        unsafe { GetTokenInformation(token, class, buffer.as_mut_ptr().cast(), bytes, &mut bytes) },
        "token-read",
    )?;
    Ok(buffer)
}
fn token_facts(process: HANDLE, expected_sid: Option<PSID>) -> Result<Value> {
    let mut token = null_mut();
    check(
        unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) },
        "open-token",
    )?;
    let token = Handle(token);
    let container = token_bytes(token.0, TokenIsAppContainer)?;
    let is_container = unsafe { *container.as_ptr().cast::<u32>() } != 0;
    let caps = token_bytes(token.0, TokenCapabilities)?;
    let count = unsafe { *caps.as_ptr().cast::<u32>() };
    let matches = if let Some(sid) = expected_sid {
        let raw = token_bytes(token.0, TokenAppContainerSid)?;
        let actual =
            unsafe { (*raw.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>()).TokenAppContainer };
        Some(unsafe { EqualSid(sid, actual) != 0 })
    } else {
        None
    };
    Ok(json!({"isAppContainer":is_container,"capabilityCount":count,"expectedIdentity":matches}))
}
struct Attributes {
    _storage: Vec<usize>,
    ptr: LPPROC_THREAD_ATTRIBUTE_LIST,
}
impl Attributes {
    fn new(caps: &SECURITY_CAPABILITIES) -> Result<Self> {
        Self::with_handles(caps, &[])
    }
    fn with_handles(caps: &SECURITY_CAPABILITIES, handles: &[HANDLE]) -> Result<Self> {
        Self::with_handles_and_jobs(caps, handles, &[])
    }
    fn with_handles_and_jobs(
        caps: &SECURITY_CAPABILITIES,
        handles: &[HANDLE],
        jobs: &[HANDLE],
    ) -> Result<Self> {
        Self::with_creation_policy(caps, handles, jobs, None)
    }
    fn with_creation_policy(
        caps: &SECURITY_CAPABILITIES,
        handles: &[HANDLE],
        jobs: &[HANDLE],
        early: Option<(&u64, &u32)>,
    ) -> Result<Self> {
        let count = 1
            + u32::from(!handles.is_empty())
            + u32::from(!jobs.is_empty())
            + if early.is_some() { 2 } else { 0 };
        let mut bytes = 0;
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), count, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(fail("attribute-size"));
        }
        let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
        let ptr = storage.as_mut_ptr().cast();
        check(
            unsafe { InitializeProcThreadAttributeList(ptr, count, 0, &mut bytes) },
            "initialize-attributes",
        )?;
        let value = Self {
            _storage: storage,
            ptr,
        };
        check(
            unsafe {
                UpdateProcThreadAttribute(
                    ptr,
                    0,
                    PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
                    (caps as *const SECURITY_CAPABILITIES).cast(),
                    size_of::<SECURITY_CAPABILITIES>(),
                    null_mut(),
                    null(),
                )
            },
            "set-security-capabilities",
        )?;
        if !handles.is_empty() {
            check(
                unsafe {
                    UpdateProcThreadAttribute(
                        ptr,
                        0,
                        PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                        handles.as_ptr().cast(),
                        std::mem::size_of_val(handles),
                        null_mut(),
                        null(),
                    )
                },
                "set-handle-list",
            )?;
        }
        if !jobs.is_empty() {
            check(
                unsafe {
                    UpdateProcThreadAttribute(
                        ptr,
                        0,
                        PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                        jobs.as_ptr().cast(),
                        std::mem::size_of_val(jobs),
                        null_mut(),
                        null(),
                    )
                },
                "set-creation-job-list",
            )?;
        }
        if let Some((mitigations, child_policy)) = early {
            check(
                unsafe {
                    UpdateProcThreadAttribute(
                        ptr,
                        0,
                        PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY as usize,
                        (mitigations as *const u64).cast(),
                        size_of::<u64>(),
                        null_mut(),
                        null(),
                    )
                },
                "bp2-creation-mitigation",
            )?;
            check(
                unsafe {
                    UpdateProcThreadAttribute(
                        ptr,
                        0,
                        PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY as usize,
                        (child_policy as *const u32).cast(),
                        size_of::<u32>(),
                        null_mut(),
                        null(),
                    )
                },
                "bp2-creation-child-policy",
            )?;
        }
        Ok(value)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.ptr);
        }
    }
}

fn write_json(path: &Path, data: &Value) -> Result<()> {
    io(
        "write-report",
        fs::write(path, serde_json::to_vec_pretty(data).unwrap()),
    )
}
fn baseline() -> Result<(Vec<TcpListener>, Vec<UdpSocket>, Vec<u16>)> {
    let mut tcp = Vec::new();
    let mut udp = Vec::new();
    let mut ports = Vec::new();
    for host in ["127.0.0.1:0", "[::1]:0"] {
        let listener = io("control-tcp-bind", TcpListener::bind(host))?;
        let address = io("control-tcp-address", listener.local_addr())?;
        let mut sender = io(
            "control-tcp-connect",
            TcpStream::connect_timeout(&address, Duration::from_secs(2)),
        )?;
        io("control-tcp-send", sender.write_all(&[42]))?;
        let (mut receiver, _) = io("control-tcp-accept", listener.accept())?;
        io(
            "control-tcp-deadline",
            receiver.set_read_timeout(Some(Duration::from_secs(2))),
        )?;
        let mut byte = [0];
        io("control-tcp-receive", receiver.read_exact(&mut byte))?;
        if byte != [42] {
            return Err(Failure {
                stage: "control-tcp-content",
                code: -1,
            });
        }
        ports.push(address.port());
        tcp.push(listener);
    }
    for host in ["127.0.0.1:0", "[::1]:0"] {
        let receiver = io("control-udp-bind", UdpSocket::bind(host))?;
        let address = io("control-udp-address", receiver.local_addr())?;
        io(
            "control-udp-deadline",
            receiver.set_read_timeout(Some(Duration::from_secs(2))),
        )?;
        let sender = io("control-udp-sender", UdpSocket::bind(host))?;
        io("control-udp-send", sender.send_to(&[42], address))?;
        let mut byte = [0];
        let (size, _) = io("control-udp-receive", receiver.recv_from(&mut byte))?;
        if size != 1 || byte != [42] {
            return Err(Failure {
                stage: "control-udp-content",
                code: -1,
            });
        }
        ports.push(address.port());
        udp.push(receiver);
    }
    Ok((tcp, udp, ports))
}
fn probes(ports: &[u16]) -> Value {
    let mut results = Vec::new();
    for (label, address) in [
        ("tcp-loopback-v4", format!("127.0.0.1:{}", ports[0])),
        ("tcp-loopback-v6", format!("[::1]:{}", ports[1])),
        ("tcp-public-v4", "1.1.1.1:443".into()),
        ("tcp-public-v6", "[2606:4700:4700::1111]:443".into()),
    ] {
        let address: SocketAddr = address.parse().unwrap();
        let code = TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .err()
            .and_then(|e| e.raw_os_error());
        results.push(json!({"case":label,"errorCode":code,"accessDenied":code==Some(10013)}));
    }
    for (label, host, address) in [
        (
            "udp-loopback-v4",
            "0.0.0.0:0",
            format!("127.0.0.1:{}", ports[2]),
        ),
        ("udp-loopback-v6", "[::]:0", format!("[::1]:{}", ports[3])),
        ("udp-public-v4", "0.0.0.0:0", "1.1.1.1:9".into()),
        ("udp-public-v6", "[::]:0", "[2606:4700:4700::1111]:9".into()),
    ] {
        let result = UdpSocket::bind(host).and_then(|socket| socket.send_to(&[42], &address));
        let code = result.err().and_then(|e| e.raw_os_error());
        results.push(json!({"case":label,"errorCode":code,"accessDenied":code==Some(10013)}));
    }
    json!(results)
}
fn child(data: &Path, ports: Vec<u16>, leaf: bool) -> Result<()> {
    let facts = token_facts(unsafe { GetCurrentProcess() }, None)?;
    if facts["isAppContainer"] != true || facts["capabilityCount"] != 0 {
        return Err(Failure {
            stage: "child-requires-contained-token",
            code: -1,
        });
    }
    let attempts = probes(&ports);
    let dns = ("example.com", 443)
        .to_socket_addrs()
        .map(|items| items.count());
    let dns_report = match dns {
        Ok(count) => json!({"resolved":count>0,"addressCount":count}),
        Err(e) => json!({"resolved":false,"errorCode":e.raw_os_error()}),
    };
    let passed = facts["isAppContainer"] == true
        && facts["capabilityCount"] == 0
        && attempts
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["accessDenied"] == true);
    write_json(
        &data.join(if leaf { "leaf.json" } else { "child.json" }),
        &json!({"token":facts,"attempts":attempts,"fixedExampleDns":dns_report,"passed":passed}),
    )?;
    if !leaf {
        let status = io(
            "spawn-descendant",
            Command::new(io("current-exe", std::env::current_exe())?)
                .arg("leaf")
                .arg(data)
                .arg(
                    ports
                        .iter()
                        .map(u16::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .creation_flags(CREATE_NO_WINDOW)
                .status(),
        )?;
        if !status.success() {
            return Err(Failure {
                stage: "descendant-failed",
                code: status.code().unwrap_or(-1).into(),
            });
        }
    }
    if !passed {
        return Err(Failure {
            stage: "network-policy-not-proved",
            code: 1,
        });
    }
    Ok(())
}
fn launch(root: &Path, app: Option<&Path>, resume: bool) -> Result<Value> {
    let temp = io("temp-root", std::env::temp_dir().canonicalize())?;
    let name = root.file_name().unwrap_or_default().to_string_lossy();
    if !crate::policy::valid_root(root, resume) {
        return Err(Failure {
            stage: "fresh-temp-root-required",
            code: -1,
        });
    }
    if resume {
        if app.is_none() || io("resume-root", root.canonicalize())? != temp.join(name.as_ref()) {
            return Err(Failure {
                stage: "resume-root-mismatch",
                code: -1,
            });
        }
        let previous: Value =
            serde_json::from_slice(&io("resume-state", fs::read(root.join("result.json")))?)
                .map_err(|_| Failure {
                    stage: "resume-state-invalid",
                    code: -1,
                })?;
        if !crate::policy::clean_resume(&previous) {
            return Err(Failure {
                stage: "resume-not-clean",
                code: -1,
            });
        }
    } else {
        io("create-root", fs::create_dir(root))?;
    }
    let mut profile = Profile::create(name.strip_prefix("kraqel-isolation-").unwrap())?;
    write_json(
        &root.join("identity.json"),
        &json!({"ownedProfileName":String::from_utf16_lossy(&profile.name[..profile.name.len()-1]),"capabilitiesGranted":0}),
    )?;
    grant(root, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    let bin = root.join("bin");
    let data = root.join("data");
    if !resume {
        io("create-bin", fs::create_dir(&bin))?;
        io("create-data", fs::create_dir(&data))?;
    }
    for (path, child) in [(&bin, "bin"), (&data, "data")] {
        if io("owned-directory", path.canonicalize())? != temp.join(name.as_ref()).join(child) {
            return Err(Failure {
                stage: "owned-directory-mismatch",
                code: -1,
            });
        }
    }
    grant(
        &data,
        profile.sid,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE | FILE_DELETE_CHILD,
    )?;
    low_label(&data)?;
    let executable = bin.join(if app.is_some() {
        "kraqel-desktop.exe"
    } else {
        "kraqel-isolation-probe.exe"
    });
    if !resume {
        io(
            "stage-executable",
            fs::copy(
                app.map(Path::to_path_buf)
                    .unwrap_or(io("current-exe", std::env::current_exe())?),
                &executable,
            ),
        )?;
    } else if io("resume-executable", executable.canonicalize())?
        != temp.join(name.as_ref()).join("bin/kraqel-desktop.exe")
    {
        return Err(Failure {
            stage: "resume-executable-mismatch",
            code: -1,
        });
    }
    let (_tcp, _udp, ports) = baseline()?;
    let public_control =
        TcpStream::connect_timeout(&"1.1.1.1:443".parse().unwrap(), Duration::from_secs(2)).is_ok();
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let attributes = Attributes::new(&caps)?;
    let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.lpAttributeList = attributes.ptr;
    let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
    let arguments = if app.is_some() {
        format!("\"{}\"", executable.display())
    } else {
        format!(
            "\"{}\" child \"{}\" {}",
            executable.display(),
            data.display(),
            ports
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let mut command = wide(arguments);
    let system = std::env::var_os("SystemRoot").unwrap_or("C:\\Windows".into());
    let mut environment = Vec::new();
    for (name, value) in [
        ("APPDATA", data.clone()),
        ("LOCALAPPDATA", data.clone()),
        ("PATH", PathBuf::from(&system).join("System32")),
        ("SystemRoot", PathBuf::from(&system)),
        ("TEMP", data.clone()),
        ("TMP", data.clone()),
        ("USERPROFILE", data.clone()),
        ("WINDIR", PathBuf::from(&system)),
    ] {
        environment.extend(wide(format!("{name}={}", value.display())));
    }
    environment.push(0);
    let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
    if job.0.is_null() {
        return Err(fail("create-job"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    check(
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        },
        "set-job-limits",
    )?;
    check(
        unsafe {
            CreateProcessW(
                wide(&executable).as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                0,
                EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_UNICODE_ENVIRONMENT
                    | CREATE_SUSPENDED
                    | CREATE_NO_WINDOW,
                environment.as_ptr().cast(),
                wide(&data).as_ptr(),
                &startup.StartupInfo,
                &mut info,
            )
        },
        "create-contained-process",
    )?;
    let process = Handle(info.hProcess);
    let thread_handle = Handle(info.hThread);
    // If assignment fails, kill this still-suspended process. Never resume without containment.
    if unsafe { AssignProcessToJobObject(job.0, process.0) } == 0 {
        let error = fail("assign-job");
        unsafe {
            TerminateProcess(process.0, 1);
            WaitForSingleObject(process.0, 5000);
        }
        return Err(error);
    }
    let token = token_facts(process.0, Some(profile.sid))?;
    if token["isAppContainer"] != true
        || token["capabilityCount"] != 0
        || token["expectedIdentity"] != true
    {
        return Err(Failure {
            stage: "suspended-token-mismatch",
            code: -1,
        });
    }
    write_json(
        &root.join("launch.json"),
        &json!({"pid":info.dwProcessId,"token":token,"capabilitiesGranted":0}),
    )?;
    if unsafe { ResumeThread(thread_handle.0) } == u32::MAX {
        return Err(fail("resume-contained-process"));
    }
    let deadline = Instant::now() + Duration::from_secs(if app.is_some() { 600 } else { 30 });
    let mut stopped = false;
    while unsafe { WaitForSingleObject(process.0, 100) } == WAIT_TIMEOUT {
        if root.join("STOP").exists() || Instant::now() > deadline {
            stopped = true;
            unsafe {
                TerminateJobObject(job.0, 1);
            }
            break;
        }
    }
    unsafe {
        WaitForSingleObject(process.0, 5000);
    }
    let mut code = 0;
    check(
        unsafe { GetExitCodeProcess(process.0, &mut code) },
        "process-exit-code",
    )?;
    unsafe {
        TerminateJobObject(job.0, 1);
    }
    // Keep the job handle while waiting for all assigned descendants to exit.
    let end = Instant::now() + Duration::from_secs(5);
    let mut active = 1;
    while active != 0 && Instant::now() < end {
        let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicAccountingInformation,
                    (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            "job-accounting",
        )?;
        active = accounting.ActiveProcesses;
        if active != 0 {
            thread::sleep(Duration::from_millis(50));
        }
    }
    drop(thread_handle);
    drop(process);
    drop(job);
    let cleaned = active == 0 && profile.remove();
    let mut report = json!({"formatVersion":1,"diagnosticOnly":true,"appMode":app.is_some(),"resumed":resume,"tokenBeforeResume":token,"localPositiveControls":4,"publicIpv4TcpControlConnected":public_control,"processExitCode":code,"deadlineOrStop":stopped,"activeJobProcessesAtEnd":active,"profileRemoved":cleaned,"productionAcceptance":false});
    if app.is_none() {
        for name in ["child", "leaf"] {
            report[name] = serde_json::from_slice(&io(
                "read-child-report",
                fs::read(data.join(format!("{name}.json"))),
            )?)
            .map_err(|_| Failure {
                stage: "invalid-child-report",
                code: -1,
            })?;
        }
    }
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}
pub fn main() {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|a| a == "bp3-debug-worker") {
        let result = debug_guardian::worker(&args[1..]);
        std::process::exit(if result.is_ok() { 0 } else { 2 });
    }
    if args.first().is_some_and(|a| a == "bp2-leaf-worker") {
        let result = exchange::bp2_worker(&args[1..]);
        std::process::exit(if result.is_ok() { 0 } else { 2 });
    }
    if args.first().is_some_and(|a| a == "bp2-must-not-start") {
        std::process::exit(2);
    }
    if args.first().is_some_and(|a| a == "c4-root-probe-worker") {
        let result = exchange::c4_root_worker(&args[1..]);
        std::process::exit(if result.is_ok() { 0 } else { 2 });
    }
    if args.first().is_some_and(|a| a == "c3-synthetic-worker") {
        let code = if exchange::group_worker(&args[1..]).is_ok() {
            0
        } else {
            2
        };
        std::process::exit(code);
    }
    // Never enter generic diagnostics/network-probe dispatch from this worker role.
    if args.first().is_some_and(|a| a == "workspace-probe-worker") {
        let code = if exchange::product_workspace_probe_worker(&args[1..]).is_ok() {
            0
        } else {
            2
        };
        std::process::exit(code);
    }
    let result = (|| {
        if args.first().is_some_and(|a| a == "product-c3-group-child") {
            return exchange::group_child(&args[1..]);
        }
        if args.first().is_some_and(|a| a == "product-p2c-data-child") {
            return exchange::product_workspace_persistent_child(&args[1..]);
        }
        if args.len() == 3 && args[0] == "product-control-series" {
            return exchange::product_crash_series(Path::new(&args[1]), Path::new(&args[2]));
        }
        if args.len() == 4 && args[0] == "product-transition-series" {
            return exchange::product_transition_series(
                Path::new(&args[1]),
                Path::new(&args[2]),
                &args[3],
            );
        }
        if args.len() == 7 && args[0] == "product-transition-child" {
            return exchange::product_transition_child(&args[1..]);
        }
        if args.len() == 4 && args[0] == "product-session-series" {
            return exchange::product_session_series(
                Path::new(&args[1]),
                Path::new(&args[2]),
                &args[3],
            );
        }
        if args.len() == 6 && args[0] == "product-session-child" {
            return exchange::product_session_child(&args[1..]);
        }
        if args.len() == 6 && args[0] == "product-workspace-child" {
            return exchange::product_workspace_child(&args[1..]);
        }
        if args.len() == 4 && args[0] == "product-workspace-probe-child" {
            return exchange::product_workspace_probe_child(&args[1..]);
        }
        if args.len() == 4 && args[0] == "product-start-series" {
            return exchange::product_start_series(
                Path::new(&args[1]),
                Path::new(&args[2]),
                &args[3],
            );
        }
        if args.len() == 6 && args[0] == "product-control-child" {
            return exchange::product_crash_child(&args[1..]);
        }
        if args.len() == 3 && args[0] == "select-p3-fixture" {
            return p3_picker::choose(&args[1], &args[2]);
        }
        if args.first().is_some_and(|arg| arg == "broker-store") {
            return exchange::product_storage(&args[1..]);
        }
        if args.len() == 5 && args[0] == "ui-broker-live-seed" {
            return exchange::seed_live(
                Path::new(&args[1]),
                Path::new(&args[2]),
                &args[3].to_string_lossy(),
                &args[4].to_string_lossy(),
            );
        }
        if args.len() == 2 && args[0] == "ui-broker-live-next" {
            return exchange::next_live(Path::new(&args[1]));
        }
        if args.len() == 3 && args[0] == "ui-combined-series" {
            return exchange::combined_series(Path::new(&args[1]), Path::new(&args[2]));
        }
        if args.len() == 5 && args[0] == "exchange-child" {
            return exchange::child(&args[1..]);
        }
        if args.len() == 5 && args[0] == "import-child" {
            return exchange::import_child(&args[1..]);
        }
        if args.len() == 3 && args[0] == "app-import-fixture" {
            return exchange::launch_import(Path::new(&args[1]), &args[2].to_string_lossy());
        }
        if args.len() == 3 && args[0] == "ui-crash-series" {
            return exchange::launch_restart_series(Path::new(&args[1]), Path::new(&args[2]));
        }
        if args.len() == 3 && args[0] == "ui-controller-crash-series" {
            return exchange::controller_crash_series(Path::new(&args[1]), Path::new(&args[2]));
        }
        if args.len() == 4 && args[0] == "ui-broker-seed" {
            return exchange::seed_broker(
                Path::new(&args[1]),
                Path::new(&args[2]),
                &args[3].to_string_lossy(),
            );
        }
        if args.len() == 3 && args[0] == "ui-broker-recover" {
            return exchange::recover_broker(Path::new(&args[1]), &args[2].to_string_lossy());
        }
        if args.len() == 3
            && (args[0] == "ui-assistive-interrupted" || args[0] == "ui-assistive-completed")
        {
            return exchange::launch_assistive(
                Path::new(&args[1]),
                Path::new(&args[2]),
                args[0] == "ui-assistive-completed",
            );
        }
        if args.len() == 3 && (args[0] == "ui-live" || args[0] == "ui-live-transfer-hold") {
            return exchange::launch_live_ui(
                Path::new(&args[1]),
                Path::new(&args[2]),
                args[0] == "ui-live-transfer-hold",
            );
        }
        if args.len() == 3 && (args[0] == "ui-fixture" || args[0] == "ui-select") {
            return exchange::launch_ui(
                Path::new(&args[1]),
                Path::new(&args[2]),
                args[0] == "ui-select",
            );
        }
        if args.len() == 2 && args[0] == "exchange-fixture" {
            return exchange::launch_fixture(Path::new(&args[1]));
        }
        if args.len() == 2 && args[0] == "exchange" {
            return exchange::launch(Path::new(&args[1]));
        }
        if args.len() == 3 && (args[0] == "child" || args[0] == "leaf") {
            let ports = crate::policy::ports(&args[2].to_string_lossy()).ok_or(Failure {
                stage: "invalid-ports",
                code: -1,
            })?;
            child(Path::new(&args[1]), ports, args[0] == "leaf")?;
            return Ok(json!({"childComplete":true}));
        }
        if args.len() == 2 && args[0] == "inspect" {
            let ids = args[1]
                .to_string_lossy()
                .split(',')
                .map(str::parse::<u32>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| Failure {
                    stage: "invalid-pids",
                    code: -1,
                })?;
            let mut facts = Vec::new();
            for pid in ids {
                let process =
                    Handle(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) });
                if process.0.is_null() {
                    return Err(fail("inspect-open-process"));
                }
                facts.push(json!({"pid":pid,"token":token_facts(process.0,None)?}));
            }
            return Ok(
                json!({"diagnosticOnly":true,"processes":facts,"productionAcceptance":false}),
            );
        }
        if args.len() == 2 && args[0] == "network" {
            launch(Path::new(&args[1]), None, false)
        } else if args.len() == 3 && args[0] == "app" {
            launch(Path::new(&args[1]), Some(Path::new(&args[2])), false)
        } else if args.len() == 2 && args[0] == "resume" {
            launch(
                Path::new(&args[1]),
                Some(&Path::new(&args[1]).join("bin/kraqel-desktop.exe")),
                true,
            )
        } else {
            Err(Failure {
                stage: "usage-network-root-or-app-root-executable",
                code: -1,
            })
        }
    })();
    match result {
        Ok(value) => {
            println!("{value}");
            if value["exchange"] == true && value["passed"] != true
                || value["exchange"] != true
                    && value.get("tokenBeforeResume").is_some()
                    && !crate::policy::successful_run(&value)
            {
                std::process::exit(1)
            }
        }
        Err(error) => {
            if args.len() == 5
                && args[0] == "import-child"
                && token_facts(unsafe { GetCurrentProcess() }, None)
                    .is_ok_and(|t| t["isAppContainer"] == true && t["capabilityCount"] == 0)
            {
                let _ = write_json(
                    &Path::new(&args[1]).join("child-error.json"),
                    &json!({"stage":error.stage,"code":error.code}),
                );
            }
            eprintln!(
                "{}",
                json!({"stage":error.stage,"code":error.code,"productionAcceptance":false})
            );
            std::process::exit(1)
        }
    }
}
