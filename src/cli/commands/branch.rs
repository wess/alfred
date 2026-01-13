use anyhow::Result;
use colored::Colorize;

use crate::{git, llm, ui};

pub fn new_branch(name: Option<String>) -> Result<()> {
  let branch_name = match name {
    Some(n) => n,
    None => {
      let description = match ui::prompt("Describe what this branch is for:") {
        Some(d) => d,
        None => {
          ui::info("Aborted");
          return Ok(());
        }
      };

      ui::info("Generating branch name...");

      match llm::suggest_branch_name(&description) {
        Ok(suggested) => {
          ui::info(format!("Suggested: {}", suggested.cyan()));

          if ui::confirm("Use this name?", true) {
            suggested
          } else {
            match ui::prompt("Enter branch name:") {
              Some(n) => n,
              None => {
                ui::info("Aborted");
                return Ok(());
              }
            }
          }
        }
        Err(e) => {
          ui::error(format!("Failed to generate name: {}", e));
          match ui::prompt("Enter branch name:") {
            Some(n) => n,
            None => {
              ui::info("Aborted");
              return Ok(());
            }
          }
        }
      }
    }
  };

  if branch_name.is_empty() {
    ui::info("Aborted");
    return Ok(());
  }

  // Sanitize branch name
  let sanitized: String = branch_name
    .to_lowercase()
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == '/' || c == '_' || c == '-' {
        c
      } else {
        '-'
      }
    })
    .collect::<String>()
    .split('-')
    .filter(|s| !s.is_empty())
    .collect::<Vec<&str>>()
    .join("-");

  match git::create_branch(&sanitized) {
    Ok(()) => {
      ui::success(format!("Created and switched to: {}", sanitized));
    }
    Err(e) => {
      ui::error(format!("Failed to create branch: {}", e));
    }
  }

  Ok(())
}

pub fn clean(force: bool) -> Result<()> {
  let status = git::status()?;
  let branches = git::get_branches()?;

  // Find base branch
  let base_branch = if branches.contains(&"main".to_string()) {
    "main"
  } else if branches.contains(&"master".to_string()) {
    "master"
  } else {
    branches.first().map(|s| s.as_str()).unwrap_or("main")
  };

  ui::info(format!(
    "Checking for branches merged into {}...",
    base_branch.cyan()
  ));

  let merged = git::get_merged_branches(base_branch)?;

  if merged.is_empty() {
    ui::success("No merged branches to clean up!");
    return Ok(());
  }

  ui::heading(format!("Found {} merged branch(es):", merged.len()));
  for b in &merged {
    let current = if *b == status.branch {
      format!(" {}", "(current)".yellow())
    } else {
      String::new()
    };
    ui::list_item(format!("{}{}", b, current));
  }
  println!();

  let to_delete: Vec<&String> = merged.iter().filter(|b| *b != &status.branch).collect();

  if to_delete.is_empty() {
    ui::info("All merged branches are currently checked out");
    return Ok(());
  }

  if force {
    for branch in to_delete {
      git::delete_branch(branch, false)?;
      ui::success(format!("Deleted: {}", branch));
    }
  } else if ui::confirm(
    &format!("Delete {} merged branch(es)?", to_delete.len()),
    true,
  ) {
    for branch in to_delete {
      match git::delete_branch(branch, false) {
        Ok(()) => {
          ui::success(format!("Deleted: {}", branch));
        }
        Err(e) => {
          ui::error(format!("Failed to delete {}: {}", branch, e));
        }
      }
    }
  } else {
    ui::info("Aborted");
  }

  Ok(())
}

pub fn list(all: bool) -> Result<()> {
  let status = git::status()?;
  let branches = git::get_branches()?;

  ui::heading("Local branches:");
  for branch in &branches {
    let marker = if *branch == status.branch {
      format!("{} ", "*".green())
    } else {
      "  ".to_string()
    };
    println!("{}{}", marker, branch);
  }

  if all {
    let remote = git::get_remote_branches()?;
    if !remote.is_empty() {
      println!();
      ui::heading("Remote branches:");
      for b in &remote {
        println!("  {}", b.red());
      }
    }
  }

  Ok(())
}

pub fn show_help() {
  println!(
    r#"
{} - Smart branch management

{}
  alfred branch <subcommand>

{}
  {}, {}    Create new branch with AI-suggested name
  {}, {}  Delete merged branches
  {}, {}       List branches

{}
  --all, -a    Show remote branches (for list)
  --force, -f  Delete without confirmation (for clean)

{}
  alfred branch new                 Create branch with AI name suggestion
  alfred branch new feature/auth    Create specific branch
  alfred branch clean               Clean up merged branches
  alfred branch list --all          List all branches including remotes
"#,
    "alfred branch".bold(),
    "USAGE".bold(),
    "SUBCOMMANDS".bold(),
    "new".cyan(),
    "create".cyan(),
    "clean".cyan(),
    "cleanup".cyan(),
    "list".cyan(),
    "ls".cyan(),
    "OPTIONS".bold(),
    "EXAMPLES".bold(),
  );
}
