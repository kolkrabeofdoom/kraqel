//! Broker-only authenticator. The window receives an opaque ticket on its standard input.
use super::*;

const DOMAIN: &[u8] = b"KRAQEL-C4B-WINDOW-BOOTSTRAP\0\x01";

fn authenticator(
    store: &Store,
    image: &fs::File,
    pi: &PROCESS_INFORMATION,
    created: u64,
) -> Result<HmacSha256> {
    let (actor, anchor) = store
        .state
        .win32_anchor
        .ok_or(invalid("win32-bootstrap-anchor"))?;
    if actor != store.state.actor {
        return Err(invalid("win32-bootstrap-actor"));
    }
    let (volume, file_id) = secure_fs::file_id(image)?;
    let mut mac =
        HmacSha256::new_from_slice(&store.key[..]).map_err(|_| invalid("win32-bootstrap-key"))?;
    mac.update(DOMAIN);
    mac.update(store.state.installation.as_bytes());
    mac.update(&store.state.head);
    mac.update(&actor);
    mac.update(&anchor);
    mac.update(&volume.to_le_bytes());
    mac.update(&file_id);
    mac.update(&pi.dwProcessId.to_le_bytes());
    mac.update(&pi.dwThreadId.to_le_bytes());
    mac.update(&created.to_le_bytes());
    Ok(mac)
}

pub(super) fn stage(
    store: &Store,
    image: &fs::File,
    pi: &PROCESS_INFORMATION,
    endpoint: HANDLE,
    writer: &mut fs::File,
    surface: bool,
) -> Result<[u8; 32]> {
    let created = workspace_probe::lifetime::creation_time(pi.hProcess)?;
    let mac = authenticator(store, image, pi, created)?;
    // The receipt-only protocol exposes its tag. Derive the UI secret in a separate domain.
    let mut secret_mac = mac.clone();
    secret_mac.update(b"window-open-v1");
    let secret: [u8; 32] = secret_mac.finalize().into_bytes().into();
    let tag = if surface {
        crate::window_receipt::commitment(&secret)
    } else {
        mac.finalize().into_bytes().into()
    };
    io("win32-bootstrap-write", writer.write_all(&tag))?;

    // Inspect the actual recipient handle without consuming the ticket or running window code.
    let mut local = null_mut();
    check(
        unsafe {
            DuplicateHandle(
                pi.hProcess,
                endpoint,
                GetCurrentProcess(),
                &mut local,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        },
        "win32-bootstrap-recipient",
    )?;
    let local = Handle(local);
    let mut received = [0u8; 32];
    let (mut read, mut available) = (0, 0);
    check(
        unsafe {
            PeekNamedPipe(
                local.0,
                received.as_mut_ptr().cast(),
                32,
                &mut read,
                &mut available,
                null_mut(),
            )
        },
        "win32-bootstrap-readback",
    )?;
    if read != 32 || available != 32 {
        return Err(invalid("win32-bootstrap-size"));
    }
    if received != tag {
        return Err(invalid("win32-bootstrap-authentication"));
    }
    Ok(if surface { secret } else { tag })
}

pub(super) fn resume_and_confirm(
    pi: &PROCESS_INFORMATION,
    reply: &fs::File,
    expected: [u8; 32],
) -> Result<()> {
    if unsafe { ResumeThread(pi.hThread) } != 1 {
        return Err(invalid("win32-window-resume"));
    }
    if unsafe { WaitForSingleObject(pi.hProcess, 5000) } != WAIT_OBJECT_0 {
        return Err(invalid("win32-window-receipt-timeout"));
    }
    let mut exit = 0;
    check(
        unsafe { GetExitCodeProcess(pi.hProcess, &mut exit) },
        "win32-window-exit",
    )?;
    if exit != 0 {
        return Err(Failure {
            stage: "win32-window-receipt-exit",
            code: i64::from(exit),
        });
    }
    // After process exit the only writer is gone. Inspect the entire frame without waiting for EOF.
    let mut ack = [0u8; 32];
    let (mut read, mut available) = (0, 0);
    check(
        unsafe {
            PeekNamedPipe(
                reply.as_raw_handle(),
                ack.as_mut_ptr().cast(),
                32,
                &mut read,
                &mut available,
                null_mut(),
            )
        },
        "win32-window-receipt-read",
    )?;
    if read != 32 || available != 32 || ack != expected {
        return Err(invalid("win32-window-receipt-mismatch"));
    }
    Ok(())
}

// Read only complete fixed frames; the outer job tears down every timeout or protocol error.
fn live_frame<const N: usize>(reply: &mut fs::File, process: HANDLE) -> Result<[u8; N]> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut available = 0;
        check(
            unsafe {
                PeekNamedPipe(
                    reply.as_raw_handle(),
                    null_mut(),
                    0,
                    null_mut(),
                    &mut available,
                    null_mut(),
                )
            },
            "win32-window-live-pipe",
        )?;
        if available as usize == N {
            let mut frame = [0; N];
            io("win32-window-live-frame", reply.read_exact(&mut frame))?;
            return Ok(frame);
        }
        if available as usize > N
            || Instant::now() >= deadline
            || unsafe { WaitForSingleObject(process, 10) } != WAIT_TIMEOUT
        {
            return Err(invalid("win32-window-live-frame"));
        }
    }
}

