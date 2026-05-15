# Changelog

## 0.1.1 - 2026-05-15

### User-facing

#### Added

- Open external HTTP(S) links and redirect/download URLs in the user's default browser when running inside Tauri.
- Add profile search and availability filters.
- Reveal the exported diagnostics ZIP in the file browser after export.
- Write important launcher success and error logs to logs/{date}-{time}.log next to the executable.
- Add a Settings toggle for verbose debug launcher logs.
- Add launcher update checks from GitHub releases with a startup prompt and Settings controls.
- Add secondary Settings tabs for Game, Patcher, and Launcher status.
- Add a Steam launch executable override with disabled Game install controls until enabled.
- Add potential overlap warnings before launch with an option to disable future warnings.

#### Changed

- Keep launcher panel widths stable when switching between Mods and Profiles.
- Align Mods and Profiles header actions and simplify filter separators.
- Make Settings easier to scan with clearer install, launch method, patcher source, and diagnostics sections.
- Show launcher success and error feedback as toasts instead of inline alert panels.
- Simplify Patcher settings status with a compact indicator.
- Clarify launcher update status when GitHub has a different release but no installable asset.
- Highlight potentially overlapping mods in Installed Mods and Profiles with orange warning styling.

#### Fixed

- Throttle automatic launcher update checks and show a clearer GitHub rate-limit message.

### Developer

#### Changed

- Split launcher runtime and logging helpers out of LauncherPage.
- Refactor Rust installer code into focused modules with typed errors and result aliases.
- Split Tauri commands into feature-scoped command modules.
- Move command implementations out of lib.rs into feature command modules.
- Move remaining Rust command DTOs and launcher/mod helpers out of lib.rs into focused modules.
- Move Steam detection and Mods inventory/metadata helpers under feature command modules.
- Move launcher game commands into the `commands/launcher/game` module directory.
- Split hosted/external mod installation and mod enable toggling into smaller tested Rust helpers.
- Add a Rust Mods overlap detector used by launcher health checks.

#### Tests

- Add focused Rust tests for Mods metadata parsing, ZIP injection, paths, keys, and inventory.
- Add Rust tests for mod folder toggling, diagnostics status helpers, installer release parsing, and updater asset filtering.
- Add Rust tests for launcher log formatting and diagnostics ZIP export content.
- Add Rust tests for patcher ZIP/DLL installation, backup behavior, and unsafe archive path rejection.
- Add Rust edge-case tests for updater digests/assets, Steam detection, diagnostics logs/status, mod toggles, installer restore, URL validation, and metadata parsing.
- Run Rust formatting, unit tests, and `clippy::pedantic` cleanly.
- Add Rust tests for potential overlap detection and config migration defaults.
