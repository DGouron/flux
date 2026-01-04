# SPEC-007 : Interactive configuration wizard (`flux init`)

## User Story

As a new Flux user,
I want an interactive wizard to configure Flux on first use,
so that I can quickly set up my preferences without manually editing TOML files.

## Context

Currently, if no config file exists, Flux uses hardcoded defaults. Users discovering Flux don't know which options exist (e.g., tray icon is disabled by default). This leads to a poor first-run experience.

`flux init` provides guided configuration with sensible defaults, shadcn-style (one question at a time, suggested default values).

## Business Rules

- `flux init` creates `~/.config/flux/config.toml`
- If config file already exists, command fails with error message
- `--force` flag allows overwriting existing config
- `flux start` without config file shows message and exits (does not auto-run wizard)
- Questions are asked one-by-one with default values shown
- User can accept default by pressing Enter

## CLI Behavior

### Happy path

```
$ flux init

Welcome to Flux! Let's configure your focus sessions.

? Enable system tray icon? (Y/n)
? Default focus duration in minutes? (25)
? Check-in interval in minutes? (25)
? Enable notification sounds? (Y/n)

Configuration saved to ~/.config/flux/config.toml
Run `flux start` to begin your first focus session.
```

### Config already exists

```
$ flux init
Error: Configuration already exists at ~/.config/flux/config.toml
Use --force to overwrite.
```

### Force overwrite

```
$ flux init --force
Warning: Overwriting existing configuration.

? Enable system tray icon? (Y/n)
...
```

### Start without config

```
$ flux start
Error: No configuration found. Run `flux init` to set up Flux.
```

## Questions Specification

| Order | Question | Type | Default | Config key |
|-------|----------|------|---------|------------|
| 1 | Enable system tray icon? | Y/n | Yes | `tray.enabled` |
| 2 | Default focus duration in minutes? | number | 25 | `focus.default_duration_minutes` |
| 3 | Check-in interval in minutes? | number | 25 | `focus.check_in_interval_minutes` |
| 4 | Enable notification sounds? | Y/n | Yes | `notifications.sound_enabled` |

### Input validation

- Duration: positive integer, 1-480 (8 hours max)
- Check-in interval: positive integer, 5-120
- Boolean: accepts y/Y/yes/n/N/no, empty = default

## Generated Config Example

```toml
[tray]
enabled = true

[focus]
default_duration_minutes = 25
check_in_interval_minutes = 25

[notifications]
sound_enabled = true
```

## Acceptance Criteria

### Scenario: First-time setup

```gherkin
Given no config file exists at ~/.config/flux/config.toml
When user runs `flux init`
And answers all questions with defaults (pressing Enter)
Then config file is created with default values
And success message is displayed
```

### Scenario: Custom values

```gherkin
Given no config file exists
When user runs `flux init`
And enters "n" for tray icon
And enters "50" for focus duration
And enters "30" for check-in interval
And enters "y" for sounds
Then config file contains tray.enabled = false
And focus.default_duration_minutes = 50
And focus.check_in_interval_minutes = 30
And notifications.sound_enabled = true
```

### Scenario: Config already exists

```gherkin
Given config file exists at ~/.config/flux/config.toml
When user runs `flux init`
Then error message is displayed
And existing config is not modified
And exit code is non-zero
```

### Scenario: Force overwrite

```gherkin
Given config file exists at ~/.config/flux/config.toml
When user runs `flux init --force`
Then warning is displayed
And wizard proceeds normally
And new config overwrites existing file
```

### Scenario: Start without config

```gherkin
Given no config file exists
When user runs `flux start`
Then error message mentions `flux init`
And daemon is not started
And exit code is non-zero
```

### Scenario: Invalid input retry

```gherkin
Given user is in flux init wizard
When user enters "abc" for focus duration
Then error message is shown
And same question is asked again
```

## Out of Scope

- GitLab/GitHub provider configuration (too complex for init)
- Config migration from older versions
- `flux config` subcommand for editing existing config
- Non-interactive mode (`--yes` to accept all defaults)

## Questions ouvertes

- Should we detect terminal capabilities and fall back to simpler prompts if not interactive?

## Technical Notes

Suggested crate for interactive prompts: `dialoguer` (already popular in Rust CLI ecosystem, shadcn-like experience).

## INVEST Evaluation

| Criteria | Status | Note |
|----------|--------|------|
| Independent | ✅ | No dependencies on other features |
| Negotiable | ✅ | Questions can be adjusted |
| Valuable | ✅ | Solves real first-run UX problem |
| Estimable | ✅ | Clear scope, ~2-3h implementation |
| Small | ✅ | Single command, 4 questions |
| Testable | ✅ | Clear Gherkin scenarios |
