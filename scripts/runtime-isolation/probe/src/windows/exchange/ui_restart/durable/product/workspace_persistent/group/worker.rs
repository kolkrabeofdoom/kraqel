//! Only pinned synthetic native roles. No GUI or generic network-probe dispatch.
use super::*;
pub(super) const LEAF_TEXT: &[u8] =
    b"C3: authenticated synthetic leaf completed after root exit.\n";
const ROOT_TEXT: &[u8] = b"C3: authenticated continue received before data access.\n";

fn handle_args(args: &[OsString]) -> Result<Vec<HANDLE>> {
    let mut out = Vec::new();
    for arg in args {
        let value = arg
            .to_str()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0 && *v != usize::MAX)
            .ok_or(invalid("c3-worker-handle"))?;
        if out.contains(&(value as HANDLE)) {
            return Err(invalid("c3-worker-handle-alias"));
        }
        out.push(value as HANDLE);
    }
    Ok(out)
}
pub(in crate::windows) fn worker(args: &[OsString]) -> Result<()> {
    let role = args
        .first()
        .and_then(|s| s.to_str())
        .ok_or(invalid("c3-worker-role"))?;
    let count = match role {
        "root" => 4,
        "leaf" => 3,
        _ => return Err(invalid("c3-worker-role")),
    };
    if args.len() != count + 2 {
        return Err(invalid("c3-worker-arguments"));
    }
    let facts = token_facts(unsafe { GetCurrentProcess() }, None)?;
    if facts["isAppContainer"] != true || facts["capabilityCount"] != 0 {
        return Err(invalid("c3-worker-uncontained"));
    }
    let raw = handle_args(&args[1..=count])?;
    inherited(&raw, false)?;
    let mut boot = unsafe { fs::File::from_raw_handle(raw[0]) };
    let mut secret = unsafe { fs::File::from_raw_handle(raw[1]) };
    let mut events = unsafe { fs::File::from_raw_handle(raw[2]) };
    let mut go = if count == 4 {
        Some(unsafe { fs::File::from_raw_handle(raw[3]) })
    } else {
        None
    };
    let peers = [peer(&boot)?, peer(&secret)?];
    let bytes = Zeroizing::new({
        let mut b = Vec::new();
        io("c3-worker-key", (&mut secret).take(33).read_to_end(&mut b))?;
        b
    });
    let key: &[u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| invalid("c3-worker-key-size"))?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let grant: Grant = decode(&read_frame(&mut boot, deadline)?, key)?;
    drop(boot);
    drop(secret);
    let launch = Uuid::parse_str(
        args[count + 1]
            .to_str()
            .ok_or(invalid("c3-worker-launch"))?,
    )
    .map_err(|_| invalid("c3-worker-launch"))?;
    if grant.version != 1
        || grant.purpose != "c3-synthetic-group"
        || grant.role != role
        || grant.launch.id != *launch.as_bytes()
        || grant.launch.generation == 0
        || grant.epoch.is_nil()
        || grant.nonce == [0; 32]
        || grant.binding_sha256 == [0; 32]
        || !grant.guardian.valid()
        || peers != [grant.parent_pid; 2]
        || grant.parent_pid == unsafe { GetCurrentProcessId() }
        || grant.profile_name != name(grant.workspace)
    {
        return Err(invalid("c3-worker-grant"));
    }
    if role == "root"
        && (grant.parent_pid != grant.guardian.pid
            || peer(go.as_ref().unwrap())? != grant.guardian.pid)
    {
        return Err(invalid("c3-worker-guardian-peer"));
    }
    let p = profile(grant.workspace)?;
    let facts = token_facts(unsafe { GetCurrentProcess() }, Some(p.sid))?;
    if facts["expectedIdentity"] != true {
        return Err(invalid("c3-worker-profile"));
    }
    let mut assigned = 0;
    check(
        unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut assigned) },
        "c3-worker-job",
    )?;
    if assigned == 0 {
        return Err(invalid("c3-worker-no-job"));
    }
    let root = local_path(&grant.root)?;
    if root.file_name()
        != Some(std::ffi::OsStr::new(&format!(
            "kraqel-isolation-p2c-{}",
            grant.workspace.simple()
        )))
    {
        return Err(invalid("c3-worker-root"));
    }
    let exe = io("c3-worker-exe", std::env::current_exe())?;
    if local_path(&exe)? != root.join("c3-code/worker.exe") || hash(&exe)? != grant.code.sha256 {
        return Err(invalid("c3-worker-code"));
    }
    let me = identity(unsafe { GetCurrentProcess() })?;
    if role == "root" {
        io(
            "c3-worker-accepted",
            events.write_all(&encode(&message(&grant, "accepted", 1, me, 0)?, key)?),
        )?;
        let continue_frame: Message = decode(&read_frame(go.as_mut().unwrap(), deadline)?, key)?;
        verify_message(&continue_frame, &grant, "continue", 2)?;
        if continue_frame.process != grant.guardian {
            return Err(invalid("c3-worker-continue-peer"));
        }
        drop(go);
        // First data access is after the authenticated, persisted Continue gate.
        if hash(&root.join("projects/synthetic.txt"))? != grant.data_sha256 {
            return Err(invalid("c3-worker-data"));
        }
        let directory = root
            .join("projects")
            .join(format!("c3-{}", launch.simple()));
        write_new(&directory.join("root.txt"), ROOT_TEXT)?;
        let mut delegated = grant.clone();
        delegated.role = "leaf".into();
        delegated.parent_pid = me.pid;
        let mut wire = encode(&delegated, key)?;
        if grant.bad_leaf {
            wire[4] ^= 1;
        }
        let boot = prepaid(&wire)?;
        let secret = prepaid(key)?;
        let handles = [
            boot.as_raw_handle(),
            secret.as_raw_handle(),
            events.as_raw_handle(),
        ];
        inherited(&handles, true)?;
        // Child inherits the restricted primary token/job. Only these three probe handles inherit.
        let attrs = leaf_attributes(&handles)?;
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.lpAttributeList = attrs.ptr;
        let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
        let mut cmd = wide(format!(
            "\"{}\" c3-synthetic-worker leaf {} {} {} {}",
            exe.display(),
            handles[0] as usize,
            handles[1] as usize,
            handles[2] as usize,
            launch
        ));
        let ok = unsafe {
            CreateProcessW(
                wide(&exe).as_ptr(),
                cmd.as_mut_ptr(),
                null(),
                null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | DETACHED_PROCESS,
                null(),
                wide(exe.parent().ok_or(invalid("c3-code-parent"))?).as_ptr(),
                &startup.StartupInfo,
                &mut pi,
            )
        };
        let error = (ok == 0).then(|| fail("c3-worker-create-leaf"));
        let child = (ok != 0).then(|| (Handle(pi.hProcess), Handle(pi.hThread)));
        inherited(&handles, false)?;
        drop(attrs);
        drop(boot);
        drop(secret);
        if let Some(e) = error {
            return Err(e);
        }
        let (process, _thread) = child.ok_or(invalid("c3-worker-leaf"))?;
        let child_id = identity(process.0)?;
        io(
            "c3-worker-child-ready",
            events.write_all(&encode(
                &message(&grant, "child-prepared", 3, child_id, pi.dwThreadId)?,
                key,
            )?),
        )?;
        // Never resume the child here. Guardian validates and records it, waits for root exit,
        // then resumes its exact native thread. Dropping these handles does not terminate it.
        Ok(())
    } else {
        let directory = root
            .join("projects")
            .join(format!("c3-{}", launch.simple()));
        if io("c3-leaf-read", fs::read(directory.join("root.txt")))? != ROOT_TEXT
            || hash(&root.join("projects/synthetic.txt"))? != grant.data_sha256
        {
            return Err(invalid("c3-leaf-data"));
        }
        write_new(&directory.join("leaf.txt"), LEAF_TEXT)?;
        io(
            "c3-leaf-done",
            events.write_all(&encode(&message(&grant, "leaf-done", 4, me, 0)?, key)?),
        )
    }
}

fn leaf_attributes(handles: &[HANDLE]) -> Result<Attributes> {
    let mut bytes = 0;
    unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut bytes) };
    if bytes == 0 {
        return Err(fail("c3-leaf-attribute-size"));
    }
    let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    let ptr = storage.as_mut_ptr().cast();
    check(
        unsafe { InitializeProcThreadAttributeList(ptr, 1, 0, &mut bytes) },
        "c3-leaf-attributes",
    )?;
    let attrs = Attributes {
        _storage: storage,
        ptr,
    };
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
        "c3-leaf-handle-list",
    )?;
    Ok(attrs)
}
