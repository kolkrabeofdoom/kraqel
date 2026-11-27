//! C3 synthetic group guardian. No UI ABI, renderer command, provider or lost-job recovery.
use super::*;
use std::collections::BTreeSet;
mod processes;
mod protocol;
mod worker;
use processes::*;
use protocol::pipe;
use protocol::*;
pub(in crate::windows) use worker::worker;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct Identity {
    pid: u32,
    created: u64,
}
impl Identity {
    fn valid(self) -> bool {
        self.pid > 4 && self.created > 0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in super::super) struct Code {
    root: RootIdentityV1,
    sha256: [u8; 32],
    size: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Committed,
    Staged,
    RootBound,
    Accepted,
    Running,
    ChildBound,
    Drained,
}
#[derive(Clone)]
struct Pending {
    launch: crate::product_start_p2c::LaunchIdentity,
    epoch: Uuid,
    guardian: Identity,
    nonce: [u8; 32],
    phase: Phase,
    root: Option<Identity>,
    child: Option<Identity>,
}
#[derive(Clone, Default)]
pub(in super::super) struct GroupState {
    code: Option<Code>,
    generation: u64,
    used: BTreeSet<[u8; 16]>,
    pending: Option<Pending>,
    last_passed: Option<bool>,
}
impl GroupState {
    pub(in super::super) fn pending(&self) -> bool {
        self.pending.is_some()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(in super::super) enum GroupEvent {
    CodeBound {
        code: Code,
    },
    Commit {
        launch: crate::product_start_p2c::LaunchIdentity,
        epoch: Uuid,
        guardian: Identity,
        nonce: [u8; 32],
    },
    Staged {
        root: RootIdentityV1,
    },
    RootBound {
        identity: Identity,
    },
    Accepted,
    Running,
    ChildBound {
        identity: Identity,
    },
    Drained,
    Finish {
        passed: bool,
    },
}
pub(in super::super) fn apply(
    state: &mut State,
    actor: [u8; 32],
    event: &GroupEvent,
) -> Result<()> {
    let c2 = state.c2.as_ref().ok_or(invalid("c3-no-c2"))?;
    if state.schema != 6
        || actor != state.actor
        || !c2.profile_ready
        || c2.binding.is_none()
        || c2.retiring
    {
        return Err(invalid("c3-control-binding"));
    }
    let g = &mut state.c3;
    match event {
        GroupEvent::CodeBound { code }
            if g.code.is_none()
                && g.pending.is_none()
                && code.root.valid()
                && code.sha256 == actor
                && (1..=128 * 1024 * 1024).contains(&code.size) =>
        {
            g.code = Some(code.clone())
        }
        GroupEvent::Commit {
            launch,
            epoch,
            guardian,
            nonce,
        } if g.code.is_some()
            && g.pending.is_none()
            && launch.id != [0; 16]
            && g.generation.checked_add(1) == Some(launch.generation)
            && !g.used.contains(&launch.id)
            && !epoch.is_nil()
            && guardian.valid()
            && *nonce != [0; 32] =>
        {
            g.generation = launch.generation;
            g.used.insert(launch.id);
            g.pending = Some(Pending {
                launch: *launch,
                epoch: *epoch,
                guardian: *guardian,
                nonce: *nonce,
                phase: Phase::Committed,
                root: None,
                child: None,
            });
        }
        _ => {
            let p = g.pending.as_mut().ok_or(invalid("c3-no-pending"))?;
            p.phase = match (event, p.phase) {
                (GroupEvent::Staged { root }, Phase::Committed) if root.valid() => Phase::Staged,
                (GroupEvent::RootBound { identity }, Phase::Staged)
                    if identity.valid() && identity.pid != p.guardian.pid =>
                {
                    p.root = Some(*identity);
                    Phase::RootBound
                }
                (GroupEvent::Accepted, Phase::RootBound) => Phase::Accepted,
                (GroupEvent::Running, Phase::Accepted) => Phase::Running,
                (GroupEvent::ChildBound { identity }, Phase::Running)
                    if identity.valid()
                        && p.root.is_some_and(|r| {
                            identity.pid != r.pid && identity.created >= r.created
                        })
                        && identity.pid != p.guardian.pid =>
                {
                    p.child = Some(*identity);
                    Phase::ChildBound
                }
                (GroupEvent::Drained, phase) if phase != Phase::Drained => Phase::Drained,
                (GroupEvent::Finish { passed }, Phase::Drained) => {
                    if *passed && (p.root.is_none() || p.child.is_none()) {
                        return Err(invalid("c3-success-without-group"));
                    }
                    g.last_passed = Some(*passed);
                    g.pending = None;
                    return Ok(());
                }
                _ => return Err(invalid("c3-transition")),
            };
        }
    }
    Ok(())
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    BadMac,
    Unassigned,
    BadLeaf,
    CrashGuardian,
}
fn code_root(id: Uuid) -> PathBuf {
    data_root(id).join("c3-code")
}
fn launch_root(id: Uuid, launch: [u8; 16]) -> PathBuf {
    data_root(id)
        .join("projects")
        .join(format!("c3-{}", Uuid::from_bytes(launch).simple()))
}
fn code_entries(c: &Code) -> Vec<InventoryEntry> {
    vec![InventoryEntry {
        path: "worker.exe".into(),
        size: c.size,
        sha256: c.sha256,
        role: FileRole::Program,
    }]
}
fn read_data(path: &Path) -> Result<Vec<u8>> {
    let _pins = pin_directories(path.parent().ok_or(invalid("c3-data-parent"))?)?;
    let mut file = open_file(path, false, false)?;
    let mut bytes = Vec::new();
    io(
        "c3-data-read",
        (&mut file).take(4097).read_to_end(&mut bytes),
    )?;
    if bytes.len() > 4096 {
        return Err(invalid("c3-data-budget"));
    }
    Ok(bytes)
}

impl Store {
    fn bind_c3(&mut self, actor: [u8; 32], head: [u8; 32], source: &mut fs::File) -> Result<()> {
        let _c2pins = self.verify_c2(actor, head)?;
        if MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 3 {
            return Err(invalid("c3-code-budget"));
        }
        if self.state.c3.code.is_some() {
            return Err(invalid("c3-code-already-bound"));
        }
        let id = self.state.c2.as_ref().unwrap().id;
        let root = code_root(id);
        secure_directory(&root)?;
        let _pins = pin_directories(&root)?;
        io("c3-code-rewind", source.seek(SeekFrom::Start(0)))?;
        let mut file = io(
            "c3-code-create",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(root.join("worker.exe")),
        )?;
        io("c3-code-copy", std::io::copy(source, &mut file))?;
        io("c3-code-sync", file.sync_all())?;
        drop(file);
        let p = profile(id)?;
        set_profile_acl(&root, true, p.sid, RX)?;
        set_profile_acl(&root.join("worker.exe"), false, p.sid, RX)?;
        let code = Code {
            root: root_identity(&root)?,
            sha256: actor,
            size: io("c3-code-size", source.metadata())?.len(),
        };
        let _files = verify_package(&root, &code_entries(&code), Some(p.sid))?;
        let (_dirs, mut executable, sha) = binary_pin(&root.join("worker.exe"))?;
        package::verify_group(&mut executable)?;
        if sha != actor {
            return Err(invalid("c3-code-hash"));
        }
        self.append(
            actor,
            head,
            Action::C3 {
                event: GroupEvent::CodeBound { code },
            },
        )
    }
    fn run_c3(&mut self, actor: [u8; 32], head: [u8; 32], mode: Mode) -> Result<Value> {
        let _c2pins = self.verify_c2(actor, head)?;
        let id = self.state.c2.as_ref().unwrap().id;
        let p = profile(id)?;
        let code = self
            .state
            .c3
            .code
            .clone()
            .ok_or(invalid("c3-code-unbound"))?;
        if code.root != root_identity(&code_root(id))? {
            return Err(invalid("c3-code-root"));
        }
        let _codepins = verify_package(&code_root(id), &code_entries(&code), Some(p.sid))?;
        if MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 10 {
            return Err(invalid("c3-budget"));
        }
        let launch = crate::product_start_p2c::LaunchIdentity {
            id: *Uuid::new_v4().as_bytes(),
            generation: self
                .state
                .c3
                .generation
                .checked_add(1)
                .ok_or(invalid("c3-generation"))?,
        };
        let mut nonce = [0; 32];
        getrandom::fill(&mut nonce).map_err(|_| invalid("c3-nonce"))?;
        self.append(
            actor,
            head,
            Action::C3 {
                event: GroupEvent::Commit {
                    launch,
                    epoch: Uuid::new_v4(),
                    guardian: identity(unsafe { GetCurrentProcess() })?,
                    nonce,
                },
            },
        )?;
        // A collision/failure here leaves the committed launch pending. Never reopen/adopt an old job.
        let job = GroupJob::create(self)?;
        let result = (|| {
            let root = launch_root(id, launch.id);
            secure_directory(&root)?;
            let _pins = pin_directories(&root)?;
            set_profile_acl(&root, true, p.sid, MODIFY)?;
            low_label(&root)?;
            check_low_label(&root)?;
            job.append(
                self,
                GroupEvent::Staged {
                    root: root_identity(&root)?,
                },
            )?;
            processes::run(self, &job, &p, id, &code, mode)
        })();
        job.stop_and_drain()?;
        job.append(self, GroupEvent::Drained)?;
        job.append(
            self,
            GroupEvent::Finish {
                passed: result.is_ok(),
            },
        )?;
        let (passed, details) = match result {
            Ok(v) => (true, v),
            Err(e) => (false, json!({"stage":e.stage,"code":e.code})),
        };
        Ok(
            json!({"passed":passed,"details":details,"groupDrained":true,"launchConsumed":true,
            "profilePreserved":present(&p)?,"uiStarted":false,"networkMeasurement":false}),
        )
    }
}
pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 3 {
        return Err(invalid("c3-arguments"));
    }
    let path = Path::new(&args[0]);
    scope(path)?;
    let action = args[1].to_str().ok_or(invalid("c3-action"))?;
    if !matches!(
        action,
        "bind"
            | "start"
            | "start-bad-mac"
            | "start-unassigned"
            | "start-bad-leaf"
            | "start-crash-root-ended"
            | "status"
    ) {
        return Err(invalid("c3-action"));
    }
    let head = parse_hash(&args[2])?;
    let (_parents, mut file, actor) = binary_pin(&io("c3-exe", std::env::current_exe())?)?;
    package::verify_group(&mut file)?;
    let mut store = Store::open(path)?;
    store.actor(actor)?;
    if store.state.schema != 6 || store.state.head != head {
        return Err(invalid("c3-head-or-schema"));
    }
    if action == "start-crash-root-ended" {
        crash::arm(path, crash::Point::C3RootEnded)?;
    }
    let outcome = match action {
        "bind" => {
            store.bind_c3(actor, head, &mut file)?;
            json!(null)
        }
        "status" => json!(null),
        _ => store.run_c3(
            actor,
            head,
            match action {
                "start-bad-mac" => Mode::BadMac,
                "start-unassigned" => Mode::Unassigned,
                "start-bad-leaf" => Mode::BadLeaf,
                "start-crash-root-ended" => Mode::CrashGuardian,
                _ => Mode::Normal,
            },
        )?,
    };
    Ok(
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"control":store.summary(),
        "c3CodeBound":store.state.c3.code.is_some(),"pendingGroup":store.state.c3.pending(),"generation":store.state.c3.generation,
        "lastPassed":store.state.c3.last_passed,"outcome":outcome,"uiAbiSupported":0,"uiStarted":false}),
    )
}

#[cfg(test)]
mod tests;
