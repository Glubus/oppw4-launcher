# OPPW4 Launcher

Desktop launcher and local mod manager for ONE PIECE: PIRATE WARRIORS 4.

The app bundles a SvelteKit frontend with a Tauri desktop backend. It keeps the
OPPW4 Mods Browser experience, then adds native launcher actions for installing
the patcher, managing local mods, profiles, diagnostics, and game launch.

## Features

- Detect Steam installs for app `1089090`.
- Launch through Steam or a configured executable override.
- Install/update the `dinput8.dll` patcher from GitHub releases.
- Import local mod ZIPs and install hosted mods from the browser.
- Manage installed mods, profiles, profile search, and profile availability filters.
- Warn before launch when active mods may overlap on the same character/type.
- Open external HTTP(S) links and redirect/download URLs in the user's default browser.
- Export diagnostics and reveal the ZIP in the file browser.
- Write launcher logs next to the executable in `logs/{date}-{time}.log`.
- Enable verbose debug logs from Settings.
- Check and install launcher updates from GitHub releases.

## Safety

- Backup replaced patcher files before writing.
- Restore files installed by the launcher.
- Reject patcher ZIP entries with absolute paths or `..`.
- Show potential mod overlaps as warnings only; launching is still allowed.
- Keep success and error feedback in app toasts.

## Development

```sh
npm install
npm run tauri dev
```

By default, the cloned browser talks to:

```txt
https://oppw4.prism.am/api
```

Override it during development with `VITE_API_BASE`.

Useful checks:

```sh
npm run test
cd src-tauri
cargo fmt
RUSTC_WRAPPER= cargo test
RUSTC_WRAPPER= cargo clippy -- -W clippy::pedantic
```

## Patcher Release

The launcher expects a GitHub repository slug like:

```txt
owner/repository
```

It defaults to `Glubus/oppw4-patcher` and downloads the first `.zip` or `.dll`
asset from the latest release. A ZIP should contain the files that must be copied
into the OPPW4 game folder, for example `dinput8.dll` and patcher config/assets.

Zip entries with absolute paths or `..` are rejected.

## Launcher Updates

Launcher self-update checks read the latest release from:

```txt
Glubus/oppw4-launcher
```

If a supported asset exists for the current platform, the launcher can download
and open it. Startup checks are throttled to avoid GitHub rate limits.
