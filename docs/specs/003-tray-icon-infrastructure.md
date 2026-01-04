# SPEC-003: Tray Icon Infrastructure (Linux)

## User Story

As a Linux user,
I want to see a Flux icon in my system tray,
so that I can tell at a glance whether the daemon is running.

## Context

Currently, users have no persistent visual feedback on Flux state. They must run `flux status` to know if a session is active. A system tray icon solves this problem non-intrusively.

This ticket lays the technical foundation: adding `ksni`, creating `TrayActor`, feature flag, and fallback handling.

## Technical Choices

- **Crate**: `ksni` (D-Bus StatusNotifierItem)
- **Architecture**: `TrayActor` in daemon (same pattern as `NotifierActor`)
- **Icons**: SVG embedded in binary
- **Platform**: Linux only (`#[cfg(target_os = "linux")]`)

## Business Rules

- Tray icon is disabled by default (opt-in via config)
- If tray fails to initialize, daemon continues running (warning logged)
- Icon displayed is static in this ticket ("daemon active" state)

## Configuration

```toml
# ~/.config/flux/config.toml
[tray]
enabled = true
```

## Acceptance Criteria

### Scenario: Tray enabled and functional

```gherkin
Given a Linux system with a compatible tray manager
And configuration tray.enabled = true
When the daemon starts
Then a Flux icon appears in the system tray
And the daemon logs "tray icon initialized"
```

### Scenario: Tray enabled but no tray manager

```gherkin
Given a Linux system without tray manager (or D-Bus unavailable)
And configuration tray.enabled = true
When the daemon starts
Then the daemon logs a warning "tray initialization failed: <reason>"
And the daemon continues running normally
And focus sessions work without the tray
```

### Scenario: Tray disabled

```gherkin
Given configuration tray.enabled = false (or absent)
When the daemon starts
Then no icon appears in the tray
And no tray-related logs are emitted
```

### Scenario: Daemon shutdown with tray

```gherkin
Given the daemon with active tray icon
When the daemon receives a shutdown signal
Then the icon disappears from the tray
And the daemon terminates cleanly
```

## Required Assets

Create an SVG icon file:
- `assets/icons/flux-icon.svg`: Base icon (monochrome, 24x24 or scalable)

## Suggested Implementation

```
crates/flux-daemon/
├── src/
│   ├── actors/
│   │   ├── mod.rs          # Add tray (cfg linux)
│   │   └── tray.rs         # TrayActor + TrayHandle
│   └── main.rs             # Spawn TrayActor if enabled
├── assets/
│   └── icons/
│       └── flux-icon.svg
```

## Out of Scope

- Different visual states (active/paused/etc.) → SPEC-004
- Tooltip with remaining time → SPEC-005
- Context menu → SPEC-006
- Windows/macOS support

## Open Questions

- Do we need a fallback for icons (embedded PNG if SVG not supported)?

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ✅ | No dependencies, first ticket in series |
| Negotiable | ✅ | Crate choice flexible, feature flag allows opt-out |
| Valuable | ✅ | Immediate visual feedback for user |
| Estimable | ✅ | ~1 day: add dep, TrayActor, config, tests |
| Small | ✅ | Minimal scope: just infra + static icon |
| Testable | ✅ | Clear and verifiable criteria |
