const express = require('express');
const router = express.Router();
const pool = require('../config/database');
const { authenticateToken, authorize } = require('../middleware/auth');

router.use(authenticateToken);
router.use(authorize(['admin']));

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

router.get('/:productId/logs', async (req, res) => {
  try {
    const { productId } = req.params;
    const result = await pool.query(`
      SELECT il.*, p.name as product_name 
      FROM inventory_logs il
      JOIN products p ON il.product_id = p.id
      WHERE il.product_id = $1
      ORDER BY il.created_at DESC
    `, [productId]);
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
  const client = await pool.connect();
  try {
    await client.query('BEGIN');
    const { productId } = req.params;
    const { quantity, min_stock_level, reason, change_amount } = req.body;

    // Get current quantity and min level
    const currentRes = await client.query('SELECT quantity, min_stock_level, id FROM inventory WHERE product_id = $1', [productId]);
    if (currentRes.rows.length === 0) {
      await client.query('ROLLBACK');
      return res.status(404).json({ error: 'Inventory not found' });
    }
    const inv = currentRes.rows[0];
    const oldQty = inv.quantity;
    const currentMin = inv.min_stock_level;
    const invId = inv.id;

    // Use current min_stock_level if not provided
    const finalMin = min_stock_level !== undefined ? min_stock_level : currentMin;

    // Update inventory
    const result = await client.query(
      'UPDATE inventory SET quantity = $1, min_stock_level = $2, last_updated = CURRENT_TIMESTAMP WHERE product_id = $3 RETURNING *',
      [quantity, finalMin, productId]
    );

    // Filter out null or undefined
    const actualChange = change_amount !== undefined ? change_amount : (quantity - oldQty);

    // Log the change
    await client.query(
      'INSERT INTO inventory_logs (inventory_id, product_id, change_amount, new_quantity, reason) VALUES ($1, $2, $3, $4, $5)',
      [invId, productId, actualChange, quantity, reason || 'Manual Adjustment']
    );

    await client.query('COMMIT');
    res.json(result.rows[0]);
  } catch (err) {
    await client.query('ROLLBACK');
    res.status(500).json({ error: err.message });
  } finally {
    client.release();
  }
});

module.exports = router;