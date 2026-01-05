-- Shop Management System Database Setup
-- Run this file with: sudo -u postgres psql < setup-db.sql

-- Create user
CREATE USER shopuser WITH PASSWORD 'shoppass123';

-- Create database
CREATE DATABASE shop_management OWNER shopuser;

-- Connect to the database
\c shop_management

-- Run the schema
\i database/init.sql

-- Grant all privileges
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO shopuser;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO shopuser;

-- Verify tables
\dt
