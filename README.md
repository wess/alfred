# Alfred

**AI-powered git workflow assistant** — A drop-in replacement for git with local AI superpowers.

Alfred wraps git with intelligent features powered by a local LLM (no API keys, no subscriptions, runs entirely offline). Use it exactly like git, but with AI-assisted commits, rebasing, conflict resolution, and branch management.

## Features

- **Drop-in git replacement** — All git commands pass through seamlessly
- **AI commit messages** — Generate conventional commit messages from your staged changes
- **Smart rebasing** — Get AI suggestions for squashing, reordering, and rewording commits
- **Conflict resolution** — AI-assisted merge conflict resolution
- **Branch management** — AI-suggested branch names and smart cleanup of merged branches
- **100% local** — Uses llama.cpp with Phi-3 Mini, runs entirely on your machine
- **No subscriptions** — Free forever, no API keys needed

## Installation

### Quick Install (macOS/Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
```

Then download the AI model:

```bash
alfred setup
```

That's it! You're ready to use Alfred.

### Install Options

**Custom install directory:**
```bash
ALFRED_INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
```

**Specific version:**
```bash
ALFRED_VERSION=v0.1.0 curl -fsSL https://raw.githubusercontent.com/wesscope/alfred/main/install.sh | bash
```

### Build from Source

If you prefer to build from source or pre-built binaries aren't available for your platform:

**Prerequisites:**
- [Rust](https://rustup.rs/) (1.70+)
- Clang (for llama.cpp compilation)

```bash
# Clone the repository
git clone https://github.com/wesscope/alfred.git
cd alfred

# Build the binaries
cargo build --release

# Install to PATH
sudo cp target/release/alfred target/release/alferd /usr/local/bin/

# Download the AI model
alfred setup
```

### Windows

Download the latest release from [GitHub Releases](https://github.com/wesscope/alfred/releases) or build from source with Cargo:

```powershell
cargo install --git https://github.com/wesscope/alfred.git
alfred setup
```

## Usage

Alfred works as a transparent wrapper around git. Any command not handled by Alfred passes directly to git:

```bash
# These pass through to git
alfred status
alfred push origin main
alfred log --oneline
alfred stash pop

# These are AI-enhanced
alfred commit          # Generate AI commit message
alfred rebase main     # Smart rebase with AI suggestions
alfred resolve         # AI-assisted conflict resolution
alfred branch new      # Create branch with AI-suggested name
```

### AI-Enhanced Commands

#### `alfred commit`

Generate a conventional commit message from your staged changes:

```bash
alfred add -A
alfred commit
```

Alfred analyzes your diff and generates a commit message following the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
feat(auth): add OAuth2 login flow with Google provider
```

Options:
- `--edit`, `-e` — Edit the generated message before committing

#### `alfred rebase [branch]`

Interactive rebase with AI suggestions:

```bash
alfred rebase main --ai
```

Alfred analyzes your commits and suggests:
- Which commits to squash together
- Better ordering for logical flow
- Commits that should be reworded

#### `alfred resolve`

AI-assisted merge conflict resolution:

```bash
# During a merge or rebase with conflicts
alfred resolve
```

Alfred reads the conflicting files, analyzes both versions, and suggests merged resolutions that preserve the intent of both changes.

Options:
- `alfred resolve <file>` — Resolve a specific file only

#### `alfred branch`

Smart branch management:

```bash
# Create a new branch with AI-suggested name
alfred branch new
# Prompts: "What is this branch for?"
# Suggests: feature/add-user-authentication

# Create with a specific name
alfred branch new feature/my-feature

# Clean up merged branches
alfred branch clean

# List branches
alfred branch list
alfred branch list --all  # Include remotes
```

### Configuration

```bash
# Show current configuration
alfred config

# Set custom model path
alfred config --model=/path/to/model.gguf

# Reset to defaults
alfred config --reset
```

Configuration is stored in `~/.alfred/config.yaml`.

### Daemon

Alfred includes a background daemon (`alferd`) that keeps the AI model loaded in memory for instant responses:

