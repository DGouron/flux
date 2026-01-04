# SPEC-004: Tray Icon Visual States

## User Story

As a Linux user with tray enabled,
I want the icon to change based on my session state,
so that I can instantly see if I'm focused, paused, or inactive.

## Context

With SPEC-003, we have a static icon. This ticket adds the dynamic dimension: the icon reflects the actual session state.

## Dependency

- SPEC-003 (Tray Icon Infrastructure)

## Business Rules

- Icon changes immediately when state changes
- 4 distinct visual states:
  - **Inactive**: No session running (gray/neutral)
  - **Active**: Session in progress (green/vivid color)
  - **Paused**: Session paused (orange/yellow)
  - **Check-in pending**: Awaiting user response (distinct icon with attention indicator)

## Required Assets

Create 4 SVG icons:
- `assets/icons/flux-inactive.svg`: Gray, idle state
- `assets/icons/flux-active.svg`: Green/blue, active session
- `assets/icons/flux-paused.svg`: Orange/yellow, paused
- `assets/icons/flux-checkin.svg`: With attention indicator (red dot?)

## Acceptance Criteria

### Scenario: Session starts

```gherkin
Given the daemon with active tray
And no session running (inactive icon)
When a focus session starts
Then the icon changes to "active" state
And the change is visible within 500ms
```

### Scenario: Session paused

```gherkin
Given an active session (active icon)
When the user pauses the session
Then the icon changes to "paused" state
```

### Scenario: Session resumed

```gherkin
Given a paused session (paused icon)
When the user resumes the session
Then the icon changes to "active" state
```

### Scenario: Session ended

```gherkin
Given an active session (active icon)
When the session ends (stop or natural completion)
Then the icon changes to "inactive" state
```

### Scenario: Check-in triggered

```gherkin
Given an active session
When a check-in is triggered (periodic reminder)
Then the icon changes to "check-in pending" state
And remains in this state until response or timeout
```

### Scenario: Check-in resolved

```gherkin
Given a pending check-in (check-in icon)
When the user responds to check-in (continue/pause/stop)
Then the icon changes to the state matching the response
```

## Suggested Implementation

```rust
// TrayActor receives state changes via channel
pub enum TrayState {
    Inactive,
    Active { mode: FocusMode },
    Paused,
    CheckInPending,
}

// TimerActor broadcasts state changes
// TrayActor subscribes and updates icon
```

## Out of Scope

- Visual differentiation by focus mode (prompting/review/etc.) - maybe v2
- Icon animation (GTK complexity)
- Tooltip → SPEC-005
- Menu → SPEC-006

## Decisions Made

- Check-in: distinct icon only (no blinking/animation)

## Open Questions

- Should we visually differentiate focus modes (prompting vs review)?

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ⚠️ | Depends on SPEC-003 |
| Negotiable | ✅ | Icon design flexible |
| Valuable | ✅ | Rich visual feedback without interaction |
| Estimable | ✅ | ~0.5 day: 4 SVGs + switch logic |
| Small | ✅ | Well-defined scope |
| Testable | ✅ | Discrete states, clear transitions |
