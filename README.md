# Flux

[![CI](https://github.com/DGouron/flux/actions/workflows/ci.yml/badge.svg)](https://github.com/DGouron/flux/actions/workflows/ci.yml)
[![Release](https://github.com/DGouron/flux/actions/workflows/release.yml/badge.svg)](https://github.com/DGouron/flux/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Deep focus tool for AI-Augmented developers

## What is Flux?

Flux is a focus management tool designed for developers who work with AI assistants. It helps you maintain deep focus by tracking your time, detecting distractions, and providing smart check-ins to keep you on task.

## Features

- **Focus Sessions** - Start, pause, resume, and stop timed focus sessions
- **Distraction Detection** - Monitors active windows and alerts you when switching to distracting apps
- **Smart Suggestions** - Learns your app usage patterns and suggests new distractions to block
- **Statistics & Analytics** - Track focus time, context switches, and productivity trends
- **Weekly Digest** - Automated summary of your weekly focus performance
- **GUI Dashboard** - Visual interface for stats, session control, and configuration
- **System Tray** - Quick access and notifications from your system tray
- **Multiple Profiles** - Switch between different focus configurations
- **Self-Update** - Built-in update mechanism
- **Multilingual** - English and French support

## Installation

### Automatic (Linux/macOS)

```bash
curl -sSL https://raw.githubusercontent.com/DGouron/flux/main/install.sh | bash
```

### Manual

1. Download the latest release from [GitHub Releases](https://github.com/DGouron/flux/releases)
2. Extract the archive:
   ```bash
   tar -xzf flux-v*.tar.gz
   ```
3. Move binaries to your PATH:
   ```bash
   mv flux flux-daemon flux-gui ~/.local/bin/
   ```

### From Source

```bash
cargo install --path crates/flux-cli
cargo install --path crates/flux-daemon
cargo install --path crates/flux-gui
```

## Quick Start

```bash
# First-time setup
flux init

# Start a focus session
flux start

# Check your status
flux status

# Open the dashboard
flux dashboard
```

## Commands Reference

| Command | Description |
|---------|-------------|
| `flux init` | Interactive setup wizard |
| `flux start` | Start a focus session |
| `flux stop` | Stop the current session |
| `flux pause` | Pause the current session |
| `flux resume` | Resume a paused session |
| `flux status` | Show session status |
| `flux stats` | Display usage statistics |
| `flux digest` | Show weekly summary |
| `flux dashboard` | Open GUI dashboard |
| `flux profile` | Manage configuration profiles |
| `flux distractions` | Manage distraction apps |
| `flux suggestions` | View detected distraction suggestions |
| `flux update` | Update Flux to latest version |
| `flux lang` | Change display language |
| `flux clear` | Delete all completed sessions |
| `flux delete` | Delete a specific session |

### Start Options

```bash
flux start                       # Default: 25 min, ai-assisted mode
flux start -d 45                 # 45 minutes
flux start -m review             # Review mode
flux start -d 30 -m architecture # Combined
```

## Focus Modes

| Mode | Description | Interruptions |
|------|-------------|---------------|
| `ai-assisted` | AI-assisted development | Enabled |
| `review` | Code review and validation | Enabled |
| `architecture` | System design and planning | Enabled |
| `veille` | Research and reading | Disabled |
| `custom` | User-defined modes | Enabled |

## Configuration

Configuration is stored in `~/.config/flux/config.toml`.

### Profiles

Create different profiles for different work contexts:

```bash
flux profile list              # List all profiles
flux profile create coding     # Create a new profile
flux profile switch coding     # Switch to a profile
```

### Distraction Management

```bash
flux distractions list         # List blocked apps
flux distractions add slack    # Add app to blocklist
flux distractions remove slack # Remove from blocklist
```

## Architecture

```
┌─────────────┐              ┌──────────────┐
│  flux-cli   │◄────────────►│ flux-daemon  │
└─────────────┘  Unix Socket └──────────────┘
       │                            │
       │                     ┌──────┴──────┐
       │                     │   SQLite    │
       │                     └─────────────┘
       │
┌──────┴──────┐
│  flux-gui   │
└─────────────┘
```

- **flux-cli** - Command-line interface
- **flux-daemon** - Background service managing sessions, tracking, and notifications
- **flux-gui** - GUI dashboard built with egui
- **flux-core** - Shared domain logic
- **flux-protocol** - IPC protocol definitions
- **flux-adapters** - Infrastructure adapters (SQLite, notifications)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

See [SECURITY.md](SECURITY.md) for reporting vulnerabilities.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## License

MIT License - see [LICENSE](LICENSE) for details.
