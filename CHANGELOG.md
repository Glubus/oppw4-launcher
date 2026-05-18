# Changelog

## 0.1.2 - 2026-05-16

### User-facing

#### Features

- Move linked local ZIP mods into the game mods folder while embedding skin metadata.
- Add a Browse filter to show or hide skins that are already installed locally.

#### Changed

- Make the top navigation use the full available width on narrow browser windows.
- Align Browse and plugin pages with the launcher layout for more consistent cards, headers, and filtering.
- Split installed Mods and Plugins views so each panel shows the right actions, filters, totals, and empty states.

#### Fixed

- Show a repair action when the tracked patcher install is missing from the game folder.

## 0.1.1 "Sunny Side Up" - 2026-05-15

### User-facing

#### Features

- Open external HTTP(S) links and redirect/download URLs in the user's default browser when running inside Tauri.
- Add profile search and availability filters.
- Add launcher update checks from GitHub releases with a startup prompt and Settings controls.
- Add secondary Settings tabs for Game, Patcher, and Launcher status.
- Add a Steam launch executable override with disabled Game install controls until enabled.
- Add potential overlap warnings before launch with an option to disable future warnings.

#### Added

- Reveal the exported diagnostics ZIP in the file browser after export.
- Write important launcher success and error logs to logs/{date}-{time}.log next to the executable.
- Add a Settings toggle for verbose debug launcher logs.

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
- Prevent a Windows command prompt from opening when starting the launcher.

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
