# Changelog

## 0.1.1 - 2026-05-15

- Open external HTTP(S) links and redirect/download URLs in the user's default browser when running inside Tauri.
- Keep launcher panel widths stable when switching between Mods and Profiles.
- Add profile search and availability filters.
- Align Mods and Profiles header actions and simplify filter separators.
- Make Settings easier to scan with clearer install, launch method, patcher source, and diagnostics sections.
- Reveal the exported diagnostics ZIP in the file browser after export.
- Show launcher success and error feedback as toasts instead of inline alert panels.
- Write important launcher success and error logs to logs/{date}-{time}.log next to the executable.
- Add a Settings toggle for verbose debug launcher logs.
- Split launcher runtime and logging helpers out of LauncherPage.
