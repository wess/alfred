use anyhow::Result;
use colored::Colorize;

use crate::{git, llm, ui};

pub fn run(edit: bool) -> Result<()> {
    let status = git::status()?;

    if status.staged.is_empty() {
        if !status.unstaged.is_empty() || !status.untracked.is_empty() {
            ui::warn("No staged changes. Stage files first with 'git add'");
            println!();

            if !status.unstaged.is_empty() {
                ui::dim("Modified files:");
                for f in &status.unstaged {
                    println!("  {} {}", "M".yellow(), f);
                }
            }

            if !status.untracked.is_empty() {
                ui::dim("Untracked files:");
                for f in &status.untracked {
                    println!("  {} {}", "?".red(), f);
                }
            }

            println!();

            if ui::confirm("Stage all changes?", true) {
                let all_files: Vec<String> = status
                    .unstaged
                    .into_iter()
                    .chain(status.untracked.into_iter())
                    .collect();
                git::add(&all_files)?;
                ui::success("Staged all changes");
            } else {
                return Ok(());
            }
        } else {
            ui::info("Nothing to commit");
            return Ok(());
        }
    }

    ui::info("Generating commit message...");

    let diff = git::diff(true)?;
    if diff.is_empty() {
        ui::error("Could not get diff of staged changes");
        return Ok(());
    }

    let mut message = match llm::generate_commit_message(&diff) {
        Ok(msg) => msg,
        Err(e) => {
            ui::error(format!("Failed to generate message: {}", e));
            ui::dim("Make sure you have run 'alfred setup'");
            return Ok(());
        }
    };

    println!();
    ui::heading("Generated commit message:");
    ui::separator();
    println!("{}", message);
    ui::separator();
    println!();

    if edit {
        if let Some(edited) = ui::prompt("Edit message (or press Enter to keep):") {
            message = edited;
        }
    }

    if ui::confirm("Commit with this message?", true) {
        git::commit(&message)?;
        ui::success("Committed!");
    } else {
        ui::info("Aborted");
    }

    Ok(())
}
