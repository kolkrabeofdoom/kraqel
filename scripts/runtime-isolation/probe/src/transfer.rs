//! Bounded byte transport only: format parsers remain inside the isolated application.
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

pub const LIMIT: u64 = 8 * 1024 * 1024;

pub fn valid_worker(report: &serde_json::Value, expected: u64) -> bool {
    report["passed"] == true
        && report["bytes"] == expected
        && report["unselectedFileAccessDenied"] == true
        && report["token"]["isAppContainer"] == true
        && report["token"]["capabilityCount"] == 0
        && report["sha256"].as_str().is_some_and(|hash| {
            hash.len() == 64
                && hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}

// Experimental flat synthetic .txt namespace; no arbitrary user paths or shell verbs.
pub fn flat_name(name: &str) -> bool {
    name.len() <= 64
        && name.ends_with(".txt")
        && name.strip_suffix(".txt").is_some_and(|stem| {
            !stem.is_empty()
                && stem.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                && ![
                    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
                    "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7",
                    "LPT8", "LPT9",
                ]
                .contains(&stem.to_ascii_uppercase().as_str())
        })
}

pub fn copy_exact(
    mut input: impl Read,
    mut output: impl Write,
    expected: u64,
) -> io::Result<String> {
    if expected == 0 || expected > LIMIT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "transport size",
        ));
    }
    let mut hash = Sha256::new();
    let mut remaining = expected;
    let mut buffer = [0u8; 16384];
    while remaining != 0 {
        let bound = remaining.min(buffer.len() as u64) as usize;
        let count = input.read(&mut buffer[..bound])?;
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short transport",
            ));
        }
        output.write_all(&buffer[..count])?;
        hash.update(&buffer[..count]);
        remaining -= count as u64;
    }
    if input.read(&mut buffer[..1])? != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "long transport"));
    }
    output.flush()?;
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_paths_devices_links_and_alternate_stream_names() {
        assert!(flat_name("input-1.txt"));
        for name in [
            "../a.txt",
            "C:\\a.txt",
            "\\\\host\\a.txt",
            "a.txt:stream",
            "a.lnk",
            "a..txt",
            "a .txt",
            "CON.txt",
            "lpt9.txt",
            ".txt",
        ] {
            assert!(!flat_name(name), "{name}");
        }
    }

    #[test]
    fn byte_budget_and_declared_length_are_enforced() {
        let mut out = Vec::new();
        assert!(copy_exact(&b"abc"[..], &mut out, LIMIT + 1).is_err());
        assert!(out.is_empty());
        assert!(copy_exact(&b"abc"[..], &mut out, 0).is_err());
        assert!(copy_exact(&b"ab"[..], Vec::new(), 3).is_err());
        assert!(copy_exact(&b"abcd"[..], Vec::new(), 3).is_err());
        assert_eq!(
            copy_exact(&b"abc"[..], &mut out, 3).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(out, b"abc");
    }

    #[test]
    fn write_failure_is_never_a_successful_transfer() {
        assert!(copy_exact(&b"abc"[..], io::sink(), 3).is_ok());
        assert!(copy_exact(&b"abc"[..], &mut [0u8; 1][..], 3).is_err());
    }

    #[test]
    fn incomplete_or_wrong_identity_reports_are_not_publishable() {
        let report = serde_json::json!({"passed":true,"bytes":3,"unselectedFileAccessDenied":true,
            "token":{"isAppContainer":true,"capabilityCount":0},"sha256":"0".repeat(64)});
        assert!(valid_worker(&report, 3));
        assert!(!valid_worker(&report, 4));
        assert!(!valid_worker(&serde_json::json!({"passed":true}), 3));
        for (key, value) in [
            (
                "token",
                serde_json::json!({"isAppContainer":true,"capabilityCount":1}),
            ),
            ("sha256", serde_json::json!("bad")),
            ("unselectedFileAccessDenied", serde_json::json!(false)),
        ] {
            let mut changed = report.clone();
            changed[key] = value;
            assert!(!valid_worker(&changed, 3));
        }
    }
}