```bash
# Start the daemon
alfred daemon start

# Check status
alfred daemon status

# Stop the daemon
alfred daemon stop

# Install as system service (starts at login)
alfred daemon install
```

Without the daemon, each command takes 2-3 seconds to load the model. With the daemon running, responses are instant.

See [docs/daemon.md](docs/daemon.md) for full documentation.

### Setup

```bash
alfred setup
```

Downloads and configures:
- **AI model** — Phi-3 Mini 4K Q4 by default (~2.4GB)

Available models:
- **Phi-3 Mini 4K (Q4)** — Recommended, fast, 2.4GB
- **Phi-3 Mini 4K (Q8)** — Higher quality, 4.1GB
- **Qwen2.5-Coder 1.5B (Q4)** — Code-focused, 1.0GB

Files are stored in `~/.alfred/`:
```
~/.alfred/
├── config.yaml
└── models/
    └── phi-3-mini-q4.gguf
```

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                        Alfred CLI                        │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │    Clap     │  │ Git Wrapper │  │  llama-cpp-2    │ │
│  │             │  │             │  │                 │ │
│  │ • Commands  │  │ • status    │  │ • Native Rust   │ │
│  │ • Flags     │  │ • diff      │  │ • Phi-3 Mini    │ │
│  │ • Passthru  │  │ • commit    │  │ • Tokenization  │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │     Git     │
                    └─────────────┘
```

### Native LLM Integration

Alfred uses the `llama-cpp-2` Rust crate for native llama.cpp integration, providing safe bindings to the C++ inference engine without FFI overhead.

### Git Passthrough

Any command not recognized as an Alfred command passes directly to git with full stdin/stdout/stderr inheritance.

## Development

### Project Structure

```
alfred/
├── Cargo.toml
├── install.sh               # Curl-based installer
├── src/
│   ├── main.rs              # CLI entry point (clap)
│   ├── lib.rs               # Library exports
│   ├── config.rs            # YAML configuration
│   ├── daemon_client.rs     # Daemon TCP client
│   ├── git.rs               # Git command wrapper
│   ├── llm.rs               # LLM integration (llama-cpp-2)
│   ├── ui.rs                # Terminal UI helpers
│   ├── bin/
│   │   └── alferd.rs        # Background daemon binary
│   └── cli/
│       └── commands/        # Command handlers
│           ├── setup.rs
│           ├── commit.rs
│           ├── rebase.rs
│           ├── resolve.rs
│           ├── branch.rs
│           ├── config.rs
│           ├── daemon.rs
│           └── help.rs
├── docs/                    # Documentation
│   ├── README.md
│   ├── getting-started.md
│   ├── tutorial.md
│   ├── commands.md
│   ├── daemon.md
│   └── configuration.md
└── README.md
```

### Building

```bash
cargo build              # Debug build
cargo build --release    # Release build (optimized)
cargo run -- <command>   # Run directly
```

### Dependencies

- `clap` — CLI framework with derive macros
- `llama-cpp-2` — Native Rust bindings to llama.cpp
- `serde` + `serde_yaml` — YAML configuration
- `tokio` — Async runtime
- `dialoguer` — Interactive prompts
- `colored` — Terminal colors
- `reqwest` — HTTP downloads
- `indicatif` — Progress bars

## Troubleshooting

### "Model not found" Error

Run `alfred setup` to download the required model:

```bash
alfred setup
```

### Build Errors

Ensure you have Clang installed for llama.cpp compilation:
- macOS: `xcode-select --install`
- Linux: `apt install clang` or `dnf install clang`

### Slow First Run

The first inference takes longer as the model loads into memory. Subsequent calls are faster.

### High Memory Usage

The default Phi-3 Mini model uses ~3-4GB RAM during inference. For lower memory usage, try the Qwen 1.5B model during setup.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [llama.cpp](https://github.com/ggerganov/llama.cpp) — Inference engine
- [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) — Rust bindings
- [Phi-3](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf) — Default AI model by Microsoft
