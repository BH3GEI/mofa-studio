# Packaging (macOS Full Bundle)

This repo ships a full-bundle macOS packager that embeds the whole repo, Python
runtime, dora CLI, and prebuilt Rust binaries into a `.app` + `.dmg`.

## Script

```bash
./scripts/package-macos-full.sh
```

Output is placed in `build/macos-full/`.

## What it does

- Builds `mofa-studio` and required Rust nodes (release)
- Installs `dora-cli` into the bundle
- Embeds a Python runtime and installs `dora-rs` + `dora-*` Python nodes
- Copies the repo into the bundle
- On first run, extracts the repo to:
  `~/Library/Application Support/MoFA Studio/repo`
- Starts the app using bundled tools

## Models

If `MODEL_SRC` is set (or `~/.dora/models` exists), models are copied into the
bundle and then synced to `~/.dora/models` on first run.

Example:

```bash
MODEL_SRC="$HOME/.dora/models" ./scripts/package-macos-full.sh
```

## Signing / Notarization (optional)

```bash
SIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
APPLE_ID="you@example.com" \
TEAM_ID="TEAMID" \
APP_PASSWORD="app-specific-password" \
./scripts/package-macos-full.sh --notarize
```

## Notes

- The first run may take time due to repo extraction.
- `apps/*/dataflow/out` is regenerated in the extracted repo (writable).
