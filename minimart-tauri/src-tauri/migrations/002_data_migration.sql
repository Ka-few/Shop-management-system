-- Phase 2 Data Backfill
-- Idempotent defaults and integrity preparation for the enhanced minimart schema.

PRAGMA foreign_keys = ON;

-- Keep default categories available even when this migration is run against an
-- existing database created before the full category seed existed.
INSERT OR IGNORE INTO categories (name, icon, description) VALUES
('Beverages', '🥤', 'Sodas, Water, Juice, Coffee, Tea, Energy Drinks'),
('Snacks', '🍿', 'Chips, Cookies, Nuts, Candy, Biscuits'),
('Toiletries', '🧴', 'Soap, Shampoo, Toothpaste, Tissues, Deodorant'),
('Food Items', '🍳', 'Eggs, Milk, Rice, Sugar, Flour, Oil, Salt'),
('Spices & Condiments', '🧂', 'Salt, Pepper, Soy Sauce, Cooking Oil, Herbs'),
('Cleaning Supplies', '🧴', 'Detergent, Bleach, Disinfectant, Cleaning Products'),
('Packaged Foods', '📦', 'Pasta, Bread, Butter, Cheese, Canned Goods'),
('Fresh Produce', '🍋', 'Fruits, Vegetables (when added in future)');

-- Normalize legacy or partially imported products that reached the new schema
-- without Phase 2-required values.
UPDATE products
SET sku = printf('MIN-%06d', id)
WHERE sku IS NULL OR trim(sku) = '';

UPDATE products
SET barcode_format = 'EAN13'
WHERE barcode_format IS NULL OR trim(barcode_format) = '';

UPDATE products
SET reorder_level = 10
WHERE reorder_level IS NULL;

UPDATE products
SET quantity_in_stock = 0
WHERE quantity_in_stock IS NULL;

-- Seed an initial inventory audit record for stock that predates the
-- transaction table. The NOT EXISTS guard keeps this migration repeatable.
INSERT INTO inventory_transactions (
  product_id,
  transaction_type,
  quantity,
  notes,
  created_at
)
SELECT
  p.id,
  'adjustment',
  p.quantity_in_stock,
  'Initial stock import from Phase 2 migration',
  COALESCE(p.created_at, CURRENT_TIMESTAMP)
FROM products p
WHERE p.quantity_in_stock > 0
  AND NOT EXISTS (
    SELECT 1
    FROM inventory_transactions it
    WHERE it.product_id = p.id
      AND it.transaction_type = 'adjustment'
      AND it.notes = 'Initial stock import from Phase 2 migration'
  );

-- Validation views used by the database diagnostics command.
CREATE VIEW IF NOT EXISTS phase2_product_integrity_issues AS
SELECT 'missing_sku' AS issue, COUNT(*) AS count
FROM products
WHERE sku IS NULL OR trim(sku) = ''
UNION ALL
SELECT 'missing_barcode', COUNT(*)
FROM products
WHERE barcode IS NULL OR trim(barcode) = ''
UNION ALL
SELECT 'invalid_category', COUNT(*)
FROM products p
LEFT JOIN categories c ON c.id = p.category_id
WHERE c.id IS NULL
UNION ALL
SELECT 'duplicate_sku', COUNT(*)
FROM (
  SELECT sku
  FROM products
  WHERE sku IS NOT NULL AND trim(sku) <> ''
  GROUP BY sku
  HAVING COUNT(*) > 1
)
UNION ALL
SELECT 'duplicate_barcode', COUNT(*)
FROM (
  SELECT barcode
  FROM products
  WHERE barcode IS NOT NULL AND trim(barcode) <> ''
  GROUP BY barcode
  HAVING COUNT(*) > 1
);
