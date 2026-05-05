-- Minimart POS Database Migration
-- Enhanced schema for Kenyan minimart with barcode support

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Users table required by auth, sales, scan logs, and audit trail
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT UNIQUE NOT NULL,
  password TEXT NOT NULL,
  email TEXT UNIQUE NOT NULL,
  role TEXT DEFAULT 'user',
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Customers table for sales attribution
CREATE TABLE IF NOT EXISTS customers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  phone TEXT,
  email TEXT UNIQUE,
  address TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Categories table for minimart product categories
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT UNIQUE NOT NULL,
  icon TEXT, -- emoji or icon identifier
  description TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Enhanced products table with barcode support
CREATE TABLE IF NOT EXISTS products (
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
  reorder_level INTEGER DEFAULT 10,
  expiry_date DATE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

-- Enhanced sales table
CREATE TABLE IF NOT EXISTS sales (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  customer_id INTEGER,
  total_amount REAL NOT NULL,
  payment_method TEXT DEFAULT 'cash', -- cash, card, partial
  payment_amount REAL NOT NULL,
  change_amount REAL DEFAULT 0,
  status TEXT DEFAULT 'completed', -- completed, pending, cancelled
  created_by INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (customer_id) REFERENCES customers(id),
  FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Enhanced sale_items table with barcode tracking
CREATE TABLE IF NOT EXISTS sale_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sale_id INTEGER NOT NULL,
  product_id INTEGER NOT NULL,
  quantity INTEGER NOT NULL,
  unit_price REAL NOT NULL,
  barcode_scanned TEXT, -- track which barcode was scanned
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (sale_id) REFERENCES sales(id) ON DELETE CASCADE,
  FOREIGN KEY (product_id) REFERENCES products(id)
);

-- Inventory transactions table (audit trail)
CREATE TABLE IF NOT EXISTS inventory_transactions (
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

-- Barcode scans log table
CREATE TABLE IF NOT EXISTS barcode_scans (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  product_id INTEGER NOT NULL,
  barcode TEXT,
  scan_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  user_id INTEGER,
  FOREIGN KEY (product_id) REFERENCES products(id),
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Suppliers table for inventory management
CREATE TABLE IF NOT EXISTS suppliers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  contact_person TEXT,
  phone TEXT,
  email TEXT,
  address TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Purchase orders table
CREATE TABLE IF NOT EXISTS purchase_orders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  supplier_id INTEGER NOT NULL,
  total_amount REAL NOT NULL,
  status TEXT DEFAULT 'pending', -- pending, received, cancelled
  expected_date DATE,
  received_date DATE,
  created_by INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (supplier_id) REFERENCES suppliers(id),
  FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Purchase order items table
CREATE TABLE IF NOT EXISTS purchase_order_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  purchase_order_id INTEGER NOT NULL,
  product_id INTEGER NOT NULL,
  quantity_ordered INTEGER NOT NULL,
  quantity_received INTEGER DEFAULT 0,
  unit_cost REAL NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (purchase_order_id) REFERENCES purchase_orders(id) ON DELETE CASCADE,
  FOREIGN KEY (product_id) REFERENCES products(id)
);

-- Settings table for app configuration
CREATE TABLE IF NOT EXISTS settings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  key TEXT UNIQUE NOT NULL,
  value TEXT,
  description TEXT,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert default categories for Kenyan minimart
INSERT OR IGNORE INTO categories (name, icon, description) VALUES
('Beverages', '🥤', 'Sodas, Water, Juice, Coffee, Tea, Energy Drinks'),
('Snacks', '🍿', 'Chips, Cookies, Nuts, Candy, Biscuits'),
('Toiletries', '🧴', 'Soap, Shampoo, Toothpaste, Tissues, Deodorant'),
('Food Items', '🍳', 'Eggs, Milk, Rice, Sugar, Flour, Oil, Salt'),
('Spices & Condiments', '🧂', 'Salt, Pepper, Soy Sauce, Cooking Oil, Herbs'),
('Cleaning Supplies', '🧴', 'Detergent, Bleach, Disinfectant, Cleaning Products'),
('Packaged Foods', '📦', 'Pasta, Bread, Butter, Cheese, Canned Goods'),
('Fresh Produce', '🍋', 'Fruits, Vegetables (when added in future)');

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value, description) VALUES
('store_name', 'Minimart POS', 'Name of the store'),
('store_address', '', 'Physical address of the store'),
('store_phone', '', 'Contact phone number'),
('store_email', '', 'Contact email address'),
('currency', 'KES', 'Default currency (KES, USD, etc.)'),
('tax_rate', '0.16', 'VAT tax rate (16% for Kenya)'),
('barcode_format', 'EAN13', 'Default barcode format for new products'),
('low_stock_threshold', '10', 'Default reorder level for products'),
('receipt_footer', 'Thank you for shopping with us!', 'Footer text for receipts');

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_barcode ON products(barcode);
CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku);
CREATE INDEX IF NOT EXISTS idx_customers_phone ON customers(phone);
CREATE INDEX IF NOT EXISTS idx_sale_items_sale ON sale_items(sale_id);
CREATE INDEX IF NOT EXISTS idx_sale_items_product ON sale_items(product_id);
CREATE INDEX IF NOT EXISTS idx_sales_date ON sales(created_at);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_product ON inventory_transactions(product_id);
CREATE INDEX IF NOT EXISTS idx_inventory_transactions_date ON inventory_transactions(created_at);
CREATE INDEX IF NOT EXISTS idx_barcode_scans_product ON barcode_scans(product_id);
CREATE INDEX IF NOT EXISTS idx_barcode_scans_time ON barcode_scans(scan_time);

-- Create triggers for automatic timestamp updates
CREATE TRIGGER IF NOT EXISTS update_products_timestamp
  AFTER UPDATE ON products
  FOR EACH ROW
  BEGIN
    UPDATE products SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_categories_timestamp
  AFTER UPDATE ON categories
  FOR EACH ROW
  BEGIN
    UPDATE categories SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_users_timestamp
  AFTER UPDATE ON users
  FOR EACH ROW
  BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_customers_timestamp
  AFTER UPDATE ON customers
  FOR EACH ROW
  BEGIN
    UPDATE customers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_sales_timestamp
  AFTER UPDATE ON sales
  FOR EACH ROW
  BEGIN
    UPDATE sales SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_suppliers_timestamp
  AFTER UPDATE ON suppliers
  FOR EACH ROW
  BEGIN
    UPDATE suppliers SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_purchase_orders_timestamp
  AFTER UPDATE ON purchase_orders
  FOR EACH ROW
  BEGIN
    UPDATE purchase_orders SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;

CREATE TRIGGER IF NOT EXISTS update_settings_timestamp
  AFTER UPDATE ON settings
  FOR EACH ROW
  BEGIN
    UPDATE settings SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
  END;
