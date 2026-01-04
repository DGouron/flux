# SPEC-006: Tray Icon Context Menu

## User Story

As a Linux user with tray enabled,
I want to control Flux from the tray menu,
so that I can act quickly without opening a terminal.

## Context

The icon and tooltip provide information (SPEC-003/004/005). The context menu enables action: pause, stop, configure.

## Dependency

- SPEC-005 (Tooltip) - or at minimum SPEC-003

## Business Rules

- Right-click on icon opens the menu
- Available actions depend on current state
- Actions have the same effect as corresponding CLI commands
- "Open configuration" opens the config file in the system default editor

## Menu Structure

### State: Inactive

```
┌─────────────────────────┐
│ ⚙ Open configuration    │
│ ✕ Quit                  │
└─────────────────────────┘
```

### State: Active

```
┌─────────────────────────┐
│ ⏸ Pause                 │
│ ⏹ Stop                  │
│ ─────────────────────── │
│ ⚙ Open configuration    │
│ ✕ Quit                  │
└─────────────────────────┘
```

### State: Paused

```
┌─────────────────────────┐
│ ▶ Resume                │
│ ⏹ Stop                  │
│ ─────────────────────── │
│ ⚙ Open configuration    │
│ ✕ Quit                  │
└─────────────────────────┘
```

### State: Check-in pending

```
┌─────────────────────────┐
│ ▶ Continue              │
│ ⏸ Pause                 │
│ ⏹ Stop                  │
│ ─────────────────────── │
│ ⚙ Open configuration    │
│ ✕ Quit                  │
└─────────────────────────┘
```

## Acceptance Criteria

### Scenario: Pause from menu

```gherkin
Given an active session
When the user clicks "Pause"
Then the session is paused
And the icon changes to "paused" state
```

### Scenario: Resume from menu

```gherkin
Given a paused session
When the user clicks "Resume"
Then the session resumes
And the icon changes to "active" state
```

### Scenario: Stop from menu

```gherkin
Given an active or paused session
When the user clicks "Stop"
Then the session ends
And the icon changes to "inactive" state
```

### Scenario: Open configuration

```gherkin
Given the context menu is open
When the user clicks "Open configuration"
Then the file ~/.config/flux/config.toml opens
And in the system default editor (xdg-open)
```

### Scenario: Quit

```gherkin
Given the context menu is open
When the user clicks "Quit"
Then the daemon shuts down cleanly (no confirmation)
And the icon disappears from the tray
And any running session is terminated
```

### Scenario: Menu reflects current state

```gherkin
Given an active session
When the user opens the menu
Then "Pause" and "Stop" are visible
And "Resume" is not visible
```

## Suggested Implementation

```rust
impl ksni::Tray for FluxTray {
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let mut items = vec![];

        match self.state {
            TrayState::Inactive => {
                // No specific actions in inactive state
            }
            TrayState::Active { .. } => {
                items.push(/* Pause */);
                items.push(/* Stop */);
            }
            TrayState::Paused { .. } => {
                items.push(/* Resume */);
                items.push(/* Stop */);
            }
            TrayState::CheckInPending => {
                items.push(/* Continue */);
                items.push(/* Pause */);
                items.push(/* Stop */);
            }
        }

        if !items.is_empty() {
            items.push(MenuItem::Separator);
        }
        items.push(/* Open configuration */);
        items.push(/* Quit */);

        items
    }
}
```

## Out of Scope

- Start session from menu (YAGNI - use CLI)
- Submenu for focus mode selection
- Submenu for duration selection
- Global keyboard shortcuts
- Notifications from menu

## Decisions Made

- No "Start session" in menu (YAGNI)
- "Quit": no confirmation, immediate shutdown
- "Quit" shuts down the entire daemon (not just hide tray)

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ⚠️ | Depends on SPEC-003 minimum |
| Negotiable | ✅ | Menu structure adjustable |
| Valuable | ✅ | Full control without terminal |
| Estimable | ✅ | ~1 day: menu + actions + xdg-open |
| Small | ✅ | Clear scope, known actions |
| Testable | ✅ | Discrete actions, verifiable results |
