//! C4a metadata and simulation only. No I/O, native grants, MACs or control-store reader support.
#![forbid(unsafe_code)]
use super::{Digest, Id, LaunchIdentity, ModelJob, Rejection, WorkspaceBindingV2, path_key};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;

const PACKAGE_DOMAIN: &[u8] = b"KRAQEL-C4-RUNTIME-PACKAGE\0\x01";
const POLICY_DOMAIN: &[u8] = b"KRAQEL-C4-UI-POLICY\0\x01";
const RECORD_DOMAIN: &[u8] = b"KRAQEL-C4-UI-MODEL\0\x01";
const BINDING_DOMAIN: &[u8] = b"KRAQEL-C4-WORKSPACE-BINDING\0\x01";
const MAX_DESCRIPTOR: usize = 16 * 1024;
const MAX_PROCESSES: u16 = 64;
const LAUNCH_BUDGET: u16 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum Role {
    Root = 1,
    Browser = 2,
    Renderer = 3,
    Gpu = 4,
    Utility = 5,
    CrashHandler = 6,
}
impl Role {
    fn read(v: u8) -> Result<Self, Rejection> {
        match v {
            1 => Ok(Self::Root),
            2 => Ok(Self::Browser),
            3 => Ok(Self::Renderer),
            4 => Ok(Self::Gpu),
            5 => Ok(Self::Utility),
            6 => Ok(Self::CrashHandler),
            _ => Err(Rejection::Shape),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RolePolicyV1 {
    pub(crate) role: Role,
    pub(crate) image: String,
    pub(crate) max_instances: u16,
    pub(crate) arguments_sha256: Digest,
    pub(crate) token_sha256: Digest,
}
/// Digests describe required policies; they cannot attest their native enforcement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiPolicyV1 {
    pub(crate) version: u16,
    pub(crate) bootstrap_version: u16,
    pub(crate) command_version: u16,
    pub(crate) record_version: u16,
    pub(crate) max_processes: u16,
    pub(crate) job_sha256: Digest,
    pub(crate) acl_sha256: Digest,
    pub(crate) environment_sha256: Digest,
    pub(crate) commands_sha256: Digest,
    pub(crate) roles: Vec<RolePolicyV1>,
}

fn nonzero(d: &Digest) -> Result<(), Rejection> {
    if *d == [0; 32] {
        Err(Rejection::Binding)
    } else {
        Ok(())
    }
}
fn version(v: u16) -> Result<(), Rejection> {
    if v == 1 {
        Ok(())
    } else {
        Err(Rejection::Shape)
    }
}
fn put16(out: &mut Vec<u8>, v: u16) {
    out.extend(v.to_le_bytes());
}
fn put64(out: &mut Vec<u8>, v: u64) {
    out.extend(v.to_le_bytes());
}
fn string(out: &mut Vec<u8>, v: &str) {
    put16(out, v.len() as u16);
    out.extend(v.as_bytes());
}

impl UiPolicyV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, Rejection> {
        for v in [
            self.version,
            self.bootstrap_version,
            self.command_version,
            self.record_version,
        ] {
            version(v)?;
        }
        if !(3..=MAX_PROCESSES).contains(&self.max_processes)
            || !(3..=6).contains(&self.roles.len())
        {
            return Err(Rejection::Limit);
        }
        let mut sorted: Vec<_> = self.roles.iter().collect();
        sorted.sort_by_key(|r| r.role);
        let mut seen = BTreeSet::new();
        let mut sum = 0u16;
        for r in &sorted {
            path_key(&r.image)?;
            nonzero(&r.arguments_sha256)?;
            nonzero(&r.token_sha256)?;
            if !seen.insert(r.role) {
                return Err(Rejection::Duplicate);
            }
            if r.max_instances == 0
                || r.max_instances > self.max_processes
                || (r.role == Role::Root && r.max_instances != 1)
            {
                return Err(Rejection::Limit);
            }
            sum = sum.checked_add(r.max_instances).ok_or(Rejection::Limit)?;
        }
        if ![Role::Root, Role::Browser, Role::Renderer]
            .iter()
            .all(|r| seen.contains(r))
            || sum > self.max_processes
        {
            return Err(Rejection::Shape);
        }
        let mut out = POLICY_DOMAIN.to_vec();
        for v in [
            self.version,
            self.bootstrap_version,
            self.command_version,
            self.record_version,
            self.max_processes,
        ] {
            put16(&mut out, v);
        }
        // Policy v1 fixes: no network capabilities/loopback, kill-on-close, no breakaway,
        // root-only bootstrap, no inherited debug overrides. No permissive boolean toggles.
        out.push(1);
        for d in [
            &self.job_sha256,
            &self.acl_sha256,
            &self.environment_sha256,
            &self.commands_sha256,
        ] {
            nonzero(d)?;
            out.extend(d);
        }
        out.push(sorted.len() as u8);
        for r in sorted {
            out.push(r.role as u8);
            string(&mut out, &r.image);
            put16(&mut out, r.max_instances);
            out.extend(r.arguments_sha256);
            out.extend(r.token_sha256);
        }
        if out.len() > MAX_DESCRIPTOR {
            return Err(Rejection::Limit);
        }
        Ok(out)
    }
    pub(crate) fn digest(&self) -> Result<Digest, Rejection> {
        Ok(Sha256::digest(self.encode()?).into())
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, Rejection> {
        let mut c = Cursor::new(bytes, POLICY_DOMAIN)?;
        let (version, bootstrap_version, command_version, record_version, max_processes) =
            (c.u16()?, c.u16()?, c.u16()?, c.u16()?, c.u16()?);
        if c.byte()? != 1 {
            return Err(Rejection::Shape);
        }
        let (job_sha256, acl_sha256, environment_sha256, commands_sha256) =
            (c.digest()?, c.digest()?, c.digest()?, c.digest()?);
        let count = c.byte()?;
        if !(3..=6).contains(&count) {
            return Err(Rejection::Limit);
        }
        let mut roles = Vec::new();
        for _ in 0..count {
            roles.push(RolePolicyV1 {
                role: Role::read(c.byte()?)?,
                image: c.string()?,
                max_instances: c.u16()?,
                arguments_sha256: c.digest()?,
                token_sha256: c.digest()?,
            });
        }
        let p = Self {
            version,
            bootstrap_version,
            command_version,
            record_version,
            max_processes,
            job_sha256,
            acl_sha256,
            environment_sha256,
            commands_sha256,
            roles,
        };
        if p.encode()? != bytes {
            return Err(Rejection::Payload);
        }
        Ok(p)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoaderV1 {
    StaticInUi,
    Dynamic { path: String, sha256: Digest },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UiRuntimePackageV1 {
    pub(crate) version: u16,
    pub(crate) host_inventory_sha256: Digest,
    pub(crate) ui_inventory_sha256: Digest,
    pub(crate) ui_exe: String,
    pub(crate) runtime_root: String,
    pub(crate) runtime_version: [u16; 4],
    pub(crate) loader: LoaderV1,
    pub(crate) frontend_sha256: Digest,
    pub(crate) configuration_sha256: Digest,
    pub(crate) policy: UiPolicyV1,
    pub(crate) db_read_min: u16,
    pub(crate) db_read_max: u16,
    pub(crate) db_write: u16,
    pub(crate) webview_data_version: u16,
    pub(crate) binding_revision: u64,
}
impl UiRuntimePackageV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, Rejection> {
        version(self.version)?;
        path_key(&self.ui_exe)?;
        path_key(&self.runtime_root)?;
        if self.runtime_version == [0; 4]
            || self.binding_revision == 0
            || self.webview_data_version == 0
            || self.db_read_min == 0
            || self.db_read_min > self.db_write
            || self.db_write > self.db_read_max
        {
            return Err(Rejection::Binding);
        }
        let policy = self.policy.encode()?;
        let root_key = path_key(&self.runtime_root)?;
        if root_key.starts_with(&(path_key(&self.ui_exe)? + "/"))
            || path_key(&self.ui_exe)?.starts_with(&(root_key.clone() + "/"))
            || path_key(&self.ui_exe)? == root_key
        {
            return Err(Rejection::Path);
        }
        for r in &self.policy.roles {
            if (r.role == Role::Root && r.image != self.ui_exe)
                || (r.role != Role::Root
                    && !r.image.starts_with(&(self.runtime_root.clone() + "/")))
            {
                return Err(Rejection::Binding);
            }
        }
        let mut out = PACKAGE_DOMAIN.to_vec();
        put16(&mut out, self.version);
        // Fixed purpose c4-minimal-ui=1, architecture windows-x64=1.
        out.extend([1, 1]);
        for d in [&self.host_inventory_sha256, &self.ui_inventory_sha256] {
            nonzero(d)?;
            out.extend(d);
        }
        string(&mut out, &self.ui_exe);
        string(&mut out, &self.runtime_root);
        for v in self.runtime_version {
            put16(&mut out, v);
        }
        match &self.loader {
            LoaderV1::StaticInUi => out.push(1),
            LoaderV1::Dynamic { path, sha256 } => {
                path_key(path)?;
                nonzero(sha256)?;
                out.push(2);
                string(&mut out, path);
                out.extend(sha256);
            }
        }
        for d in [&self.frontend_sha256, &self.configuration_sha256] {
            nonzero(d)?;
            out.extend(d);
        }
        put16(&mut out, policy.len() as u16);
        out.extend(policy);
        for v in [
            self.db_read_min,
            self.db_read_max,
            self.db_write,
            self.webview_data_version,
        ] {
            put16(&mut out, v);
        }
        put64(&mut out, self.binding_revision);
        if out.len() > MAX_DESCRIPTOR {
            return Err(Rejection::Limit);
        }
        Ok(out)
    }
    pub(crate) fn digest(&self) -> Result<Digest, Rejection> {
        Ok(Sha256::digest(self.encode()?).into())
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, Rejection> {
        let mut c = Cursor::new(bytes, PACKAGE_DOMAIN)?;
        let version = c.u16()?;
        if c.take(2)? != [1, 1] {
            return Err(Rejection::Shape);
        }
        let (host_inventory_sha256, ui_inventory_sha256) = (c.digest()?, c.digest()?);
        let (ui_exe, runtime_root) = (c.string()?, c.string()?);
        let runtime_version = [c.u16()?, c.u16()?, c.u16()?, c.u16()?];
        let loader = match c.byte()? {
            1 => LoaderV1::StaticInUi,
            2 => LoaderV1::Dynamic {
                path: c.string()?,
                sha256: c.digest()?,
            },
            _ => return Err(Rejection::Shape),
        };
        let (frontend_sha256, configuration_sha256) = (c.digest()?, c.digest()?);
        let length = usize::from(c.u16()?);
        let policy = UiPolicyV1::decode(c.take(length)?)?;
        let p = Self {
            version,
            host_inventory_sha256,
            ui_inventory_sha256,
            ui_exe,
            runtime_root,
            runtime_version,
            loader,
            frontend_sha256,
            configuration_sha256,
            policy,
            db_read_min: c.u16()?,
            db_read_max: c.u16()?,
            db_write: c.u16()?,
            webview_data_version: c.u16()?,
            binding_revision: c.u64()?,
        };
        if p.encode()? != bytes {
            return Err(Rejection::Payload);
        }
        Ok(p)
    }
    fn matches(&self, b: &WorkspaceBindingV2) -> Result<(), Rejection> {
        self.encode()?;
        if !b.valid()
            || b.host_package_sha256 != self.host_inventory_sha256
            || b.ui_package_sha256 != self.ui_inventory_sha256
            || b.config_sha256 != self.configuration_sha256
            || b.commands_sha256 != self.policy.commands_sha256
            || b.job_policy_sha256 != self.policy.job_sha256
            || b.acl_policy_sha256 != self.policy.acl_sha256
            || b.max_processes != u32::from(self.policy.max_processes)
            || b.revision != self.binding_revision
            || (b.db_read_min, b.db_read_max, b.db_write)
                != (self.db_read_min, self.db_read_max, self.db_write)
        {
            return Err(Rejection::Binding);
        }
        Ok(())
    }
}

/// Supplied workspace metadata only; hashes no actual files and authenticates no root.
fn binding_digest(p: &UiRuntimePackageV1, b: &WorkspaceBindingV2) -> Result<Digest, Rejection> {
    p.matches(b)?;
    let mut out = BINDING_DOMAIN.to_vec();
    for v in [b.version, b.control_schema, b.ui_abi] {
        put16(&mut out, v);
    }
    out.extend(b.installation_id);
    out.extend(b.workspace_id);
    string(&mut out, &b.profile_name);
    string(&mut out, &b.profile_sid);
    for root in [&b.projects, &b.webview, &b.session_root] {
        out.extend(root.canonical_path_sha256);
        put64(&mut out, root.volume_id);
        out.extend(root.file_id);
    }
    for d in [
        b.host_package_sha256,
        b.ui_package_sha256,
        b.config_sha256,
        b.commands_sha256,
        b.job_policy_sha256,
        b.acl_policy_sha256,
        p.digest()?,
    ] {
        out.extend(d);
    }
    for v in [b.workspace_format, b.db_read_min, b.db_read_max, b.db_write] {
        put16(&mut out, v);
    }
    out.extend(b.max_processes.to_le_bytes());
    put64(&mut out, b.revision);
    Ok(Sha256::digest(out).into())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8], domain: &[u8]) -> Result<Self, Rejection> {
        if bytes.len() > MAX_DESCRIPTOR {
            return Err(Rejection::Limit);
        }
        let mut c = Self { bytes, offset: 0 };
        if c.take(domain.len())? != domain {
            return Err(Rejection::Payload);
        }
        Ok(c)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], Rejection> {
        let end = self.offset.checked_add(n).ok_or(Rejection::Limit)?;
        let out = self.bytes.get(self.offset..end).ok_or(Rejection::Payload)?;
        self.offset = end;
        Ok(out)
    }
    fn byte(&mut self) -> Result<u8, Rejection> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, Rejection> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| Rejection::Payload)?,
        ))
    }
    fn u64(&mut self) -> Result<u64, Rejection> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| Rejection::Payload)?,
        ))
    }
    fn digest(&mut self) -> Result<Digest, Rejection> {
        self.take(32)?.try_into().map_err(|_| Rejection::Payload)
    }
    fn string(&mut self) -> Result<String, Rejection> {
        let n = usize::from(self.u16()?);
        if n == 0 || n > 1024 {
            return Err(Rejection::Limit);
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| Rejection::Payload)?
            .to_owned())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum StepV1 {
    Commit = 1,
    RootBound = 2,
    Accepted = 3,
    Running = 4,
    Ready = 5,
    Closing = 6,
    Drained = 7,
    Finish = 8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModelRecordV1 {
    pub(crate) version: u16,
    pub(crate) step: StepV1,
    pub(crate) workspace: Id,
    pub(crate) binding_revision: u64,
    pub(crate) package_sha256: Digest,
    pub(crate) binding_sha256: Digest,
    pub(crate) launch: LaunchIdentity,
    pub(crate) expected_head: Digest,
    pub(crate) next_head: Digest,
}
impl ModelRecordV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, Rejection> {
        version(self.version)?;
        if self.workspace == [0; 16]
            || self.launch.id == [0; 16]
            || self.launch.generation == 0
            || self.binding_revision == 0
        {
            return Err(Rejection::Binding);
        }
        for d in [
            &self.package_sha256,
            &self.binding_sha256,
            &self.expected_head,
            &self.next_head,
        ] {
            nonzero(d)?;
        }
        if self.expected_head == self.next_head {
            return Err(Rejection::Head);
        }
        let mut out = RECORD_DOMAIN.to_vec();
        put16(&mut out, self.version);
        out.push(self.step as u8);
        out.extend(self.workspace);
        put64(&mut out, self.binding_revision);
        out.extend(self.package_sha256);
        out.extend(self.binding_sha256);
        out.extend(self.launch.id);
        put64(&mut out, self.launch.generation);
        out.extend(self.expected_head);
        out.extend(self.next_head);
        Ok(out)
    }
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, Rejection> {
        let mut c = Cursor::new(bytes, RECORD_DOMAIN)?;
        let version = c.u16()?;
        let step = match c.byte()? {
            1 => StepV1::Commit,
            2 => StepV1::RootBound,
            3 => StepV1::Accepted,
            4 => StepV1::Running,
            5 => StepV1::Ready,
            6 => StepV1::Closing,
            7 => StepV1::Drained,
            8 => StepV1::Finish,
            _ => return Err(Rejection::Shape),
        };
        let p = Self {
            version,
            step,
            workspace: c.take(16)?.try_into().map_err(|_| Rejection::Payload)?,
            binding_revision: c.u64()?,
            package_sha256: c.digest()?,
            binding_sha256: c.digest()?,
            launch: LaunchIdentity {
                id: c.take(16)?.try_into().map_err(|_| Rejection::Payload)?,
                generation: c.u64()?,
            },
            expected_head: c.digest()?,
            next_head: c.digest()?,
        };
        if p.encode()? != bytes {
            return Err(Rejection::Payload);
        }
        Ok(p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhaseV1 {
    Idle,
    Committed,
    RootBound,
    Accepted,
    Running,
    Ready,
    Closing,
    Drained,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RootClaim {
    pub(crate) pid: u32,
    pub(crate) created: u64,
}
/// Caller-supplied simulation claims. None proves a native fact or authorizes data access.
#[derive(Clone, Copy)]
pub(crate) struct ModelObservationV1 {
    pub(crate) lease_held: bool,
    pub(crate) control_consistent: bool,
    pub(crate) package_verified: bool,
    pub(crate) runtime_roles_verified: bool,
    pub(crate) root: Option<RootClaim>,
    pub(crate) job: ModelJob,
    pub(crate) elapsed_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LifecycleModelV1 {
    binding: WorkspaceBindingV2,
    package: Digest,
    binding_sha256: Digest,
    head: Digest,
    remaining: u16,
    generation: u64,
    used: BTreeSet<Id>,
    launch: Option<LaunchIdentity>,
    owner: Option<(Id, Id)>,
    root: Option<RootClaim>,
    phase: PhaseV1,
    elapsed_ms: u64,
}
impl LifecycleModelV1 {
    pub(crate) fn new(
        p: &UiRuntimePackageV1,
        b: WorkspaceBindingV2,
        head: Digest,
        remaining: u16,
    ) -> Result<Self, Rejection> {
        p.matches(&b)?;
        nonzero(&head)?;
        if remaining > 256 {
            return Err(Rejection::Budget);
        }
        Ok(Self {
            binding_sha256: binding_digest(p, &b)?,
            binding: b,
            package: p.digest()?,
            head,
            remaining,
            generation: 0,
            used: BTreeSet::new(),
            launch: None,
            owner: None,
            root: None,
            phase: PhaseV1::Idle,
            elapsed_ms: 0,
        })
    }
    pub(crate) fn phase(&self) -> PhaseV1 {
        self.phase
    }
    pub(crate) fn guardian_lost(&mut self) {
        if self.phase != PhaseV1::Idle {
            self.phase = PhaseV1::Blocked;
        }
    }
    pub(crate) fn apply(
        &mut self,
        r: &ModelRecordV1,
        o: ModelObservationV1,
    ) -> Result<(), Rejection> {
        r.encode()?;
        if r.workspace != self.binding.workspace_id
            || r.binding_revision != self.binding.revision
            || r.package_sha256 != self.package
            || r.binding_sha256 != self.binding_sha256
        {
            return Err(Rejection::Binding);
        }
        if r.expected_head != self.head {
            return Err(Rejection::Head);
        }
        if !o.lease_held || !o.control_consistent || !o.package_verified {
            return Err(Rejection::Witness);
        }
        if self.remaining == 0 {
            return Err(Rejection::Budget);
        }
        if r.step == StepV1::Commit {
            if o.elapsed_ms != 0 {
                return Err(Rejection::Deadline);
            }
        } else if o.elapsed_ms < self.elapsed_ms
            || (matches!(
                r.step,
                StepV1::RootBound | StepV1::Accepted | StepV1::Running
            ) && o.elapsed_ms >= 30_000)
        {
            return Err(Rejection::Deadline);
        }
        let next = if r.step == StepV1::Commit {
            if self.phase != PhaseV1::Idle {
                return Err(Rejection::Phase);
            }
            if o.job != ModelJob::NoGroup {
                return Err(Rejection::Witness);
            }
            if self.used.contains(&r.launch.id) {
                return Err(Rejection::Replay);
            }
            if self.generation.checked_add(1) != Some(r.launch.generation) {
                return Err(Rejection::Generation);
            }
            if self.remaining < LAUNCH_BUDGET {
                return Err(Rejection::Budget);
            }
            PhaseV1::Committed
        } else {
            if self.launch != Some(r.launch) {
                return Err(Rejection::Generation);
            }
            let (job, epoch, count) = match o.job {
                ModelJob::Owned {
                    job_id,
                    guardian_epoch,
                    active_processes,
                } if job_id != [0; 16]
                    && guardian_epoch != [0; 16]
                    && active_processes <= self.binding.max_processes =>
                {
                    (job_id, guardian_epoch, active_processes)
                }
                _ => return Err(Rejection::Witness),
            };
            if r.step == StepV1::RootBound {
                if self.phase != PhaseV1::Committed
                    || count != 1
                    || !o.root.is_some_and(|r| r.pid > 0 && r.created > 0)
                {
                    return Err(Rejection::Witness);
                }
                PhaseV1::RootBound
            } else {
                if self.owner != Some((job, epoch)) {
                    return Err(Rejection::Witness);
                }
                match (self.phase, r.step) {
                    (PhaseV1::RootBound, StepV1::Accepted) if count > 0 && o.root == self.root => {
                        PhaseV1::Accepted
                    }
                    (PhaseV1::Accepted, StepV1::Running) if count > 0 && o.root == self.root => {
                        PhaseV1::Running
                    }
                    (PhaseV1::Running, StepV1::Ready)
                        if count >= 3 && o.root == self.root && o.runtime_roles_verified =>
                    {
                        PhaseV1::Ready
                    }
                    (
                        PhaseV1::RootBound | PhaseV1::Accepted | PhaseV1::Running | PhaseV1::Ready,
                        StepV1::Closing,
                    ) => PhaseV1::Closing,
                    (PhaseV1::Closing, StepV1::Drained) if count == 0 => PhaseV1::Drained,
                    (PhaseV1::Drained, StepV1::Finish) if count == 0 => PhaseV1::Idle,
                    _ => return Err(Rejection::Phase),
                }
            }
        };
        // All checks precede mutations: rejected records preserve head, budget and consumed IDs.
        if r.step == StepV1::Commit {
            self.used.insert(r.launch.id);
            self.generation = r.launch.generation;
            self.launch = Some(r.launch);
        }
        if r.step == StepV1::RootBound {
            if let ModelJob::Owned {
                job_id,
                guardian_epoch,
                ..
            } = o.job
            {
                self.owner = Some((job_id, guardian_epoch));
            }
            self.root = o.root;
        }
        if r.step == StepV1::Finish {
            self.launch = None;
            self.owner = None;
            self.root = None;
        }
        self.remaining -= 1;
        self.head = r.next_head;
        self.phase = next;
        self.elapsed_ms = o.elapsed_ms;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
