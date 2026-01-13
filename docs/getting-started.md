# Getting Started with Alfred

This guide will help you install Alfred, download the AI model, and run your first commands.

## Prerequisites

Before installing Alfred, make sure you have:

- **Git** - Alfred is a git workflow tool, so git must be installed
- **2GB free disk space** - For the AI model
- **Internet connection** - Only needed for initial setup

## Installation

### Quick Install (Recommended)

Run the installer script:

```bash
curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
```

This automatically:
- Detects your OS and architecture
- Downloads the appropriate binary
- Installs to `/usr/local/bin`

**Custom install location:**
```bash
ALFRED_INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
```

### From Source

If pre-built binaries aren't available for your platform, build from source:

**Prerequisites for building:**
- [Rust toolchain](https://rustup.rs) (1.70 or later)
- Clang (for llama.cpp compilation)

```bash
git clone https://github.com/wesscope/alfred.git
cd alfred
cargo build --release
```

The binaries will be at:
- `target/release/alfred` - Main CLI
- `target/release/alferd` - Background daemon

Install to PATH:

```bash
sudo cp target/release/alfred target/release/alferd /usr/local/bin/
```

### Verify Installation

```bash
alfred --version
# alfred 0.1.0
```

## Initial Setup

Run the setup command to download the AI model:

```bash
alfred setup
```

This will:
1. Create the `~/.alfred` directory
2. Download the Phi-3 Mini model (~2GB)
3. Verify the model works correctly

The download takes a few minutes depending on your internet speed. You'll see a progress bar:

```
Alfred Setup
Downloading Phi-3 Mini model...
████████████████████████████████████████ 100% (1.8 GB)
✓ Model downloaded successfully!
Testing model...
✓ Model loaded and working!
Setup complete!
```

## Your First Command

Navigate to any git repository with staged changes:

```bash
cd your-project
git add .
alfred commit
```

Alfred will analyze your changes and generate a commit message:

```
Analyzing staged changes...
Generated commit message:

  feat(auth): add password reset functionality

  - Add forgot password email template
  - Implement token generation and validation
  - Create reset password API endpoint

? Use this message? (Y/n)
```

Press Enter to accept, or `n` to edit the message.

## Starting the Daemon (Optional)

For instant responses, start the background daemon:

```bash
alfred daemon start
```

The daemon keeps the AI model loaded in memory, eliminating the 2-3 second startup time. See [Daemon Documentation](./daemon.md) for more details.

## Verifying Everything Works

Run these commands to verify your installation:

```bash
# Check Alfred version
alfred --version

# Check daemon status
alfred daemon status

# View available commands
alfred --help

# Test in a git repo
cd any-git-repo
alfred branch list
```

## Troubleshooting

### "Model not found" Error

```
Error: Model not found at /Users/you/.alfred/models/phi-3-mini-q4.gguf
Run 'alfred setup' to download it.
```

**Solution**: Run `alfred setup` to download the model.

### Setup Download Fails

If the download fails partway through:

```bash
# Remove partial download and retry
rm -rf ~/.alfred/models
alfred setup
```

### Slow First Command

The first command after a reboot takes 2-3 seconds while the model loads. This is normal. Start the daemon for instant responses:

```bash
alfred daemon start
```

### Permission Denied

If you get permission errors when installing:

```bash
# Use sudo for system-wide install
sudo cp target/release/alfred /usr/local/bin/

# Or install to user directory
mkdir -p ~/.local/bin
cp target/release/alfred ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

### Git Repository Not Found

```
Error: Not a git repository
```

**Solution**: Make sure you're in a directory that's part of a git repository.

## Next Steps

- Read the [Tutorial](./tutorial.md) for a complete workflow walkthrough
- See [Command Reference](./commands.md) for all available commands
- Configure Alfred with [Configuration Guide](./configuration.md)
- Set up the [Daemon](./daemon.md) for faster responses