pub(super) fn confirm_and_open(
    pi: &PROCESS_INFORMATION,
    mut writer: fs::File,
    mut reply: fs::File,
    secret: [u8; 32],
) -> Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, WM_CLOSE,
    };
    if unsafe { ResumeThread(pi.hThread) } != 1 {
        return Err(invalid("win32-window-resume"));
    }
    let expected = crate::window_receipt::receipt(&crate::window_receipt::commitment(&secret));
    if live_frame::<32>(&mut reply, pi.hProcess)? != expected {
        return Err(invalid("win32-window-receipt-mismatch"));
    }
    io("win32-window-open-grant", writer.write_all(&secret))?;
    drop(writer);
    let handle = u64::from_le_bytes(live_frame::<8>(&mut reply, pi.hProcess)?);
    let hwnd = usize::try_from(handle).map_err(|_| invalid("win32-window-handle"))? as HWND;
    let mut pid = 0;
    let mut class = [0u16; 64];
    let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    let len = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) } as usize;
    if pid != pi.dwProcessId
        || tid != pi.dwThreadId
        || unsafe { IsWindowVisible(hwnd) } == 0
        || !class[..len]
            .iter()
            .copied()
            .eq("KRAQEL.Synthetic.Edit".encode_utf16())
    {
        return Err(invalid("win32-window-surface-identity"));
    }
    check(
        unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) },
        "win32-window-surface-close",
    )?;
    if unsafe { WaitForSingleObject(pi.hProcess, 5000) } != WAIT_OBJECT_0 {
        return Err(invalid("win32-window-surface-timeout"));
    }
    let mut exit = 0;
    check(
        unsafe { GetExitCodeProcess(pi.hProcess, &mut exit) },
        "win32-window-exit",
    )?;
    if exit != 40 {
        return Err(invalid("win32-window-surface-exit"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reused_ids_cannot_authenticate_a_previous_window_generation() {
        let root = std::env::temp_dir().join(format!("kraqel-window-ticket-{}", Uuid::new_v4()));
        let mut store = Store::initialize(&root, [1; 32]).unwrap();
        store.state.win32_anchor = Some((store.state.actor, [2; 32]));
        let image = fs::File::open(std::env::current_exe().unwrap()).unwrap();
        let pi = PROCESS_INFORMATION {
            dwProcessId: 100,
            dwThreadId: 200,
            ..Default::default()
        };
        let previous = authenticator(&store, &image, &pi, 1)
            .unwrap()
            .finalize()
            .into_bytes();
        assert!(
            authenticator(&store, &image, &pi, 2)
                .unwrap()
                .verify_slice(&previous)
                .is_err()
        );
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }
}
