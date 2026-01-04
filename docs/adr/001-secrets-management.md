# ADR-001: Secrets and Credentials Management

**Date**: 2026-01-04
**Status**: Accepted

## Context

Flux needs to connect to external providers (GitLab, GitHub) to track code review activity. These connections require authentication tokens and user identifiers.

The initial design proposed storing credentials directly in `~/.config/flux/config.toml`, which poses security risks:
- Accidental commit of the config file
- Exposure via cloud backup
- Reading by third-party applications

## Decision

Separate public configuration from secrets with a cascading resolution strategy:

1. **Environment variables** (highest priority)
   - `FLUX_GITLAB_TOKEN`, `FLUX_GITLAB_USER_ID`
   - `FLUX_GITHUB_TOKEN`, `FLUX_GITHUB_USER_ID`

2. **secrets.toml file** (fallback)
   - `~/.config/flux/secrets.toml`
   - Restrictive permissions (chmod 600)
   - Never versioned

3. **Public config** (can be versioned)
   - `~/.config/flux/config.toml`
   - Contains only `base_url` for providers
   - No secrets

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Token in config.toml | Simple, single file | Commit risk, no separation |
| System keyring (libsecret) | More secure | Heavy dependency, cross-platform complex |
| `token_command` only | Delegates to password manager | Requires user setup |
| Environment variables only | CI/CD standard | Not practical for local dev |

## Consequences

### Positive
- Standard pattern (same approach as gh, glab, docker CLI)
- CI/CD compatible via env vars
- secrets.toml file for dev convenience
- `/security` skill can scan for leaks

### Negative
- Two files to manage (config.toml + secrets.toml)
- User must understand the separation

## Notes

- Rule added to CLAUDE.md: never store tokens in plain text in code
- `/security` skill created to scan before commit
- `/security-scan` command for repo audit
