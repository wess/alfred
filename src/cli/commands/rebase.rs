use anyhow::Result;
use colored::Colorize;

use crate::{git, llm, ui};

pub fn run(onto: Option<String>, use_ai: bool) -> Result<()> {
    let status = git::status()?;

    if !status.staged.is_empty() || !status.unstaged.is_empty() {
        ui::warn("You have uncommitted changes. Commit or stash them first.");
        return Ok(());
    }

    let onto = match onto {
        Some(branch) => branch,
        None => {
            let branches = git::get_branches()?;
            let common_bases = ["main", "master", "develop", "dev"];

            let mut options: Vec<String> = common_bases
                .iter()
                .filter(|b| branches.contains(&b.to_string()))
                .map(|s| s.to_string())
                .collect();

            options.extend(
                branches
                    .into_iter()
                    .filter(|b| !common_bases.contains(&b.as_str()) && *b != status.branch),
            );

            if options.is_empty() {
                ui::error("No branches available to rebase onto");
                return Ok(());
            }

            ui::select("Rebase onto which branch?", &options)
                .ok_or_else(|| anyhow::anyhow!("No branch selected"))?
        }
    };

    ui::info(format!(
        "Analyzing commits for rebase onto {}...",
        onto.cyan()
    ));

    let commits = git::get_rebase_commits(&onto)?;

    if commits.is_empty() {
        ui::info(format!(
            "No commits to rebase. {} is already up to date with {}.",
            status.branch, onto
        ));
        return Ok(());
    }

    println!();
    ui::heading(format!("Commits to rebase ({}):", commits.len()));
    for c in &commits {
        ui::list_item(c);
    }
    println!();

    if use_ai {
        ui::info("Getting AI suggestions...");

        match llm::suggest_rebase_strategy(&commits, &onto) {
            Ok(suggestion) => {
                println!();
                ui::heading("AI Suggestion:");
                ui::separator();
                println!("{}", suggestion);
                ui::separator();
                println!();
            }
            Err(e) => {
                ui::error(format!("Failed to get suggestions: {}", e));
            }
        }
    }

    let options = [
        "Start interactive rebase",
        "Auto-rebase (no interaction)",
        "Abort",
    ];

    let action = ui::select("How would you like to proceed?", &options)
        .unwrap_or("Abort");

    if action == "Abort" {
        ui::info("Aborted");
        return Ok(());
    }

    let interactive = action == "Start interactive rebase";

    if interactive {
        ui::info("Starting interactive rebase...");
        ui::dim("This will open your editor. Use 'pick', 'squash', 'reword', etc.");
    } else {
        ui::info("Starting auto-rebase...");
    }

    match git::rebase(&onto, interactive) {
        Ok(()) => {
            ui::success(format!("Successfully rebased onto {}", onto));
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("conflict") || msg.contains("CONFLICT") {
                ui::warn("Rebase stopped due to conflicts");
                ui::info("Use 'alfred resolve' to resolve conflicts with AI assistance");
                ui::dim("Or use 'git rebase --abort' to cancel");
            } else {
                ui::error(format!("Rebase failed: {}", msg));
            }
        }
    }

    Ok(())
}
