const { Pool } = require('pg');
require('dotenv').config();

const pool = new Pool({
    connectionString: process.env.DATABASE_URL,
});

class Inventory {
    static async create(inventoryData) {
        const { product_id, quantity, min_stock_level } = inventoryData;
        const query = `
      INSERT INTO inventory (product_id, quantity, min_stock_level)
      VALUES ($1, $2, $3)
      RETURNING *
    `;
        const values = [product_id, quantity, min_stock_level || 10];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async findAll() {
        const query = `
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i
      JOIN products p ON i.product_id = p.id
      ORDER BY p.name
    `;
        const result = await pool.query(query);
        return result.rows;
    }

    static async findById(id) {
        const query = `
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i
      JOIN products p ON i.product_id = p.id
      WHERE i.id = $1
    `;
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async findByProductId(productId) {
        const query = `
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i
      JOIN products p ON i.product_id = p.id
      WHERE i.product_id = $1
    `;
        const result = await pool.query(query, [productId]);
        return result.rows[0];
    }

    static async update(id, inventoryData) {
        const { quantity, min_stock_level } = inventoryData;
        const query = `
      UPDATE inventory 
      SET quantity = $1, min_stock_level = $2, last_updated = CURRENT_TIMESTAMP
      WHERE id = $3
      RETURNING *
    `;
        const values = [quantity, min_stock_level, id];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async updateQuantity(productId, quantityChange) {
        const query = `
      UPDATE inventory 
      SET quantity = quantity + $1, last_updated = CURRENT_TIMESTAMP
      WHERE product_id = $2
      RETURNING *
    `;
        const result = await pool.query(query, [quantityChange, productId]);
        return result.rows[0];
    }

    static async getLowStockItems() {
        const query = `
      SELECT i.*, p.name as product_name, p.sku 
      FROM inventory i
      JOIN products p ON i.product_id = p.id
      WHERE i.quantity <= i.min_stock_level
      ORDER BY i.quantity ASC
    `;
        const result = await pool.query(query);
        return result.rows;
    }

    static async getLowStockCount() {
        const query = `
      SELECT COUNT(*) as count 
      FROM inventory 
      WHERE quantity <= min_stock_level
    `;
        const result = await pool.query(query);
        return parseInt(result.rows[0].count);
    }

    static async delete(id) {
        const query = 'DELETE FROM inventory WHERE id = $1 RETURNING *';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }
}

module.exports = Inventory;
