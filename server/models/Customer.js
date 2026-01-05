const { Pool } = require('pg');
require('dotenv').config();

const pool = new Pool({
    connectionString: process.env.DATABASE_URL,
});

class Customer {
    static async create(customerData) {
        const { name, email, phone, address } = customerData;
        const query = `
      INSERT INTO customers (name, email, phone, address)
      VALUES ($1, $2, $3, $4)
      RETURNING *
    `;
        const values = [name, email, phone, address];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async findAll() {
        const query = 'SELECT * FROM customers ORDER BY created_at DESC';
        const result = await pool.query(query);
        return result.rows;
    }

    static async findById(id) {
        const query = 'SELECT * FROM customers WHERE id = $1';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async update(id, customerData) {
        const { name, email, phone, address } = customerData;
        const query = `
      UPDATE customers 
      SET name = $1, email = $2, phone = $3, address = $4
      WHERE id = $5
      RETURNING *
    `;
        const values = [name, email, phone, address, id];
        const result = await pool.query(query, values);
        return result.rows[0];
    }

    static async delete(id) {
        const query = 'DELETE FROM customers WHERE id = $1 RETURNING *';
        const result = await pool.query(query, [id]);
        return result.rows[0];
    }

    static async findByEmail(email) {
        const query = 'SELECT * FROM customers WHERE email = $1';
        const result = await pool.query(query, [email]);
        return result.rows[0];
    }

    static async search(searchTerm) {
        const query = `
      SELECT * FROM customers 
      WHERE name ILIKE $1 OR email ILIKE $1 OR phone ILIKE $1
      ORDER BY name
    `;
        const result = await pool.query(query, [`%${searchTerm}%`]);
        return result.rows;
    }
}

module.exports = Customer;
