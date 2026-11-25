//! Synthetic local copy controller. The private store is never an ordinary preview installation.
use super::*;

#[derive(PartialEq, Eq)]
pub(super) enum Operation {
    Authorize,
    AuthorizeComparison,
    ReadComparison,
    Reopen,
    Cleanup,
    Read,
    Profiles,
}

pub(super) fn serve(
    preview: &Path,
    image: &fs::File,
    actor: [u8; 32],
    operation: Operation,
) -> Result<Value> {
    let comparison = matches!(
        operation,
        Operation::AuthorizeComparison | Operation::ReadComparison
    );
    let create = matches!(
        operation,
        Operation::Authorize | Operation::AuthorizeComparison
    );
    let read = matches!(operation, Operation::Read | Operation::ReadComparison);
    let cleanup = operation == Operation::Cleanup;
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
        || (home.file_name().and_then(|s| s.to_str()) == Some("comparison-install")) != comparison
        || !matches!(
            home.file_name().and_then(|s| s.to_str()),
            Some("install-a" | "install-b" | "selection-install" | "comparison-install")
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
    if operation == Operation::Profiles {
        let begin = store
            .state
            .p3_pipe_begin
            .ok_or(invalid("p3-profile-generation"))?;
        store.idle()?;
        Store::job_idle(&wide(format!(
            "Local\\KRAQEL.Scene.{}",
            store.state.installation.simple()
        )))?;
        // Check both identities before any removal; a missing creation witness never grants deletion.
        let mut owned = Vec::new();
        for data in [false, true] {
            let identity = derived_profile(begin, data)?;
            let current = folder_identity(&store, &identity)?;
            let expected = store.state.p3_profiles[usize::from(data)];
            if current.is_some() && current != expected {
                return Err(invalid("p3-unowned-profile"));
            }
            if let Some(folder) = expected {
                owned.push((data, identity, folder));
            }
        }
        let mut deleted = 0;
        for (data, identity, folder) in owned {
            deleted += usize::from(retire_profile(&mut store, begin, data, &identity, folder)?);
        }
        return Ok(
            json!({"synthetic":true, "profileFoldersAbsent":true, "deletedProfiles":deleted,
            "sessionOpen":matches!(store.state.window_session, win32_binding::Session::Open(_))}),
        );
    }
    store.append_guard(actor, store.state.head)?;
    if read && store.state.p3_evidence[2].is_none() {
        return Err(invalid("p3-publication-required"));
    }
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
        if read && slot == 6 && mode == 0 && verified == [true; 3] {
            use crate::window_receipt as protocol;
            let row = io("p3-row", protocol::read_p3(&mut input, comparison))?;
            let root = store.root.join("bin");
            let image_path = root.join("pipe-service.exe");
            let (_dirs, image, digest) = binary_pin(&image_path)?;
            let (_window_dirs, window, window_digest) = binary_pin(&root.join("native-leaf.exe"))?;
            let binding: [u8; 32] = Sha256::new()
                .chain_update(if comparison {
                    b"KRAQEL-P3-COMPARISON-v2".as_slice()
                } else {
                    b"KRAQEL-P3-PREVIEW-v2".as_slice()
                })
                .chain_update(&frame[2..])
                .chain_update(digest)
                .chain_update(window_digest)
                .chain_update(&row)
                .finalize()
                .into();
            if !protocol::valid_p3_row(&row, comparison)
                || Some(binding) != store.state.p3_evidence[0]
                || comparison
                    && (store.state.p3_evidence[1].as_ref().map(|v| v.as_slice())
                        != Some(&row[32..64])
                        || store.state.p3_evidence[2].as_ref().map(|v| v.as_slice())
                            != Some(&row[64..96]))
            {
                return Err(invalid("p3-unbound-row"));
            }
            if store.state.sequence + 6 >= MAX_RECORDS {
                return Err(invalid("p3-delivery-budget"));
            }
            store.append(
                actor,
                store.state.head,
                Action::BeginP3PipePreview {
                    binding,
                    packages: packages_folder()?.1,
                },
            )?;
            let begin = store.state.head;
            io("p3-delivery-ready", output.write_all(b"BOUND\n"))?;
            io("p3-delivery-ready", output.flush())?;
            let mut start = [0];
            io("p3-delivery-start", input.read_exact(&mut start))?;
            let interactive = match start {
                [1] => false,
                [2] => true,
                _ => return Err(invalid("p3-delivery-start")),
            };
            let (choice, _) = win32_binding::deliver_p3(
                &mut store,
                &root,
                &image,
                &window,
                binding,
                &row,
                interactive,
                None,
                || {
                    io("p3-visible", output.write_all(b"VISIBLE\n"))?;
                    io("p3-visible", output.flush())
                },
            )?;
            store.append_with_checked_context(
                actor,
                store.state.head,
                Action::EndWin32Window { begin },
            )?;
            if comparison {
                io("p3-comparison-choice", output.write_all(&[choice]))?;
            }
            io("p3-service-confirmed", output.write_all(b"DELIVERED\n"))?;
            io("p3-service-confirmed", output.flush())?;
            break;
        }
        if read && (mode != 0 || slot > 2) {
            return Err(invalid("p3-read-only"));
        }
        if slot == 4 || slot == 5 {
            // The cursor is accessible only after verifying the exact cleanup decision.
            if !cleanup || !verified[2] || mode > 1 || frame[6..].iter().any(|b| *b != 0) {
                return Err(invalid("p3-removal-order"));
            }
            let step = u32::from_le_bytes(frame[2..6].try_into().unwrap());
            let cursor = store.state.p3_removal;
            if slot == 5 {
                if mode == 1 && !store.state.p3_removed {
                    store.append(
                        actor,
                        store.state.head,
                        Action::P3FixtureRemoved { count: step },
                    )?;
                }
                if store.state.p3_removed && step != cursor {
                    return Err(invalid("p3-removal-count"));
                }
                let reply: &[u8] = if store.state.p3_removed {
                    b"DONE\n"
                } else {
                    b"OPEN\n"
                };
                io("p3-final-reply", output.write_all(reply))?;
                io("p3-final-reply", output.flush())?;
                continue;
            }
            if store.state.p3_removed {
                return Err(invalid("p3-removal-closed"));
            }
            if mode == 0 {
                // At most one deletion may have been authorized but not yet completed.
                if step != cursor && step.checked_add(1) != Some(cursor) {
                    return Err(invalid("p3-removal-remainder"));
                }
            } else if step != cursor {
                store.append(actor, store.state.head, Action::P3FixtureRemoval { step })?;
            }
            io("p3-removal-reply", output.write_all(b"OK\n"))?;
            io("p3-removal-reply", output.flush())?;
            continue;
        }
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

pub(super) fn retire_profile(
    store: &mut Store,
    begin: [u8; 32],
    data: bool,
    identity: &Profile,
    folder: [u8; 32],
) -> Result<bool> {
    let current = folder_identity(store, identity)?;
    if let Some(current) = current {
        if folder != current {
            return Err(invalid("p3-replaced-profile"));
        }
        let hr = unsafe { DeleteAppContainerProfile(identity.name.as_ptr()) };
        if hr < 0 {
            return Err(Failure {
                stage: "p3-profile-delete",
                code: hr.into(),
            });
        }
    }
    if folder_identity(store, identity)?.is_some() {
        return Err(invalid("p3-profile-remains"));
    }
    store.append_with_checked_context(
        store.state.actor,
        store.state.head,
        Action::P3Profile {
            begin,
            data,
            folder,
            removed: true,
        },
    )?;
    Ok(current.is_some())
}

pub(super) fn profile_id(begin: [u8; 32], data: bool) -> String {
    let id = Uuid::from_bytes(begin[..16].try_into().unwrap());
    format!("{}.{}", if data { "data" } else { "ui" }, id.simple())
}

fn derived_profile(begin: [u8; 32], data: bool) -> Result<Profile> {
    let name = wide(format!("org.kraqel.probe.{}", profile_id(begin, data)));
    let mut sid = null_mut();
    let hr = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
    if hr < 0 {
        return Err(Failure {
            stage: "p3-profile-sid",
            code: hr.into(),
        });
    }
    Ok(Profile {
        name,
        sid,
        active: false,
    })
}

pub(super) fn packages_folder() -> Result<(PathBuf, (u64, [u8; 16]), Vec<fs::File>)> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::System::Environment::ExpandEnvironmentStringsForUserW;
    // AppContainer profiles belong to the Windows account. A redirected desktop
    // project directory must never redefine the trusted parent for profile removal.
    let mut raw = null_mut();
    check(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_DUPLICATE, &mut raw) },
        "p3-profile-account-token",
    )?;
    let token = Handle(raw);
    let name = wide("%LOCALAPPDATA%");
    let mut buf = vec![0u16; 32768];
    check(
        unsafe {
            ExpandEnvironmentStringsForUserW(
                token.0,
                name.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as u32,
            )
        },
        "p3-profile-account-path",
    )?;
    let len = buf
        .iter()
        .position(|c| *c == 0)
        .ok_or(invalid("p3-profile-account-path"))?;
    let base = PathBuf::from(std::ffi::OsString::from_wide(&buf[..len]));
    let path = local_path(&base.join("Packages"))?;
    let pins = pin_directories(&path)?;
    let id = secure_fs::file_id(pins.last().ok_or(invalid("p3-packages-pin"))?)?;
    Ok((path, id, pins))
}

