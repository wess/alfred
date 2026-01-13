# Alfred Documentation

Alfred is an AI-powered git workflow assistant that runs entirely on your local machine. It uses a small, efficient language model (Phi-3 Mini) to help you write better commit messages, resolve merge conflicts, manage branches, and streamline your git workflow.

## Table of Contents

- [Getting Started](./getting-started.md) - Installation, setup, and your first commands
- [Tutorial](./tutorial.md) - Step-by-step walkthrough of common workflows
- [Command Reference](./commands.md) - Complete list of all commands and options
- [Daemon](./daemon.md) - Background service for faster inference
- [Configuration](./configuration.md) - Customizing Alfred's behavior

## Quick Start

```bash
# Install (after building)
cargo install --path .

# Download the AI model
alfred setup

# Generate a commit message from staged changes
git add .
alfred commit

# Create a branch with AI-suggested name
alfred branch new
```

## Features

### AI-Powered Commit Messages
Alfred analyzes your staged changes and generates conventional commit messages following best practices.

### Smart Branch Naming
Describe what you're working on, and Alfred suggests appropriate branch names following common conventions (feature/, bugfix/, hotfix/, chore/).

### Merge Conflict Resolution
When you hit a merge conflict, Alfred can analyze both sides and suggest a resolution that preserves the intent of both changes.

### Rebase Strategy Suggestions
Get AI recommendations on which commits to squash, reorder, or reword during interactive rebases.

### Git Passthrough
Any command Alfred doesn't recognize gets passed through to git, so you can use `alfred` as a drop-in replacement for `git`.

## Architecture

Alfred consists of two binaries:

| Binary | Description |
|--------|-------------|
| `alfred` | Main CLI tool for all git operations |
| `alferd` | Background daemon that keeps the model loaded for instant inference |

When you run an Alfred command, it first tries to connect to the daemon. If the daemon is running, you get instant responses. If not, Alfred loads the model on-demand (which takes a few seconds on first use).

## Requirements

- **Operating System**: macOS, Linux, or Windows
- **Disk Space**: ~2GB for the AI model
- **RAM**: 4GB minimum, 8GB recommended
- **CPU**: Any modern x86_64 or ARM64 processor

## Privacy

Alfred runs 100% locally. Your code never leaves your machine:

- The AI model runs on your CPU
- No internet connection required after setup
- No telemetry or analytics
- No cloud API calls

## License

MIT License - see [LICENSE](../LICENSE) for details.
