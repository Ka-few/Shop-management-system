# Minimart POS Tauri App

Desktop POS app built with Tauri 2, Vue 3, TypeScript, Rust, and SQLite.

## Commands

```bash
npm install
npm run tauri dev
npm run build
npm run tauri build
```

## Layout

```text
src/                        Vue frontend
src/components/             POS and login components
src-tauri/                  Rust backend and Tauri config
src-tauri/src/commands/     Command handlers exposed to the frontend
src-tauri/src/db/           SQLite setup and access
src-tauri/src/models/       Shared Rust models
src-tauri/migrations/       Database migrations
```
