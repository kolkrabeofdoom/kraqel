//! One-shot synthetic Shell broker. No format parsing, shell execution or network APIs here.
use super::*;
use crate::transfer;
use std::{
    ffi::OsString,
    fs::OpenOptions,
    os::windows::{
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle},
    },
};
use windows::{
    Win32::{System::Com::*, UI::Shell::*},
    core::{Interface, PCWSTR},
};
use windows_sys::Win32::Storage::FileSystem::*;
mod app_import;
pub(super) use app_import::{child as import_child, launch as launch_import};
mod ui_grant;
mod ui_live;
mod ui_restart;
pub(super) use ui_grant::launch as launch_ui;
pub(super) use ui_live::launch as launch_live_ui;
pub(super) use ui_restart::launch as launch_restart_series;
pub(super) use ui_restart::launch_assistive;
pub(super) use ui_restart::product_storage;
pub(super) use ui_restart::{combined_series, next_live, seed_live};
pub(super) use ui_restart::{controller_crash_series, recover_broker, seed_broker};
pub(super) use ui_restart::{
    product_crash_child, product_crash_series, product_session_child, product_session_series,
    product_start_series,
};

fn invalid(stage: &'static str) -> Failure {
    Failure { stage, code: -1 }
}
fn com<T>(stage: &'static str, value: windows::core::Result<T>) -> Result<T> {
    value.map_err(|e| Failure {
        stage,
        code: e.code().0.into(),
    })
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
struct SelectionText(windows::core::PWSTR);
impl Drop for SelectionText {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.0.cast())) };
    }
}

fn choose(host: &Path, save: bool) -> Result<Option<PathBuf>> {
    choose_named(
        host,
        save,
        if save { "output.txt" } else { "input.txt" },
        false,
    )
}
fn choose_named(
    host: &Path,
    save: bool,
    suggested: &str,
    snapshot: bool,
) -> Result<Option<PathBuf>> {
    // Called on the one-shot controller's sole STA thread, never the renderer's thread.
    com("shell-sta", unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()
    })?;
    let _apartment = Apartment;
    let dialog: IFileDialog = if save {
        com("save-dialog", unsafe {
            CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
        })?
    } else {
        com("open-dialog", unsafe {
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
        })?
    };
    let shell_host = shell_folder_path(host)?;
    let folder: IShellItem = com("shell-folder", unsafe {
        SHCreateItemFromParsingName(PCWSTR(wide(&shell_host).as_ptr()), None)
    })?;
    let second: IFileDialog2 = com("shell-navigation-interface", dialog.cast())?;
    unsafe {
        com("shell-navigation-root", second.SetNavigationRoot(&folder))?;
        com("shell-folder", dialog.SetFolder(&folder))?;
        com("shell-default-folder", dialog.SetDefaultFolder(&folder))?;
        com(
            "shell-options",
            dialog.SetOptions(
                FOS_FORCEFILESYSTEM
                    | FOS_NOCHANGEDIR
                    | FOS_PATHMUSTEXIST
                    | FOS_DONTADDTORECENT
                    | FOS_NODEREFERENCELINKS
                    | if save {
                        FOS_OVERWRITEPROMPT
                    } else {
                        FOS_FILEMUSTEXIST
                    },
            ),
        )?;
        com(
            "shell-title",
            dialog.SetTitle(PCWSTR(
                wide(if save {
                    "KRAQEL Austauschprobe: neue Ausgabe"
                } else {
                    "KRAQEL Austauschprobe: synthetische Eingabe"
                })
                .as_ptr(),
            )),
        )?;
        com(
            "shell-filename",
            dialog.SetFileName(PCWSTR(wide(suggested).as_ptr())),
        )?;
        let shown = dialog.Show(None);
        if let Err(error) = shown {
            if error.code().0 as u32 == 0x800704c7 {
                return Ok(None);
            }
            return Err(Failure {
                stage: "shell-show",
                code: error.code().0.into(),
            });
        }
        let item = com("shell-result", dialog.GetResult())?;
        let text = SelectionText(com(
            "shell-filesystem-path",
            item.GetDisplayName(SIGDN_FILESYSPATH),
        )?);
        let path = PathBuf::from(text.0.to_string().map_err(|_| invalid("shell-path-text"))?);
        // Shell navigation is guidance; this independent allowlist is the security check.
        scoped_selection(&path, host, snapshot).map(Some)
    }
}

