use anyhow::{Context, Result};
use colored::Colorize;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::Write;

use crate::{config, ui};

struct ModelInfo {
  name: &'static str,
  size: &'static str,
  url: &'static str,
  filename: &'static str,
}

const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "Phi-3 Mini 4K (Q4) - Recommended",
        size: "2.4 GB",
        url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf",
        filename: "phi-3-mini-q4.gguf",
    },
    ModelInfo {
        name: "Phi-3 Mini 4K (Q8)",
        size: "4.1 GB",
        url: "https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q8.gguf",
        filename: "phi-3-mini-q8.gguf",
    },
    ModelInfo {
        name: "Qwen2.5-Coder 1.5B (Q4)",
        size: "1.0 GB",
        url: "https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf",
        filename: "qwen2.5-coder-1.5b-q4.gguf",
    },
];

async fn download_with_progress(url: &str, dest_path: &std::path::Path) -> Result<()> {
  let client = reqwest::Client::new();
  let response = client
    .get(url)
    .send()
    .await
    .with_context(|| format!("Failed to download from {}", url))?;

  if !response.status().is_success() {
    return Err(anyhow::anyhow!(
      "Download failed: {} {}",
      response.status(),
      response.status().canonical_reason().unwrap_or("")
    ));
  }

  let total_size = response.content_length().unwrap_or(0);

  let pb = ProgressBar::new(total_size);
  pb.set_style(
    ProgressStyle::default_bar()
      .template("  {spinner:.cyan} [{bar:40.cyan/dim}] {bytes}/{total_bytes} ({eta})")
      .unwrap()
      .progress_chars("█▓░"),
  );

  let mut file = fs::File::create(dest_path)
    .with_context(|| format!("Failed to create file {}", dest_path.display()))?;

  let mut stream = response.bytes_stream();
  let mut downloaded: u64 = 0;

  while let Some(chunk) = stream.next().await {
    let chunk = chunk.with_context(|| "Failed to read response chunk")?;
    file
      .write_all(&chunk)
      .with_context(|| "Failed to write to file")?;
    downloaded += chunk.len() as u64;
    pb.set_position(downloaded);
  }

  pb.finish_and_clear();
  Ok(())
}

pub async fn run() -> Result<()> {
  ui::heading("Alfred Setup");
  println!();
  println!(
    "Alfred uses a local AI model for git assistance - no API keys or subscriptions needed."
  );
  println!();

  // Create directories
  let models_dir = config::models_dir();
  fs::create_dir_all(&models_dir)
    .with_context(|| format!("Failed to create directory {}", models_dir.display()))?;

  // Check for existing models
  let existing_models: Vec<String> = fs::read_dir(&models_dir)?
    .filter_map(|entry| entry.ok())
    .filter(|entry| {
      entry
        .path()
        .extension()
        .map(|e| e == "gguf")
        .unwrap_or(false)
    })
    .filter_map(|entry| entry.file_name().into_string().ok())
    .collect();

  if !existing_models.is_empty() {
    ui::info("Found existing models:");
    for m in &existing_models {
      ui::list_item(m);
    }
    println!();
  }

  println!("Available models:");
  for (i, model) in MODELS.iter().enumerate() {
    println!(
      "  {}. {} ({})",
      (i + 1).to_string().cyan(),
      model.name,
      model.size
    );
  }
  println!();

  let model_names: Vec<&str> = MODELS.iter().map(|m| m.name).collect();
  let selected = ui::select("Select a model to download:", &model_names)
    .ok_or_else(|| anyhow::anyhow!("No model selected"))?;

  let model = MODELS
    .iter()
    .find(|m| m.name == selected)
    .ok_or_else(|| anyhow::anyhow!("Invalid selection"))?;

  let model_path = models_dir.join(model.filename);

  if model_path.exists() {
    ui::success(format!("Model already downloaded: {}", model.filename));
  } else {
    ui::info(format!("Downloading {}...", model.name));

    download_with_progress(model.url, &model_path).await?;
    ui::success("Model downloaded!");
  }

  // Save config
  let mut cfg = config::load()?;
  cfg.model_path = Some(model_path.to_string_lossy().to_string());
  config::save(&cfg)?;

  println!();
  ui::heading("Setup Complete!");
  println!(
    r#"
  Model: {}

  You're ready to use alfred! Try:
    {}       - Generate AI commit messages
    {}       - Smart rebasing with AI suggestions
    {}      - AI-assisted conflict resolution
    {}   - Create branches with AI naming

  All commands also pass through to git:
    {}       - Same as git status
    {}         - Same as git push
"#,
    model_path.display(),
    "alfred commit".cyan(),
    "alfred rebase".cyan(),
    "alfred resolve".cyan(),
    "alfred branch new".cyan(),
    "alfred status".cyan(),
    "alfred push".cyan(),
  );

  Ok(())
}
