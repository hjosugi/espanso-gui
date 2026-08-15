use directories::BaseDirs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct EspansoStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub service: Option<String>,
    pub config_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub output: String,
}

pub fn detect() -> EspansoStatus {
    let version_output = Command::new("espanso").arg("--version").output();
    let (installed, version) = match version_output {
        Ok(output) if output.status.success() => {
            (true, non_empty_output(&output.stdout, &output.stderr))
        }
        _ => (false, None),
    };
    let config_root = if installed {
        detect_config_root().unwrap_or_else(default_config_root)
    } else {
        default_config_root()
    };
    let service = installed
        .then(|| Command::new("espanso").arg("status").output().ok())
        .flatten()
        .and_then(|output| non_empty_output(&output.stdout, &output.stderr));

    EspansoStatus {
        installed,
        version,
        service,
        config_root,
    }
}

pub fn action(action: &str) -> anyhow::Result<ActionResult> {
    anyhow::ensure!(
        matches!(action, "start" | "stop" | "restart" | "status"),
        "許可されていないEspanso操作です"
    );
    let output = Command::new("espanso").arg(action).output()?;
    Ok(ActionResult {
        success: output.status.success(),
        output: non_empty_output(&output.stdout, &output.stderr).unwrap_or_default(),
    })
}

fn detect_config_root() -> Option<PathBuf> {
    let output = Command::new("espanso").arg("path").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().map(str::trim).find_map(|line| {
        let candidate = line
            .strip_prefix("Config:")
            .or_else(|| line.strip_prefix("Config path:"))
            .unwrap_or(line)
            .trim();
        let path = PathBuf::from(candidate);
        path.is_absolute().then_some(path)
    })
}

pub fn default_config_root() -> PathBuf {
    BaseDirs::new()
        .map(|dirs| dirs.config_dir().join("espanso"))
        .unwrap_or_else(|| PathBuf::from("espanso"))
}

fn non_empty_output(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stdout.is_empty() {
        Some(stdout)
    } else if !stderr.is_empty() {
        Some(stderr)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_actions_are_accepted() {
        let result = action("anything-else");
        assert!(result.is_err());
    }

    #[test]
    fn output_prefers_stdout() {
        assert_eq!(
            non_empty_output(b"running\n", b"warning"),
            Some("running".into())
        );
        assert_eq!(non_empty_output(b"", b"stopped\n"), Some("stopped".into()));
    }
}
