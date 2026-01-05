const { Pool } = require('pg');
const fs = require('fs');
const path = require('path');
require('dotenv').config();

const pool = new Pool({
    user: process.env.DB_USER,
    host: process.env.DB_HOST,
    database: process.env.DB_NAME,
    password: process.env.DB_PASSWORD,
    port: process.env.DB_PORT,
});

async function run() {
    try {
        // 1. Check existing tables and owners
        const tables = await pool.query(`
      SELECT tablename, tableowner 
      FROM pg_tables 
      WHERE schemaname = 'public'
    `);
        console.log('Existing tables and owners:', tables.rows);

        // 2. Try to run init.sql statement by statement
        const sqlPath = path.join(__dirname, 'database', 'init.sql');
        const sql = fs.readFileSync(sqlPath, 'utf8');
        const statements = sql.split(';').filter(s => s.trim().length > 0);

        for (let statement of statements) {
            try {
                await pool.query(statement);
                console.log('Success:', statement.trim().substring(0, 50) + '...');
            } catch (e) {
                console.error('Failed statement:', statement.trim().substring(0, 50) + '...');
                console.error('Error:', e.message);
            }
        }

        process.exit(0);
    } catch (err) {
        console.error('Fatal error:', err.message);
        process.exit(1);
    }
}

run();
