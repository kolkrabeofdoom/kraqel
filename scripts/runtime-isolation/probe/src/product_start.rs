//! P1 contract model only. Parsing/model transitions authenticate nothing and grant no access.
//! No filesystem, key, transport, clock, token, process or UI operations belong in this module.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const MAX_PAYLOAD: usize = 8192;
const MAX_MODEL_RECORDS: u16 = 256; // Bounded model, not product checkpointing/retention.
type Id = [u8; 16];
type Digest = [u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkspaceBindingV1 {
    version: u16,
    control_schema: u16,
    installation_id: Id,
    workspace_id: Id,
    profile_id: Id,
    broker_package_sha256: Digest,
    ui_package_sha256: Digest,
    config_sha256: Digest,
    workspace_root_sha256: Digest,
    workspace_format_version: u16,
    offline_policy_version: u16,
}
impl WorkspaceBindingV1 {
    fn valid(&self) -> bool {
        self.version == 1
            && self.control_schema == 5 // Proposed contract; native store still supports only 1..4.
            && self.workspace_format_version == 1 // Workspace format, not SQLite schema version.
            && self.offline_policy_version == 1
            && [self.installation_id, self.workspace_id, self.profile_id]
                .iter().all(|id| *id != [0; 16])
            && [self.broker_package_sha256, self.ui_package_sha256, self.config_sha256, self.workspace_root_sha256]
                .iter().all(|hash| *hash != [0; 32])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Purpose {
    #[serde(rename = "workspace-start")]
    WorkspaceStart,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StartProposalV1 {
    version: u16,
    purpose: Purpose,
    binding: WorkspaceBindingV1,
    launch_id: Id,
    generation: u64,
    expected_head: Digest,
    bootstrap_timeout_ms: u32,
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
struct UntrustedStartProposal(StartProposalV1);
fn parse_payload(bytes: &[u8]) -> Result<UntrustedStartProposal, Rejection> {
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
enum Rejection {
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
enum ModelJob {
    Empty,
    OwnedSuspendedChecked,
    Running,
    ForeignOrUnknown,
}
#[derive(Clone, Copy)]
struct ModelConditions {
    consistent_control: bool,
    pending: bool,
    job: ModelJob,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LaunchIdentity {
    id: Id,
    generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelPhase {
    Idle,
    Committed(LaunchIdentity),
    Running(LaunchIdentity),
}
#[derive(Clone, Copy)]
enum Settlement {
    NormalClose,
    Interrupted,
}

// Non-serializable simulation of a control decision, never a persisted/authenticated store.
#[derive(Clone, Debug, PartialEq, Eq)]
struct StartModel {
    binding: WorkspaceBindingV1,
    head: Digest,
    generation: u64,
    used_launches: BTreeSet<Id>,
    remaining_records: u16,
    phase: ModelPhase,
}
impl StartModel {
    fn new(binding: WorkspaceBindingV1, head: Digest, records: u16) -> Result<Self, Rejection> {
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
    // Models an append only; returns no OS launch token. Future host must persist/authenticate first.
    fn commit(
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
    fn mark_running(
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
    fn settle(
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
}

#[cfg(test)]
mod tests;
