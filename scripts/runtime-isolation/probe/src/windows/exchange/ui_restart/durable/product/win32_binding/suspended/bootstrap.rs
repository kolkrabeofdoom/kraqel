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
) -> Result<[u8; 32]> {
    let created = workspace_probe::lifetime::creation_time(pi.hProcess)?;
    let tag = authenticator(store, image, pi, created)?
        .finalize()
        .into_bytes();
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
    authenticator(
        store,
        image,
        pi,
        workspace_probe::lifetime::creation_time(pi.hProcess)?,
    )?
    .verify_slice(&received)
    .map_err(|_| invalid("win32-bootstrap-authentication"))?;
    Ok(crate::window_receipt::receipt(&received))
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
