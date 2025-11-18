use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub wpm: usize,
    pub extensions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wpm: 200,
            extensions: vec!["md".to_string(), "txt".to_string()],
        }
    }
}

impl Config {
    /// Load configuration with fallback chain
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self> {
        // If explicit config path provided, try to load it
        if let Some(path) = config_path {
            return Self::load_from_file(path)
                .context(format!("Failed to load config from {}", path.display()));
        }

        // Try primary location: ~/.config/<project>/<project>.yml
        if let Some(config_dir) = dirs::config_dir() {
            let project_name = env!("CARGO_PKG_NAME");
            let primary_config = config_dir
                .join(project_name)
                .join(format!("{}.yml", project_name));
            if primary_config.exists() {
                match Self::load_from_file(&primary_config) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        log::warn!(
                            "Failed to load config from {}: {}",
                            primary_config.display(),
                            e
                        );
                    }
                }
            }
        }

        // Try fallback location: ./<project>.yml
        let project_name = env!("CARGO_PKG_NAME");
        let fallback_config = PathBuf::from(format!("{}.yml", project_name));
        if fallback_config.exists() {
            match Self::load_from_file(&fallback_config) {
                Ok(config) => return Ok(config),
                Err(e) => {
                    log::warn!(
                        "Failed to load config from {}: {}",
                        fallback_config.display(),
                        e
                    );
                }
            }
        }

        // No config file found, use defaults
        log::info!("No config file found, using defaults");
        Ok(Self::default())
    }

    fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path).context("Failed to read config file")?;

        let config: Self = serde_yaml::from_str(&content).context("Failed to parse config file")?;

        log::info!("Loaded config from: {}", path.as_ref().display());
        Ok(config)
    }
}
