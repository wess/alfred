# Configuration Guide

Alfred stores its configuration in `~/.alfred/config.yaml`. This guide covers all available options.

## Configuration File Location

```
~/.alfred/
├── config.yaml          # Configuration file
├── models/
│   └── phi-3-mini-q4.gguf  # AI model
├── alferd.pid           # Daemon PID file (when running)
├── alferd.log           # Daemon log (macOS only)
└── alferd.error.log     # Daemon error log (macOS only)
```

## Default Configuration

If no config file exists, Alfred uses these defaults:

```yaml
# Model path (optional - uses default if not set)
# model_path: ~/.alfred/models/phi-3-mini-q4.gguf

# Daemon settings
daemon:
  port: 7654
  idle_timeout_minutes: 30
  auto_start: false
```

## Configuration Options

### model_path

Path to the GGUF model file.

```yaml
model_path: /path/to/your/model.gguf
```

**Default:** `~/.alfred/models/phi-3-mini-q4.gguf`

**Notes:**
- Must be a valid GGUF format model
- Supports any llama.cpp compatible model
- Larger models require more RAM but may give better results

**Using a different model:**

```yaml
# Use a larger model for better quality
model_path: ~/.alfred/models/phi-3-medium-q4.gguf

# Or a smaller model for faster inference
model_path: ~/.alfred/models/phi-3-mini-q8.gguf
```

### daemon.port

TCP port for daemon communication.

```yaml
daemon:
  port: 7654
```

**Default:** `7654`

**Notes:**
- Must be available (not used by another process)
- Both `alfred` and `alferd` read this setting
- Only binds to localhost (127.0.0.1) for security

### daemon.idle_timeout_minutes

Minutes of inactivity before the daemon auto-shuts down.

```yaml
daemon:
  idle_timeout_minutes: 30
```

**Default:** `30`

**Options:**
- `0` - Never timeout (daemon runs until manually stopped)
- `1-1440` - Minutes until auto-shutdown

**Examples:**

```yaml
# Aggressive timeout for low-memory systems
daemon:
  idle_timeout_minutes: 5

# Keep running during work hours
daemon:
  idle_timeout_minutes: 480  # 8 hours

# Never auto-shutdown
daemon:
  idle_timeout_minutes: 0
```

### daemon.auto_start

Reserved for future use. Currently has no effect.

```yaml
daemon:
  auto_start: false
```

## Managing Configuration

### View Current Configuration

```bash
alfred config
```

Output:
```
Alfred Configuration

  Model: ~/.alfred/models/phi-3-mini-q4.gguf
  Config: ~/.alfred/config.yaml

Daemon:
  Port: 7654
  Idle timeout: 30 minutes
```

### Set Model Path

```bash
alfred config --model /path/to/model.gguf
```

This updates `config.yaml` with the new path.

### Reset to Defaults

```bash
alfred config --reset
```

This removes `config.yaml`, reverting to default settings.

### Manual Editing

You can edit `~/.alfred/config.yaml` directly:

```bash
# Open in your editor
$EDITOR ~/.alfred/config.yaml

# Or use any text editor
nano ~/.alfred/config.yaml
vim ~/.alfred/config.yaml
code ~/.alfred/config.yaml
```

## Example Configurations

### Minimal (Defaults)

```yaml
# Empty file or no file - uses all defaults
```

### Custom Model

```yaml
model_path: /Users/me/models/mistral-7b-q4.gguf
```

### Low Memory System

```yaml
daemon:
  idle_timeout_minutes: 5  # Free memory quickly
```

### Development Machine

```yaml
model_path: ~/.alfred/models/phi-3-medium-q4.gguf  # Better quality
daemon:
  port: 7654
  idle_timeout_minutes: 0  # Keep running all day
```

### Server/CI Environment

```yaml
# No daemon, just use local loading
daemon:
  idle_timeout_minutes: 1  # Shutdown immediately after use
```

## Environment Variables

Alfred doesn't currently use environment variables, but you can override the config location by changing `$HOME`:

```bash
# Use a different home directory (and thus different config)
HOME=/custom/home alfred commit
```

## Troubleshooting

### Config File Syntax Error

If you see:
```
Error: Failed to parse config YAML
```

Check your YAML syntax:
```bash
# Validate YAML
cat ~/.alfred/config.yaml | python3 -c "import sys, yaml; yaml.safe_load(sys.stdin)"
```

Common issues:
- Missing colons after keys
- Incorrect indentation (use spaces, not tabs)
- Unquoted special characters

### Model Not Found

If you see:
```
Error: Model not found at /custom/path/model.gguf
```

Check that:
1. The file exists at the specified path
2. The path in config.yaml is correct
3. You have read permissions

### Port Conflict

If the daemon won't start due to port conflict:

1. Find what's using the port:
   ```bash
   lsof -i :7654
   ```

2. Either stop that process or change Alfred's port:
   ```yaml
   daemon:
     port: 8765
   ```

3. Restart the daemon:
   ```bash
   alfred daemon stop
   alfred daemon start
   ```

## Config File Schema

For reference, here's the complete schema:

```yaml
# Optional: Path to GGUF model file
# Type: string (file path)
# Default: ~/.alfred/models/phi-3-mini-q4.gguf
model_path: string

# Daemon configuration
daemon:
  # TCP port for daemon communication
  # Type: integer (1-65535)
  # Default: 7654
  port: integer

  # Auto-shutdown after N minutes of inactivity
  # Type: integer (0 = disabled)
  # Default: 30
  idle_timeout_minutes: integer

  # Reserved for future use
  # Type: boolean
  # Default: false
  auto_start: boolean
```
