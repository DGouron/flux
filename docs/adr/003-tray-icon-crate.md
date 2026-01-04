# ADR-003: System Tray Icon Crate Selection

**Date**: 2026-01-04
**Status**: Accepted

## Context

Flux needs a system tray icon on Linux to provide persistent visual feedback about session state. The daemon runs as a background process, and users currently have no way to know if a session is active without running `flux status`.

We need a Rust crate that:
- Works on Linux desktop environments (GNOME, KDE, XFCE, etc.)
- Supports dynamic icon updates
- Integrates well with Tokio async runtime
- Has minimal dependencies

## Decision

Use **ksni** (v0.2) for Linux system tray support.

## Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| **ksni** | Pure Rust, D-Bus native, async-friendly, lightweight (~50KB), follows StatusNotifierItem spec | Linux-only, requires D-Bus |
| **tray-icon** | Cross-platform (Windows, macOS, Linux) | Heavy dependencies (GTK/Qt on Linux), sync API, larger binary size |
| **systray** | Cross-platform, simple API | Unmaintained since 2020, sync-only, GTK dependency on Linux |
| **libappindicator-rs** | Direct bindings to system library | C bindings, complex build, distribution-specific issues |

## Consequences

### Positive
- Native D-Bus integration without C dependencies
- Small binary footprint
- StatusNotifierItem is the modern standard (KDE native, GNOME via extension)
- Async-compatible with Tokio
- Clean failure mode: if D-Bus unavailable, daemon continues without tray

### Negative
- Linux-only (Windows/macOS would need different solution)
- Requires StatusNotifierItem-compatible tray (GNOME needs AppIndicator extension)
- Limited to what StatusNotifierItem protocol supports

## Notes

- StatusNotifierItem spec: https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/
- GNOME users need `gnome-shell-extension-appindicator` package
- KDE Plasma supports StatusNotifierItem natively
- Feature is opt-in via `tray.enabled = true` in config