// Shell parsing does not accept every canonical Win32 file namespace. Re-express only a local
// disk path and prove the same directory before using it; selection validation stays canonical.
fn shell_folder_path(host: &Path) -> Result<PathBuf> {
    use std::path::{Component, Prefix};
    let canonical = io("shell-host-canonical", host.canonicalize())?;
    let mut components = canonical.components();
    let Some(Component::Prefix(prefix)) = components.next() else {
        return Err(invalid("shell-host-namespace"));
    };
    let Prefix::VerbatimDisk(drive) = prefix.kind() else {
        return Err(invalid("shell-host-local-disk-only"));
    };
    if components.next() != Some(Component::RootDir) {
        return Err(invalid("shell-host-root"));
    }
    let mut plain = PathBuf::from(format!("{}:\\", char::from(drive)));
    for component in components {
        let Component::Normal(name) = component else {
            return Err(invalid("shell-host-component"));
        };
        plain.push(name);
    }
    if wide(&plain).len() > 260 || io("shell-host-identity", plain.canonicalize())? != canonical {
        return Err(invalid("shell-host-identity-or-length"));
    }
    Ok(plain)
}

fn scoped_selection(path: &Path, host: &Path, snapshot: bool) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or(invalid("selection-outside-synthetic-root"))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|n| transfer::flat_name(n) || (snapshot && *n == "input.kraqel"))
        .ok_or(invalid("selection-outside-synthetic-root"))?;
    // Shell returns long paths even when TEMP used an 8.3 alias. Reconstruct the result
    // from the controller-owned root; file handle metadata checks remain mandatory.
    let expected = io("selection-host-directory", host.canonicalize())?;
    if io("selection-parent-directory", parent.canonicalize())? != expected {
        return Err(invalid("selection-outside-synthetic-root"));
    }
    Ok(host.join(name))
}

fn host_for(root: &Path) -> Result<PathBuf> {
    let suffix = root
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("kraqel-isolation-"))
        .ok_or(invalid("exchange-root-name"))?;
    Ok(root
        .parent()
        .ok_or(invalid("exchange-root-parent"))?
        .join(format!("kraqel-exchange-host-{suffix}")))
}

fn file_info(file: &fs::File) -> Result<BY_HANDLE_FILE_INFORMATION> {
    let mut info = unsafe { zeroed() };
    check(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
        "selected-file-info",
    )?;
    // Reject reparse/cloud placeholders, directories and multiply-linked files, without hydrating.
    let forbidden = FILE_ATTRIBUTE_REPARSE_POINT
        | FILE_ATTRIBUTE_DIRECTORY
        | FILE_ATTRIBUTE_OFFLINE
        | FILE_ATTRIBUTE_RECALL_ON_OPEN
        | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS;
    if info.dwFileAttributes & forbidden != 0 || info.nNumberOfLinks != 1 {
        return Err(invalid("selected-file-not-plain-local"));
    }
    Ok(info)
}
struct Pending {
    path: PathBuf,
    file: Option<fs::File>,
}
impl Pending {
    fn create(path: PathBuf) -> Result<Self> {
        let file = io(
            "create-selected-output",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(&path),
        )?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }
    fn close(&mut self) {
        self.file.take();
    }
    fn publish(&mut self, destination: &Path) -> Result<()> {
        self.close();
        check(
            unsafe {
                MoveFileExW(
                    wide(&self.path).as_ptr(),
                    wide(destination).as_ptr(),
                    MOVEFILE_WRITE_THROUGH,
                )
            },
            "exchange-publish-no-replace",
        )
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.close();
        let _ = fs::remove_file(&self.path);
    }
}
// Even a failure before job assignment must terminate the still-suspended child.
struct Worker(Handle, Handle);
impl Drop for Worker {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0.0, 1);
            WaitForSingleObject(self.0.0, 5000);
        }
    }
}

