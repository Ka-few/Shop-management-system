# Minimart POS

A Tauri 2 desktop point-of-sale app for a Kenyan minimart. The active application lives in `minimart-tauri/` and uses Vue 3 + TypeScript for the UI, Rust Tauri commands for the backend, and SQLite for local data.

## Features

- Point of sale cart with barcode/SKU lookup and product search
- Product, category, and inventory management
- Manual stock adjustments with inventory transaction logging
- Sales dashboard, recent transactions, top products, and daily reports
- VAT shown on receipts as the included 16% VAT portion without reducing or increasing the payable total
- Receipt popup generated after sale completion, with print preview and print button
- Black, white, and gold UI theme
- Toast notifications and custom prompt dialogs across app workflows
- Local SQLite database migrations and default setup

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

## Requirements

- Node.js 18+
- npm
- Rust toolchain
- Tauri system dependencies for your OS

## Development

```bash
cd minimart-tauri
npm install
npm run tauri dev
```

The app opens as a Tauri desktop window. During development, Vite serves the frontend on the port configured by Tauri.

## Build

```bash
cd minimart-tauri
npm run build
npm run tauri build
```

## Useful Commands

```bash
cd minimart-tauri
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Default App Flow

1. Log in.
2. Use the POS screen to scan a barcode, enter an SKU, search products, or click product tiles.
3. Enter the amount received and complete the sale.
4. A receipt popup is generated automatically with subtotal, included VAT, total, paid amount, and change.
5. Use the print icon in the receipt popup to print only the receipt.

## Notes

The old Electron/Express structure has been removed from the active project tree. Historical migration context remains in `TAURI_MIGRATION_PLAN.md`.
