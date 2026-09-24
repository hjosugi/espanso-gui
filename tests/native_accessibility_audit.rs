//! Keeps the native accessibility audit wired into CI and aligned with the shipped catalog.

use serde_yaml_ng::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const SCRIPT: &str = "scripts/native-accessibility-audit.py";

fn read(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn test_job() -> Value {
    let workflow: Value =
        serde_yaml_ng::from_str(&read(".github/workflows/ci.yml")).expect("parse CI workflow");
    workflow
        .get("jobs")
        .and_then(|jobs| jobs.get("test"))
        .cloned()
        .expect("CI must have a test job")
}

/// Returns every string literal passed as the first argument to `text(` in the script, plus the
/// catalog keys named in its `SECTIONS` table.
fn catalog_keys_used_by_script(script: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for (index, _) in script.match_indices("text(\"") {
        let rest = &script[index + "text(\"".len()..];
        let key = rest.split('"').next().expect("closing quote");
        keys.insert(key.to_owned());
    }
    let sections = script
        .split("SECTIONS = (")
        .nth(1)
        .and_then(|rest| rest.split("\n)").next())
        .expect("the script must declare its SECTIONS table");
    for line in sections.lines() {
        let fields: Vec<&str> = line.split('"').collect();
        if fields.len() >= 4 {
            keys.insert(fields[3].to_owned());
        }
    }
    keys
}

#[test]
fn native_audit_runs_through_each_platform_accessibility_api_in_ci() {
    let job = test_job();
    let platforms: Vec<&str> = job["strategy"]["matrix"]["os"]
        .as_sequence()
        .expect("test matrix")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for prefix in ["ubuntu-", "windows-", "macos-"] {
        assert!(
            platforms.iter().any(|os| os.starts_with(prefix)),
            "the CI test matrix must include {prefix}* so its native audit runs"
        );
    }

    let steps = job["steps"].as_sequence().expect("test steps");
    let audit = steps
        .iter()
        .find(|step| step["name"].as_str() == Some("Audit through the native accessibility API"))
        .and_then(|step| step["run"].as_str())
        .expect("the test job must run the native accessibility audit");
    for (platform, binary) in [
        ("Linux)", "target/release/espanso-gui "),
        ("Windows)", "target/release/espanso-gui.exe"),
        ("macOS)", "target/release/espanso-gui "),
    ] {
        let branch = audit
            .split(platform)
            .nth(1)
            .and_then(|rest| rest.split(";;").next())
            .unwrap_or_else(|| panic!("the audit step must handle {platform}"));
        assert!(branch.contains(SCRIPT), "{platform} must run {SCRIPT}");
        assert!(
            branch.contains(binary),
            "{platform} must audit the release binary"
        );
    }
    assert!(
        audit.contains("dbus-run-session") && audit.contains("xvfb-run"),
        "Linux needs a private session bus and X server for AT-SPI and keyboard input"
    );
    assert!(
        steps.iter().any(|step| {
            step["uses"]
                .as_str()
                .is_some_and(|uses| uses.starts_with("actions/upload-artifact@"))
                && step["if"].as_str() == Some("always()")
        }),
        "audit reports must be uploaded even when the audit fails"
    );
}

#[test]
fn native_audit_expectations_come_from_existing_catalog_keys() {
    let script = read(SCRIPT);
    let catalog = read("src/i18n.rs");
    let keys = catalog_keys_used_by_script(&script);
    for required in [
        "Snippets",
        "Profiles",
        "Globals",
        "Diagnostics",
        "SettingsNav",
        "Search",
        "Language",
        "AddFile",
        "NewMatchFileTitle",
    ] {
        assert!(
            keys.contains(required),
            "the audit no longer checks {required}"
        );
    }
    for key in keys {
        assert!(
            catalog.contains(&format!("    {key} => (")),
            "the audit expects catalog key {key}, which src/i18n.rs does not define"
        );
    }
}

#[test]
fn native_audit_client_dependencies_are_pinned_per_platform() {
    let requirements = read("scripts/native-accessibility-audit-requirements.txt");
    let pins: Vec<&str> = requirements
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(pins.len(), 2, "one UI Automation and one AX client");
    for pin in pins {
        let (requirement, marker) = pin.split_once(';').expect("platform marker");
        assert!(requirement.contains("=="), "{requirement} must be pinned");
        assert!(
            marker.contains("sys_platform == \"win32\"")
                || marker.contains("sys_platform == \"darwin\""),
            "{pin} must only install on its own platform"
        );
    }
}
