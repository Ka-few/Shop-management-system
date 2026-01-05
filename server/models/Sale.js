const { Pool } = require('pg');
require('dotenv').config();

const pool = new Pool({
    connectionString: process.env.DATABASE_URL,
});

class Sale {
    static async create(saleData) {
        const client = await pool.connect();
        try {
            await client.query('BEGIN');

            const { customer_id, total_amount, payment_method, items } = saleData;

            // Create the sale
            const saleQuery = `
        INSERT INTO sales (customer_id, total_amount, payment_method)
        VALUES ($1, $2, $3)
        RETURNING *
      `;
            const saleResult = await client.query(saleQuery, [customer_id, total_amount, payment_method]);
            const sale = saleResult.rows[0];

            // Create sale items and update inventory
            if (items && items.length > 0) {
                for (const item of items) {
                    const { product_id, quantity, unit_price, subtotal } = item;

                    // Insert sale item
                    const itemQuery = `
            INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, subtotal)
            VALUES ($1, $2, $3, $4, $5)
          `;
                    await client.query(itemQuery, [sale.id, product_id, quantity, unit_price, subtotal]);

                    // Update inventory
                    const inventoryQuery = `
            UPDATE inventory 
            SET quantity = quantity - $1, last_updated = CURRENT_TIMESTAMP
            WHERE product_id = $2
          `;
                    await client.query(inventoryQuery, [quantity, product_id]);
                }
            }

            await client.query('COMMIT');
            return sale;
        } catch (error) {
            await client.query('ROLLBACK');
            throw error;
        } finally {
            client.release();
        }
    }

    static async findAll() {
        const query = `
      SELECT s.*, c.name as customer_name 
      FROM sales s
      LEFT JOIN customers c ON s.customer_id = c.id
      ORDER BY s.sale_date DESC
    `;
        const result = await pool.query(query);
        return result.rows;
    }

    static async findById(id) {
        const query = `
      SELECT s.*, c.name as customer_name 
      FROM sales s
      LEFT JOIN customers c ON s.customer_id = c.id
      WHERE s.id = $1
    `;
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async findSaleItems(saleId) {
        const query = `
      SELECT si.*, p.name as product_name 
      FROM sale_items si
      JOIN products p ON si.product_id = p.id
      WHERE si.sale_id = $1
    `;
        const result = await pool.query(query, [saleId]);
        return result.rows;
    }

    static async getTotalSales() {
        const query = 'SELECT COALESCE(SUM(total_amount), 0) as total FROM sales';
        const result = await pool.query(query);
        return parseFloat(result.rows[0].total);
    }

    static async getSalesByDateRange(startDate, endDate) {
        const query = `
      SELECT s.*, c.name as customer_name 
      FROM sales s
      LEFT JOIN customers c ON s.customer_id = c.id
      WHERE s.sale_date BETWEEN $1 AND $2
      ORDER BY s.sale_date DESC
    `;
        const result = await pool.query(query, [startDate, endDate]);
        return result.rows;
    }

    static async delete(id) {
        const query = 'DELETE FROM sales WHERE id = $1 RETURNING *';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }
}

module.exports = Sale;
