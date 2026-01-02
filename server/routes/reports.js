const express = require('express');
const router = express.Router();
const pool = require('../config/database');

router.get('/sales', async (req, res) => {
  try {
    const { from, to } = req.query;
    
    const salesData = await pool.query(`
      SELECT 
        COUNT(*) as total_transactions,
        SUM(total_amount) as total_sales,
        AVG(total_amount) as avg_sale
      FROM sales 
      WHERE sale_date BETWEEN $1 AND $2
    `, [from, to]);
    
    const topProducts = await pool.query(`
      SELECT 
        p.name as product_name,
        SUM(si.quantity) as total_quantity,
        SUM(si.subtotal) as total_revenue
      FROM sale_items si
      JOIN products p ON si.product_id = p.id
      JOIN sales s ON si.sale_id = s.id
      WHERE s.sale_date BETWEEN $1 AND $2
      GROUP BY p.id, p.name
      ORDER BY total_revenue DESC
      LIMIT 10
    `, [from, to]);
    
    const profitData = await pool.query(`
      SELECT 
        SUM(si.subtotal - (p.cost * si.quantity)) as total_profit
      FROM sale_items si
      JOIN products p ON si.product_id = p.id
      JOIN sales s ON si.sale_id = s.id
      WHERE s.sale_date BETWEEN $1 AND $2
    `, [from, to]);
    
    res.json({
      totalTransactions: parseInt(salesData.rows[0].total_transactions) || 0,
      totalSales: parseFloat(salesData.rows[0].total_sales) || 0,
      avgSale: parseFloat(salesData.rows[0].avg_sale) || 0,
      totalProfit: parseFloat(profitData.rows[0].total_profit) || 0,
      topProducts: topProducts.rows
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

module.exports = router;