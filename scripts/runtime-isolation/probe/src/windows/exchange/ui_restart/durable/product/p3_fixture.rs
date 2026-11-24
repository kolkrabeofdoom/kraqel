//! Synthetic local copy controller. The private store is never an ordinary preview installation.
use super::*;

pub(super) fn serve(
    preview: &Path,
    image: &fs::File,
    actor: [u8; 32],
    create: bool,
    cleanup: bool,
) -> Result<Value> {
    let home = preview.parent().ok_or(invalid("p3-fixture-home"))?;
    let fixture = home.parent().ok_or(invalid("p3-fixture-root"))?;
    let parent = fixture.parent().ok_or(invalid("p3-temp-parent"))?;
    let parent_pins = pin_directories(parent)?;
    let temp_pins = pin_directories(&std::env::temp_dir())?;
    let identity = |f: &fs::File| -> Result<(u32, u32, u32)> {
        let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
        check(
            unsafe { GetFileInformationByHandle(f.as_raw_handle(), &mut i) },
            "p3-file-identity",
        )?;
        Ok((i.dwVolumeSerialNumber, i.nFileIndexHigh, i.nFileIndexLow))
    };
    let valid = fixture
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("kraqel-p3-"))
        .is_some_and(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()));
    if !valid
        || identity(parent_pins.last().unwrap())? != identity(temp_pins.last().unwrap())?
        || !matches!(
            home.file_name().and_then(|s| s.to_str()),
            Some("install-a" | "install-b")
        )
    {
        return Err(invalid("p3-fixture-scope"));
    }
    let (_pins, expected, _) = binary_pin(&home.join("p3-starter.exe"))?;
    if identity(&expected)? != identity(image)? {
        return Err(invalid("p3-starter-identity"));
    }
    if [
        std::io::stdin().as_raw_handle(),
        std::io::stdout().as_raw_handle(),
    ]
    .iter()
    .any(|h| unsafe { GetFileType(*h) } != FILE_TYPE_PIPE)
    {
        return Err(invalid("p3-pipe-required"));
    }
    let path = home.join("p3-control");
    let mut store = if create {
        Store::initialize(&path, actor)?
    } else {
        Store::open(&path)?
    };
    store.actor(actor)?;
    store.append_guard(actor, store.state.head)?;
    if (cleanup && store.state.p3_evidence[2].is_some())
        || (!cleanup && store.state.p3_cleanup.is_some())
    {
        return Err(invalid("p3-terminal-decision"));
    }
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    io("p3-ready", output.write_all(b"P3/1\n"))?;
    io("p3-ready", output.flush())?;
    // Retain owner.lock for the lifetime of the local pipe. Only fixed-size hashes and
    // purpose-specific receipts cross it. EOF never creates or completes evidence.
    let mut verified = [false; 3];
    loop {
        let mut frame = [0u8; 34];
        if io("p3-input", input.read(&mut frame[..1]))? == 0 {
            break;
        }
        io("p3-frame", input.read_exact(&mut frame[1..]))?;
        let [mode, slot] = [frame[0], frame[1]];
        // Cleanup and publication are mutually exclusive terminal decisions.
        if (slot == 3 && !cleanup) || (cleanup && (slot == 2 || (mode == 1 && slot != 3))) {
            return Err(invalid("p3-operation"));
        }
        let index = if slot == 3 { 2 } else { usize::from(slot) };
        if mode > 1 || index >= 3 || verified[..index].contains(&false) {
            return Err(invalid("p3-request-order"));
        }
        let digest: [u8; 32] = frame[2..].try_into().unwrap();
        let saved = if slot == 3 {
            store.state.p3_cleanup
        } else {
            store.state.p3_evidence[index]
        };
        match saved {
            Some(saved) if saved == digest => (),
            None if mode == 1 && (slot != 0 || create) => {
                store.append(
                    actor,
                    store.state.head,
                    if slot == 3 {
                        Action::P3FixtureCleanup { digest }
                    } else {
                        Action::P3FixtureEvidence { slot, digest }
                    },
                )?;
            }
            _ => return Err(invalid("p3-unbound-evidence")),
        }
        verified[index] = true;
        let mut mac =
            HmacSha256::new_from_slice(store.key.as_ref()).map_err(|_| invalid("p3-key"))?;
        mac.update(b"KRAQEL-P3-FIXTURE-ATTESTATION\0\x01");
        mac.update(store.state.installation.as_bytes());
        mac.update(&[slot]);
        mac.update(&digest);
        io(
            "p3-reply",
            writeln!(output, "{}", hex(&mac.finalize().into_bytes().into())),
        )?;
        io("p3-reply", output.flush())?;
    }
    Ok(json!({"synthetic":true,"networkMeasurement":false}))
}
