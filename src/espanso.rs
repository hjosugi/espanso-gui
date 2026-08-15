use directories::BaseDirs;
use std::fmt;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EspansoAction {
    Start,
    Stop,
    Restart,
}

impl EspansoAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
        }
    }
}

impl fmt::Display for EspansoAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
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

pub fn action(action: EspansoAction) -> anyhow::Result<ActionResult> {
    let output = Command::new("espanso").arg(action.as_str()).output()?;
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
    fn actions_map_to_expected_cli_arguments() {
        assert_eq!(EspansoAction::Start.as_str(), "start");
        assert_eq!(EspansoAction::Stop.as_str(), "stop");
        assert_eq!(EspansoAction::Restart.as_str(), "restart");
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
