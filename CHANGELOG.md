# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.13] - 2025-01-16

### Added
- Veille mode: new focus mode that disables all interruptions
- Check-in focused notification for better user engagement

### Changed
- Renamed "Prompting" focus mode to "AiAssisted" for clarity

## [0.2.12] - 2025-01-15

### Added
- Window title tracking during focus sessions
- Title-based distraction detection using configurable patterns
- Browser-based distraction detection via title patterns

## [0.2.11] - 2025-01-14

### Added
- Context switch details section in GUI dashboard
- Whitelist support for context switches

## [0.2.10] - 2025-01-13

### Fixed
- Lifetime issue in non-linux notification fallback

## [0.2.9] - 2025-01-12

### Fixed
- Cross-platform support for interactive notifications

## [0.2.8] - 2025-01-11

### Added
- Configuration profiles support
- Weekly digest with CLI command (`flux digest`) and auto-notification
- Friction mode for ambiguous apps
- Toggle button in GUI to mark apps as focus/distraction
- Focus metrics display with circular gauge in GUI
- Distraction discovery with smart suggestions
- Distinct sound and urgency levels for distraction alerts
- Separate focus and distraction apps in statistics

### Fixed
- Reduced short burst threshold from 30s to 15s for better accuracy

## [0.2.7] - 2025-01-10

### Added
- Auto-refresh stats in GUI when session ends
- Interactive check-ins with auto-pause on timeout

## [0.2.6] - 2025-01-09

### Added
- Session control in GUI (start/pause/stop from dashboard)
- Distraction detection and alerts
- Application time tracking during focus sessions

### Fixed
- Renamed AppTrackerMessage variants to fix clippy warning

## [0.2.5] - 2025-01-08

### Added
- Delete and clear sessions in GUI dashboard

## [0.2.4] - 2025-01-07

### Added
- Delete command in CLI and start session button in GUI
- Clear command to purge completed sessions

## [0.2.3] - 2025-01-06

### Added
- Auto-restart daemon and GUI after self-update

## [0.2.2] - 2025-01-05

### Fixed
- Include flux-gui in install script
- Backup mechanism during updates

## [0.2.1] - 2025-01-04

### Fixed
- Include flux-gui in release artifacts

## [0.2.0] - 2025-01-03

### Added
- GUI dashboard (`flux-gui`) with egui
- System tray integration (Linux)
- Statistics visualization
- Session history view

[Unreleased]: https://github.com/DGouron/flux/compare/v0.2.13...HEAD
[0.2.13]: https://github.com/DGouron/flux/compare/v0.2.12...v0.2.13
[0.2.12]: https://github.com/DGouron/flux/compare/v0.2.11...v0.2.12
[0.2.11]: https://github.com/DGouron/flux/compare/v0.2.10...v0.2.11
[0.2.10]: https://github.com/DGouron/flux/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/DGouron/flux/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/DGouron/flux/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/DGouron/flux/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/DGouron/flux/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/DGouron/flux/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/DGouron/flux/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/DGouron/flux/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/DGouron/flux/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/DGouron/flux/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/DGouron/flux/releases/tag/v0.2.0
