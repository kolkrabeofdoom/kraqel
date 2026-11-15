//! Pure guard checks are also exercised on Linux; they don't create security profiles.
use serde_json::Value;
use std::path::Path;

pub fn profile_name(name: &str) -> Option<String> {
    let suffix = name.strip_prefix("kraqel-isolation-")?;
    if suffix.is_empty() || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
        return None;
    }
    let identity = format!("org.kraqel.probe.{suffix}");
    (identity.len() <= 64).then_some(identity)
}

pub fn valid_root(root: &Path, resume: bool) -> bool {
    let Ok(temp) = std::env::temp_dir().canonicalize() else {
        return false;
    };
    let parent = root.parent().and_then(|p| p.canonicalize().ok());
    root.is_absolute()
        && parent.as_deref() == Some(temp.as_path())
        && root
            .file_name()
            .and_then(|x| x.to_str())
            .and_then(profile_name)
            .is_some()
        && (resume || !root.exists())
}

pub fn clean_resume(previous: &Value) -> bool {
    previous["profileRemoved"] == true
        && previous["activeJobProcessesAtEnd"] == 0
        && previous["appMode"] == true
        && previous["processExitCode"] == 0
        && previous["deadlineOrStop"] == false
        && previous["tokenBeforeResume"]["isAppContainer"] == true
        && previous["tokenBeforeResume"]["capabilityCount"] == 0
        && previous["tokenBeforeResume"]["expectedIdentity"] == true
}

pub fn successful_run(report: &Value) -> bool {
    report["profileRemoved"] == true
        && report["activeJobProcessesAtEnd"] == 0
        && report["processExitCode"] == 0
        && report["deadlineOrStop"] == false
        && report["localPositiveControls"] == 4
        && report["publicIpv4TcpControlConnected"] == true
        && report["tokenBeforeResume"]["isAppContainer"] == true
        && report["tokenBeforeResume"]["capabilityCount"] == 0
        && report["tokenBeforeResume"]["expectedIdentity"] == true
        && (report["appMode"] == true
            || (report["child"]["passed"] == true && report["leaf"]["passed"] == true))
}

pub fn ports(text: &str) -> Option<Vec<u16>> {
    let values = text
        .split(',')
        .take(5)
        .map(str::parse::<u16>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (values.len() == 4 && values.iter().all(|value| *value != 0)).then_some(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn clean() -> Value {
        json!({"profileRemoved":true,"activeJobProcessesAtEnd":0,"appMode":true,
            "processExitCode":0,"deadlineOrStop":false,"localPositiveControls":4,
            "publicIpv4TcpControlConnected":true,
            "tokenBeforeResume":{"isAppContainer":true,"capabilityCount":0,"expectedIdentity":true}})
    }

    #[test]
    fn names_cannot_escape_the_test_namespace_or_exceed_win32_limits() {
        for text in [
            "org.kraqel.desktop",
            "kraqel-isolation-",
            "kraqel-isolation-../private",
            "kraqel-isolation-a\"b",
            "kraqel-isolation-ä",
        ] {
            assert!(profile_name(text).is_none());
        }
        assert!(profile_name(&format!("kraqel-isolation-{}", "a".repeat(64))).is_none());
        assert_eq!(
            profile_name("kraqel-isolation-app-123"),
            Some("org.kraqel.probe.app-123".into())
        );
    }

    #[test]
    fn existing_and_non_temp_directories_are_rejected_before_any_acl_operation() {
        assert!(!valid_root(Path::new("kraqel-isolation-relative"), false));
        assert!(!valid_root(
            &std::env::current_dir()
                .unwrap()
                .join("kraqel-isolation-rejected"),
            false
        ));
        let root = std::env::temp_dir().join(format!(
            "kraqel-isolation-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert!(valid_root(&root, false));
        std::fs::create_dir(&root).unwrap();
        assert!(!valid_root(&root, false));
        assert!(valid_root(&root, true));
        std::fs::remove_dir(&root).unwrap();
    }

    #[test]
    fn resume_requires_successful_exit_identity_and_confirmed_cleanup() {
        assert!(clean_resume(&clean()));
        for (key, value) in [
            ("profileRemoved", json!(false)),
            ("activeJobProcessesAtEnd", json!(1)),
            ("appMode", json!(false)),
            ("processExitCode", json!(1)),
            ("deadlineOrStop", json!(true)),
        ] {
            let mut state = clean();
            state[key] = value;
            assert!(!clean_resume(&state));
        }
        let mut state = clean();
        state["tokenBeforeResume"]["expectedIdentity"] = json!(false);
        assert!(!clean_resume(&state));
        assert!(!clean_resume(&json!({})));
    }

    #[test]
    fn diagnostic_success_does_not_ignore_missing_controls_or_an_uncontained_child() {
        assert!(successful_run(&clean()));
        let mut state = clean();
        state["localPositiveControls"] = json!(3);
        assert!(!successful_run(&state));
        let mut state = clean();
        state["tokenBeforeResume"]["capabilityCount"] = json!(1);
        assert!(!successful_run(&state));
        let mut state = clean();
        state["appMode"] = json!(false);
        state["child"] = json!({"passed":true});
        state["leaf"] = json!({"passed":false});
        assert!(!successful_run(&state));
        assert!(!successful_run(&json!({})));
    }

    #[test]
    fn port_arguments_are_bounded_and_never_treated_as_commands() {
        assert_eq!(ports("1,2,3,65535"), Some(vec![1, 2, 3, 65535]));
        for text in [
            "",
            "0,2,3,4",
            "65536,2,3,4",
            "1,2,3",
            "1,2,3,4,5",
            "1,2,3,$(secret)",
        ] {
            assert!(ports(text).is_none());
        }
    }
}
