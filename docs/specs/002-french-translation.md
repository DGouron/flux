# SPEC-002: Text Translations (EN default + FR)

## User Story

As a French-speaking Flux user,
I want all messages to be available in French,
so that I can easily understand the displayed information.

## Context

This issue is the **follow-up to SPEC-001** (i18n mechanism). It covers:
1. Externalizing the 97 current French strings into translation files
2. Translating these strings to English (new default language)
3. Keeping French as an alternative translation

**Dependency**: SPEC-001 must be implemented first.

## Business Rules

- All user-facing texts must be externalized
- English becomes the default language
- French is a complete translation (no partial fallback)
- Technical logs remain in English (not translated, already the case)

## Text Inventory

### By category

| Category | Files | String count |
|----------|-------|--------------|
| CLI help/about | main.rs | 11 |
| CLI commands | commands/*.rs | 70 |
| CLI errors | commands/*.rs | 6 |
| Daemon errors | server.rs | 6 |
| Notifications | notifier.rs | 15 |
| Alerts | timer.rs | 2 |

**Total: 97 strings** (excluding examples/debug)

### Impacted files

1. `crates/flux-cli/src/main.rs`
2. `crates/flux-cli/src/commands/start.rs`
3. `crates/flux-cli/src/commands/stop.rs`
4. `crates/flux-cli/src/commands/pause.rs`
5. `crates/flux-cli/src/commands/resume.rs`
6. `crates/flux-cli/src/commands/status.rs`
7. `crates/flux-cli/src/commands/stats.rs`
8. `crates/flux-daemon/src/server.rs`
9. `crates/flux-daemon/src/actors/notifier.rs`
10. `crates/flux-daemon/src/actors/timer.rs`

## Acceptance Criteria

### Scenario: CLI messages in English (default)

```gherkin
Given a fresh Flux installation
When the user runs "flux start"
Then the displayed message is "Focus session started"
And the details are "Duration: 25 min"
```

### Scenario: CLI messages in French

```gherkin
Given the configured language is "fr"
When the user runs "flux start"
Then the displayed message is "Session focus démarrée"
And the details are "Durée: 25 min"
```

### Scenario: Notifications in French

```gherkin
Given the configured language is "fr"
And a focus session is active
When the check-in triggers
Then the notification has title "Flux - Check-in"
And the body is "25min écoulées. Toujours concentré ?"
```

### Scenario: Daemon errors in English

```gherkin
Given the configured language is "en"
When a session error occurs
Then the error message is "unable to start session"
```

### Scenario: Statistics in French

```gherkin
Given the configured language is "fr"
When the user runs "flux stats"
Then the displayed labels are in French
And "Total time:" becomes "Temps total:"
And "Sessions:" stays "Sessions:"
```

### Scenario: Complete coverage

```gherkin
Given the translation file fr.toml
When counting keys
Then there are exactly the same number as in en.toml
And no key has an empty value
```

## Translation Examples

| Key | EN (default) | FR |
|-----|--------------|-----|
| `session.started` | Focus session started | Session focus démarrée |
| `session.stopped` | Focus session ended | Session focus terminée |
| `session.paused` | Session paused | Session mise en pause |
| `session.resumed` | Session resumed | Session reprise |
| `status.active` | Focus session active | Session focus active |
| `status.no_session` | No active session | Aucune session active |
| `daemon.not_running` | Daemon not running | Le daemon n'est pas démarré |
| `notification.checkin.title` | Flux - Check-in | Flux - Check-in |
| `notification.checkin.body` | {minutes}min elapsed. Still focused? | {minutes}min écoulées. Toujours concentré ? |
| `stats.total_time` | Total time: | Temps total: |
| `stats.period.today` | today | aujourd'hui |
| `stats.period.week` | this week | cette semaine |

## Translation File Structure

```
crates/flux-core/src/
├── i18n/
│   ├── mod.rs          # i18n module, t!() or get_text() function
│   ├── en.toml         # 97 strings in English
│   └── fr.toml         # 97 strings in French
```

## Out of Scope

- Adding new languages (DE, ES, etc.)
- Documentation translation
- Technical log translation (tracing)
- Advanced pluralization (1 session vs N sessions)

## Open Questions

1. **Interpolation**: How to handle `{minutes}min`?
   - Simple `format!` with replacement
   - Or dedicated lib if pluralization needed

2. **Emojis**: Keep the same emojis for all languages? (probably yes)

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ⚠️ | Depends on SPEC-001 |
| Negotiable | ✅ | File format flexible |
| Valuable | ✅ | Makes Flux usable in FR and EN |
| Estimable | ✅ | ~1-2 days (extraction + translation) |
| Small | ✅ | Mechanical work, clear scope |
| Testable | ✅ | Verifiable by file comparison |
