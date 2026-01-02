const express = require('express');
const router = express.Router();
const pool = require('../config/database');

router.get('/', async (req, res) => {
  try {
    const result = await pool.query(`
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i 
      JOIN products p ON i.product_id = p.id 
      ORDER BY p.name
    `);
    res.json(result.rows);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

router.get('/low-stock', async (req, res) => {
  try {
    const result = await pool.query(`
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i 
      JOIN products p ON i.product_id = p.id 
      WHERE i.quantity <= i.min_stock_level
    `);
    res.json(result.rows);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

router.get('/:productId', async (req, res) => {
  try {
    const { productId } = req.params;
    const result = await pool.query(`
      SELECT i.*, p.name as product_name 
      FROM inventory i 
      JOIN products p ON i.product_id = p.id 
      WHERE i.product_id = $1
    `, [productId]);
    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Inventory not found' });
    }
    res.json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

router.post('/', async (req, res) => {
  try {
    const { product_id, quantity, min_stock_level } = req.body;
    const result = await pool.query(
      'INSERT INTO inventory (product_id, quantity, min_stock_level) VALUES ($1, $2, $3) RETURNING *',
      [product_id, quantity, min_stock_level || 10]
    );
    res.status(201).json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

router.put('/:productId', async (req, res) => {
  try {
    const { productId } = req.params;
    const { quantity, min_stock_level } = req.body;
    const result = await pool.query(
      'UPDATE inventory SET quantity = $1, min_stock_level = $2, last_updated = CURRENT_TIMESTAMP WHERE product_id = $3 RETURNING *',
      [quantity, min_stock_level, productId]
    );
    if (result.rows.length === 0) {
      return res.status(404).json({ error: 'Inventory not found' });
    }
    res.json(result.rows[0]);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

module.exports = router;