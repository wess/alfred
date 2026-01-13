#![allow(dead_code)]

mod cli;
mod config;
mod daemon_client;
mod git;
mod llm;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "alfred",
    about = "AI-powered git workflow assistant",
    version,
    arg_required_else_help = false
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Arguments to pass through to git
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    git_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Download AI model and configure alfred
    Setup,

    /// Generate AI commit message from staged changes
    Commit {
        /// Edit the message before committing
        #[arg(short, long)]
        edit: bool,
    },

    /// Interactive rebase with AI suggestions
    Rebase {
        /// Branch to rebase onto
        onto: Option<String>,

        /// Get AI suggestions for the rebase
        #[arg(long)]
        ai: bool,

        /// Alias for --ai
        #[arg(long)]
        suggest: bool,
    },

    /// AI-assisted merge conflict resolution
    Resolve {
        /// Specific file to resolve
        file: Option<String>,
    },

    /// Smart branch management
    Branch {
        #[command(subcommand)]
        subcmd: Option<BranchCommands>,
    },

    /// Configure alfred settings
    Config {
        /// Set custom model path
        #[arg(long)]
        model: Option<String>,

        /// Reset configuration to defaults
        #[arg(long)]
        reset: bool,
    },

    /// Manage the alfred daemon (keeps model loaded for fast inference)
    Daemon {
        #[command(subcommand)]
        action: Option<DaemonAction>,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Check daemon status
    Status,
    /// Install as system service (launchd on macOS, systemd on Linux)
    Install,
    /// Uninstall system service
    Uninstall,
}

#[derive(Subcommand)]
enum BranchCommands {
    /// Create new branch with AI-suggested name
    #[command(alias = "create")]
    New {
        /// Branch name (optional, will prompt if not provided)
        name: Option<String>,
    },

    /// Delete merged branches
    #[command(alias = "cleanup")]
    Clean {
        /// Delete without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// List branches
    #[command(alias = "ls")]
    List {
        /// Show remote branches too
        #[arg(short, long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            ui::error(format!("{}", e));
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<i32> {
    let cli = Cli::parse();

    // If no subcommand and we have git args, pass through to git
    if cli.command.is_none() {
        if cli.git_args.is_empty() {
            // No args at all, show our custom help
            cli::commands::help::run();
            return Ok(0);
        }
        // Pass through to git
        let code = git::passthrough(&cli.git_args)?;
        return Ok(code);
    }

    // Handle alfred commands
    match cli.command.unwrap() {
        Commands::Setup => {
            cli::commands::setup::run().await?;
        }
        Commands::Commit { edit } => {
            ensure_git_repo()?;
            cli::commands::commit::run(edit)?;
        }
        Commands::Rebase { onto, ai, suggest } => {
            ensure_git_repo()?;
            cli::commands::rebase::run(onto, ai || suggest)?;
        }
        Commands::Resolve { file } => {
            ensure_git_repo()?;
            cli::commands::resolve::run(file)?;
        }
        Commands::Branch { subcmd } => {
            ensure_git_repo()?;
            match subcmd {
                Some(BranchCommands::New { name }) => {
                    cli::commands::branch::new_branch(name)?;
                }
                Some(BranchCommands::Clean { force }) => {
                    cli::commands::branch::clean(force)?;
                }
                Some(BranchCommands::List { all }) => {
                    cli::commands::branch::list(all)?;
                }
                None => {
                    cli::commands::branch::show_help();
                }
            }
        }
        Commands::Config { model, reset } => {
            cli::commands::config::run(model, reset)?;
        }
        Commands::Daemon { action } => {
            match action {
                Some(DaemonAction::Start) => {
                    cli::commands::daemon::start()?;
                }
                Some(DaemonAction::Stop) => {
                    cli::commands::daemon::stop()?;
                }
                Some(DaemonAction::Status) => {
                    cli::commands::daemon::status()?;
                }
                Some(DaemonAction::Install) => {
                    cli::commands::daemon::install()?;
                }
                Some(DaemonAction::Uninstall) => {
                    cli::commands::daemon::uninstall()?;
                }
                None => {
                    cli::commands::daemon::show_help();
                }
            }
        }
    }

    Ok(0)
}

fn ensure_git_repo() -> Result<()> {
    if !git::is_git_repo() {
        ui::error("Not a git repository");
        ui::dim("Run this command from within a git repository");
        std::process::exit(1);
    }
    Ok(())
}
