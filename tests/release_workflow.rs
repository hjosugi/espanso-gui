use serde_yaml_ng::Value;
use std::fs;
use std::path::PathBuf;

fn workflow() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release.yml");
    let source = fs::read_to_string(path).expect("read release workflow");
    serde_yaml_ng::from_str(&source).expect("parse release workflow")
}

fn key<'a>(value: &'a Value, name: &str) -> &'a Value {
    value
        .get(name)
        .unwrap_or_else(|| panic!("missing workflow key: {name}"))
}

fn job<'a>(workflow: &'a Value, name: &str) -> &'a Value {
    key(key(workflow, "jobs"), name)
}

fn steps<'a>(workflow: &'a Value, job_name: &str) -> &'a [Value] {
    key(job(workflow, job_name), "steps")
        .as_sequence()
        .unwrap_or_else(|| panic!("{job_name} steps must be a sequence"))
}

fn named_step<'a>(workflow: &'a Value, job_name: &str, name: &str) -> &'a Value {
    steps(workflow, job_name)
        .iter()
        .find(|step| step.get("name").and_then(Value::as_str) == Some(name))
        .unwrap_or_else(|| panic!("missing {job_name} step: {name}"))
}

fn step_run<'a>(workflow: &'a Value, job_name: &str, name: &str) -> &'a str {
    key(named_step(workflow, job_name, name), "run")
        .as_str()
        .unwrap_or_else(|| panic!("{job_name} step {name} must have a run script"))
}

fn step_condition<'a>(workflow: &'a Value, job_name: &str, name: &str) -> &'a str {
    key(named_step(workflow, job_name, name), "if")
        .as_str()
        .unwrap_or_else(|| panic!("{job_name} step {name} must have a condition"))
}

#[test]
fn package_jobs_are_read_only_and_publish_is_tag_gated() {
    let workflow = workflow();
    assert_eq!(
        key(key(&workflow, "permissions"), "contents").as_str(),
        Some("read")
    );

    let publish = job(&workflow, "publish");
    assert_eq!(
        key(key(publish, "permissions"), "contents").as_str(),
        Some("write")
    );
    assert_eq!(
        key(publish, "if").as_str(),
        Some("startsWith(github.ref, 'refs/tags/')")
    );

    for job_name in ["package", "publish"] {
        let checkout = steps(&workflow, job_name)
            .iter()
            .find(|step| {
                step.get("uses")
                    .and_then(Value::as_str)
                    .is_some_and(|uses| uses.starts_with("actions/checkout@"))
            })
            .expect("checkout step");
        assert_eq!(
            key(key(checkout, "with"), "persist-credentials").as_bool(),
            Some(false)
        );
    }
}

#[test]
fn macos_keeps_the_signed_app_for_post_package_verification() {
    let workflow = workflow();
    let package = job(&workflow, "package");
    let matrix = key(key(package, "strategy"), "matrix");
    let macos = key(matrix, "include")
        .as_sequence()
        .expect("matrix entries")
        .iter()
        .find(|entry| key(entry, "name").as_str() == Some("macos-aarch64"))
        .expect("macOS matrix entry");
    assert_eq!(key(macos, "formats").as_str(), Some("app,dmg"));

    let verification = named_step(&workflow, "package", "Notarize and staple macOS disk image");
    let environment = key(verification, "env");
    for secret in [
        "APPLE_SIGNING_IDENTITY",
        "APPLE_ID",
        "APPLE_PASSWORD",
        "APPLE_TEAM_ID",
    ] {
        assert!(environment.get(secret).is_some(), "missing {secret}");
    }

    let upload = named_step(&workflow, "package", "Upload packages");
    assert!(
        key(key(upload, "with"), "path")
            .as_str()
            .expect("upload paths")
            .contains("!dist/**/*.app/**"),
        "the retained app bundle must not be published separately"
    );
}

