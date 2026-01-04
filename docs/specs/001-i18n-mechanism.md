# SPEC-001: Internationalization Mechanism (i18n)

## User Story

As a Flux user,
I want to choose the interface language (English or French),
so that I can use the application in my preferred language.

## Context

Flux currently displays all text in French. To broaden the audience and follow best practices, English should become the default language with French as an option.

This issue covers the **technical infrastructure only**. Translations are in a separate issue (SPEC-002).

## Business Rules

- Default language: **English** (if not configured)
- Supported languages: `en`, `fr`
- Preference is persisted in `~/.config/flux/config.toml`
- Language change is immediate (no daemon restart required)
- Clap texts (help/about) also follow the configured language

## Acceptance Criteria

### Scenario: Change language to French

```gherkin
Given a fresh Flux installation
And the default language is "en"
When the user runs "flux lang fr"
Then the config.toml file contains "language = \"fr\""
And the CLI displays "Langue définie : français"
```

### Scenario: Change language to English

```gherkin
Given the configured language is "fr"
When the user runs "flux lang en"
Then the config.toml file contains "language = \"en\""
And the CLI displays "Language set: English"
```

### Scenario: Display current language

```gherkin
Given the configured language is "fr"
When the user runs "flux lang"
Then the CLI displays "Langue actuelle : français (fr)"
```

### Scenario: Invalid language

```gherkin
Given the user runs "flux lang de"
When the command is processed
Then the CLI displays an error "Unsupported language: de. Available languages: en, fr"
And the configuration is not modified
```

### Scenario: First use without config

```gherkin
Given no config.toml file exists
When the user runs "flux start"
Then messages are displayed in English
```

### Scenario: Daemon messages

```gherkin
Given the configured language is "fr"
When a check-in notification is sent
Then the notification text is in French
```

## Technical Design (negotiable)

### Option A: Static TOML/JSON files
- `locales/en.toml`, `locales/fr.toml`
- Loaded at runtime, embedded via `include_str!`
- Simple, no external dependency

### Option B: `rust-i18n` crate
- `t!("key")` macros
- Pluralization support
- More robust but additional dependency

### Option C: `fluent` crate
- Mozilla standard, very flexible
- Probably overkill for 2 languages

**Recommendation**: Option A for simplicity. Flux has only ~100 strings and 2 languages.

### Config structure

```toml
# ~/.config/flux/config.toml
[general]
language = "fr"  # "en" | "fr", default: "en"
```

### New CLI command

```
flux lang [LANG]

Arguments:
  [LANG]  Language to set (en, fr). Without argument: displays current language.

Examples:
  flux lang        # Display current language
  flux lang fr     # Switch to French
  flux lang en     # Switch to English
```

## Out of Scope

- Text translations (SPEC-002)
- Automatic system locale detection
- Support for languages other than en/fr
- Documentation translation

## Open Questions

1. **Clap texts**: The `#[command(about = "...")]` macros are evaluated at compile time. Options:
   - Build-time: compile 2 binaries (complex)
   - Runtime: use dynamic `about_fn` or `help_template`
   - Accept that `--help` is always in English (pragmatic)

2. **Daemon hot-reload**: Should the daemon reload config on each message, or only at startup?

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ✅ | Deliverable alone, enables EN texts by default |
| Negotiable | ✅ | Multiple implementations possible (A/B/C) |
| Valuable | ✅ | Opens Flux to international audience |
| Estimable | ✅ | ~2-3 days of work |
| Small | ✅ | Infrastructure only, no translations |
| Testable | ✅ | Clear and automatable Gherkin scenarios |
