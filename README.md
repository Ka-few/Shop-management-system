# Minimart POS

A Tauri 2 desktop point-of-sale app for a Kenyan minimart, built with Vue 3, TypeScript, Rust, and SQLite.

## Project Structure

```text
minimart-tauri/               Tauri 2 + Vue desktop app
  src/                        Vue frontend
  src/components/             Frontend components
  src-tauri/                  Rust backend and Tauri config
  src-tauri/src/commands/     Tauri command handlers
  src-tauri/src/db/           SQLite database layer
  src-tauri/src/models/       Rust data models
  src-tauri/migrations/       SQLite migrations
TAURI_MIGRATION_PLAN.md       Migration notes and remaining roadmap
```

Generated dependency and build directories are intentionally not committed.

## Development

```bash
cd minimart-tauri
npm install
npm run tauri dev
```

## Build

```bash
cd minimart-tauri
npm run build
npm run tauri build
```

The old Electron/Express structure has been removed from the active project tree. Historical migration context remains in `TAURI_MIGRATION_PLAN.md`.