#[test]
fn optional_credentials_gate_signing_but_not_packaging() {
    let workflow = workflow();

    let windows_validation = step_run(
        &workflow,
        "package",
        "Validate optional Windows signing secrets",
    );
    for required in [
        "$provided.Count -ne 0 -and $provided.Count -ne 2",
        "SIGNING_ENABLED=$enabled",
    ] {
        assert!(
            windows_validation.contains(required),
            "Windows optional-secret validation is missing {required}"
        );
    }

    let macos_validation = step_run(
        &workflow,
        "package",
        "Validate optional macOS signing secrets",
    );
    for required in [
        "provided=0",
        "\"${provided}\" -ne 0",
        "\"${provided}\" -ne 6",
        "SIGNING_ENABLED=false",
    ] {
        assert!(
            macos_validation.contains(required),
            "macOS optional-secret validation is missing {required}"
        );
    }

    for signing_step in [
        "Sign Windows application binary",
        "Sign Windows installers",
        "Notarize and staple macOS disk image",
    ] {
        assert!(
            step_condition(&workflow, "package", signing_step)
                .contains("env.SIGNING_ENABLED == 'true'"),
            "{signing_step} must only run with complete credentials"
        );
    }
    for package_step in ["Build Linux and Windows packages", "Build macOS packages"] {
        assert!(
            !step_condition(&workflow, "package", package_step).contains("SIGNING_ENABLED"),
            "{package_step} must remain available to unsigned forks"
        );
    }
}

#[test]
fn windows_signed_path_covers_binary_and_installer_artifacts() {
    let workflow = workflow();
    let binary_signing = step_run(&workflow, "package", "Sign Windows application binary");
    assert!(binary_signing.contains("target/release/espanso-gui.exe"));

    let installer_signing = step_run(&workflow, "package", "Sign Windows installers");
    for required in [
        "Get-ChildItem dist -Recurse -File",
        "\".exe\", \".msi\"",
        "./scripts/sign-windows.ps1 -Path $artifacts.FullName",
    ] {
        assert!(
            installer_signing.contains(required),
            "Windows installer signing is missing {required}"
        );
    }
}

#[test]
fn publish_attaches_status_notes_and_checksums() {
    let workflow = workflow();
    let notes = step_run(
        &workflow,
        "publish",
        "Prepare release notes with signing status",
    );
    for required in [
        "## Signing status",
        "${#statuses[@]} != 3",
        "cat \"${status}\" >> release-notes.md",
    ] {
        assert!(
            notes.contains(required),
            "release notes are missing {required}"
        );
    }

    let checksums = step_run(&workflow, "publish", "Generate checksums");
    for required in [
        "find . -type f -print0",
        "sort -z",
        "sha256sum",
        "> SHA256SUMS",
    ] {
        assert!(
            checksums.contains(required),
            "checksum generation is missing {required}"
        );
    }

    let release = step_run(&workflow, "publish", "Create release");
    for command in ["gh release upload", "gh release create"] {
        let line = release
            .lines()
            .find(|line| line.contains(command))
            .unwrap_or_else(|| panic!("missing {command}"));
        assert!(
            line.contains("release-assets/* SHA256SUMS"),
            "{command} must attach packages and SHA256SUMS"
        );
    }
    assert!(release.contains("--notes-file release-notes.md"));
}

#[test]
fn macos_verifier_checks_distribution_security_properties() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/notarize-macos-dmg.sh");
    let verifier = fs::read_to_string(path).expect("read macOS verifier");
    for required in [
        "Authority=${APPLE_SIGNING_IDENTITY}",
        "TeamIdentifier=${APPLE_TEAM_ID}",
        "Timestamp=",
        "flags=.*\\(runtime\\)",
        "spctl --verbose=4 --assess --type exec",
        "hdiutil verify",
        "xcrun stapler validate",
    ] {
        assert!(verifier.contains(required), "missing check: {required}");
    }
}

#[test]
fn windows_signer_verifies_identity_timestamp_and_all_signatures() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/sign-windows.ps1");
    let verifier = fs::read_to_string(path).expect("read Windows signer");
    for required in [
        "/fd SHA256",
        "/tr http://timestamp.digicert.com",
        "/td SHA256",
        "verify /pa /all /v",
        "Get-PfxCertificate",
        "SignerCertificate.Thumbprint -ne $certificate.Thumbprint",
        "TimeStamperCertificate",
    ] {
        assert!(verifier.contains(required), "missing check: {required}");
    }
}
