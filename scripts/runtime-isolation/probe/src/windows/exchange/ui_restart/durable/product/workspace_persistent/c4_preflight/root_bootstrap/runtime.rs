//! Explicit Fixed-Version WebView creation, blank content only, no Tauri/project commands.
use super::*;
use webview2_com::{
    CoreWebView2EnvironmentOptions, CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{Interface, PCWSTR};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

fn win<T>(stage: &'static str, r: windows::core::Result<T>) -> Result<T> {
    r.map_err(|e| Failure {
        stage,
        code: e.code().0.into(),
    })
}
fn wait<T>(rx: std::sync::mpsc::Receiver<windows::core::Result<T>>) -> Result<T> {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        match rx.try_recv() {
            Ok(r) => return win("c4-webview-callback", r),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Err(invalid("c4-webview-disconnected"));
            }
            Err(_) => {}
        }
        if Instant::now() >= deadline {
            return Err(invalid("c4-webview-timeout"));
        }
        let mut msg = unsafe { zeroed() };
        while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
}
pub(super) fn blank(runtime: &Path, data: &Path) -> Result<Value> {
    // Defense in depth: no direct internal call may skip BP3, even for a blank WebView.
    require_preventive_runtime()?;
    win(
        "c4-webview-com",
        unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
            )
        }
        .ok(),
    )?;
    struct Com;
    impl Drop for Com {
        fn drop(&mut self) {
            unsafe { windows::Win32::System::Com::CoUninitialize() }
        }
    }
    let _com = Com;
    let browser = wide(runtime);
    let data = wide(data);
    let options: ICoreWebView2EnvironmentOptions = CoreWebView2EnvironmentOptions::default().into();
    let (tx, rx) = std::sync::mpsc::channel();
    let handler =
        CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |error, env| {
            let result = error.and_then(|()| {
                env.ok_or_else(|| {
                    windows::core::Error::from_hresult(windows::core::HRESULT(0x80004003u32 as i32))
                })
            });
            let _ = tx.send(result);
            Ok(())
        }));
    win("c4-webview-environment", unsafe {
        CreateCoreWebView2EnvironmentWithOptions(
            PCWSTR(browser.as_ptr()),
            PCWSTR(data.as_ptr()),
            &options,
            &handler,
        )
    })?;
    let environment = wait(rx)?;
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            wide("KRAQEL C4 blank runtime research").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            320,
            200,
            null_mut(),
            null_mut(),
            null_mut(),
            null(),
        )
    };
    if hwnd.is_null() {
        return Err(fail("c4-webview-window"));
    }
    struct Window(HANDLE);
    impl Drop for Window {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.0);
            }
        }
    }
    let _window = Window(hwnd);
    let (tx, rx) = std::sync::mpsc::channel();
    let handler =
        CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |error, controller| {
            let result = error.and_then(|()| {
                controller.ok_or_else(|| {
                    windows::core::Error::from_hresult(windows::core::HRESULT(0x80004003u32 as i32))
                })
            });
            let _ = tx.send(result);
            Ok(())
        }));
    win("c4-webview-controller", unsafe {
        environment.CreateCoreWebView2Controller(windows::Win32::Foundation::HWND(hwnd), &handler)
    })?;
    let controller = wait(rx)?;
    let view = win("c4-webview-view", unsafe { controller.CoreWebView2() })?;
    let settings = win("c4-webview-settings", unsafe { view.Settings() })?;
    win("c4-webview-devtools", unsafe {
        settings.SetAreDevToolsEnabled(false)
    })?;
    win("c4-webview-script", unsafe {
        settings.SetIsScriptEnabled(false)
    })?;
    win("c4-webview-webmessages", unsafe {
        settings.SetIsWebMessageEnabled(false)
    })?;
    win("c4-webview-blank", unsafe {
        view.Navigate(windows::core::w!("about:blank"))
    })?;
    let extended: ICoreWebView2Environment8 = win("c4-webview-process-api", environment.cast())?;
    let mut roles = std::collections::BTreeMap::new();
    let mut samples = 0;
    let until = Instant::now() + Duration::from_secs(2);
    let mut next_sample = Instant::now();
    while Instant::now() < until {
        let mut msg = unsafe { zeroed() };
        while unsafe { PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) } != 0 {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if Instant::now() >= next_sample {
            process_roles(&extended, &mut roles)?;
            samples += 1;
            next_sample = Instant::now() + Duration::from_millis(100);
        }
        thread::sleep(Duration::from_millis(10));
    }
    process_roles(&extended, &mut roles)?;
    samples += 1;
    win("c4-webview-close", unsafe { controller.Close() })?;
    let roles: Vec<_> = roles
        .into_iter()
        .map(|((pid, created), kind)| json!({"pid":pid,"created":created,"kind":kind}))
        .collect();
    Ok(
        json!({"runtimeCreated":true,"error":0,"webviewProcesses":roles,"processInfoSamples":samples,"roleObservationOnly":true}),
    )
}
fn process_roles(
    extended: &ICoreWebView2Environment8,
    roles: &mut std::collections::BTreeMap<(u32, u64), i32>,
) -> Result<()> {
    let processes = win("c4-webview-process-infos", unsafe {
        extended.GetProcessInfos()
    })?;
    let mut count = 0;
    win("c4-webview-process-count", unsafe {
        processes.Count(&mut count)
    })?;
    if count > 16 {
        return Err(invalid("c4-webview-process-bound"));
    }
    for n in 0..count {
        let info = win("c4-webview-process-info", unsafe {
            processes.GetValueAtIndex(n)
        })?;
        let mut pid = 0;
        let mut kind = COREWEBVIEW2_PROCESS_KIND_BROWSER;
        win("c4-webview-process-pid", unsafe {
            info.ProcessId(&mut pid)
        })?;
        win("c4-webview-process-kind", unsafe { info.Kind(&mut kind) })?;
        if pid <= 0 {
            return Err(invalid("c4-webview-process-pid"));
        }
        // Join API claims to a native generation, never to a bare recyclable PID.
        let process = Handle(unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                pid as u32,
            )
        });
        if process.0.is_null() {
            return Err(fail("c4-webview-role-process"));
        }
        let id = identity(process.0)?;
        if unsafe { WaitForSingleObject(process.0, 0) } != WAIT_TIMEOUT {
            return Err(invalid("c4-webview-role-process-ended"));
        }
        record_role(roles, id, kind.0)?;
    }
    Ok(())
}
fn record_role(
    roles: &mut std::collections::BTreeMap<(u32, u64), i32>,
    id: Identity,
    kind: i32,
) -> Result<()> {
    if !(0..=6).contains(&kind) {
        return Err(invalid("c4-webview-role-kind"));
    }
    let key = (id.pid, id.created);
    if let Some(previous) = roles.get(&key) {
        if *previous != kind {
            return Err(invalid("c4-webview-role-changed"));
        }
    } else {
        if roles.len() >= 32 {
            return Err(invalid("c4-webview-role-lifetime-bound"));
        }
        roles.insert(key, kind);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c4_roles_bind_generation_reject_conflicts_unknown_kinds_and_overflow() {
        let mut roles = std::collections::BTreeMap::new();
        let id = Identity {
            pid: 42,
            created: 1,
        };
        record_role(&mut roles, id, 0).unwrap();
        record_role(&mut roles, id, 0).unwrap();
        assert!(record_role(&mut roles, id, 1).is_err());
        record_role(&mut roles, Identity { created: 2, ..id }, 1).unwrap();
        assert!(record_role(&mut roles, Identity { created: 3, ..id }, 7).is_err());
        for created in 3..=32 {
            record_role(&mut roles, Identity { created, ..id }, 2).unwrap();
        }
        assert!(record_role(&mut roles, Identity { created: 33, ..id }, 2).is_err());
        assert_eq!(roles.len(), 32);
    }
}
