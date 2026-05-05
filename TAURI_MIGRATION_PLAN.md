# Tauri Migration & Minimart App Implementation Plan

## Project Overview
Convert the current Electron-based POS system to a **Tauri-based desktop application** for a Kenyan minimart with enhanced features including product categories, barcode scanning/generation, and optimized inventory management.

---

## Phase 1: Project Setup & Architecture (Week 1)

### 1.1 Tauri Project Initialization
- [x] Initialize new Tauri project (`minimart-tauri/`)
- [x] Choose frontend framework: **Vue 3 + TypeScript**
- [x] Setup project structure:
  ```
  minimart-app/
  ├── src-tauri/          # Rust backend
  │   ├── src/
  │   │   ├── main.rs
  │   │   ├── db/         # Database layer
  │   │   ├── models/     # Data models
  │   │   └── commands/   # Tauri commands
  │   └── Cargo.toml
  ├── src/                # Frontend (Vue/React)
  │   ├── components/
  │   ├── pages/
  │   ├── stores/         # State management
  │   └── App.vue
  └── tauri.conf.json
  ```
- [x] Remove stale Electron/Express project files from the active tree

### 1.2 Technology Stack
- **Backend**: Rust (Tauri) + SQLite
- **Frontend**: Vue 3 + TypeScript + Pinia (state management)
- **Barcode**: `barcode-rs` + `jsbarcode` for generation, barcode scanner integration
- **Database**: SQLite (existing, migrate schema)
- **Styling**: Tailwind CSS or existing CSS

### 1.3 Development Dependencies
- Tauri CLI
- Rust toolchain
- Node.js 18+
- TypeScript
- Barcode generation libraries (JS + Rust)
- QR code library (alternative/supplement to barcode)

---

## Phase 2: Database Migration & Schema Enhancement (Week 1-2)

### 2.1 New Database Schema

#### Products Table (Enhanced)
```sql
CREATE TABLE products (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  category_id INTEGER NOT NULL,
  sku TEXT UNIQUE NOT NULL,
  barcode TEXT UNIQUE,
  barcode_format TEXT DEFAULT 'EAN13', -- EAN13, EAN8, CODE128, QR
  description TEXT,
  unit_price REAL NOT NULL,
  cost_price REAL,
  quantity_in_stock INTEGER DEFAULT 0,
  reorder_level INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (category_id) REFERENCES categories(id)
);
```

