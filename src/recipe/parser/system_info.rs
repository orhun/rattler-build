use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Tool {
    /// Name of the tool.
    name: String,
    /// Version of the tool.
    version: String,
}

/// The system information for the rendered recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// `rattler-build` version.
    rattler_build_version: String,
    /// Used tools.
    tools: Vec<Tool>,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            rattler_build_version: env!("CARGO_PKG_VERSION").to_string(),
            tools: Vec::new(),
        }
    }
}

impl SystemInfo {
    /// Collects the system information.
    pub fn call_tool(&mut self, tool: &str, version_arg: &str) -> Result<Command, which::Error> {
        if let Some(version) = Self::run_os_command(tool, version_arg) {
            match self.tools.iter().find(|v| v.name == tool) {
                Some(tool) => {
                    tracing::warn!(
                        "rebuild: using {} {version} instead of {} as in the original build",
                        tool.name,
                        tool.version
                    )
                }
                None => {
                    self.tools.push(Tool {
                        name: tool.to_string(),
                        version,
                    });
                }
            }
        }
        let binary_path = which::which(tool)?;
        Ok(Command::new(binary_path))
    }

    /// Runs the given OS command and returns the result.
    fn run_os_command(bin: &str, arg: &str) -> Option<String> {
        match std::process::Command::new(bin).arg(arg).output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string()
                        .split_whitespace()
                        .last()
                        .map(String::from)
                } else {
                    tracing::error!(
                        "calling {bin} failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    None
                }
            }
            Err(e) => {
                tracing::error!("calling git failed: {e}");
                None
            }
        }
    }
}
