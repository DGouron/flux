# Flux

[![CI](https://github.com/DGouron/flux/actions/workflows/ci.yml/badge.svg)](https://github.com/DGouron/flux/actions/workflows/ci.yml)
[![Release](https://github.com/DGouron/flux/actions/workflows/release.yml/badge.svg)](https://github.com/DGouron/flux/releases)

> Deep focus tool for AI-Augmented developers

## Description

Flux is a CLI tool that helps developers maintain focus by blocking distractions and providing periodic check-ins. Designed for AI-assisted development workflows, it creates a distraction-free environment so you can stay in the zone.

## Features

- 🎯 Focus session management with customizable durations
- 🔔 Periodic check-ins to verify you're still on task
- 📊 Status tracking to monitor your focus time
- ⚡ Lightweight and fast (built with Rust)
- 🖥️ Cross-platform support (Linux, macOS, Windows)
- 🔧 Interactive configuration wizard

## Installation

### Script automatique (Linux/macOS)

```bash
curl -sSL https://raw.githubusercontent.com/DGouron/flux/main/install.sh | bash
```

### Manuel

1. Télécharge la dernière release depuis [GitHub Releases](https://github.com/DGouron/flux/releases)
2. Extrais l'archive :
```bash
tar -xzf flux-v*.tar.gz
```
3. Déplace les binaires dans ton PATH :
```bash
mv flux flux-daemon ~/.local/bin/
```

### Depuis les sources

```bash
cargo install --path crates/flux-cli
cargo install --path crates/flux-daemon
```

## Usage

### Initial setup

```bash
flux init
```

This interactive wizard configures Flux on first use:
- Enable/disable system tray icon
- Default focus duration
- Check-in interval
- Notification sounds

### Start a focus session

```bash
flux start                       # Défaut: 25 min, mode ai-assisted
flux start -d 45                 # 45 minutes
flux start -m review             # Mode review
flux start -d 30 -m ai-assisted  # Combiné
```

### Check status

```bash
flux status          # Affichage formaté
flux status --json   # Format JSON
```

### Stop session

```bash
flux stop
```

## Focus modes

| Mode | Description |
|------|-------------|
| `ai-assisted` | AI-assisted development and prompting |
| `review` | Code review and validation |
| `architecture` | System design and architecture |

## Architecture

```
┌─────────────┐     IPC      ┌──────────────┐
│  flux-cli   │◄────────────►│ flux-daemon  │
└─────────────┘  Unix Socket └──────────────┘
```

- **flux-cli** : Interface utilisateur en ligne de commande
- **flux-daemon** : Service en arrière-plan qui gère les sessions

## License

MIT License - see [LICENSE](LICENSE) for details.