#### New Categories Table
```sql
CREATE TABLE categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT UNIQUE NOT NULL,
  icon TEXT, -- emoji or icon identifier
  description TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### Barcode Log Table (for scanning history)
```sql
CREATE TABLE barcode_scans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL,
  barcode TEXT,
  scan_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  user_id INTEGER,
  FOREIGN KEY (product_id) REFERENCES products(id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

#### Sales Items Table (Enhanced)
```sql
CREATE TABLE sale_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sale_id INTEGER NOT NULL,
  product_id INTEGER NOT NULL,
  quantity INTEGER NOT NULL,
  unit_price REAL NOT NULL,
  barcode_scanned TEXT, -- track which barcode was scanned
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (sale_id) REFERENCES sales(id),
  FOREIGN KEY (product_id) REFERENCES products(id)
);
```

#### Inventory Transactions Table (Audit trail)
```sql
CREATE TABLE inventory_transactions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL,
  transaction_type TEXT NOT NULL, -- 'purchase', 'sale', 'adjustment', 'return'
  quantity INTEGER NOT NULL,
  reference_id INTEGER, -- sale_id, purchase_order_id, etc.
  notes TEXT,
  created_by INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (product_id) REFERENCES products(id),
  FOREIGN KEY (created_by) REFERENCES users(id)
);
```

### 2.2 Predefined Categories
Insert minimart categories:
- 🥤 Beverages (Sodas, Water, Juice, Coffee, Tea)
- 🍿 Snacks (Chips, Cookies, Nuts, Candy)
- 🧴 Toiletries (Soap, Shampoo, Toothpaste, Tissues)
- 🍳 Food Items (Eggs, Milk, Rice, Sugar, Flour, Oil, Salt)
- 🧂 Spices & Condiments (Salt, Pepper, Soy Sauce, Cooking Oil)
- 🧴 Cleaning Supplies (Detergent, Bleach, Disinfectant)
- 📦 Packaged Foods (Pasta, Bread, Butter, Cheese)
- 🍋 Fresh Produce (when added in future phases)

### 2.3 Migration Strategy
- [ ] Create migration script to add new tables
- [ ] Populate default categories
- [ ] Generate barcodes for existing products (auto-generate if missing)
- [ ] Data validation and integrity checks

---

## Phase 3: Backend Development (Rust/Tauri) (Week 2-3)

### 3.1 Tauri Commands Structure

#### Product Commands
```rust
#[tauri::command]
async fn get_products(category_id: Option<i32>) -> Result<Vec<Product>, String>

#[tauri::command]
async fn get_product_by_barcode(barcode: String) -> Result<Product, String>

#[tauri::command]
async fn create_product(product: NewProduct) -> Result<Product, String>

#[tauri::command]
async fn update_product(id: i32, product: UpdateProduct) -> Result<Product, String>

#[tauri::command]
async fn delete_product(id: i32) -> Result<(), String>
```

#### Barcode Commands
```rust
#[tauri::command]
async fn generate_barcode(product_id: i32, format: String) -> Result<String, String>
// Returns base64 encoded barcode image

#[tauri::command]
async fn log_barcode_scan(product_id: i32, barcode: String) -> Result<(), String>

#[tauri::command]
async fn get_scan_history(product_id: i32, limit: i32) -> Result<Vec<BarcodeScan>, String>
```

#### Inventory Commands
```rust
#[tauri::command]
async fn get_inventory_status() -> Result<Vec<InventoryStatus>, String>

#[tauri::command]
async fn adjust_inventory(product_id: i32, quantity: i32, reason: String) -> Result<(), String>

#[tauri::command]
async fn get_low_stock_items(threshold: i32) -> Result<Vec<Product>, String>
```

#### Sales Commands (Enhanced)
```rust
#[tauri::command]
async fn create_sale_from_barcode_scan(barcode: String, quantity: i32) -> Result<SaleItem, String>

#[tauri::command]
async fn complete_sale(sale_id: i32, payment_method: String) -> Result<Sale, String>

#[tauri::command]
async fn get_daily_sales_summary() -> Result<DailySummary, String>
```

### 3.2 Database Layer (SQLite)
- [ ] Implement CRUD operations for all entities
- [ ] Use `rusqlite` crate for SQLite interactions
- [ ] Add connection pooling (r2d2)
- [ ] Implement proper error handling
- [ ] Add database transactions for consistency

### 3.3 Authentication & Security
- [ ] Migrate existing JWT implementation to Tauri
- [ ] Store tokens securely in Tauri secure storage
- [ ] Implement session management
- [ ] Add audit logging for sensitive operations

### 3.4 Barcode Processing
- [ ] Integrate barcode generation library (`barcode-rs`)
- [ ] Support multiple formats: EAN13 (default), EAN8, CODE128, QR
- [ ] Barcode validation logic
- [ ] Batch barcode generation for bulk imports

---

## Phase 4: Frontend Development (Vue 3) (Week 3-4)

### 4.1 Pages/Views

#### Dashboard
- [ ] Sales summary (today, week, month)
- [ ] Quick stats (total sales, items sold, top products)
- [ ] Low stock alerts
- [ ] Recent transactions

#### Point of Sale (POS) - Core Feature
- [ ] Barcode scanner input field (auto-focus)
- [ ] Real-time product lookup
- [ ] Add items to cart
- [ ] Quantity adjustment
- [ ] Remove items from cart
- [ ] Cart summary (subtotal, tax, total)
- [ ] Payment methods (Cash, Card, Partial)
- [ ] Receipt printing/preview
- [ ] Manual product search fallback

#### Product Management
- [ ] Product list with categories
- [ ] Filter by category
- [ ] Search by name/SKU/barcode
- [ ] Add new product (with barcode generation)
- [ ] Edit product details
- [ ] Upload product photo
- [ ] Bulk import (CSV)
- [ ] Generate/print barcode labels

#### Inventory Management
- [ ] Stock levels overview
- [ ] Low stock alerts
- [ ] Inventory adjustments
- [ ] Stock taking (count & reconcile)
- [ ] Inventory history/audit trail
- [ ] Reorder recommendations

#### Reports
- [ ] Sales by category
- [ ] Sales by product
- [ ] Inventory valuation
- [ ] Daily closing report
- [ ] Employee performance (if multi-user)
- [ ] Export to PDF/Excel

#### Settings
- [ ] User management
- [ ] Store settings (name, location, phone, email)
- [ ] Barcode format preferences
- [ ] Tax settings
- [ ] Receipt customization
- [ ] Backup/Restore database

### 4.2 Components

#### ProductSelector
- Barcode input with real-time autocomplete
- Quick search by name
- Recent products quick access

#### ShoppingCart
- Line items display
- Quantity controls
- Item removal
- Price calculations (with tax)

#### BarcodeScanner
- Camera permission handling
- Real-time scanning
- Feedback (success/error)

#### ReceiptPrinter
- Format receipt
- Print preview
- Thermal printer support
- Email receipt option

#### CategoryFilter
- Collapsible category list with counts
- Search within category
- Icon display

### 4.3 State Management (Pinia)
```typescript
// stores/pos.ts
- currentCart
- activeCustomer
- currentSale
- paymentMethod

// stores/inventory.ts
- products
- categories
- lowStockItems
- selectedCategory

// stores/ui.ts
- theme
- sidebarOpen
- notificationStack
```

### 4.4 Styling & UI
- [ ] Mobile-responsive design (tablet-first for POS)
- [ ] Dark mode support (good for retail environments)
- [ ] Large touch-friendly buttons
- [ ] Clear visual hierarchy
- [ ] Quick access to critical functions
- [ ] Accessibility standards

---

## Phase 5: Barcode Integration (Week 4)

### 5.1 Barcode Scanning Hardware Support
- [ ] USB barcode scanner integration
- [ ] Keyboard wedge emulation support
- [ ] Laser scanner compatibility
- [ ] Mobile camera barcode scanning (via Tauri)

### 5.2 Barcode Generation
- [ ] Generate product barcodes during product creation
- [ ] Batch barcode label printing
- [ ] Support for multiple barcode formats per product
- [ ] QR code option (links to product info)
- [ ] Barcode label templates (Avery 8.5x11, thermal sticker)

### 5.3 Barcode Validation
- [ ] Check digit validation
- [ ] Prevent duplicate barcodes
- [ ] Handle scanning errors gracefully
- [ ] Suggest corrections for invalid barcodes

---

## Phase 6: Features & Enhancements (Week 5-6)

### 6.1 Advanced POS Features
- [ ] Multi-currency support (KES, USD, etc.)
- [ ] Discount system (percentage, fixed amount, buy-one-get-one)
- [ ] Gift cards/loyalty points (future phase)
- [ ] Customer profiles with purchase history
- [ ] Return/exchange management
- [ ] Price override (manager approval)
- [ ] Split payments

### 6.2 Inventory Features
- [ ] Supplier management
- [ ] Purchase order creation
- [ ] Stock take/count
- [ ] Damage/loss reporting
- [ ] Stock transfer between locations (multi-store future)
- [ ] Expiry date tracking
- [ ] FIFO/LIFO stock rotation

### 6.3 Reporting Features
- [ ] Real-time dashboards
- [ ] Custom report builder
- [ ] Scheduled report emails
- [ ] Data export (PDF, Excel, CSV)
- [ ] Charts and visualizations
- [ ] Comparative analysis (period-over-period)

### 6.4 Offline Support
- [ ] Work offline with local sync
- [ ] Auto-sync when connection restored
- [ ] Conflict resolution strategy
- [ ] Offline-first data handling

---

## Phase 7: Testing & Quality Assurance (Week 6-7)

### 7.1 Testing Strategy
- [ ] Unit tests (Rust backend)
- [ ] Integration tests (Tauri commands)
- [ ] Component tests (Vue frontend)
- [ ] E2E tests (complete workflows)
- [ ] Barcode scanning tests
- [ ] Performance testing (large inventory)
- [ ] Security testing

### 7.2 Quality Assurance
- [ ] Code review process
- [ ] Linting and formatting
- [ ] Database integrity checks
- [ ] Error handling & edge cases
- [ ] User acceptance testing (UAT)

---

## Phase 8: Deployment & Distribution (Week 7-8)

### 8.1 Build & Packaging
- [ ] Configure release builds
- [ ] Create installers for:
  - Windows (.msi, .exe)
  - macOS (.dmg, .app)
  - Linux (.deb, .rpm, .AppImage)
- [ ] Code signing for security
- [ ] Auto-update configuration

### 8.2 Deployment
- [ ] GitHub Releases for distribution
- [ ] Version management
- [ ] Changelog documentation
- [ ] Release notes

### 8.3 Documentation
- [ ] User manual
- [ ] Admin guide
- [ ] Installation guide
- [ ] Troubleshooting guide
- [ ] API documentation (for future integrations)

---

## Architecture Comparison

### Current (Electron)
```
Electron App
├── Node.js Backend (Express)
├── SQLite Database
└── HTML/CSS/JS Frontend
```

### New (Tauri)
```
Tauri App
├── Rust Backend (Tauri Commands)
├── SQLite Database
└── Vue 3 / TypeScript Frontend
```

**Advantages of Tauri:**
- ✅ Smaller bundle size (~10MB vs ~150MB Electron)
- ✅ Faster startup time
- ✅ Lower memory usage
- ✅ Native OS integration
- ✅ Better performance for barcode scanning
- ✅ Rust safety guarantees
- ✅ Better security model

---

## Data Migration Plan

### Step 1: Export Existing Data
- Export users, products, sales data from current SQLite
- Validate data integrity

### Step 2: Transform Data
- Map existing products to new categories
- Generate barcodes for products without them
- Normalize data format

### Step 3: Import to New Database
- Create new Tauri app database
- Import transformed data
- Verify counts and checksums

### Step 4: Validation
- Test all data is accessible
- Verify relationships (foreign keys)
- Check for data loss

---

## File Structure Preview

```
minimart-pos-tauri/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── db.rs                 # Database initialization
│   │   ├── models.rs             # Data models
│   │   └── commands/
│   │       ├── products.rs
│   │       ├── sales.rs
│   │       ├── inventory.rs
│   │       ├── barcode.rs
│   │       ├── auth.rs
│   │       └── reports.rs
│   ├── migrations/               # Database migrations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.vue
│   ├── main.ts
│   ├── components/
│   │   ├── POS/
│   │   │   ├── BarcodeScanner.vue
│   │   │   ├── ShoppingCart.vue
│   │   │   ├── PaymentModal.vue
│   │   │   └── Receipt.vue
│   │   ├── Inventory/
│   │   ├── Products/
│   │   └── Common/
│   ├── pages/
│   │   ├── Dashboard.vue
│   │   ├── POS.vue
│   │   ├── Inventory.vue
│   │   ├── Products.vue
│   │   ├── Reports.vue
│   │   └── Settings.vue
│   ├── stores/
│   │   ├── pos.ts
│   │   ├── inventory.ts
│   │   ├── auth.ts
│   │   └── ui.ts
│   ├── utils/
│   │   ├── barcode.ts
│   │   ├── api.ts
│   │   └── formatter.ts
│   └── styles/
│       └── main.css
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## Timeline Summary
- **Phase 1**: 3-4 days (Setup, Architecture)
- **Phase 2**: 4-5 days (Database)
- **Phase 3**: 5-7 days (Rust Backend)
- **Phase 4**: 7-10 days (Frontend)
- **Phase 5**: 3-4 days (Barcode)
- **Phase 6**: 5-7 days (Enhancements)
- **Phase 7**: 4-5 days (Testing)
- **Phase 8**: 3-4 days (Deployment)

**Total: 6-8 weeks** for MVP with core features

---

## Dependencies to Add

### Rust (Cargo.toml)
```toml
tauri = "2.x"
tokio = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.31", features = ["bundled", "chrono"] }
r2d2 = "0.8"
r2d2_sqlite = "0.24"
jsonwebtoken = "9.0"
bcrypt = "0.15"
barcode = "0.16"  # or similar barcode gen library
chrono = "0.4"
uuid = { version = "1.0", features = ["v4"] }
```

### Node/Frontend (package.json)
```json
{
  "dependencies": {
    "vue": "^3.x",
    "pinia": "^2.x",
    "vue-router": "^4.x",
    "@tauri-apps/api": "^2.x",
    "axios": "^1.x",
    "jsbarcode": "^3.x",
    "html5-qrcode": "^2.x",
    "date-fns": "^2.x"
  },
  "devDependencies": {
    "typescript": "^5.x",
    "vite": "^6.x",
    "vitest": "^0.x",
    "@vitejs/plugin-vue": "^5.x",
    "tailwindcss": "^3.x"
  }
}
```

---

## Next Steps
1. Continue Phase 2 database migration work inside `minimart-tauri/src-tauri/migrations/`
2. Fill out remaining Tauri command implementations and frontend workflows
3. Add focused tests around database commands, POS cart behavior, and barcode lookup
4. Start Phase 2: Database Migration in parallel

Would you like me to proceed with implementation of any specific phase?
