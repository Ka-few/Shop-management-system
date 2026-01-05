#!/bin/bash

# Shop Management System - Database Setup Script
# This script sets up the PostgreSQL database for the shop management system

echo "Setting up PostgreSQL database for Shop Management System..."

# Create database user if it doesn't exist
echo "Creating database user 'shopuser'..."
sudo -u postgres psql -c "CREATE USER shopuser WITH PASSWORD 'shoppass123';" 2>/dev/null || echo "User may already exist"

# Create database
echo "Creating database 'shop_management'..."
sudo -u postgres psql -c "CREATE DATABASE shop_management OWNER shopuser;" 2>/dev/null || echo "Database may already exist"

# Grant privileges
echo "Granting privileges..."
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE shop_management TO shopuser;"

# Run schema
echo "Running database schema..."
sudo -u postgres psql -d shop_management -f database/init.sql

# Grant privileges on tables
echo "Granting table privileges..."
sudo -u postgres psql -d shop_management -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO shopuser;"
sudo -u postgres psql -d shop_management -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO shopuser;"

echo ""
echo "✅ Database setup complete!"
echo ""
echo "Database credentials:"
echo "  Database: shop_management"
echo "  User: shopuser"
echo "  Password: shoppass123"
echo "  Host: localhost"
echo "  Port: 5432"
echo ""
echo "Update your .env file with these credentials."
