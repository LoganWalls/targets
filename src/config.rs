use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub targets: BTreeMap<String, Target>,
}

impl TryFrom<Option<PathBuf>> for Config {
    type Error = anyhow::Error;

    fn try_from(path: Option<PathBuf>) -> Result<Self> {
        let fallback_path = || {
            let mut fb_path = std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    [
                        std::env::var("HOME").expect("No config paths found"),
                        ".config".to_string(),
                    ]
                    .iter()
                    .collect()
                });
            fb_path.push("targets");
            fb_path.push("config.toml");
            fb_path
        };

        Ok(toml::from_str(&read_to_string(
            path.unwrap_or_else(fallback_path),
        )?)?)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Target {
    /// Template to populate with `values`
    pub template: PathBuf,

    /// Path to write the populated template to
    pub out: PathBuf,

    /// A command to run after writing the populated template to `out`
    pub hook: Option<Vec<String>>,
}

impl Target {
    pub fn run_hook(&self) -> Result<()> {
        if self.hook.is_none() {
            return Ok(());
        }
        let hook = self.hook.as_ref().unwrap();
        if hook.is_empty() {
            return Err(anyhow!("a hook should have at least one part"));
        }

        let status = std::process::Command::new(
            shellexpand::full(
                hook.first()
                    .expect("script command should have at least one part"),
            )
            .with_context(|| format!("Failed to expand variables in hook: {}", hook.join(" ")))?
            .into_owned(),
        )
        .args(
            hook.iter()
                .skip(1)
                .map(|h| {
                    shellexpand::full(h)
                        .with_context(|| {
                            format!("Failed to expand variables in hook: {}", hook.join(" "))
                        })
                        .map(|s| s.into_owned())
                })
                .collect::<Result<Vec<_>>>()?,
        )
        .status()
        .with_context(|| format!("Failed to execute hook {}", hook.join("")))?;

        if status.success() {
            Ok(())
        } else {
            Err(anyhow!(format!(
                "Hook '{}' failed with status: {}",
                hook.join(" "),
                status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "(command terminated by signal)".to_string())
            )))
        }
    }
}
