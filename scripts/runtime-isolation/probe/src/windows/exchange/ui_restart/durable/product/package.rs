//! Bounded read-only PE metadata. Never loads/executes a candidate or reads session payloads.
use super::*;
use std::io::{Read, Seek, SeekFrom};
const EXPECTED: [u8; 32] = [
    b'K', b'R', b'A', b'Q', b'E', b'L', b'-', b'P', b'A', b'C', b'K', b'A', b'G', b'E', 0, 0, 1, 0,
    0, 0, // package metadata v1
    1, 0, 4, 0, // all control history schemas 1..4
    3, 0, 1, 0, // restart ticket v3, authenticated profile cleanup v1
    0, 0, 0, 0, // reserved, must remain zero
];
#[used]
#[unsafe(link_section = ".kqmeta")]
static EMBEDDED: [u8; 32] = EXPECTED;
// Declares support for replaying the schema-4 native package binding record.
#[used]
#[unsafe(link_section = ".kqwin")]
static WIN32_BINDING: [u8; 32] = *b"KRAQEL-WIN32-BINDING-V1.........";
pub(super) fn verify_win32_binding(file: &mut fs::File) -> Result<()> {
    let size = io("win32-broker-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqwin\0\0",
        std::hint::black_box(&WIN32_BINDING),
    )
}
#[used]
#[unsafe(link_section = ".kqwses")]
static WINDOW_SESSION: [u8; 32] = *b"KRAQEL-WINDOW-SESSION-V1........";
pub(super) fn verify_window_session(file: &mut fs::File) -> Result<()> {
    let size = io("win32-broker-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqwses\0",
        std::hint::black_box(&WINDOW_SESSION),
    )
}
// Separate declaration: schema-5 record ABI 1, UI bootstrap ABI 0 (not supported).
const P2A: [u8; 32] = *b"KRAQEL-P2A-META\0\x01\0\0\0\x05\0\x01\0\0\0\0\0\0\0\0\0";
#[used]
#[unsafe(link_section = ".kqprod")]
static P2A_EMBEDDED: [u8; 32] = P2A;
// Synthetic worker bootstrap only. UI ABI stays zero; no authoring UI is advertised.
const P2B: [u8; 32] = *b"KRAQEL-P2B-PROBE\x01\0\0\0\x05\0\x01\0\0\0\x03\0\0\0\0\0";
#[used]
#[unsafe(link_section = ".kqboot")]
static P2B_EMBEDDED: [u8; 32] = P2B;
// Separate fresh-store schema-6 DATA record declaration. No UI/bootstrap support.
const C2: [u8; 32] = *b"KRAQEL-P2C-DATA\0\x01\0\0\0\x06\0\x01\0\0\0\0\0\0\0\0\0";
#[used]
#[unsafe(link_section = ".kqc2")]
static C2_EMBEDDED: [u8; 32] = C2;
const C3: [u8; 32] = *b"KRAQEL-P2C-GRUP\0\x01\0\0\0\x06\0\x01\0\0\0\0\0\0\0\0\0";
#[used]
#[unsafe(link_section = ".kqc3")]
static C3_EMBEDDED: [u8; 32] = C3;
pub(super) fn verify_group(file: &mut fs::File) -> Result<()> {
    let size = io("c3-package-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqc3\0\0\0",
        std::hint::black_box(&C3_EMBEDDED),
    )
}
pub(super) fn verify_persistent_records(file: &mut fs::File) -> Result<()> {
    let size = io("c2-package-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqc2\0\0\0",
        std::hint::black_box(&C2_EMBEDDED),
    )
}
pub(super) fn verify_probe(file: &mut fs::File) -> Result<()> {
    let size = io("probe-package-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqboot\0",
        std::hint::black_box(&P2B_EMBEDDED),
    )
}
pub(super) fn verify_workspace_records(file: &mut fs::File) -> Result<()> {
    let size = io("package-size", file.metadata())?.len();
    verify_named(
        file,
        size,
        b".kqprod\0",
        std::hint::black_box(&P2A_EMBEDDED),
    )
}

