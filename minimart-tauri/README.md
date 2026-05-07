# Minimart POS Tauri App

Desktop POS app built with Tauri 2, Vue 3, TypeScript, Rust, and SQLite.

## Current Capabilities

- Login and session state
- POS cart with barcode/SKU lookup, product search, and category filtering
- Sale completion with stock reduction and inventory transaction logging
- Receipt popup generated on sale completion
- Receipt print preview with print icon button
- VAT display as included 16% VAT, while total remains the payable cart total
- Product create, edit, and delete
- Inventory view and stock adjustment
- Dashboard and reports views
- Toast notifications and custom prompt dialogs
- Black, white, and gold styling
- Startup splash screen with minimart storefront icon
- Per-machine/user SQLite database initialized on first launch

## Commands

```bash
npm install
npm run tauri dev
npm run build
npm run package
npm run package:deb
npm run package:windows
```

For Rust-only validation:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

## Packaging

Linux packages are built from Linux:

```bash
npm run package
```

`npm run package` is the default Linux packaging command and is equivalent to `npm run package:deb`.

The `.deb` output is written under `src-tauri/target/release/bundle/deb/`, for example `Minimart POS_0.1.0_amd64.deb`.

Windows installers can be cross-built from Linux when the `x86_64-pc-windows-gnu` Rust target, MinGW, and NSIS are installed:

```bash
npm run package:windows
```

The Windows output is written under `src-tauri/target/x86_64-pc-windows-gnu/release/bundle/nsis/` as an `.exe` installer.

Tauri cross-platform Windows builds are unsigned by default from Linux. For signed production installers, build/sign on Windows or configure a custom Windows signing command.

The installer does not bundle a prebuilt SQLite database. On first launch, the app creates `minimart.db` in the operating system app-data directory and runs migrations/default seed data there. That gives each PC an independent database and avoids install-time write-permission errors.

## Layout

```text
src/                          Vue frontend
src/App.vue                   App shell and session switching
src/components/Login.vue      Login screen
src/components/POS.vue        Main POS, dashboard, products, inventory, reports, settings
src/components/NotificationHost.vue
src/composables/              Shared Vue composables
src-tauri/                    Rust backend and Tauri config
src-tauri/src/commands/       Command handlers exposed to the frontend
src-tauri/src/db/             SQLite setup and access
src-tauri/src/models/         Shared Rust models
src-tauri/migrations/         Database migrations
```

## Receipt Behavior

After a sale completes, the app immediately opens a receipt popup. The receipt includes line items, subtotal, VAT 16% included, total, paid amount, and change. The print button uses the browser/Tauri print dialog and print CSS hides the rest of the app so only the receipt is printed.

VAT is calculated for display only as the included portion of the total:

```text
VAT = total * 16 / 116
```

The total charged to the customer is not reduced by VAT and VAT is not added again.
