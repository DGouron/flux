# Contributing to Flux

First off, thank you for considering contributing to Flux! This document provides guidelines and information for contributors.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 1.75+** (check with `rustc --version`)
- **System dependencies** (Linux):
  ```bash
  # Debian/Ubuntu
  sudo apt install libdbus-1-dev pkg-config libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

  # Fedora
  sudo dnf install dbus-devel pkgconfig libxcb-devel
  ```

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/DGouron/flux.git
   cd flux
   ```

2. **Build the project**
   ```bash
   cargo build --workspace
   ```

3. **Run tests**
   ```bash
   cargo test --workspace
   ```

4. **Run the daemon** (in one terminal)
   ```bash
   cargo run -p flux-daemon
   ```

5. **Run the CLI** (in another terminal)
   ```bash
   cargo run -p flux-cli -- start
   ```

## Before Submitting

**Always verify your changes pass CI locally:**

```bash
cargo build --workspace && cargo test --workspace && cargo clippy --workspace && cargo fmt --check
```

This is mandatory before pushing. Do not skip this step.

## Commit Guidelines

- **Language**: Commit messages must be in **English**
- **Style**: Use conventional commits when applicable (`feat:`, `fix:`, `docs:`, `refactor:`, etc.)
- **Atomicity**: Keep commits small and focused on a single change
- **No AI mentions**: Do not mention AI tools in commit messages

Good examples:
```
feat: add weekly digest notification
fix: prevent crash when config file is missing
refactor: extract timer logic into separate module
```

## Code Style

### Language Rules

| Context | Language |
|---------|----------|
| Error messages shown to users | French |
| UI text and notifications | French |
| Tests | English |
| Technical logs | English |
| Code comments (when necessary) | English |
| Documentation in `docs/` | English |

### Example

```rust
// UI message (French)
println!("Session focus terminée !");

// Technical log (English)
tracing::debug!("focus session completed, duration={}s", duration);
```

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files/modules | snake_case | `timer_manager.rs` |
| Types/Traits | PascalCase | `TimerManager` |
| Functions/variables | snake_case | `start_timer()` |
| Constants | SCREAMING_SNAKE | `DEFAULT_DURATION` |
| CLI commands | kebab-case | `flux start-timer` |

**Important**: Always use full words. Never abbreviations.
- `configuration`, `session`, `duration`, `index`
- `cfg`, `sess`, `dur`, `idx`, `i`

### Error Handling

- Library code (`flux-core`, `flux-protocol`): Use `thiserror`
- Application code (`flux-cli`, `flux-daemon`): Use `anyhow`

### Comments

Avoid comments unless absolutely necessary for understanding. Code should be self-documenting through clear naming.

## Pull Request Process

1. **Create a feature branch** from `main`
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Make your changes** following the guidelines above

3. **Run the full CI check locally**
   ```bash
   cargo build --workspace && cargo test --workspace && cargo clippy --workspace && cargo fmt --check
   ```

4. **Push your branch** and open a Pull Request

5. **Fill out the PR template** describing your changes

6. **Wait for review** - maintainers will review and provide feedback

## Reporting Issues

- Use GitHub Issues for bug reports and feature requests
- Check existing issues before creating a new one
- Use the provided issue templates

## Questions?

Feel free to open a GitHub Discussion if you have questions about contributing.
