//! Explicit, idempotent profile cleanup derived only from authenticated retired state.
use super::*;
impl Store {
    pub(in super::super) fn cleanup_profile(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
    ) -> Result<Value> {
        self.session_guard(actor, head)?;
        let s = self
            .state
            .session
            .as_ref()
            .ok_or(invalid("product-session-unbound"))?;
        if !s.retired {
            return Err(invalid("product-profile-not-retired"));
        }
        let id = s.binding.session;
        let already_recorded = s.profile_cleaned;
        // Derive a name, never create/reopen a grant. Neither folders nor caller-supplied IDs
        // authorize deletion. The private Store lease remains held across API call and record.
        let profile = super::super::super::reopen_profile(id)?;
        let hr = unsafe { DeleteAppContainerProfile(profile.name.as_ptr()) };
        // Only success or the precise Win32 not-found results permit reconciliation. All other
        // failures retain the signed retirement and can be retried with its current head.
        if hr < 0 && !matches!(hr as u32, 0x80070490 | 0x80070002) {
            return Err(Failure {
                stage: "product-profile-delete",
                code: hr.into(),
            });
        }
        crash::hit(crash::Point::ProfileRemoved);
        if self.state.schema == 4 && !already_recorded {
            self.append(actor, self.state.head, Action::ProfileCleaned)?;
        }
        Ok(
            json!({"nativePreviewOnly":true,"productionAcceptance":false,"retired":true,
            "profileRemoved":true,"profileAbsent":true,"cleanupRecorded":self.state.schema==4,
            "alreadyRecorded":already_recorded,"activeJobProcessesAtEnd":0,"networkMeasurement":false}),
        )
    }
}
