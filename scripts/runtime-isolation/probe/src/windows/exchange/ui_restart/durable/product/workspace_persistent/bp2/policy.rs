//! Native SDK policy translation/readback. No BP1 model output is treated as authority.
use super::*;
use windows_sys::Win32::System::SystemServices::{
    PROCESS_MITIGATION_CHILD_PROCESS_POLICY, PROCESS_MITIGATION_IMAGE_LOAD_POLICY,
};
// WinBase.h, Windows SDK 10.0.19041/22621, not BP1 semantic tag numbers.
pub(super) const REQUIRED: u64 = (1u64 << 52) | (1u64 << 56);
pub(super) const DENY_CHILDREN: u32 = 1;
pub(super) fn support() -> Result<[u64; 2]> {
    let mut mask = [0u64; 2];
    check(
        unsafe {
            GetProcessMitigationPolicy(
                GetCurrentProcess(),
                ProcessMitigationOptionsMask,
                mask.as_mut_ptr().cast(),
                size_of_val(&mask),
            )
        },
        "bp2-support-query",
    )?;
    Ok(mask)
}
pub(super) fn require_supported(required: u64, mask: [u64; 2]) -> Result<()> {
    if required & mask[0] != required {
        return Err(invalid("bp2-unsupported-requirement"));
    }
    Ok(())
}
pub(super) fn readback(process: HANDLE) -> Result<(u32, u32)> {
    let mut image: PROCESS_MITIGATION_IMAGE_LOAD_POLICY = unsafe { zeroed() };
    let mut child: PROCESS_MITIGATION_CHILD_PROCESS_POLICY = unsafe { zeroed() };
    check(
        unsafe {
            GetProcessMitigationPolicy(
                process,
                ProcessImageLoadPolicy,
                (&mut image as *mut PROCESS_MITIGATION_IMAGE_LOAD_POLICY).cast(),
                size_of_val(&image),
            )
        },
        "bp2-image-readback",
    )?;
    check(
        unsafe {
            GetProcessMitigationPolicy(
                process,
                ProcessChildProcessPolicy,
                (&mut child as *mut PROCESS_MITIGATION_CHILD_PROCESS_POLICY).cast(),
                size_of_val(&child),
            )
        },
        "bp2-child-readback",
    )?;
    Ok(unsafe { (image.Anonymous.Flags, child.Anonymous.Flags) })
}
pub(super) fn require_readback(facts: (u32, u32), mismatch: bool) -> Result<()> {
    let wanted = if mismatch { 1 } else { 3 };
    if facts.0 & 3 != wanted || facts.0 & 0x18 != 0 || facts.1 & 1 != 1 || facts.1 & 6 != 0 {
        return Err(invalid("bp2-policy-readback-mismatch"));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bp2_policy_translation_never_masks_missing_bits_or_accepts_audit() {
        assert_eq!(REQUIRED, 0x0110000000000000);
        assert!(require_supported(REQUIRED, [REQUIRED, 0]).is_ok());
        assert!(require_supported(REQUIRED, [1 << 52, 0]).is_err());
        assert!(require_readback((3, 1), false).is_ok());
        for facts in [(1, 1), (2, 1), (3 | 8, 1), (3, 0), (3, 3), (3, 5)] {
            assert!(require_readback(facts, false).is_err());
        }
        assert!(require_readback((3, 1), true).is_err());
    }
}