pub(super) fn child(args: &[OsString]) -> Result<Value> {
    let data = Path::new(&args[0]);
    let token = token_facts(unsafe { GetCurrentProcess() }, None)?;
    if token["isAppContainer"] != true || token["capabilityCount"] != 0 {
        return Err(invalid("exchange-child-token"));
    }
    let parse = |value: &OsString| {
        value
            .to_str()
            .and_then(|n| n.parse::<usize>().ok())
            .filter(|n| *n != 0 && *n != usize::MAX)
            .ok_or(invalid("exchange-child-argument"))
    };
    let input_handle = parse(&args[1])?;
    let output_handle = parse(&args[2])?;
    if input_handle == output_handle {
        return Err(invalid("exchange-duplicate-handle"));
    }
    let expected = parse(&args[3])? as u64;
    let mut input = unsafe { fs::File::from_raw_handle(input_handle as HANDLE) };
    let mut output = unsafe { fs::File::from_raw_handle(output_handle as HANDLE) };
    file_info(&input)?;
    file_info(&output)?;
    let host = host_for(data.parent().ok_or(invalid("exchange-data-root"))?)?;
    let denied = fs::File::open(host.join("withheld.txt"))
        .err()
        .and_then(|e| e.raw_os_error())
        == Some(5);
    if !denied {
        return Err(invalid("unselected-file-not-denied"));
    }
    let hash = io(
        "bounded-copy",
        transfer::copy_exact(&mut input, &mut output, expected),
    )?;
    io("flush-exchange", output.sync_all())?;
    let report = json!({"bytes":expected,"sha256":hash,"unselectedFileAccessDenied":denied,"token":token,"passed":true});
    write_json(&data.join("exchange.json"), &report)?;
    Ok(json!({"exchangeChildComplete":true}))
}

pub(super) fn launch(root: &Path) -> Result<Value> {
    launch_inner(root, false)
}
pub(super) fn launch_fixture(root: &Path) -> Result<Value> {
    launch_inner(root, true)
}
fn launch_inner(root: &Path, fixture: bool) -> Result<Value> {
    if !crate::policy::valid_root(root, false) {
        return Err(invalid("exchange-needs-fresh-temp-root"));
    }
    let host = host_for(root)?;
    // Separate sibling: no inheritable package ACL may reach unselected host files.
    io("create-host-root", fs::create_dir(&host))?;
    io(
        "seed-input",
        fs::write(
            host.join("input.txt"),
            "KRAQEL Austauschprobe: ÄÖÜ ß Ελληνικά Кириллица.\n",
        ),
    )?;
    io(
        "seed-withheld",
        fs::write(
            host.join("withheld.txt"),
            "Not selected. No package grant.\n",
        ),
    )?;
    let cancelled = || json!({"exchange":true,"cancelled":true,"passed":true,"transferred":false,"productionAcceptance":false});
    let Some(source) = (if fixture {
        Some(host.join("input.txt"))
    } else {
        choose(&host, false)?
    }) else {
        return Ok(cancelled());
    };
    if source.file_name() != Some(std::ffi::OsStr::new("input.txt")) {
        return Err(invalid("exchange-only-synthetic-input"));
    }
    let Some(destination) = (if fixture {
        Some(host.join("output.txt"))
    } else {
        choose(&host, true)?
    }) else {
        return Ok(cancelled());
    };
    if destination.exists() {
        return Err(invalid("exchange-refuses-overwrite"));
    }
    let input = io(
        "open-selected-input",
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(source),
    )?;
    let info = file_info(&input)?;
    let length = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
    if length == 0 || length > transfer::LIMIT {
        return Err(invalid("exchange-size-budget"));
    }
    // create_new and a no-replace rename protect existing targets, including races.
    let mut pending = Pending::create(host.join("pending-transfer.tmp"))?;
    let output = pending.file.as_ref().unwrap();
    file_info(output)?;
    let diagnostics = run_worker(
        root,
        &input,
        output,
        "exchange-child",
        &length.to_string(),
        |_| Ok(()),
    )?;
    drop(input);
    pending.close();
    let data = root.join("data");
    let mut report_bytes = Vec::new();
    io(
        "exchange-worker-report",
        io(
            "open-exchange-report",
            fs::File::open(data.join("exchange.json")),
        )?
        .take(4097)
        .read_to_end(&mut report_bytes),
    )?;
    if report_bytes.len() > 4096 {
        return Err(invalid("exchange-report-too-large"));
    }
    let worker: Value =
        serde_json::from_slice(&report_bytes).map_err(|_| invalid("exchange-report-invalid"))?;
    if !transfer::valid_worker(&worker, length)
        || io("exchange-output-size", fs::metadata(&pending.path))?.len() != length
    {
        return Err(invalid("exchange-report-mismatch"));
    }
    pending.publish(&destination)?;
    let mut report = diagnostics;
    report["formatVersion"] = json!(1);
    report["exchange"] = json!(true);
    report["passed"] = json!(true);
    report["transferred"] = json!(true);
    report["inheritedFileHandles"] = json!(2);
    report["hostDirectoryAclGranted"] = json!(false);
    report["worker"] = worker;
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}