pub(super) fn folder_identity(store: &Store, profile: &Profile) -> Result<Option<[u8; 32]>> {
    let name = String::from_utf16(&profile.name[..profile.name.len() - 1])
        .map_err(|_| invalid("p3-profile-name"))?;
    let (packages, id, _pins) = packages_folder()?;
    if Some(id) != store.state.p3_packages {
        return Err(invalid("p3-profile-parent"));
    }
    let mut text = null_mut();
    check(
        unsafe { ConvertSidToStringSidW(profile.sid, &mut text) },
        "p3-profile-sid-text",
    )?;
    let _text = Local(text.cast());
    let mut folder = null_mut();
    let hr = unsafe { GetAppContainerFolderPath(text, &mut folder) };
    if hr as u32 == 0x80070002 {
        return match fs::symlink_metadata(local_path(&packages.join(&name))?) {
            Err(e) if e.raw_os_error() == Some(ERROR_FILE_NOT_FOUND as i32) => Ok(None),
            _ => Err(invalid("p3-profile-absence-unconfirmed")),
        };
    }
    if hr < 0 || folder.is_null() {
        return Err(Failure {
            stage: "p3-profile-folder",
            code: hr.into(),
        });
    }
    let path = unsafe { ::windows::core::PWSTR(folder).to_string() };
    unsafe { ::windows::Win32::System::Com::CoTaskMemFree(Some(folder.cast())) };
    let path = PathBuf::from(path.map_err(|_| invalid("p3-profile-path"))?);
    let root = path.parent().ok_or(invalid("p3-profile-root"))?;
    if path.file_name().and_then(|s| s.to_str()) != Some("AC")
        || !root
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case(&name))
    {
        return Err(invalid("p3-profile-layout"));
    }
    let pins = pin_directories(&path)?;
    // Windows resolves the AC subdirectory, while profile removal includes its parent.
    let [.., parent, package, ac] = pins.as_slice() else {
        return Err(invalid("p3-profile-folder-pin"));
    };
    if secure_fs::file_id(parent)? != id {
        return Err(invalid("p3-profile-parent"));
    }
    let mut digest = Sha256::new();
    for directory in [package, ac] {
        let (volume, file) = secure_fs::file_id(directory)?;
        digest.update(volume.to_le_bytes());
        digest.update(file);
    }
    Ok(Some(digest.finalize().into()))
}
