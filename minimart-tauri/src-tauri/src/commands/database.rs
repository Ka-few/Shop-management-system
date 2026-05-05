use crate::db::{DbPool, backup_database, get_db_stats, validate_database_integrity};
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn backup_database_cmd(
    _pool: State<'_, Arc<DbPool>>,
    backup_path: String,
) -> Result<(), String> {
    // For now, we'll use a simple backup approach
    // In a real app, you'd want to get the actual database path
    let db_path = "minimart.db".to_string(); // This should be configurable

    backup_database(&db_path, &backup_path)
        .map_err(|e| format!("Database backup failed: {}", e))
}

#[tauri::command]
pub async fn get_database_stats(
    pool: State<'_, Arc<DbPool>>,
) -> Result<serde_json::Value, String> {
    get_db_stats(&pool)
        .map_err(|e| format!("Failed to get database stats: {}", e))
}

#[tauri::command]
pub async fn validate_database(
    pool: State<'_, Arc<DbPool>>,
) -> Result<serde_json::Value, String> {
    validate_database_integrity(&pool)
        .map_err(|e| format!("Database validation failed: {}", e))
}

#[tauri::command]
pub async fn reset_database(
    pool: State<'_, Arc<DbPool>>,
) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // This is a dangerous operation - in production you'd want confirmation
    // For now, we'll just clear all data except settings

    let tables_to_clear = vec![
        "barcode_scans",
        "inventory_transactions",
        "sale_items",
        "sales",
        "products",
        "categories",
        "customers",
        "users",
    ];

    for table in &tables_to_clear {
        conn.execute(&format!("DELETE FROM {}", table), [])
            .map_err(|e| format!("Error clearing table {}: {}", table, e))?;
    }

    // Reset auto-increment counters
    for table in &tables_to_clear {
        conn.execute(&format!("DELETE FROM sqlite_sequence WHERE name = '{}'", table), [])
            .ok(); // Ignore errors for tables without auto-increment
    }

    // Re-initialize with default data
    crate::db::initialize_database(&pool)
        .map_err(|e| format!("Database initialization failed: {}", e))?;

    Ok(())
}
