use crate::db::DbPool;
use crate::models::{Category, NewCategory};
use chrono::Utc;
use rusqlite::Result;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_categories(pool: State<'_, Arc<DbPool>>) -> Result<Vec<Category>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let categories = conn.prepare("SELECT id, name, icon, description, created_at, updated_at FROM categories ORDER BY name")
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(categories)
}

#[tauri::command]
pub async fn create_category(
    pool: State<'_, Arc<DbPool>>,
    category: NewCategory,
) -> Result<Category, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO categories (name, icon, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            category.name,
            category.icon,
            category.description,
            now,
            now,
        ]
    ).map_err(|e| format!("Category creation error: {}", e))?;

    let category_id = conn.last_insert_rowid() as i32;

    // Return the created category
    let category = conn.query_row(
        "SELECT id, name, icon, description, created_at, updated_at FROM categories WHERE id = ?",
        [category_id],
        |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }
    ).map_err(|e| format!("Category retrieval error: {}", e))?;

    Ok(category)
}
