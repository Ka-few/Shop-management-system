use crate::db::DbPool;
use crate::models::{Setting, UpdateSetting};
use chrono::Utc;
use rusqlite::Result;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_settings(pool: State<'_, Arc<DbPool>>) -> Result<Vec<Setting>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let settings = conn
        .prepare("SELECT id, key, value, description, updated_at FROM settings ORDER BY key")
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map([], |row| {
            Ok(Setting {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                description: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(settings)
}

#[tauri::command]
pub async fn update_setting(
    pool: State<'_, Arc<DbPool>>,
    key: String,
    update: UpdateSetting,
) -> Result<Setting, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.execute(
        "UPDATE settings SET value = ?, updated_at = ? WHERE key = ?",
        [&update.value, &Utc::now().to_rfc3339(), &key],
    )
    .map_err(|e| format!("Setting update error: {}", e))?;

    // Return the updated setting
    let setting = conn
        .query_row(
            "SELECT id, key, value, description, updated_at FROM settings WHERE key = ?",
            [&key],
            |row| {
                Ok(Setting {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    description: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        )
        .map_err(|e| format!("Setting retrieval error: {}", e))?;

    Ok(setting)
}

#[tauri::command]
pub async fn get_setting_value(
    pool: State<'_, Arc<DbPool>>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let value: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?", [&key], |row| {
            row.get(0)
        })
        .ok(); // Return None if not found

    Ok(value)
}
