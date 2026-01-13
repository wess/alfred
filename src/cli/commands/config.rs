use anyhow::Result;
use colored::Colorize;
use std::path::Path;

use crate::{config, ui};

pub fn run(model_path: Option<String>, reset: bool) -> Result<()> {
  if reset {
    config::save(&config::Config::default())?;
    ui::success("Configuration reset to defaults");
    return Ok(());
  }

  if let Some(path) = model_path {
    let mut cfg = config::load()?;
    cfg.model_path = Some(path.clone());
    config::save(&cfg)?;
    ui::success(format!("Model path set to: {}", path));
    return Ok(());
  }

  // Show current config
  let cfg = config::load()?;

  ui::heading("Alfred Configuration");
  println!(
    "{}",
    format!("Config file: {}", config::config_path().display()).dimmed()
  );
  println!();

  // Check model
  let model_path = cfg
    .model_path
    .map(|p| Path::new(&p).to_path_buf())
    .unwrap_or_else(config::default_model_path);

  if model_path.exists() {
    let size_mb = std::fs::metadata(&model_path)
      .map(|m| m.len() as f64 / (1024.0 * 1024.0))
      .unwrap_or(0.0);
    ui::success(format!(
      "Model: {} ({:.1} MB)",
      model_path.display(),
      size_mb
    ));
  } else {
    ui::warn(format!("Model not found: {}", model_path.display()));
    ui::dim("Run 'alfred setup' to download a model");
  }

  // Check library (not needed for Rust version with llama-cpp-2)
  ui::info("Using llama-cpp-2 Rust bindings (no external library required)");

  println!(
    r#"
{}
  alfred config --model=/path/to/model.gguf
  alfred config --reset
"#,
    "Options:".bold()
  );

  Ok(())
}
