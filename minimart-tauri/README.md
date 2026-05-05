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

## Commands

```bash
npm install
npm run tauri dev
npm run build
npm run tauri build
```

For Rust-only validation:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

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
