# PostgreSQL Database Setup Guide

## Quick Setup (Recommended)

Run these commands in your terminal:

```bash
# 1. Access PostgreSQL as superuser
sudo -u postgres psql

# 2. Once in psql, run these commands:
CREATE USER shopuser WITH PASSWORD 'shoppass123';
CREATE DATABASE shop_management OWNER shopuser;
\q

# 3. Initialize the database schema
sudo -u postgres psql -d shop_management -f database/init.sql

# 4. Grant privileges
sudo -u postgres psql -d shop_management -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO shopuser;"
sudo -u postgres psql -d shop_management -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO shopuser;"
```

## Update .env File

Edit your `.env` file and update these values:

```env
DB_USER=shopuser
DB_HOST=localhost
DB_NAME=shop_management
DB_PASSWORD=shoppass123
DB_PORT=5432
PORT=3000
```

## Alternative: Use Existing Postgres User

If you prefer to use the default `postgres` user:

```bash
# 1. Set a password for postgres user (if not already set)
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'your_secure_password';"

# 2. Create database
sudo -u postgres createdb shop_management

# 3. Initialize schema
sudo -u postgres psql -d shop_management -f database/init.sql
```

Then update `.env`:
```env
DB_USER=postgres
DB_PASSWORD=your_secure_password
```

## Verify Setup

After setup, verify the database is working:

```bash
# Connect to database
psql -U shopuser -d shop_management -h localhost

# List tables (should show: products, customers, sales, sale_items, inventory, users)
\dt

# Exit
\q
```

## Restart Server

After database setup, restart your server:

```bash
# Stop the current server (Ctrl+C if running)
# Then start it again
node server/server.js
```

You should see: `Connected to PostgreSQL database`

## Troubleshooting

### "role does not exist"
Run: `sudo -u postgres psql -c "CREATE USER shopuser WITH PASSWORD 'shoppass123';"`

### "database does not exist"
Run: `sudo -u postgres createdb shop_management`

### "password authentication failed"
Make sure your `.env` file has the correct password matching what you set.

### "FATAL: Peer authentication failed"
Edit `/etc/postgresql/18/main/pg_hba.conf` and change:
```
local   all   all   peer
```
to:
```
local   all   all   md5
```
Then restart PostgreSQL: `sudo systemctl restart postgresql`
