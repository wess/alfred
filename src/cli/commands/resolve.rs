use anyhow::Result;
use colored::Colorize;
use std::fs;

use crate::{git, llm, ui};

pub fn run(target_file: Option<String>) -> Result<()> {
  let status = git::status()?;

  if status.conflicts.is_empty() {
    ui::success("No conflicts to resolve!");
    return Ok(());
  }

  ui::heading(format!(
    "Found {} conflicted file(s):",
    status.conflicts.len()
  ));
  for f in &status.conflicts {
    println!("  {} {}", "!".red(), f);
  }
  println!();

  let files_to_resolve: Vec<String> = if let Some(ref target) = target_file {
    let matching: Vec<String> = status
      .conflicts
      .iter()
      .filter(|f| *f == target)
      .cloned()
      .collect();

    if matching.is_empty() {
      ui::error(format!("File not in conflict: {}", target));
      return Ok(());
    }
    matching
  } else {
    status.conflicts.clone()
  };

  ui::info("Loading AI model...");
  if let Err(e) = llm::load_model() {
    ui::error(format!("Failed to load model: {}", e));
    ui::dim("Make sure you have run 'alfred setup'");
    return Ok(());
  }

  for file in &files_to_resolve {
    ui::heading(format!("Resolving: {}", file));

    let conflict_info = match git::get_conflict_info(file) {
      Ok(info) => info,
      Err(e) => {
        ui::warn(format!(
          "Cannot extract conflict versions for {}: {}",
          file, e
        ));
        continue;
      }
    };

    if conflict_info.ours.is_empty() && conflict_info.theirs.is_empty() {
      ui::warn(format!("Cannot extract conflict versions for {}", file));
      continue;
    }

    ui::info("Analyzing conflict...");

    let resolution = match llm::suggest_conflict_resolution(
      file,
      &conflict_info.ours,
      &conflict_info.theirs,
      &conflict_info.base,
    ) {
      Ok(r) => r,
      Err(e) => {
        ui::error(format!("Failed to analyze conflict: {}", e));
        continue;
      }
    };

    println!();
    ui::heading("AI Suggested Resolution:");
    ui::separator();
    println!("{}", resolution);
    ui::separator();
    println!();

    let options = [
      "Apply this resolution",
      "Keep ours (current branch)",
      "Keep theirs (incoming)",
      "Skip this file",
    ];

    let action = ui::select("What would you like to do?", &options).unwrap_or("Skip this file");

    let content = match action {
      "Apply this resolution" => resolution,
      "Keep ours (current branch)" => conflict_info.ours,
      "Keep theirs (incoming)" => conflict_info.theirs,
      _ => continue,
    };

    fs::write(file, &content)?;
    git::stage_file(file)?;
    ui::success(format!("Resolved: {}", file));
  }

  // Check remaining conflicts
  let remaining = git::status()?;
  if remaining.conflicts.is_empty() {
    ui::success("All conflicts resolved!");

    if ui::confirm("Continue rebase/merge?", true) {
      match git::continue_rebase() {
        Ok(()) => {
          ui::success("Rebase continued successfully");
        }
        Err(e) => {
          let msg = e.to_string();
          if msg.contains("No rebase") {
            ui::dim("No active rebase. If this was a merge, run 'git commit' to complete.");
          } else if msg.contains("conflict") {
            ui::warn("More conflicts encountered");
            ui::dim("Run 'alfred resolve' again");
          } else {
            ui::error(msg);
          }
        }
      }
    }
  } else {
    ui::warn(format!(
      "{} conflict(s) remaining",
      remaining.conflicts.len()
    ));
  }

  Ok(())
}