// Shared research runner: only static child modes and controller-generated numeric/UUID arguments.
fn run_worker(
    root: &Path,
    input: &fs::File,
    output: &fs::File,
    child_mode: &'static str,
    argument: &str,
    prepare: impl FnOnce(&Path) -> Result<()>,
) -> Result<Value> {
    run_program(
        root,
        input,
        output,
        Program::Probe {
            child_mode,
            argument,
        },
        prepare,
    )
}
enum Program<'a> {
    Probe {
        child_mode: &'static str,
        argument: &'a str,
    },
    Ui {
        executable: &'a Path,
        id: uuid::Uuid,
    },
    UiLive {
        executable: &'a Path,
        id: uuid::Uuid,
    },
}
fn run_program(
    root: &Path,
    input: &fs::File,
    output: &fs::File,
    program: Program<'_>,
    prepare: impl FnOnce(&Path) -> Result<()>,
) -> Result<Value> {
    run_program_serviced(root, input, output, program, prepare, || Ok(()))
}
fn run_program_serviced(
    root: &Path,
    input: &fs::File,
    output: &fs::File,
    program: Program<'_>,
    prepare: impl FnOnce(&Path) -> Result<()>,
    mut service: impl FnMut() -> Result<()>,
) -> Result<Value> {
    io("create-exchange-root", fs::create_dir(root))?;
    let suffix = root
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .strip_prefix("kraqel-isolation-")
        .unwrap();
    let mut profile = Profile::create(suffix)?;
    grant(root, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    let bin = root.join("bin");
    let data = root.join("data");
    io("create-exchange-bin", fs::create_dir(&bin))?;
    io("create-exchange-data", fs::create_dir(&data))?;
    grant(
        &data,
        profile.sid,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE | FILE_DELETE_CHILD,
    )?;
    low_label(&data)?;
    prepare(&data)?;
    let executable = bin.join(
        if matches!(program, Program::Ui { .. } | Program::UiLive { .. }) {
            "kraqel-desktop.exe"
        } else {
            "kraqel-isolation-probe.exe"
        },
    );
    let source = match &program {
        Program::Ui { executable, .. } | Program::UiLive { executable, .. } => {
            executable.to_path_buf()
        }
        _ => io("current-exe", std::env::current_exe())?,
    };
    io("stage-exchange-worker", fs::copy(source, &executable))?;
    let handles = [input.as_raw_handle(), output.as_raw_handle()];
    for handle in handles {
        check(
            unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
            "allow-selected-handle-inheritance",
        )?;
    }
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let attributes = Attributes::with_handles(&caps, &handles)?;
    let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup.lpAttributeList = attributes.ptr;
    let arguments = match &program {
        Program::Probe {
            child_mode,
            argument,
        } => format!(
            "\"{}\" {child_mode} \"{}\" {} {} {argument}",
            executable.display(),
            data.display(),
            handles[0] as usize,
            handles[1] as usize
        ),
        Program::Ui { id, .. } => format!(
            "\"{}\" ui-grant {} {} {id}",
            executable.display(),
            handles[0] as usize,
            handles[1] as usize
        ),
        Program::UiLive { id, .. } => format!(
            "\"{}\" ui-live {} {} {id}",
            executable.display(),
            handles[0] as usize,
            handles[1] as usize
        ),
    };
    let mut command = wide(arguments);
    let system = std::env::var_os("SystemRoot").unwrap_or("C:\\Windows".into());
    let mut environment = Vec::new();
    for (key, value) in [
        ("APPDATA", data.clone()),
        ("LOCALAPPDATA", data.clone()),
        ("PATH", PathBuf::from(&system).join("System32")),
        ("SystemRoot", PathBuf::from(&system)),
        ("TEMP", data.clone()),
        ("TMP", data.clone()),
        ("USERPROFILE", data.clone()),
        ("WINDIR", PathBuf::from(&system)),
    ] {
        environment.extend(wide(format!("{key}={}", value.display())));
    }
    environment.push(0);
    let job = Handle(unsafe { CreateJobObjectW(null(), null()) });
    if job.0.is_null() {
        return Err(fail("exchange-create-job"));
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
        "exchange-job-limit",
    )?;
    let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
    let created = unsafe {
        CreateProcessW(
            wide(&executable).as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_UNICODE_ENVIRONMENT
                | CREATE_SUSPENDED
                | CREATE_NO_WINDOW,
            environment.as_ptr().cast(),
            wide(&data).as_ptr(),
            &startup.StartupInfo,
            &mut info,
        )
    };
    // Preserve CreateProcess's error before subsequent Win32 calls.
    let creation_error = (created == 0).then(|| fail("exchange-create-worker"));
    let worker = (created != 0).then(|| Worker(Handle(info.hProcess), Handle(info.hThread)));
    for handle in handles {
        check(
            unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) },
            "reset-handle-inheritance",
        )?;
    }
    if let Some(error) = creation_error {
        return Err(error);
    }
    let worker = worker.ok_or(invalid("exchange-missing-worker"))?;
    let process = &worker.0;
    let thread_handle = &worker.1;
    if unsafe { AssignProcessToJobObject(job.0, process.0) } == 0 {
        let error = fail("exchange-assign-job");
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
        return Err(invalid("exchange-suspended-token"));
    }
    write_json(
        &root.join("launch.json"),
        &json!({"pid":info.dwProcessId,"token":token}),
    )?;
    if unsafe { ResumeThread(thread_handle.0) } == u32::MAX {
        return Err(fail("exchange-resume"));
    }
    let finished = if matches!(program, Program::Ui { .. } | Program::UiLive { .. }) {
        let start = Instant::now();
        loop {
            if unsafe { WaitForSingleObject(process.0, 100) } == WAIT_OBJECT_0 {
                break true;
            }
            service()?;
            if root.join("STOP").exists()
                || start.elapsed() > Duration::from_secs(600)
                || (start.elapsed() > Duration::from_secs(20)
                    && !data.join("ui-ready.json").exists())
            {
                break false;
            }
        }
    } else {
        (unsafe { WaitForSingleObject(process.0, 30000) }) == WAIT_OBJECT_0
    };
    if !finished {
        unsafe {
            TerminateJobObject(job.0, 1);
            WaitForSingleObject(process.0, 5000);
        }
    }
    let mut code = 1;
    check(
        unsafe { GetExitCodeProcess(process.0, &mut code) },
        "exchange-exit-code",
    )?;
    unsafe {
        TerminateJobObject(job.0, 1);
    }
    let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
    let cleanup_deadline = Instant::now() + Duration::from_secs(5);
    loop {
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
            "exchange-job-accounting",
        )?;
        if accounting.ActiveProcesses == 0 || Instant::now() >= cleanup_deadline {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    drop(worker);
    drop(job);
    let removed = accounting.ActiveProcesses == 0 && profile.remove();
    write_json(
        &root.join("worker-lifecycle.json"),
        &json!({"finished":finished,"processExitCode":code,"activeJobProcessesAtEnd":accounting.ActiveProcesses,"profileRemoved":removed}),
    )?;
    if !finished || code != 0 || !removed {
        return Err(invalid("exchange-worker-or-cleanup-failed"));
    }
    Ok(
        json!({"tokenBeforeResume":token,"processExitCode":code,"activeJobProcessesAtEnd":accounting.ActiveProcesses,"profileRemoved":removed,"productionAcceptance":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "kraqel-isolation-files-{}",
                uuid::Uuid::new_v4().simple()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Flat, exclusively synthetic fixtures, no recursive filesystem operation.
            for item in fs::read_dir(&self.0).unwrap() {
                fs::remove_file(item.unwrap().path()).unwrap();
            }
            fs::remove_dir(&self.0).unwrap();
        }
    }
    #[test]
    fn selection_accepts_resolved_temp_alias_but_rejects_a_sibling() {
        let root = Fixture::new();
        let sibling = Fixture::new();
        let canonical = root.0.canonicalize().unwrap();
        assert_eq!(
            scoped_selection(&canonical.join("input.kraqel"), &root.0, true).unwrap(),
            root.0.join("input.kraqel")
        );
        assert!(scoped_selection(&sibling.0.join("input.kraqel"), &root.0, true).is_err());
        assert!(scoped_selection(&canonical.join("input.kraqel:stream"), &root.0, true).is_err());
    }
    #[test]
    #[ignore = "opens and automatically cancels a native synthetic Shell dialog; not for CI"]
    fn native_owned_dialog_can_be_closed_and_joins() {
        let root = Fixture::new();
        let owner = unsafe { GetCurrentThreadId() };
        let closer = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                let posted = super::ui_live::close_owned_dialog(owner);
                if posted > 0 {
                    return posted;
                }
                if Instant::now() >= deadline {
                    return 0;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
        assert!(
            choose_named(&root.0, false, "input.kraqel", true)
                .unwrap()
                .is_none()
        );
        assert!(closer.join().unwrap() > 0);
    }
    #[test]
    #[ignore = "opens and automatically cancels native Shell dialogs; explicit local namespace comparison only"]
    fn native_dialog_canonical_namespace_comparison() {
        let root = Fixture::new();
        let canonical = root.0.canonicalize().unwrap();
        let owner = unsafe { GetCurrentThreadId() };
        let closer = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            while Instant::now() < deadline {
                let posted = super::ui_live::close_owned_dialog(owner);
                if posted > 0 {
                    return posted;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            0
        });
        let result = choose_named(&canonical, false, "input.kraqel", true);
        let posted = closer.join().unwrap();
        assert!(result.unwrap().is_none());
        assert!(posted > 0);
    }
    #[test]
    fn shell_folder_representation_preserves_canonical_identity_without_verbatim_prefix() {
        let root = Fixture::new();
        let canonical = root.0.canonicalize().unwrap();
        let plain = shell_folder_path(&canonical).unwrap();
        assert_eq!(plain.canonicalize().unwrap(), canonical);
        assert!(!plain.as_os_str().to_string_lossy().starts_with("\\\\?\\"));
    }
    #[test]
    fn pending_creation_never_owns_an_existing_file() {
        let root = Fixture::new();
        let path = root.0.join("existing.tmp");
        fs::write(&path, b"preserve").unwrap();
        assert!(Pending::create(path.clone()).is_err());
        assert_eq!(fs::read(path).unwrap(), b"preserve");
    }
    #[test]
    fn publication_cannot_replace_a_destination_created_during_transfer() {
        let root = Fixture::new();
        let path = root.0.join("pending.tmp");
        let destination = root.0.join("output.txt");
        {
            let mut pending = Pending::create(path.clone()).unwrap();
            pending.file.as_mut().unwrap().write_all(b"new").unwrap();
            fs::write(&destination, b"preserve").unwrap();
            assert!(pending.publish(&destination).is_err());
        }
        assert_eq!(fs::read(destination).unwrap(), b"preserve");
        assert!(!path.exists());
    }
    #[test]
    fn failed_copy_closes_and_removes_partial_output() {
        let root = Fixture::new();
        let path = root.0.join("pending.tmp");
        {
            let mut pending = Pending::create(path.clone()).unwrap();
            assert!(
                transfer::copy_exact(&b"short"[..], pending.file.as_mut().unwrap(), 20).is_err()
            );
        }
        assert!(!path.exists());
    }
    #[test]
    fn plain_file_metadata_rejects_hardlinks() {
        let root = Fixture::new();
        let path = root.0.join("input.txt");
        fs::write(&path, b"synthetic").unwrap();
        assert!(file_info(&fs::File::open(&path).unwrap()).is_ok());
        fs::hard_link(&path, root.0.join("alias.txt")).unwrap();
        assert!(file_info(&fs::File::open(path).unwrap()).is_err());
    }
}
