use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
  #[serde(default = "default_port")]
  pub port: u16,
  #[serde(default = "default_idle_timeout")]
  pub idle_timeout_minutes: u32,
  #[serde(default)]
  pub auto_start: bool,
}

fn default_port() -> u16 {
  7654
}

fn default_idle_timeout() -> u32 {
  30
}

impl Default for DaemonConfig {
  fn default() -> Self {
    Self {
      port: default_port(),
      idle_timeout_minutes: default_idle_timeout(),
      auto_start: false,
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub model_path: Option<String>,
  #[serde(default)]
  pub daemon: DaemonConfig,
}

pub fn alfred_dir() -> PathBuf {
  dirs::home_dir()
    .expect("Could not find home directory")
    .join(".alfred")
}

pub fn config_path() -> PathBuf {
  alfred_dir().join("config.yaml")
}

pub fn models_dir() -> PathBuf {
  alfred_dir().join("models")
}

pub fn lib_dir() -> PathBuf {
  alfred_dir().join("lib")
}

pub fn default_model_path() -> PathBuf {
  models_dir().join("phi-3-mini-q4.gguf")
}

pub fn pid_file() -> PathBuf {
  alfred_dir().join("alferd.pid")
}

pub fn load() -> Result<Config> {
  let path = config_path();

  if !path.exists() {
    return Ok(Config::default());
  }

  let content = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read config from {}", path.display()))?;

  let config: Config =
    serde_yaml::from_str(&content).with_context(|| "Failed to parse config YAML")?;

  Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
  let dir = alfred_dir();
  fs::create_dir_all(&dir)
    .with_context(|| format!("Failed to create directory {}", dir.display()))?;

  let path = config_path();
  let content =
    serde_yaml::to_string(config).with_context(|| "Failed to serialize config to YAML")?;

  fs::write(&path, content)
    .with_context(|| format!("Failed to write config to {}", path.display()))?;

  Ok(())
}

pub fn get_model_path() -> PathBuf {
  load()
    .ok()
    .and_then(|c| c.model_path)
    .map(PathBuf::from)
    .unwrap_or_else(default_model_path)
}

pub fn get_daemon_config() -> DaemonConfig {
  load().ok().map(|c| c.daemon).unwrap_or_default()
}
