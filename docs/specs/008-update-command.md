# SPEC-008 : Self-update command (`flux update`)

## User Story

As a Flux user,
I want to update Flux to the latest version with a single command,
so that I can stay up-to-date without manual downloads.

## Context

Currently, updating Flux requires manually downloading binaries from GitHub releases. This friction reduces adoption of new features and bug fixes.

## Business Rules

- `flux update` downloads and installs the latest version from GitHub releases
- Uses the existing `install.sh` script for consistency
- If daemon is running, prompt for confirmation before stopping it
- Backup current binaries before update for rollback
- If update fails, restore previous version automatically
- Configuration files are never modified

## CLI Behavior

### Happy path (no daemon running)

```
$ flux update
Checking for updates...
Current version: v0.1.8
Latest version: v0.1.9

Downloading update...
Installing...

✅ Flux updated to v0.1.9
```

### Daemon running

```
$ flux update
Checking for updates...
Current version: v0.1.8
Latest version: v0.1.9

⚠️  The Flux daemon is currently running.
Stop the daemon and continue? (Y/n)

Stopping daemon...
Downloading update...
Installing...

✅ Flux updated to v0.1.9
```

### Already up-to-date

```
$ flux update
Checking for updates...
Current version: v0.1.9
Latest version: v0.1.9

✅ Flux is already up-to-date.
```

### Update failed (rollback)

```
$ flux update
Checking for updates...
Current version: v0.1.8
Latest version: v0.1.9

Downloading update...
Installing...

❌ Update failed: installation script error
Rolling back to v0.1.8...

✅ Rollback successful. Flux is still at v0.1.8.
```

### Skip confirmation

```
$ flux update --yes
```

## Technical Implementation

### Version detection

1. Embed version in binary at compile time (`env!("CARGO_PKG_VERSION")`)
2. Fetch latest release from GitHub API: `https://api.github.com/repos/DGouron/flux/releases/latest`

### Update process

1. Check if daemon is running (via socket connection)
2. If running, prompt for confirmation (unless `--yes`)
3. Stop daemon if confirmed
4. Backup current binaries to `/tmp/flux-backup-{timestamp}/`
5. Download and run `install.sh`
6. Verify new version works (`flux --version`)
7. If verification fails, restore from backup
8. Clean up backup on success

### Rollback mechanism

```
/tmp/flux-backup-1704000000/
├── flux
└── flux-daemon
```

## Acceptance Criteria

### Scenario: Update available

```gherkin
Given Flux v0.1.8 is installed
And v0.1.9 is available on GitHub
When user runs `flux update`
Then Flux is updated to v0.1.9
And success message is displayed
```

### Scenario: Daemon running requires confirmation

```gherkin
Given Flux daemon is running
When user runs `flux update`
Then confirmation prompt is displayed
And update proceeds only if confirmed
```

### Scenario: Skip confirmation with --yes

```gherkin
Given Flux daemon is running
When user runs `flux update --yes`
Then daemon is stopped without prompting
And update proceeds automatically
```

### Scenario: Already up-to-date

```gherkin
Given Flux v0.1.9 is installed
And v0.1.9 is the latest version
When user runs `flux update`
Then "already up-to-date" message is displayed
And no download occurs
```

### Scenario: Rollback on failure

```gherkin
Given Flux v0.1.8 is installed
When user runs `flux update`
And installation fails
Then previous version is restored
And rollback message is displayed
```

### Scenario: Config preserved

```gherkin
Given config.toml exists with custom settings
When user runs `flux update`
Then config.toml is unchanged after update
```

## Out of Scope

- Automatic update checks at daemon startup (future feature)
- "Check for updates" in tray menu (future, depends on this)
- Downgrade to specific version
- Update channels (stable/beta)

## INVEST Evaluation

| Criterion | Status | Note |
|-----------|--------|------|
| Independent | ✅ | No dependencies |
| Negotiable | ✅ | Version check complexity negotiable |
| Valuable | ✅ | Reduces update friction |
| Estimable | ✅ | Clear scope |
| Small | ✅ | Single command |
| Testable | ✅ | Discrete scenarios |
