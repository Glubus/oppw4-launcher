# OPPW4 Launcher

Desktop launcher for ONE PIECE: PIRATE WARRIORS 4.

V1 goals:

- Detect Steam installs for app `1089090`.
- Allow a manual executable or game folder fallback.
- Install/update a `dinput8.dll` style modloader from a GitHub Release zip.
- Backup replaced files before writing.
- Restore files installed by the launcher.
- Launch the game through Steam URI or a configured executable.

## Development

```sh
npm install
npm run tauri dev
```

## Modloader Release

The launcher expects a GitHub repository slug like:

```txt
owner/repository
```

It downloads the first `.zip` asset from the latest release. The zip should contain the files that must be copied into the OPPW4 game folder, for example `dinput8.dll` and loader config/assets.

Zip entries with absolute paths or `..` are rejected.

