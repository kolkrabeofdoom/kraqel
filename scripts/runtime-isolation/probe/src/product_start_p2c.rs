//! P2c C1 simulation only: no I/O, native witnesses, authentication or execution capabilities.
//! Schema/ABI values below describe proposed types, never the native store's reader range.
#![forbid(unsafe_code)]
#![cfg_attr(not(test), allow(dead_code))] // Intentionally not wired to the native broker.

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

type Id = [u8; 16];
type Digest = [u8; 32];
const INVENTORY_DOMAIN: &[u8] = b"KRAQEL-P2C-PACKAGESET\0\x01";
const BOOTSTRAP_DOMAIN: &str = "KRAQEL-P2C-UI-START\0\x01";
const MAX_FRAME: usize = 8192;
const MAX_FILES: usize = 32_768;
const MAX_INVENTORY: usize = 4 * 1024 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;
const MAX_CONTENT: u64 = 2 * 1024 * 1024 * 1024;
const MAX_RECORDS: u16 = 256;
const LAUNCH_RECORDS: u16 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Rejection {
    Shape,
    Payload,
    Path,
    Duplicate,
    Limit,
    Binding,
    Head,
    Generation,
    Replay,
    Budget,
    Phase,
    Witness,
    Deadline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PackageRole {
    Host = 1,
    Ui = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum FileRole {
    Program = 1,
    Library = 2,
    Resource = 3,
    Configuration = 4,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct InventoryEntry {
    pub(crate) path: String,
    pub(crate) size: u64,
    pub(crate) sha256: Digest,
    pub(crate) role: FileRole,
}

fn path_key(path: &str) -> Result<String, Rejection> {
    if path.is_empty() || path.len() > 1024 || !path.nfc().eq(path.chars()) {
        return Err(Rejection::Path);
    }
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() > 32 {
        return Err(Rejection::Path);
    }
    for part in parts {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "\\:<>\"|?*".contains(c))
        {
            return Err(Rejection::Path);
        }
        let stem = part.split('.').next().unwrap_or("").to_uppercase();
        if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix).is_some_and(|tail| {
                    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "¹", "²", "³"].contains(&tail)
                })
            })
        {
            return Err(Rejection::Path);
        }
    }
    // Portable conservative collision key, NOT proof of Windows ordinal name identity.
    // C2 must additionally compare actual names/file IDs using native pinned handles.
    Ok(path.to_uppercase())
}

