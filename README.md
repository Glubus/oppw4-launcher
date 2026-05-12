# OPPW4 Launcher

Desktop launcher for ONE PIECE: PIRATE WARRIORS 4.

The frontend is a local SvelteKit clone of the OPPW4 Mods Browser, with an extra
`/launcher` page for native desktop actions.

V1 goals:

- Detect Steam installs for app `1089090`.
- Allow a manual executable or game folder fallback.
- Install/update the `dinput8.dll` patcher from a GitHub Release zip.
- Backup replaced files before writing.
- Restore files installed by the launcher.
- Launch the game through Steam URI or a configured executable.

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

## Patcher Release

The launcher expects a GitHub repository slug like:

```txt
owner/repository
```

It defaults to `Glubus/oppw4-patcher` and downloads the first `.zip` asset from the latest release. The zip should contain the files that must be copied into the OPPW4 game folder, for example `dinput8.dll` and patcher config/assets.

Zip entries with absolute paths or `..` are rejected.
