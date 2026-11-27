//! Pure contract rules. Parsing/transitions authenticate nothing and grant no access.
//! No filesystem, key, transport, clock, token, process or UI operations belong in this module.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_PAYLOAD: usize = 8192;
const MAX_MODEL_RECORDS: u16 = 256; // Bounded model, not product checkpointing/retention.
pub(crate) type Id = [u8; 16];
pub(crate) type Digest = [u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WorkspaceBindingV1 {
    pub(crate) version: u16,
    pub(crate) control_schema: u16,
    pub(crate) installation_id: Id,
    pub(crate) workspace_id: Id,
    pub(crate) profile_id: Id,
    pub(crate) broker_package_sha256: Digest,
    pub(crate) ui_package_sha256: Digest,
    pub(crate) config_sha256: Digest,
    pub(crate) workspace_root_sha256: Digest,
    pub(crate) workspace_format_version: u16,
    pub(crate) offline_policy_version: u16,
}
impl WorkspaceBindingV1 {
    fn valid(&self) -> bool {
        self.version == 1
            && self.control_schema == 5 // Record-only P2a; no native UI bootstrap.
            && self.workspace_format_version == 1 // Workspace format, not SQLite schema version.
            && self.offline_policy_version == 1
            && [self.installation_id, self.workspace_id, self.profile_id]
                .iter().all(|id| *id != [0; 16])
            && [self.broker_package_sha256, self.ui_package_sha256, self.config_sha256, self.workspace_root_sha256]
                .iter().all(|hash| *hash != [0; 32])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Purpose {
    #[serde(rename = "workspace-start")]
    WorkspaceStart,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StartProposalV1 {
    pub(crate) version: u16,
    pub(crate) purpose: Purpose,
    pub(crate) binding: WorkspaceBindingV1,
    pub(crate) launch_id: Id,
    pub(crate) generation: u64,
    pub(crate) expected_head: Digest,
    pub(crate) bootstrap_timeout_ms: u32,
}
impl StartProposalV1 {
    fn valid(&self) -> bool {
        self.version == 1
            && self.binding.valid()
            && self.launch_id != [0; 16]
            && self.generation > 0
            && self.expected_head != [0; 32]
            && (1..=30_000).contains(&self.bootstrap_timeout_ms)
    }
}

// Deliberately not named Authenticated/Authorized. No deserializable execution capability.
pub(crate) struct UntrustedStartProposal(StartProposalV1);
pub(crate) fn parse_payload(bytes: &[u8]) -> Result<UntrustedStartProposal, Rejection> {
    if bytes.is_empty() || bytes.len() > MAX_PAYLOAD {
        return Err(Rejection::Payload);
    }
    let proposal: StartProposalV1 =
        serde_json::from_slice(bytes).map_err(|_| Rejection::Payload)?;
    if !proposal.valid() {
        return Err(Rejection::Shape);
    }
    Ok(UntrustedStartProposal(proposal))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Rejection {
    Payload,
    Shape,
    Binding,
    Control,
    Pending,
    Job,
    Phase,
    Head,
    Generation,
    Replay,
    Budget,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))] // P2b states are tested, not exposed by P2a.
pub(crate) enum ModelJob {
    Empty,
    OwnedSuspendedChecked,
    Running,
    ForeignOrUnknown,
}
#[derive(Clone, Copy)]
pub(crate) struct ModelConditions {
    pub(crate) consistent_control: bool,
    pub(crate) pending: bool,
    pub(crate) job: ModelJob,
}
impl ModelConditions {
    fn check(self, job: ModelJob) -> Result<(), Rejection> {
        if !self.consistent_control {
            return Err(Rejection::Control);
        }
        if self.pending {
            return Err(Rejection::Pending);
        }
        if self.job != job {
            return Err(Rejection::Job);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LaunchIdentity {
    pub(crate) id: Id,
    pub(crate) generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))] // P2b states are tested, not exposed by P2a.
enum ModelPhase {
    Idle,
    Committed(LaunchIdentity),
    Running(LaunchIdentity),
}
#[derive(Clone, Copy)]
#[cfg_attr(not(test), allow(dead_code))] // P2b states are tested, not exposed by P2a.
pub(crate) enum Settlement {
    NormalClose,
    Interrupted,
}

// Non-serializable simulation of a control decision, never a persisted/authenticated store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartModel {
    binding: WorkspaceBindingV1,
    head: Digest,
    generation: u64,
    used_launches: BTreeSet<Id>,
    remaining_records: u16,
    phase: ModelPhase,
}
impl StartModel {
    pub(crate) fn new(
        binding: WorkspaceBindingV1,
        head: Digest,
        records: u16,
    ) -> Result<Self, Rejection> {
        if !binding.valid() {
            return Err(Rejection::Binding);
        }
        if head == [0; 32] {
            return Err(Rejection::Head);
        }
        if records > MAX_MODEL_RECORDS {
            return Err(Rejection::Budget);
        }
        Ok(Self {
            binding,
            head,
            generation: 0,
            used_launches: BTreeSet::new(),
            remaining_records: records,
            phase: ModelPhase::Idle,
        })
    }
    fn changed_head(&self, expected: Digest, next: Digest) -> Result<(), Rejection> {
        if expected != self.head || next == [0; 32] || next == self.head {
            return Err(Rejection::Head);
        }
        Ok(())
    }
    // Pure transition only; no OS launch token. Native caller must persist/authenticate first.
    pub(crate) fn commit(
        &mut self,
        proposal: &UntrustedStartProposal,
        next_head: Digest,
        conditions: ModelConditions,
    ) -> Result<(), Rejection> {
        let p = &proposal.0;
        if !p.valid() {
            return Err(Rejection::Shape);
        }
        if p.binding != self.binding {
            return Err(Rejection::Binding);
        }
        conditions.check(ModelJob::Empty)?;
        if self.phase != ModelPhase::Idle {
            return Err(Rejection::Phase);
        }
        self.changed_head(p.expected_head, next_head)?;
        if self.generation.checked_add(1) != Some(p.generation) {
            return Err(Rejection::Generation);
        }
        if self.used_launches.contains(&p.launch_id) {
            return Err(Rejection::Replay);
        }
        // Reserve a terminal record too: never consume the last slot and strand a launch.
        if self.remaining_records < 2 {
            return Err(Rejection::Budget);
        }
        self.used_launches.insert(p.launch_id);
        self.generation = p.generation;
        self.remaining_records -= 1;
        self.head = next_head;
        self.phase = ModelPhase::Committed(LaunchIdentity {
            id: p.launch_id,
            generation: p.generation,
        });
        Ok(())
    }
    pub(crate) fn sync_control_head(
        &mut self,
        expected: Digest,
        next: Digest,
    ) -> Result<(), Rejection> {
        if !matches!(self.phase, ModelPhase::Committed(_)) {
            return Err(Rejection::Phase);
        }
        self.changed_head(expected, next)?;
        if self.remaining_records < 2 {
            return Err(Rejection::Budget);
        }
        self.remaining_records -= 1;
        self.head = next;
        Ok(())
    }
    pub(crate) fn mark_running(
        &mut self,
        launch: LaunchIdentity,
        expected_head: Digest,
        conditions: ModelConditions,
    ) -> Result<(), Rejection> {
        conditions.check(ModelJob::OwnedSuspendedChecked)?;
        if self.head != expected_head {
            return Err(Rejection::Head);
        }
        if self.phase != ModelPhase::Committed(launch) {
            return Err(Rejection::Phase);
        }
        self.phase = ModelPhase::Running(launch);
        Ok(())
    }
    pub(crate) fn settle(
        &mut self,
        launch: LaunchIdentity,
        expected_head: Digest,
        next_head: Digest,
        outcome: Settlement,
        conditions: ModelConditions,
    ) -> Result<(), Rejection> {
        conditions.check(ModelJob::Empty)?;
        let matches = match (self.phase, outcome) {
            (ModelPhase::Running(active), _)
            | (ModelPhase::Committed(active), Settlement::Interrupted) => active == launch,
            _ => false,
        };
        if !matches {
            return Err(Rejection::Phase);
        }
        self.changed_head(expected_head, next_head)?;
        if self.remaining_records == 0 {
            return Err(Rejection::Budget);
        }
        self.remaining_records -= 1;
        self.head = next_head;
        self.phase = ModelPhase::Idle; // Binding/profile/used launches deliberately survive.
        Ok(())
    }
    pub(crate) fn binding(&self) -> &WorkspaceBindingV1 {
        &self.binding
    }
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn pending_launch(&self) -> Option<LaunchIdentity> {
        if let ModelPhase::Committed(id) = self.phase {
            Some(id)
        } else {
            None
        }
    }
    pub(crate) fn used_launch(&self) -> Option<Id> {
        self.used_launches.first().copied()
    }
}

#[cfg(test)]
mod tests;