pub(super) fn verify(file: &mut fs::File) -> Result<()> {
    let size = io("package-size", file.metadata())?.len();
    verify_image(file, size)
}
fn read_at<R: Read + Seek, const N: usize>(r: &mut R, size: u64, offset: u64) -> Result<[u8; N]> {
    if offset.checked_add(N as u64).is_none_or(|end| end > size) {
        return Err(invalid("package-header-range"));
    }
    io("package-seek", r.seek(SeekFrom::Start(offset)))?;
    let mut bytes = [0; N];
    io("package-read", r.read_exact(&mut bytes))?;
    Ok(bytes)
}
fn u16_at(b: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(b[offset..offset + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(b[offset..offset + 4].try_into().unwrap())
}
fn verify_image<R: Read + Seek>(r: &mut R, size: u64) -> Result<()> {
    verify_named(r, size, b".kqmeta\0", std::hint::black_box(&EMBEDDED))
}
fn verify_named<R: Read + Seek>(
    r: &mut R,
    size: u64,
    name: &[u8; 8],
    expected: &[u8; 32],
) -> Result<()> {
    if !(64..=128 * 1024 * 1024).contains(&size) {
        return Err(invalid("package-size-budget"));
    }
    let dos = read_at::<_, 64>(r, size, 0)?;
    if &dos[..2] != b"MZ" {
        return Err(invalid("package-dos-header"));
    }
    let pe = u64::from(u32_at(&dos, 0x3c));
    let coff = read_at::<_, 24>(r, size, pe)?;
    let sections = u16_at(&coff, 6);
    let optional = u16_at(&coff, 20);
    if &coff[..4] != b"PE\0\0"
        || u16_at(&coff, 4) != 0x8664
        || !(1..=96).contains(&sections)
        || !(112..=4096).contains(&optional)
    {
        return Err(invalid("package-pe64-header"));
    }
    let magic = read_at::<_, 2>(r, size, pe + 24)?;
    if u16_at(&magic, 0) != 0x20b {
        return Err(invalid("package-pe64-magic"));
    }
    let table = pe + 24 + u64::from(optional);
    let table_end = table + u64::from(sections) * 40;
    if table_end > size {
        return Err(invalid("package-section-table"));
    }
    let mut found = false;
    for i in 0..sections {
        let s = read_at::<_, 40>(r, size, table + u64::from(i) * 40)?;
        if &s[..8] != name {
            continue;
        }
        let raw = u64::from(u32_at(&s, 20));
        let bytes = u32_at(&s, 16);
        let flags = u32_at(&s, 36);
        if found
            || u32_at(&s, 8) != 32
            || !(32..=4096).contains(&bytes)
            || raw < table_end
            || raw
                .checked_add(u64::from(bytes))
                .is_none_or(|end| end > size)
            || flags & 0x40000040 != 0x40000040
            || flags & 0xa2000000 != 0
        {
            return Err(invalid("package-contract-section"));
        }
        let metadata = read_at::<_, 32>(r, size, raw)?;
        // The address reference also keeps the exact read-only section in the linked executable.
        if metadata != *expected {
            return Err(invalid("package-incompatible-contract"));
        }
        found = true;
    }
    if !found {
        return Err(invalid("package-missing-contract"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn image() -> Vec<u8> {
        let mut b = vec![0; 1024];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[70..72].copy_from_slice(&1u16.to_le_bytes());
        b[84..86].copy_from_slice(&240u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        b[328..336].copy_from_slice(b".kqmeta\0");
        b[336..340].copy_from_slice(&32u32.to_le_bytes());
        b[344..348].copy_from_slice(&512u32.to_le_bytes());
        b[348..352].copy_from_slice(&512u32.to_le_bytes());
        b[364..368].copy_from_slice(&0x40000040u32.to_le_bytes());
        b[512..544].copy_from_slice(&EXPECTED);
        b
    }
    #[test]
    fn metadata_requires_one_bounded_read_only_pe64_section_and_exact_abi() {
        let b = image();
        assert!(verify_image(&mut Cursor::new(&b), b.len() as u64).is_ok());
        for change in 0..7 {
            let mut b = image();
            match change {
                0 => b[328] = b'X',
                1 => b[528] = 2,
                2 => b[367] = 0xc0,
                3 => b[348..352].copy_from_slice(&u32::MAX.to_le_bytes()),
                4 => b[60..64].copy_from_slice(&u32::MAX.to_le_bytes()),
                5 => b[88..90].copy_from_slice(&0x10bu16.to_le_bytes()),
                _ => {
                    b[70..72].copy_from_slice(&2u16.to_le_bytes());
                    let s = b[328..368].to_vec();
                    b[368..408].copy_from_slice(&s);
                }
            }
            assert!(verify_image(&mut Cursor::new(&b), b.len() as u64).is_err());
        }
    }
    #[test]
    fn linked_test_executable_contains_the_declared_package_contract() {
        let (_dirs, mut file, _sha) = binary_pin(&std::env::current_exe().unwrap()).unwrap();
        verify(&mut file).unwrap();
        verify_workspace_records(&mut file).unwrap();
    }
    #[test]
    fn record_only_declaration_rejects_legacy_or_claimed_ui_bootstrap() {
        let mut b = image();
        assert!(verify_named(&mut Cursor::new(&b), b.len() as u64, b".kqprod\0", &P2A).is_err());
        b[328..336].copy_from_slice(b".kqprod\0");
        b[512..544].copy_from_slice(&P2A);
        assert!(verify_named(&mut Cursor::new(&b), b.len() as u64, b".kqprod\0", &P2A).is_ok());
        b[536] = 1; // bootstrap ABI must remain zero: records alone do not authorize UI start.
        assert!(verify_named(&mut Cursor::new(&b), b.len() as u64, b".kqprod\0", &P2A).is_err());
    }
}
