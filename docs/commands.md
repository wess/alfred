# Command Reference

Complete reference for all Alfred commands.

## Overview

```
alfred [COMMAND] [OPTIONS]
alfred [GIT_ARGS...]
```

Alfred commands take precedence over git. Any unrecognized command is passed through to git.

## Commands

### setup

Download and configure the AI model.

```bash
alfred setup
```

This command:
- Creates `~/.alfred` directory structure
- Downloads the Phi-3 Mini model (~2GB)
- Verifies the model loads correctly

Only needs to be run once after installation.

---

### commit

Generate an AI commit message from staged changes.

```bash
alfred commit [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-e, --edit` | Open editor to modify the generated message |

**Examples:**

```bash
# Generate and commit with AI message
git add .
alfred commit

# Generate message but edit before committing
alfred commit --edit
```

**How it works:**

1. Runs `git diff --cached` to get staged changes
2. Sends the diff to the AI model
3. Generates a conventional commit message
4. Prompts for confirmation
5. Runs `git commit -m "message"` if confirmed

**Message format:**

Alfred generates messages following [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

- bullet point details
- another detail
```

Types used: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

---

### branch

Smart branch management with AI assistance.

```bash
alfred branch [SUBCOMMAND]
```

#### branch new

Create a new branch with an AI-suggested name.

```bash
alfred branch new [NAME]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `NAME` | Optional branch name. If omitted, prompts for description and suggests name |

**Examples:**

```bash
# Interactive - describe your work, get suggested name
alfred branch new
# ? What are you working on? Adding user authentication
# Suggested: feature/add-user-authentication
# ? Use this name? (Y/n)

# Direct - create branch with specific name
alfred branch new feature/my-feature
```

**Naming conventions:**

Alfred suggests names following common patterns:
- `feature/` - New features
- `bugfix/` - Bug fixes
- `hotfix/` - Urgent production fixes
- `chore/` - Maintenance tasks

#### branch clean

Delete branches that have been merged.

```bash
alfred branch clean [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-f, --force` | Delete without confirmation prompts |

**Examples:**

```bash
# Interactive - confirm each deletion
alfred branch clean

# Delete all merged branches without prompting
alfred branch clean --force
```

#### branch list

List branches with status information.

```bash
alfred branch list [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `-a, --all` | Include remote branches |

**Examples:**

```bash
# List local branches
alfred branch list

# List all branches including remotes
alfred branch list --all
```

---

### rebase

Interactive rebase with optional AI suggestions.

```bash
alfred rebase [ONTO] [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `ONTO` | Branch or commit to rebase onto (default: upstream or main) |

**Options:**

| Option | Description |
|--------|-------------|
| `--ai` | Get AI suggestions for rebase strategy |
| `--suggest` | Alias for `--ai` |

**Examples:**

```bash
# Standard rebase onto main
alfred rebase main

# Rebase with AI suggestions
alfred rebase main --ai
```

**AI suggestions include:**

- Which commits to squash together
- Commits that should be reordered
- Commit messages that could be improved

---

### resolve

AI-assisted merge conflict resolution.

```bash
alfred resolve [FILE]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `FILE` | Specific file to resolve. If omitted, resolves all conflicts |

**Examples:**

```bash
# Resolve all conflicts
alfred resolve

# Resolve specific file
alfred resolve src/main.rs
```

**How it works:**

1. Parses conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
2. Extracts "ours", "theirs", and optionally "base" versions
3. Sends to AI for analysis
4. Suggests merged resolution
5. Prompts to accept or edit

---

### config

View and modify Alfred configuration.

```bash
alfred config [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--model PATH` | Set custom model path |
| `--reset` | Reset configuration to defaults |

**Examples:**

```bash
# View current configuration
alfred config

# Set custom model path
alfred config --model /path/to/custom-model.gguf

# Reset to defaults
alfred config --reset
```

---

### daemon

Manage the background daemon for faster inference.

```bash
alfred daemon [SUBCOMMAND]
```

See [Daemon Documentation](./daemon.md) for full details.

#### daemon start

Start the background daemon.

```bash
alfred daemon start
```

#### daemon stop

Stop the running daemon.

```bash
alfred daemon stop
```

#### daemon status

Check daemon status and configuration.

```bash
alfred daemon status
```

**Output:**
```
Daemon Status

  Status: Running
  PID: 12345
  Port: 7654
  Idle timeout: 30 minutes
  Service: Installed
```

#### daemon install

Install daemon as a system service.

```bash
alfred daemon install
```

- **macOS**: Creates launchd plist at `~/Library/LaunchAgents/com.alfred.daemon.plist`
- **Linux**: Creates systemd unit at `~/.config/systemd/user/alfred.service`

#### daemon uninstall

Remove daemon system service.

```bash
alfred daemon uninstall
```

---

## Git Passthrough

Any command not listed above is passed directly to git:

```bash
# These are equivalent:
alfred status
git status

alfred log --oneline -10
git log --oneline -10

alfred push origin main
git push origin main
```

This means you can use `alfred` as your primary git command and get AI features when you need them.

## Global Options

These options work with any command:

| Option | Description |
|--------|-------------|
| `-h, --help` | Print help information |
| `-V, --version` | Print version |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 128+ | Git error (passed through from git) |