/// Encodes supplied metadata only. Cannot attest inventory completeness, file bytes or link safety.
pub(crate) fn encode_inventory(
    role: PackageRole,
    entries: &[InventoryEntry],
) -> Result<Vec<u8>, Rejection> {
    if entries.is_empty() || entries.len() > MAX_FILES {
        return Err(Rejection::Limit);
    }
    let mut sorted: Vec<_> = entries.iter().collect();
    sorted.sort_by(|a, b| a.path.as_bytes().cmp(b.path.as_bytes()));
    let mut aliases = BTreeSet::new();
    let mut length = INVENTORY_DOMAIN.len() + 5;
    let mut content = 0u64;
    for entry in &sorted {
        if entry.size > MAX_FILE || entry.sha256 == [0; 32] {
            return Err(Rejection::Limit);
        }
        content = content.checked_add(entry.size).ok_or(Rejection::Limit)?;
        length = length
            .checked_add(45 + entry.path.len())
            .ok_or(Rejection::Limit)?;
        if content > MAX_CONTENT || length > MAX_INVENTORY {
            return Err(Rejection::Limit);
        }
        if !aliases.insert(path_key(&entry.path)?) {
            return Err(Rejection::Duplicate);
        }
    }
    // A file must not also be an ancestor of another file, including case aliases.
    for key in &aliases {
        for (i, _) in key.match_indices('/') {
            if aliases.contains(&key[..i]) {
                return Err(Rejection::Duplicate);
            }
        }
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(INVENTORY_DOMAIN);
    bytes.push(role as u8);
    bytes.extend_from_slice(&(sorted.len() as u32).to_le_bytes());
    for entry in sorted {
        bytes.extend_from_slice(&(entry.path.len() as u32).to_le_bytes());
        bytes.extend_from_slice(entry.path.as_bytes());
        bytes.extend_from_slice(&entry.size.to_le_bytes());
        bytes.extend_from_slice(&entry.sha256);
        bytes.push(entry.role as u8);
    }
    Ok(bytes)
}

pub(crate) fn inventory_digest(
    role: PackageRole,
    entries: &[InventoryEntry],
) -> Result<Digest, Rejection> {
    Ok(Sha256::digest(encode_inventory(role, entries)?).into())
}

pub(crate) fn decode_inventory(
    bytes: &[u8],
) -> Result<(PackageRole, Vec<InventoryEntry>), Rejection> {
    if bytes.len() > MAX_INVENTORY {
        return Err(Rejection::Limit);
    }
    struct Cursor<'a> {
        bytes: &'a [u8],
        offset: usize,
    }
    impl<'a> Cursor<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], Rejection> {
            let end = self.offset.checked_add(n).ok_or(Rejection::Payload)?;
            let value = self.bytes.get(self.offset..end).ok_or(Rejection::Payload)?;
            self.offset = end;
            Ok(value)
        }
    }
    let mut c = Cursor { bytes, offset: 0 };
    if c.take(INVENTORY_DOMAIN.len())? != INVENTORY_DOMAIN {
        return Err(Rejection::Payload);
    }
    let role = match c.take(1)?[0] {
        1 => PackageRole::Host,
        2 => PackageRole::Ui,
        _ => return Err(Rejection::Shape),
    };
    let count = u32::from_le_bytes(c.take(4)?.try_into().map_err(|_| Rejection::Payload)?) as usize;
    if count == 0 || count > MAX_FILES {
        return Err(Rejection::Limit);
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let length =
            u32::from_le_bytes(c.take(4)?.try_into().map_err(|_| Rejection::Payload)?) as usize;
        if length == 0 || length > 1024 {
            return Err(Rejection::Path);
        }
        let path = std::str::from_utf8(c.take(length)?)
            .map_err(|_| Rejection::Path)?
            .to_owned();
        let size = u64::from_le_bytes(c.take(8)?.try_into().map_err(|_| Rejection::Payload)?);
        let sha256 = c.take(32)?.try_into().map_err(|_| Rejection::Payload)?;
        let role = match c.take(1)?[0] {
            1 => FileRole::Program,
            2 => FileRole::Library,
            3 => FileRole::Resource,
            4 => FileRole::Configuration,
            _ => return Err(Rejection::Shape),
        };
        entries.push(InventoryEntry {
            path,
            size,
            sha256,
            role,
        });
    }
    if c.offset != bytes.len() || encode_inventory(role, &entries)? != bytes {
        return Err(Rejection::Payload);
    }
    Ok((role, entries))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RootIdentityV1 {
    pub(crate) canonical_path_sha256: Digest,
    pub(crate) volume_id: u64,
    pub(crate) file_id: Id,
}
impl RootIdentityV1 {
    fn valid(&self) -> bool {
        self.canonical_path_sha256 != [0; 32] && self.file_id != [0; 16]
    }
    fn aliases(&self, other: &Self) -> bool {
        self.canonical_path_sha256 == other.canonical_path_sha256
            || (self.volume_id == other.volume_id && self.file_id == other.file_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WorkspaceBindingV2 {
    pub(crate) version: u16,
    pub(crate) control_schema: u16,
    pub(crate) ui_abi: u16,
    pub(crate) installation_id: Id,
    pub(crate) workspace_id: Id,
    pub(crate) profile_name: String,
    pub(crate) profile_sid: String,
    pub(crate) projects: RootIdentityV1,
    pub(crate) webview: RootIdentityV1,
    pub(crate) session_root: RootIdentityV1,
    pub(crate) host_package_sha256: Digest,
    pub(crate) ui_package_sha256: Digest,
    pub(crate) config_sha256: Digest,
    pub(crate) commands_sha256: Digest,
    pub(crate) job_policy_sha256: Digest,
    pub(crate) acl_policy_sha256: Digest,
    pub(crate) workspace_format: u16,
    pub(crate) db_read_min: u16,
    pub(crate) db_read_max: u16,
    pub(crate) db_write: u16,
    pub(crate) max_processes: u32,
    pub(crate) revision: u64,
}
impl WorkspaceBindingV2 {
    fn valid(&self) -> bool {
        let sid = self.profile_sid.strip_prefix("S-1-15-2-");
        self.version == 2
            && self.control_schema == 6
            && self.ui_abi == 1
            && self.installation_id != [0; 16]
            && self.workspace_id != [0; 16]
            && (1..=64).contains(&self.profile_name.len())
            && self
                .profile_name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
            && self.profile_sid.len() <= 184
            && sid.is_some_and(|s| {
                let parts: Vec<_> = s.split('-').collect();
                parts.len() == 7
                    && parts.iter().all(|p| {
                        !p.is_empty()
                            && p.bytes().all(|c| c.is_ascii_digit())
                            && (p.len() == 1 || !p.starts_with('0'))
                            && p.parse::<u32>().is_ok()
                    })
            })
            && self.projects.valid()
            && self.webview.valid()
            && self.session_root.valid()
            && !self.projects.aliases(&self.webview)
            && !self.projects.aliases(&self.session_root)
            && !self.webview.aliases(&self.session_root)
            && [
                self.host_package_sha256,
                self.ui_package_sha256,
                self.config_sha256,
                self.commands_sha256,
                self.job_policy_sha256,
                self.acl_policy_sha256,
            ]
            .iter()
            .all(|d| *d != [0; 32])
            && self.workspace_format == 1
            && self.db_read_min > 0
            && self.db_read_min <= self.db_write
            && self.db_write <= self.db_read_max
            && self.max_processes >= 2
            && self.revision > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Purpose {
    #[serde(rename = "workspace-ui-start")]
    WorkspaceUiStart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LaunchIdentity {
    pub(crate) id: Id,
    pub(crate) generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UiStartProposalV1 {
    pub(crate) version: u16,
    pub(crate) purpose: Purpose,
    pub(crate) binding: WorkspaceBindingV2,
    pub(crate) launch: LaunchIdentity,
    pub(crate) expected_head: Digest,
    pub(crate) timeout_ms: u32,
}
impl UiStartProposalV1 {
    fn valid(&self) -> bool {
        self.version == 1
            && self.binding.valid()
            && self.launch.id != [0; 16]
            && self.launch.generation > 0
            && self.expected_head != [0; 32]
            && (1..=30_000).contains(&self.timeout_ms)
    }
}

pub(crate) struct UntrustedProposal(UiStartProposalV1);
pub(crate) fn parse_proposal(bytes: &[u8]) -> Result<UntrustedProposal, Rejection> {
    let p: UiStartProposalV1 = parse_json(bytes)?;
    if !p.valid() {
        return Err(Rejection::Shape);
    }
    Ok(UntrustedProposal(p))
}
fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Rejection> {
    if bytes.is_empty() || bytes.len() > MAX_FRAME {
        return Err(Rejection::Payload);
    }
    serde_json::from_slice(bytes).map_err(|_| Rejection::Payload)
}

/// Deserializable claims, never an authenticated OS start grant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UiStartGrantV1 {
    pub(crate) proposal: UiStartProposalV1,
    pub(crate) guardian_epoch: Id,
    pub(crate) job_id: Id,
    pub(crate) nonce: Digest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BootstrapRole {
    Grant,
    Accepted,
    Continue,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BootstrapFrameV1 {
    pub(crate) domain: String,
    pub(crate) version: u16,
    pub(crate) sequence: u8,
    pub(crate) role: BootstrapRole,
    pub(crate) grant: UiStartGrantV1,
}
pub(crate) struct UntrustedFrame(BootstrapFrameV1);
pub(crate) fn parse_bootstrap(bytes: &[u8]) -> Result<UntrustedFrame, Rejection> {
    let f: BootstrapFrameV1 = parse_json(bytes)?;
    let sequence = match f.role {
        BootstrapRole::Grant => 0,
        BootstrapRole::Accepted => 1,
        BootstrapRole::Continue => 2,
    };
    if f.domain != BOOTSTRAP_DOMAIN
        || f.version != 1
        || f.sequence != sequence
        || !f.grant.proposal.valid()
        || f.grant.guardian_epoch == [0; 16]
        || f.grant.job_id == [0; 16]
        || f.grant.nonce == [0; 32]
    {
        return Err(Rejection::Shape);
    }
    Ok(UntrustedFrame(f))
}

/// All observations are supplied simulation data. No value here proves a native fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModelJob {
    NoGroup,
    Owned {
        job_id: Id,
        guardian_epoch: Id,
        active_processes: u32,
    },
    Unknown,
}
#[derive(Clone, Copy)]
pub(crate) struct ModelObservation {
    pub(crate) lease_held: bool,
    pub(crate) control_consistent: bool,
    pub(crate) job: ModelJob,
}
impl ModelObservation {
    fn control(self) -> Result<(), Rejection> {
        if !self.lease_held || !self.control_consistent {
            return Err(Rejection::Witness);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiRootBoundV1 {
    pub(crate) launch: LaunchIdentity,
    pub(crate) job_id: Id,
    pub(crate) guardian_epoch: Id,
    pub(crate) pid: u32,
    pub(crate) creation_time: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiGroupDrainedV1 {
    pub(crate) launch: LaunchIdentity,
    pub(crate) job_id: Id,
    pub(crate) guardian_epoch: Id,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Committed,
    RootBound,
    Accepted,
    Running,
    Closing,
    Drained,
    Blocked,
}

/// Non-serializable simulation. Native persistence/peer/token/job checks must be implemented in C2/C3.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LifecycleModel {
    binding: WorkspaceBindingV2,
    head: Digest,
    generation: u64,
    remaining: u16,
    used: BTreeSet<Id>,
    proposal: Option<UiStartProposalV1>,
    root: Option<UiRootBoundV1>,
    nonce: Option<Digest>,
    elapsed_ms: u32,
    phase: Phase,
}
impl LifecycleModel {
    pub(crate) fn new(
        binding: WorkspaceBindingV2,
        head: Digest,
        remaining: u16,
    ) -> Result<Self, Rejection> {
        if !binding.valid() {
            return Err(Rejection::Binding);
        }
        if head == [0; 32] {
            return Err(Rejection::Head);
        }
        if remaining > MAX_RECORDS {
            return Err(Rejection::Budget);
        }
        Ok(Self {
            binding,
            head,
            generation: 0,
            remaining,
            used: BTreeSet::new(),
            proposal: None,
            root: None,
            nonce: None,
            elapsed_ms: 0,
            phase: Phase::Idle,
        })
    }
    pub(crate) fn phase(&self) -> Phase {
        self.phase
    }
    fn record(&mut self, expected: Digest, next: Digest, phase: Phase) -> Result<(), Rejection> {
        if expected != self.head || next == [0; 32] || next == self.head {
            return Err(Rejection::Head);
        }
        if self.remaining == 0 {
            return Err(Rejection::Budget);
        }
        self.remaining -= 1;
        self.head = next;
        self.phase = phase;
        Ok(())
    }
    pub(crate) fn commit(
        &mut self,
        p: &UntrustedProposal,
        next: Digest,
        o: ModelObservation,
    ) -> Result<(), Rejection> {
        let p = &p.0;
        o.control()?;
        if o.job != ModelJob::NoGroup {
            return Err(Rejection::Witness);
        }
        if self.phase != Phase::Idle {
            return Err(Rejection::Phase);
        }
        if p.binding != self.binding {
            return Err(Rejection::Binding);
        }
        if self.used.contains(&p.launch.id) {
            return Err(Rejection::Replay);
        }
        if self.generation.checked_add(1) != Some(p.launch.generation) {
            return Err(Rejection::Generation);
        }
        if self.remaining < LAUNCH_RECORDS {
            return Err(Rejection::Budget);
        }
        self.record(p.expected_head, next, Phase::Committed)?;
        self.generation = p.launch.generation;
        self.used.insert(p.launch.id);
        self.proposal = Some(p.clone());
        Ok(())
    }
    fn owned(&self, o: ModelObservation, root: UiRootBoundV1) -> Result<u32, Rejection> {
        o.control()?;
        match o.job {
            ModelJob::Owned {
                job_id,
                guardian_epoch,
                active_processes,
            } if job_id == root.job_id
                && guardian_epoch == root.guardian_epoch
                && active_processes <= self.binding.max_processes =>
            {
                Ok(active_processes)
            }
            _ => Err(Rejection::Witness),
        }
    }
    pub(crate) fn bind_root(
        &mut self,
        root: UiRootBoundV1,
        grant: &UntrustedFrame,
        expected: Digest,
        next: Digest,
        o: ModelObservation,
    ) -> Result<(), Rejection> {
        if self.phase != Phase::Committed {
            return Err(Rejection::Phase);
        }
        if root.launch != self.proposal.as_ref().ok_or(Rejection::Phase)?.launch
            || root.job_id == [0; 16]
            || root.guardian_epoch == [0; 16]
            || root.pid == 0
            || root.creation_time == 0
        {
            return Err(Rejection::Binding);
        }
        if grant.0.role != BootstrapRole::Grant
            || Some(&grant.0.grant.proposal) != self.proposal.as_ref()
            || grant.0.grant.job_id != root.job_id
            || grant.0.grant.guardian_epoch != root.guardian_epoch
        {
            return Err(Rejection::Binding);
        }
        if self.owned(o, root)? != 1 {
            return Err(Rejection::Witness);
        }
        self.record(expected, next, Phase::RootBound)?;
        self.root = Some(root);
        self.nonce = Some(grant.0.grant.nonce);
        Ok(())
    }
    pub(crate) fn bootstrap(
        &mut self,
        frame: &UntrustedFrame,
        elapsed_ms: u32,
        expected: Digest,
        next: Digest,
        o: ModelObservation,
    ) -> Result<(), Rejection> {
        let f = &frame.0;
        let target = match (self.phase, f.role) {
            (Phase::RootBound, BootstrapRole::Accepted) => Phase::Accepted,
            (Phase::Accepted, BootstrapRole::Continue) => Phase::Running,
            _ => return Err(Rejection::Phase),
        };
        let root = self.root.ok_or(Rejection::Phase)?;
        if self.owned(o, root)? == 0 {
            return Err(Rejection::Witness);
        }
        if Some(&f.grant.proposal) != self.proposal.as_ref()
            || f.grant.job_id != root.job_id
            || f.grant.guardian_epoch != root.guardian_epoch
            || self.nonce.is_some_and(|nonce| nonce != f.grant.nonce)
        {
            return Err(Rejection::Binding);
        }
        if elapsed_ms < self.elapsed_ms || elapsed_ms >= f.grant.proposal.timeout_ms {
            return Err(Rejection::Deadline);
        }
        self.record(expected, next, target)?;
        self.nonce = Some(f.grant.nonce);
        self.elapsed_ms = elapsed_ms;
        Ok(())
    }
    /// Root/client close does not establish drain and consumes no journal record.
    pub(crate) fn request_close(&mut self) -> Result<(), Rejection> {
        if !matches!(
            self.phase,
            Phase::RootBound | Phase::Accepted | Phase::Running | Phase::Closing
        ) {
            return Err(Rejection::Phase);
        }
        self.phase = Phase::Closing;
        Ok(())
    }
    /// Lost continuous guardian ownership is permanently blocked in C1, even if root is gone.
    pub(crate) fn guardian_lost(&mut self) -> Result<(), Rejection> {
        if self.phase == Phase::Idle {
            return Err(Rejection::Phase);
        }
        self.phase = Phase::Blocked;
        Ok(())
    }
    pub(crate) fn drain(
        &mut self,
        end: UiGroupDrainedV1,
        expected: Digest,
        next: Digest,
        o: ModelObservation,
    ) -> Result<(), Rejection> {
        if !matches!(
            self.phase,
            Phase::RootBound | Phase::Accepted | Phase::Running | Phase::Closing
        ) {
            return Err(Rejection::Phase);
        }
        let root = self.root.ok_or(Rejection::Phase)?;
        if end
            != (UiGroupDrainedV1 {
                launch: root.launch,
                job_id: root.job_id,
                guardian_epoch: root.guardian_epoch,
            })
        {
            return Err(Rejection::Binding);
        }
        if self.owned(o, root)? != 0 {
            return Err(Rejection::Witness);
        }
        self.record(expected, next, Phase::Drained)
    }
    pub(crate) fn finish(
        &mut self,
        expected: Digest,
        next: Digest,
        o: ModelObservation,
    ) -> Result<(), Rejection> {
        if self.phase != Phase::Drained {
            return Err(Rejection::Phase);
        }
        if self.owned(o, self.root.ok_or(Rejection::Phase)?)? != 0 {
            return Err(Rejection::Witness);
        }
        self.record(expected, next, Phase::Idle)?;
        self.proposal = None;
        self.root = None;
        self.nonce = None;
        self.elapsed_ms = 0;
        // Binding, workspace, profile and used launches intentionally survive normal close.
        Ok(())
    }
}

#[cfg(test)]
mod tests;
