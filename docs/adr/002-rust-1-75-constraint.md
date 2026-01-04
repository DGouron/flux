# ADR-002: Rust 1.75 Constraint and Dependency Pinning

**Date**: 2026-01-04
**Status**: Accepted

## Context

The project targets Rust 1.75 (stable version available on most Linux distributions). Several recent transitive dependencies require newer Rust versions:

- `indexmap >= 2.7` requires Rust 1.82
- `zbus >= 5` requires Rust 1.77
- `notify-rust >= 4.11` depends on `zbus 5`
- `url >= 2.5.1` pulls `icu_*` crates requiring Rust 1.83

These incompatibilities break the build on Rust 1.75.

## Decision

Pin problematic dependencies to Rust 1.75-compatible versions:

```bash
# In Cargo.toml
notify-rust = "=4.8.0"
ureq = "2.9.1"

# Via cargo update
cargo update indexmap --precise 2.5.0
cargo update url --precise 2.5.0
```

Document these constraints and update when the minimum Rust version is raised.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Upgrade to Rust 1.82+ | Access to latest versions | Excludes users on stable distros |
| Pin dependencies | Rust 1.75 compatible | Manual maintenance, missing features |
| Fork dependencies | Full control | Heavy maintenance |
| Replace notify-rust | Avoids zbus | Re-implement notifications |

## Consequences

### Positive
- Builds on Rust 1.75 (Debian stable, Ubuntu LTS)
- No forks to maintain
- Simple and reversible solution

### Negative
- Frozen dependency versions
- Potential unpatched CVEs in old versions
- Must be re-evaluated regularly

## Notes

Currently pinned dependencies:
- `indexmap`: 2.5.0 (instead of 2.12.1)
- `notify-rust`: =4.8.0 (instead of 4.11.7) → avoids zbus 5
- `ureq`: 2.9.1 (instead of 2.12.1)
- `url`: 2.5.0 (instead of 2.5.7) → avoids icu_* crates

Re-evaluate when:
- The MSRV (Minimum Supported Rust Version) is raised
- A CVE affects a pinned dependency
