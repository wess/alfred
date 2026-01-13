use colored::Colorize;

pub fn run() {
  println!(
    r#"
{} - AI-powered git workflow assistant

{}
  alfred <command> [options]

  Alfred passes through all standard git commands, so you can use it
  as a drop-in replacement for git. AI-enhanced commands are listed below.

{}
  {}     Download AI model and configure alfred
  {}    Generate AI commit message from staged changes
  {}    Interactive rebase with AI suggestions
  {}   AI-assisted merge conflict resolution
  {}    Smart branch management
  {}    Configure alfred settings
  {}      Show this help message

{}
  All other commands are passed directly to git:
  {}            → git status
  {}              → git push
  {}     → git log --oneline
  {}             → git stash

{}
  alfred setup               Download AI model (run this first!)
  alfred commit              Generate commit message for staged changes
  alfred commit --edit       Generate and edit before committing
  alfred rebase main         Rebase onto main with AI suggestions
  alfred resolve             Resolve all conflicts with AI assistance
  alfred branch new          Create branch with AI-suggested name
  alfred branch clean        Clean up merged branches

{}
  Run 'alfred setup' to download llama.cpp and a local AI model.
  Everything runs locally - no API keys or subscriptions needed.
  Files stored in ~/.alfred/
"#,
    "alfred".bold(),
    "USAGE".bold(),
    "AI-ENHANCED COMMANDS".bold(),
    "setup".cyan(),
    "commit".cyan(),
    "rebase".cyan(),
    "resolve".cyan(),
    "branch".cyan(),
    "config".cyan(),
    "help".cyan(),
    "GIT PASSTHROUGH".bold(),
    "alfred status".dimmed(),
    "alfred push".dimmed(),
    "alfred log --oneline".dimmed(),
    "alfred stash".dimmed(),
    "EXAMPLES".bold(),
    "GETTING STARTED".bold(),
  );
}
