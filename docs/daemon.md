# Alfred Daemon (alferd)

The Alfred daemon is a background service that keeps the AI model loaded in memory, providing instant responses without the 2-3 second model loading time.

## Overview

```
┌─────────────────┐         TCP localhost:7654        ┌─────────────────┐
│   alfred CLI    │ ◄──────────────────────────────► │    alferd       │
│                 │         JSON-RPC messages         │   (daemon)      │
│  • commit       │                                   │                 │
│  • branch new   │                                   │  • LLM loaded   │
│  • resolve      │                                   │  • Idle timer   │
│  • rebase --ai  │                                   │  • Model mgmt   │
└─────────────────┘                                   └─────────────────┘
```

When you run an Alfred command:
1. Alfred tries to connect to the daemon on localhost:7654
2. If connected, sends the request and gets instant response
3. If not connected, falls back to loading the model locally

## Quick Start

```bash
# Start the daemon
alfred daemon start

# Check status
alfred daemon status

# Stop the daemon
alfred daemon stop
```

## Commands

### Start

Start the daemon in the background:

```bash
alfred daemon start
```

Output:
```
i Starting alfred daemon...
✓ Daemon started (PID: 12345)
Model will be loaded on first request
```

The daemon loads the model on the first request, not at startup. This keeps startup fast while still providing quick responses after the first use.

### Stop

Stop the running daemon:

```bash
alfred daemon stop
```

Output:
```
i Stopping daemon...
✓ Daemon stopped
```

### Status

Check daemon status and configuration:

```bash
alfred daemon status
```

Output:
```
Daemon Status

  Status: Running
  PID: 12345
  Port: 7654
  Idle timeout: 30 minutes
  Service: Not installed
```

### Install as Service

Install the daemon as a system service that starts automatically at login:

```bash
alfred daemon install
```

**macOS (launchd):**
```
✓ Daemon installed as launchd service
Plist: /Users/you/Library/LaunchAgents/com.alfred.daemon.plist
The daemon will start automatically at login
```

**Linux (systemd):**
```
✓ Daemon installed as systemd user service
Service file: /home/you/.config/systemd/user/alfred.service
The daemon will start automatically at login
```

### Uninstall Service

Remove the system service:

```bash
alfred daemon uninstall
```

## Configuration

Daemon settings are stored in `~/.alfred/config.yaml`:

```yaml
daemon:
  port: 7654                    # TCP port to listen on
  idle_timeout_minutes: 30      # Auto-shutdown after idle (0 = never)
  auto_start: false             # Reserved for future use
```

### Port

The daemon listens on `127.0.0.1:7654` by default. To change:

```yaml
daemon:
  port: 8888
```

Note: Both `alfred` and `alferd` read this config, so they'll automatically use the same port.

### Idle Timeout

By default, the daemon shuts down after 30 minutes of inactivity to free memory. To change:

```yaml
daemon:
  idle_timeout_minutes: 60  # 1 hour
```

To disable auto-shutdown:

```yaml
daemon:
  idle_timeout_minutes: 0  # Never timeout
```

## How It Works

### Communication Protocol

The daemon uses a simple JSON-RPC protocol over TCP:

**Request:**
```json
{"method": "generate_commit_message", "params": {"diff": "..."}, "id": 1}
```

**Response:**
```json
{"result": "feat(auth): add login endpoint", "id": 1}
```

### Available Methods

| Method | Parameters | Description |
|--------|------------|-------------|
| `ping` | none | Health check, returns "pong" |
| `shutdown` | none | Graceful shutdown |
| `generate` | `prompt`, `max_tokens` | Raw text generation |
| `generate_commit_message` | `diff` | Generate commit message |
| `suggest_branch_name` | `description` | Suggest branch name |
| `suggest_conflict_resolution` | `file`, `ours`, `theirs`, `base` | Resolve conflict |
| `suggest_rebase_strategy` | `commits`, `onto` | Rebase suggestions |

### Fallback Behavior

If the daemon isn't running, Alfred automatically falls back to local model loading:

```
alfred commit
# Daemon not running, loading model...
# (2-3 second delay)
# Generated commit message:
# ...
```

This means Alfred always works, with or without the daemon.

## Service Management

### macOS (launchd)

The launchd plist is installed at:
```
~/Library/LaunchAgents/com.alfred.daemon.plist
```

Manual control:
```bash
# Load/start
launchctl load ~/Library/LaunchAgents/com.alfred.daemon.plist

# Unload/stop
launchctl unload ~/Library/LaunchAgents/com.alfred.daemon.plist

# Check if loaded
launchctl list | grep alfred
```

Logs are written to:
- `~/.alfred/alferd.log` - Standard output
- `~/.alfred/alferd.error.log` - Error output

### Linux (systemd)

The systemd unit is installed at:
```
~/.config/systemd/user/alfred.service
```

Manual control:
```bash
# Start
systemctl --user start alfred

# Stop
systemctl --user stop alfred

# Restart
systemctl --user restart alfred

# Check status
systemctl --user status alfred

# View logs
journalctl --user -u alfred -f
```

## Troubleshooting

### Daemon Won't Start

**Port already in use:**
```
Error: Failed to bind to 127.0.0.1:7654
```

Solution: Check what's using the port and either stop it or change Alfred's port:
```bash
lsof -i :7654
```

**Model not found:**
```
Error loading model: Model not found at ...
```

Solution: Run `alfred setup` to download the model.

### Daemon Not Responding

If `alfred daemon status` shows "Running" but commands are slow:

```bash
# Stop and restart
alfred daemon stop
alfred daemon start
```

### Service Won't Install

**macOS permission error:**
```bash
# Make sure LaunchAgents directory exists
mkdir -p ~/Library/LaunchAgents
```

**Linux systemd not available:**
```bash
# Check if user systemd is running
systemctl --user status
```

### High Memory Usage

The daemon keeps the model in memory (~2-4GB). If memory is tight:

1. Reduce idle timeout to free memory sooner:
   ```yaml
   daemon:
     idle_timeout_minutes: 5
   ```

2. Or don't use the daemon - Alfred works fine without it, just with a small startup delay.

### Checking Logs

**macOS:**
```bash
tail -f ~/.alfred/alferd.log
tail -f ~/.alfred/alferd.error.log
```

**Linux:**
```bash
journalctl --user -u alfred -f
```

**Manual daemon (foreground):**

Run the daemon in the foreground to see all output:
```bash
alferd
```

## Performance Comparison

| Scenario | First Command | Subsequent Commands |
|----------|---------------|---------------------|
| No daemon | ~2-3 seconds | ~2-3 seconds |
| Daemon running | ~2-3 seconds (model load) | <100ms |
| Daemon with model loaded | <100ms | <100ms |

The daemon provides the biggest benefit when you're making multiple commits or using Alfred frequently.
