//! BP1 simulation only. No I/O, native witnesses, grants, handles, MACs or startup routes.
#![forbid(unsafe_code)]
#![cfg_attr(not(test), allow(dead_code))]
use super::{Digest, Id, c4::Role, path_key};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
mod codec;
#[cfg(test)]
mod tests;

const POLICY_DOMAIN: &[u8] = b"KRAQEL-C4-PRELOAD-POLICY\0\x01";
const MODEL_DOMAIN: &[u8] = b"KRAQEL-C4-PRELOAD-MODEL\0\x01";
const MAX_BYTES: usize = 16 * 1024;
const MAX_LAUNCHES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Shape,
    Limit,
    Context,
    Unsupported,
    PolicyMismatch,
    LoadUnproven,
    RoleUnproven,
    HostUnproven,
    NativeAuthorityUnavailable,
    Replay,
    Pending,
    Phase,
    UnclearEnd,
    GuardianLost,
}
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Architecture {
    WindowsX64 = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Purpose {
    SyntheticModel = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum Mitigation {
    NoRemoteImages = 1,
    NoLowLabelImages = 2,
    PreferSystem32 = 3,
    MicrosoftSignedOnly = 4,
    StoreSignedOnly = 5,
    ProhibitDynamicCode = 6,
    DisableExtensionPoints = 7,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Children {
    Denied = 1,
    RequiresBoundRole = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum LoadRequirement {
    ExactPackageAndPlatform = 1,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreloadRoleRuleV1 {
    pub(crate) role: Role,
    pub(crate) image: String,
    pub(crate) image_sha256: Digest,
    pub(crate) max_instances: u16,
    pub(crate) arguments_sha256: Digest,
    pub(crate) token_sha256: Digest,
    pub(crate) handles_sha256: Digest,
    pub(crate) required: BTreeSet<Mitigation>,
    pub(crate) children: Children,
    pub(crate) load: LoadRequirement,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreloadPolicyV1 {
    pub(crate) version: u16,
    pub(crate) purpose: Purpose,
    pub(crate) architecture: Architecture,
    pub(crate) min_os_build: u32,
    pub(crate) package_sha256: Digest,
    pub(crate) controller_sha256: Digest,
    pub(crate) platform_sha256: Digest,
    pub(crate) environment_sha256: Digest,
    pub(crate) max_processes: u16,
    pub(crate) roles: Vec<PreloadRoleRuleV1>,
}
fn nonzero<const N: usize>(value: &[u8; N]) -> Result<()> {
    if *value == [0; N] {
        Err(Error::Shape)
    } else {
        Ok(())
    }
}
fn version(value: u16) -> Result<()> {
    if value == 1 {
        Ok(())
    } else {
        Err(Error::Shape)
    }
}
fn controls(values: &BTreeSet<Mitigation>) -> Result<()> {
    if values.contains(&Mitigation::MicrosoftSignedOnly)
        && values.contains(&Mitigation::StoreSignedOnly)
    {
        return Err(Error::PolicyMismatch);
    }
    Ok(())
}
impl PreloadPolicyV1 {
    fn validate(&self) -> Result<()> {
        version(self.version)?;
        for d in [
            &self.package_sha256,
            &self.controller_sha256,
            &self.platform_sha256,
            &self.environment_sha256,
        ] {
            nonzero(d)?;
        }
        if self.min_os_build == 0
            || !(1..=16).contains(&self.max_processes)
            || !(1..=6).contains(&self.roles.len())
        {
            return Err(Error::Limit);
        }
        let mut seen = BTreeSet::new();
        let mut total = 0u16;
        for r in &self.roles {
            path_key(&r.image).map_err(|_| Error::Shape)?;
            for d in [
                &r.image_sha256,
                &r.arguments_sha256,
                &r.token_sha256,
                &r.handles_sha256,
            ] {
                nonzero(d)?;
            }
            if !seen.insert(r.role) {
                return Err(Error::Shape);
            }
            if r.max_instances == 0 || (r.role == Role::Root && r.max_instances != 1) {
                return Err(Error::Limit);
            }
            total = total.checked_add(r.max_instances).ok_or(Error::Limit)?;
            controls(&r.required)?;
        }
        if !seen.contains(&Role::Root) || total > self.max_processes {
            return Err(Error::Limit);
        }
        Ok(())
    }
    pub(crate) fn digest(&self) -> Result<Digest> {
        Ok(Sha256::digest(self.encode()?).into())
    }
    fn root(&self) -> &PreloadRoleRuleV1 {
        self.roles
            .iter()
            .find(|r| r.role == Role::Root)
            .expect("validated policy")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreloadSupportV1 {
    pub(crate) version: u16,
    pub(crate) architecture: Architecture,
    pub(crate) os_build: u32,
    pub(crate) policy_sha256: Digest,
    pub(crate) query_complete: bool,
    pub(crate) supported: BTreeSet<Mitigation>,
    pub(crate) child_policy_supported: bool,
    /// Model assertions only. Even true cannot produce native runtime or data authority.
    pub(crate) simulated_host_anchor: bool,
}
impl PreloadSupportV1 {
    fn validate(&self) -> Result<()> {
        version(self.version)?;
        nonzero(&self.policy_sha256)?;
        if self.os_build == 0 {
            return Err(Error::Shape);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Generation {
    pub(crate) pid: u32,
    pub(crate) created: u64,
}
impl Generation {
    fn validate(self) -> Result<()> {
        if self.pid == 0 || self.created == 0 {
            Err(Error::Context)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModelContextV1 {
    pub(crate) version: u16,
    pub(crate) policy_sha256: Digest,
    pub(crate) revision: u64,
    pub(crate) guardian: Generation,
    pub(crate) job: Id,
}
impl ModelContextV1 {
    fn validate(&self) -> Result<()> {
        version(self.version)?;
        nonzero(&self.policy_sha256)?;
        nonzero(&self.job)?;
        self.guardian.validate()?;
        if self.revision == 0 {
            return Err(Error::Context);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModelRequestV1 {
    pub(crate) context: ModelContextV1,
    pub(crate) launch: Id,
    pub(crate) nonce: Digest,
}
impl ModelRequestV1 {
    fn validate(&self) -> Result<()> {
        self.context.validate()?;
        nonzero(&self.launch)?;
        nonzero(&self.nonce)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreloadWitnessV1 {
    pub(crate) version: u16,
    pub(crate) request: ModelRequestV1,
    pub(crate) root: Generation,
    pub(crate) support_sha256: Digest,
    pub(crate) image_sha256: Digest,
    pub(crate) arguments_sha256: Digest,
    pub(crate) token_sha256: Digest,
    pub(crate) handles_sha256: Digest,
    pub(crate) readback_complete: bool,
    pub(crate) enforced: BTreeSet<Mitigation>,
    pub(crate) audited: BTreeSet<Mitigation>,
    pub(crate) actual_children: Children,
    pub(crate) thread_opt_out: bool,
    pub(crate) remote_downgrade: bool,
    pub(crate) simulated_exact_load: bool,
    pub(crate) simulated_role_gate: bool,
}
impl PreloadWitnessV1 {
    fn validate(&self) -> Result<()> {
        version(self.version)?;
        self.request.validate()?;
        self.root.validate()?;
        for d in [
            &self.support_sha256,
            &self.image_sha256,
            &self.arguments_sha256,
            &self.token_sha256,
            &self.handles_sha256,
        ] {
            nonzero(d)?;
        }
        if !self.enforced.is_disjoint(&self.audited) {
            return Err(Error::PolicyMismatch);
        }
        controls(&self.enforced)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModelEndV1 {
    pub(crate) version: u16,
    pub(crate) request: ModelRequestV1,
    pub(crate) root: Generation,
    pub(crate) root_ended: bool,
    pub(crate) same_job_drained: bool,
}
impl ModelEndV1 {
    fn validate(&self) -> Result<()> {
        version(self.version)?;
        self.request.validate()?;
        self.root.validate()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModelGoal {
    SyntheticLeaf,
    Runtime,
    Data,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreloadDecisionV1 {
    SyntheticLeafEligible,
    Denied(Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Pending {
    request: ModelRequestV1,
    policy: PreloadPolicyV1,
    support: PreloadSupportV1,
    goal: ModelGoal,
    root: Option<Generation>,
    evaluated: bool,
    guardian_lost: bool,
}
/// Volatile, non-serializable simulation. No crash recovery or durable replay guarantees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreloadModel {
    context: ModelContextV1,
    used: BTreeSet<Id>,
    nonces: BTreeSet<Digest>,
    roots: BTreeSet<(u32, u64)>,
    pending: Option<Pending>,
}
impl PreloadModel {
    pub(crate) fn new(context: ModelContextV1) -> Result<Self> {
        context.validate()?;
        Ok(Self {
            context,
            used: BTreeSet::new(),
            nonces: BTreeSet::new(),
            roots: BTreeSet::new(),
            pending: None,
        })
    }
    pub(crate) fn reserve(
        &mut self,
        policy: &PreloadPolicyV1,
        support: &PreloadSupportV1,
        request: ModelRequestV1,
        goal: ModelGoal,
    ) -> Result<()> {
        policy.validate()?;
        support.validate()?;
        request.validate()?;
        if request.context != self.context
            || policy.digest()? != self.context.policy_sha256
            || support.policy_sha256 != self.context.policy_sha256
        {
            return Err(Error::Context);
        }
        if self.used.contains(&request.launch) || self.nonces.contains(&request.nonce) {
            return Err(Error::Replay);
        }
        if self.pending.is_some() {
            return Err(Error::Pending);
        }
        if self.used.len() >= MAX_LAUNCHES {
            return Err(Error::Limit);
        }
        if !support.query_complete
            || support.architecture != policy.architecture
            || support.os_build < policy.min_os_build
            || !support.child_policy_supported
            || policy
                .roles
                .iter()
                .any(|r| !r.required.is_subset(&support.supported))
        {
            return Err(Error::Unsupported);
        }
        if goal == ModelGoal::SyntheticLeaf {
            let root = policy.root();
            if policy.roles.len() != 1
                || policy.max_processes != 1
                || root.children != Children::Denied
            {
                return Err(Error::RoleUnproven);
            }
            if ![Mitigation::NoRemoteImages, Mitigation::NoLowLabelImages]
                .iter()
                .all(|m| root.required.contains(m))
            {
                return Err(Error::PolicyMismatch);
            }
        }
        self.used.insert(request.launch);
        self.nonces.insert(request.nonce);
        self.pending = Some(Pending {
            request,
            policy: policy.clone(),
            support: support.clone(),
            goal,
            root: None,
            evaluated: false,
            guardian_lost: false,
        });
        Ok(())
    }
    pub(crate) fn bind_root(&mut self, request: &ModelRequestV1, root: Generation) -> Result<()> {
        root.validate()?;
        let p = self.pending.as_mut().ok_or(Error::Phase)?;
        if p.request != *request
            || root.pid == self.context.guardian.pid
            || root.created < self.context.guardian.created
        {
            return Err(Error::Context);
        }
        if p.guardian_lost {
            return Err(Error::GuardianLost);
        }
        if p.root.is_some() {
            return Err(Error::Phase);
        }
        if !self.roots.insert((root.pid, root.created)) {
            return Err(Error::Replay);
        }
        p.root = Some(root);
        Ok(())
    }
    pub(crate) fn evaluate(&mut self, witness: &PreloadWitnessV1) -> PreloadDecisionV1 {
        use PreloadDecisionV1::{Denied, SyntheticLeafEligible};
        let Some(p) = self.pending.as_mut() else {
            return Denied(if self.used.contains(&witness.request.launch) {
                Error::Replay
            } else {
                Error::Context
            });
        };
        if p.request != witness.request || p.root != Some(witness.root) {
            return Denied(Error::Context);
        }
        if p.guardian_lost {
            return Denied(Error::GuardianLost);
        }
        if p.evaluated {
            return Denied(Error::Replay);
        }
        // A matching attempted assessment is one-shot even when the evidence is rejected.
        p.evaluated = true;
        let check = (|| {
            witness.validate()?;
            if witness.support_sha256 != p.support.digest()? {
                return Err(Error::Context);
            }
            let root = p.policy.root();
            if witness.image_sha256 != root.image_sha256
                || witness.arguments_sha256 != root.arguments_sha256
                || witness.token_sha256 != root.token_sha256
                || witness.handles_sha256 != root.handles_sha256
            {
                return Err(Error::Context);
            }
            if !witness.readback_complete
                || !witness.enforced.is_subset(&p.support.supported)
                || !witness.audited.is_subset(&p.support.supported)
                || !root.required.is_subset(&witness.enforced)
                || !root.required.is_disjoint(&witness.audited)
                || witness.actual_children != root.children
            {
                return Err(Error::PolicyMismatch);
            }
            if root.required.contains(&Mitigation::ProhibitDynamicCode)
                && (witness.thread_opt_out || witness.remote_downgrade)
            {
                return Err(Error::PolicyMismatch);
            }
            if p.goal != ModelGoal::SyntheticLeaf {
                if !p.support.simulated_host_anchor {
                    return Err(Error::HostUnproven);
                }
                if !witness.simulated_exact_load {
                    return Err(Error::LoadUnproven);
                }
                if !witness.simulated_role_gate {
                    return Err(Error::RoleUnproven);
                }
                // No combination of model claims can grant actual runtime or data authority.
                return Err(Error::NativeAuthorityUnavailable);
            }
            Ok(())
        })();
        match check {
            Ok(()) => SyntheticLeafEligible,
            Err(e) => Denied(e),
        }
    }
    pub(crate) fn end(&mut self, end: &ModelEndV1) -> Result<()> {
        end.validate()?;
        let p = self.pending.as_ref().ok_or(Error::Phase)?;
        if p.request != end.request || p.root != Some(end.root) {
            return Err(Error::Context);
        }
        if p.guardian_lost {
            return Err(Error::GuardianLost);
        }
        if !end.root_ended || !end.same_job_drained {
            return Err(Error::UnclearEnd);
        }
        self.pending = None;
        Ok(())
    }
    pub(crate) fn lose_guardian(&mut self, request: &ModelRequestV1) -> Result<()> {
        let p = self.pending.as_mut().ok_or(Error::Phase)?;
        if p.request != *request {
            return Err(Error::Context);
        }
        p.guardian_lost = true;
        Ok(())
    }
    /// Models a failed creation before any root was bound, never a native drain witness.
    pub(crate) fn cancel_unstarted(
        &mut self,
        request: &ModelRequestV1,
        same_job_empty: bool,
    ) -> Result<()> {
        let p = self.pending.as_ref().ok_or(Error::Phase)?;
        if p.request != *request {
            return Err(Error::Context);
        }
        if p.guardian_lost {
            return Err(Error::GuardianLost);
        }
        if p.root.is_some() {
            return Err(Error::Phase);
        }
        if !same_job_empty {
            return Err(Error::UnclearEnd);
        }
        self.pending = None;
        Ok(())
    }
}
