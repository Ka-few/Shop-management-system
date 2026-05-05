use rusqlite::{params, Connection, OptionalExtension, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;

/// Initialize database connection pool
pub fn init_db_pool(db_path: &str) -> Result<DbPool> {
    if let Some(parent) = Path::new(db_path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Database directory error: {}", e),
                rusqlite::types::Type::Text,
            )
        })?;
    }

    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(manager).expect("Failed to create database pool");

    // Run migrations
    run_migrations(&pool)?;
    run_phase2_backfills(&pool)?;

    Ok(pool)
}

/// Run database migrations
fn run_migrations(pool: &DbPool) -> Result<()> {
    let conn = pool.get().expect("Failed to get database connection");

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Read and execute migration files
    let migrations = vec![
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_data_migration.sql"),
        include_str!("../../migrations/003_accounting.sql"),
    ];

    for migration in migrations {
        conn.execute_batch(migration)?;
    }

    println!("Database migrations completed successfully");
    Ok(())
}

/// Backfill generated values that are easier and safer to compute in Rust than
/// in SQLite migration SQL.
fn run_phase2_backfills(pool: &DbPool) -> Result<()> {
    let conn = get_connection(pool)?;

    let product_ids = {
        let mut stmt = conn.prepare(
            "SELECT id FROM products WHERE barcode IS NULL OR trim(barcode) = '' ORDER BY id"
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, i32>(0))?
            .collect::<Result<Vec<_>>>()?;
        rows
    };

    for product_id in product_ids {
        let barcode = create_unique_ean13_barcode(&conn, product_id)?;
        conn.execute(
            "UPDATE products SET barcode = ?, barcode_format = 'EAN13', updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![barcode, product_id],
        )?;
    }

    Ok(())
}

fn create_unique_ean13_barcode(conn: &Connection, product_id: i32) -> Result<String> {
    let id = i64::from(product_id).rem_euclid(1_000_000_000);

    for prefix in 200..=299 {
        let base = format!("{}{:09}", prefix, id);
        let candidate = append_ean13_check_digit(&base);
        let existing_product_id: Option<i32> = conn.query_row(
            "SELECT id FROM products WHERE barcode = ?",
            [&candidate],
            |row| row.get(0),
        ).optional()?;

        if existing_product_id.is_none() || existing_product_id == Some(product_id) {
            return Ok(candidate);
        }
    }

    Err(rusqlite::Error::InvalidQuery)
}

fn append_ean13_check_digit(base: &str) -> String {
    let sum: u32 = base
        .chars()
        .enumerate()
        .map(|(index, value)| {
            let digit = value.to_digit(10).unwrap_or(0);
            if index % 2 == 0 { digit } else { digit * 3 }
        })
        .sum();
    let check_digit = (10 - (sum % 10)) % 10;

    format!("{}{}", base, check_digit)
}

/// Get database connection from pool
pub fn get_connection(pool: &DbPool) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
    pool.get().map_err(|e| {
        rusqlite::Error::InvalidColumnType(0, format!("Pool error: {}", e), rusqlite::types::Type::Text)
    })
}

/// Initialize database with default data if empty
pub fn initialize_database(pool: &DbPool) -> Result<()> {
    let conn = get_connection(pool)?;

    // Check if we have any users
    let user_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users",
        [],
        |row| row.get(0)
    ).unwrap_or(0);

    if user_count == 0 {
        println!("No users found. Creating default admin user...");
        create_default_admin(&conn)?;
    }

    Ok(())
}

/// Create default admin user
fn create_default_admin(conn: &Connection) -> Result<()> {
    use bcrypt::{hash, DEFAULT_COST};
    use chrono::Utc;

    let username = "admin";
    let password = "admin123";
    let email = "admin@minimart.com";
    let role = "admin";

    let hashed_password = hash(password, DEFAULT_COST)
        .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "bcrypt error".to_string(), rusqlite::types::Type::Text))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO users (username, password, email, role, created_at) VALUES (?, ?, ?, ?, ?)",
        params![username, hashed_password, email, role, now]
    )?;

    println!("Default admin user created: admin/admin123");
    Ok(())
}

/// Backup database
pub fn backup_database(db_path: &str, backup_path: &str) -> Result<()> {
    fs::copy(db_path, backup_path).map_err(|e| {
        rusqlite::Error::InvalidColumnType(0, format!("Backup error: {}", e), rusqlite::types::Type::Text)
    })?;
    println!("Database backed up to: {}", backup_path);
    Ok(())
}

/// Get database statistics
pub fn get_db_stats(pool: &DbPool) -> Result<serde_json::Value> {
    let conn = get_connection(pool)?;

    let product_count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))?;
    let sale_count: i64 = conn.query_row("SELECT COUNT(*) FROM sales", [], |row| row.get(0))?;
    let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    let category_count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;

    let stats = serde_json::json!({
        "products": product_count,
        "sales": sale_count,
        "users": user_count,
        "categories": category_count
    });

    Ok(stats)
}

/// Validate Phase 2 schema and data assumptions.
pub fn validate_database_integrity(pool: &DbPool) -> Result<serde_json::Value> {
    let conn = get_connection(pool)?;

    let required_tables = [
        "users",
        "customers",
        "categories",
        "products",
        "sales",
        "sale_items",
        "barcode_scans",
        "inventory_transactions",
    ];

    let mut missing_tables = Vec::new();
    for table in required_tables {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            [table],
            |row| row.get(0),
        )?;

        if exists == 0 {
            missing_tables.push(table.to_string());
        }
    }

    let integrity_issues = {
        let mut stmt = conn.prepare(
            "SELECT issue, count FROM phase2_product_integrity_issues ORDER BY issue"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "issue": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        })?
        .collect::<Result<Vec<_>>>()?;
        rows
    };

    let foreign_key_issues = {
        let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
        let rows = stmt.query_map([], |row| {
            Ok(serde_json::json!({
                "table": row.get::<_, String>(0)?,
                "rowid": row.get::<_, i64>(1)?,
                "parent": row.get::<_, String>(2)?,
                "fkid": row.get::<_, i64>(3)?,
            }))
        })?
        .collect::<Result<Vec<_>>>()?;
        rows
    };

    Ok(serde_json::json!({
        "missing_tables": missing_tables,
        "integrity_issues": integrity_issues,
        "foreign_key_issues": foreign_key_issues,
    }))
}
