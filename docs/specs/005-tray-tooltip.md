# SPEC-005: Tray Icon Dynamic Tooltip

## User Story

As a Linux user with tray enabled,
I want to see remaining time when hovering over the icon,
so that I can check my progress without opening a terminal.

## Context

The icon shows state (SPEC-004), but not details. The tooltip adds temporal information on mouse hover.

## Dependency

- SPEC-004 (Visual States)

## Business Rules

- Tooltip updates in real-time (or near real-time: every second)
- Content varies based on session state
- Format is concise and readable

## Tooltip Format by State

| State | Tooltip |
|-------|---------|
| Inactive | `Flux - No active session` |
| Active | `Flux - 12:34 remaining (prompting)` |
| Paused | `Flux - Paused (12:34 remaining)` |
| Check-in | `Flux - Check-in pending` |

## Acceptance Criteria

### Scenario: Tooltip inactive session

```gherkin
Given the daemon with no active session
When the user hovers over the tray icon
Then the tooltip displays "Flux - No active session"
```

### Scenario: Tooltip active session

```gherkin
Given an active session with 15 minutes remaining
And the focus mode is "review"
When the user hovers over the tray icon
Then the tooltip displays "Flux - 15:00 remaining (review)"
```

### Scenario: Tooltip updates

```gherkin
Given an active session with 10:30 remaining
When 30 seconds elapse
And the user hovers over the icon
Then the tooltip displays "Flux - 10:00 remaining (mode)"
```

### Scenario: Tooltip paused session

```gherkin
Given a paused session with 8 minutes remaining
When the user hovers over the icon
Then the tooltip displays "Flux - Paused (8:00 remaining)"
```

### Scenario: Tooltip check-in

```gherkin
Given a pending check-in
When the user hovers over the icon
Then the tooltip displays "Flux - Check-in pending"
```

## Suggested Implementation

```rust
impl ksni::Tray for FluxTray {
    fn title(&self) -> String {
        match &self.state {
            TrayState::Inactive => "Flux - No active session".into(),
            TrayState::Active { remaining, mode } => {
                format!("Flux - {} remaining ({})",
                    format_duration(*remaining),
                    mode)
            }
            TrayState::Paused { remaining } => {
                format!("Flux - Paused ({} remaining)",
                    format_duration(*remaining))
            }
            TrayState::CheckInPending => "Flux - Check-in pending".into(),
        }
    }
}
```

## Out of Scope

- Localized time formatting (always MM:SS)
- Rich tooltip (multi-line, HTML) - not supported by SNI standard
- Context menu → SPEC-006

## Open Questions

- Refresh: Does D-Bus allow tooltip refresh without full rebuild?
- Should we include total duration in addition to remaining time?

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ⚠️ | Depends on SPEC-004 |
| Negotiable | ✅ | Text format adjustable |
| Valuable | ✅ | Time info without action |
| Estimable | ✅ | ~0.5 day: just formatting |
| Small | ✅ | Very small scope |
| Testable | ✅ | String formats verifiable |
