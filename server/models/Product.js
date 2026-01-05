const { Pool } = require('pg');
require('dotenv').config();

const pool = new Pool({
    connectionString: process.env.DATABASE_URL,
});

class Product {
    static async create(productData) {
        const { name, description, price, cost, sku, barcode, category } = productData;
        const query = `
      INSERT INTO products (name, description, price, cost, sku, barcode, category)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      RETURNING *
    `;
        const values = [name, description, price, cost, sku, barcode, category];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async findAll() {
        const query = 'SELECT * FROM products ORDER BY created_at DESC';
        const result = await pool.query(query);
        return result.rows;
    }

    static async findById(id) {
        const query = 'SELECT * FROM products WHERE id = $1';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async update(id, productData) {
        const { name, description, price, cost, sku, barcode, category } = productData;
        const query = `
      UPDATE products 
      SET name = $1, description = $2, price = $3, cost = $4, 
          sku = $5, barcode = $6, category = $7, updated_at = CURRENT_TIMESTAMP
      WHERE id = $8
      RETURNING *
    `;
        const values = [name, description, price, cost, sku, barcode, category, id];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async delete(id) {
        const query = 'DELETE FROM products WHERE id = $1 RETURNING *';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async findBySku(sku) {
        const query = 'SELECT * FROM products WHERE sku = $1';
        const result = await pool.query(query, [sku]);
        return result.rows[0];
    }

    static async findByCategory(category) {
        const query = 'SELECT * FROM products WHERE category = $1 ORDER BY name';
        const result = await pool.query(query, [category]);
        return result.rows;
    }
}

module.exports = Product;
