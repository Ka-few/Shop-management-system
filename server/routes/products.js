const express = require('express');
const router = express.Router();
const pool = require('../config/database');
const { authenticateToken, authorize } = require('../middleware/auth');

// Apply protection to all routes in this file
router.use(authenticateToken);

// Admin-only operations
const adminOnly = authorize(['admin']);

// Get all products
router.get('/', async (req, res) => {
  try {
    const result = await pool.query('SELECT * FROM products ORDER BY id DESC');
    res.json(result.rows);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Get single product
router.get('/:id', async (req, res) => {
  try {
    const { id } = req.params;
    const result = await pool.query('SELECT * FROM products WHERE id = $1', [id]);
    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Product not found' });
    }
    res.json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Create product
router.post('/', adminOnly, async (req, res) => {
  const client = await pool.connect();
  try {
    await client.query('BEGIN');
    const { name, description, price, cost, sku, barcode, category } = req.body;

    // Insert product
    const productResult = await client.query(
      'INSERT INTO products (name, description, price, cost, sku, barcode, category) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *',
      [name, description, price, cost, sku, barcode, category]
    );
    const product = productResult.rows[0];

    // Initialize inventory
    await client.query(
      'INSERT INTO inventory (product_id, quantity, min_stock_level) VALUES ($1, $2, $3)',
      [product.id, 0, 10]
    );

    // Log initial adjustment
    await client.query(
      'INSERT INTO inventory_logs (inventory_id, product_id, change_amount, new_quantity, reason) SELECT id, product_id, 0, 0, \'Initial Stock\' FROM inventory WHERE product_id = $1',
      [product.id]
    );

    await client.query('COMMIT');
    res.status(201).json(product);
  } catch (err) {
    await client.query('ROLLBACK');
    res.status(500).json({ error: err.message });
  } finally {
    client.release();
  }
});

// Update product
router.put('/:id', adminOnly, async (req, res) => {
  try {
    const { id } = req.params;
    const { name, description, price, cost, sku, barcode, category } = req.body;
    const result = await pool.query(
      'UPDATE products SET name = $1, description = $2, price = $3, cost = $4, sku = $5, barcode = $6, category = $7, updated_at = CURRENT_TIMESTAMP WHERE id = $8 RETURNING *',
      [name, description, price, cost, sku, barcode, category, id]
    );
    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Product not found' });
    }
    res.json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Delete product
router.delete('/:id', adminOnly, async (req, res) => {
  try {
    const { id } = req.params;
    await pool.query('DELETE FROM products WHERE id = $1', [id]);
    res.json({ message: 'Product deleted successfully' });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

module.exports = router;