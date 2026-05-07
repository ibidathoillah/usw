# USwitch — User Switch AI Runtime

> **1 Linux User = 1 AI Runtime (isolated)**

`usw` (USwitch) is a system administration CLI that creates isolated per-user AI runtimes backed by systemd. Each runtime gets its own Linux user, home directory, environment variables, workspace attachments, and sandboxed service unit.

## Features

- **Isolated runtimes** — each AI runtime runs as a dedicated Linux user with its own processes, service, and config
- **Workspace attachment** — bind your current working directory to the runtime with ACL-based permissions
- **Plugin system** — discover and install AI tools (binaries, npm packages, pip packages, or shell scripts)
- **Environment management** — per-user `.env` files (`usw env <user> set KEY=val`)
- **Systemd integration** — runtimes are managed via systemd service units with strict sandboxing (`ProtectSystem=strict`, `NoNewPrivileges`)
- **Atomic state** — runtime state is persisted to `/var/lib/usw/state.json` with file locking and `0o600` permissions

## Quick Start

```bash
# Build
make build

# Install (copies binary + systemd template)
sudo make install

# Create a runtime for a new user
sudo usw create myproject

# Quick switch — same as create + attach
sudo usw myproject

# Monitor all runtimes
sudo usw monitor

# List available plugins
sudo usw plugin

# Install a plugin
sudo usw install plugin-name

# Manage environment variables
sudo usw env myproject set API_KEY=sk-abc123

# Destroy a runtime
sudo usw destroy myproject
```

## Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `usw <user>` | — | Create or switch to a runtime |
| `usw create <user>` | `c`, `mk`, `up`, `add`, `new` | Create a new runtime |
| `usw destroy <user>` | `d`, `rm`, `del` | Remove a runtime |
| `usw monitor [user]` | `m`, `ps`, `s` | Show runtime status |
| `usw kill [user]` | `k`, `stop` | Stop runtime processes |
| `usw purge [user]` | `x`, `clear`, `nuke` | Completely remove all runtimes |
| `usw plugin [name]` | `p`, `pl` | List or inspect plugins |
| `usw install [tool]` | `i`, `in` | Install a plugin/tool |
| `usw env <user>` | `e` | Manage environment variables |
| `usw current` | — | Show active runtime |

## How It Works

1. `usw create <name>` creates a Linux user via `useradd`, sets up runtime directories under `/home/<name>/`, deploys plugin binaries to `/opt/ai-core/binaries/`, generates a `/home/<name>/runtime/start.sh` script, and registers the runtime in `/var/lib/usw/state.json`.

2. The start script is executed by a systemd unit template (`ai-runtime@.service`) that sandboxes the runtime with `ProtectSystem=strict`, `NoNewPrivileges=yes`, and read-only home access.

3. `usw <name>` stops any other active runtimes, starts (or restarts) the target runtime, then drops you into a login shell as that user via `su -l`.

## Requirements

- Linux (systemd-based distribution)
- Rust toolchain (for building from source)
- Root privileges (the binary checks for EUID 0 at startup)

## Build & Install

```bash
# Build release binary
make build

# Install system-wide
sudo make install

# Run tests
make test

# Clean build artifacts
make clean
```

## Project Structure

```
src/
├── main.rs          # Entry point + tracing init
├── lib.rs           # Core utilities (root check, dir setup, validation)
├── cli.rs           # Clap argument parsing (commands + aliases)
├── error.rs         # Error types (thiserror)
├── output.rs        # Colored terminal output + table rendering
├── switch.rs        # Main switch logic (create-or-switch)
├── runtime.rs       # Systemd service management
├── user.rs          # Linux user creation/teardown + scripts
├── project.rs       # Workspace attachment (ACL + symlinks)
├── plugin.rs        # Plugin manifest loading + binary discovery
├── state.rs         # JSON state persistence with file locking
└── commands/
    ├── mod.rs       # Command dispatch
    ├── create.rs    # Full runtime creation flow
    ├── destroy.rs   # Runtime teardown
    ├── kill.rs      # Process termination
    ├── purge.rs     # Bulk removal
    ├── monitor.rs   # Status table
    ├── install.rs   # Plugin installation
    ├── plugin_cmd.rs# Plugin listing/inspection
    └── env.rs       # Environment variable management
templates/
└── ai-runtime@.service  # Systemd unit template
```

## Security

- State file at `/var/lib/usw/state.json` is created with `0o600` permissions (owner-only read/write)
- Per-user environment files (`runtime/env`) use `0o600` to protect API keys
- Writes use atomic rename (write to temp file → rename) to prevent corruption
- File locking via `flock` prevents concurrent state modification
- Runtime systemd units use strict sandboxing profiles

## License

MIT
