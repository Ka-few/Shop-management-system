use crate::db::DbPool;
use crate::models::InventoryStatus;
use rusqlite::Result;
use tauri::State;
use std::sync::Arc;
use chrono::Utc;

#[tauri::command]
pub async fn get_inventory_status(
    pool: State<'_, Arc<DbPool>>,
) -> Result<Vec<InventoryStatus>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let status_items = conn.prepare(
        "SELECT p.id, p.name, p.quantity_in_stock, p.reorder_level FROM products p ORDER BY p.name"
    )
    .map_err(|e| format!("Query preparation error: {}", e))?
    .query_map([], |row| {
        let id: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let stock: i32 = row.get(2)?;
        let reorder_level: i32 = row.get(3)?;

        let status = if stock <= 0 {
            "out_of_stock"
        } else if stock <= reorder_level {
            "low_stock"
        } else {
            "in_stock"
        };

        Ok(InventoryStatus {
            product_id: id,
            product_name: name,
            current_stock: stock,
            reorder_level,
            status: status.to_string(),
        })
    })
    .map_err(|e| format!("Query execution error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(status_items)
}

#[tauri::command]
pub async fn adjust_inventory(
    pool: State<'_, Arc<DbPool>>,
    product_id: i32,
    quantity: i32,
    reason: String,
) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // Begin transaction
    conn.execute("BEGIN", [])
        .map_err(|e| format!("Transaction begin error: {}", e))?;

    // Update product stock
    let result = conn.execute(
        "UPDATE products SET quantity_in_stock = quantity_in_stock + ?, updated_at = ? WHERE id = ?",
        rusqlite::params![quantity, Utc::now().to_rfc3339(), product_id]
    );

    if let Err(e) = result {
        conn.execute("ROLLBACK", [])
            .map_err(|e| format!("Transaction rollback error: {}", e))?;
        return Err(format!("Inventory adjustment error: {}", e));
    }

    // Log the transaction
    let transaction_result = conn.execute(
        "INSERT INTO inventory_transactions (product_id, transaction_type, quantity, notes, created_at) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            product_id,
            "adjustment",
            quantity,
            reason,
            Utc::now().to_rfc3339(),
        ]
    );

    if let Err(e) = transaction_result {
        conn.execute("ROLLBACK", [])
            .map_err(|e| format!("Transaction rollback error: {}", e))?;
        return Err(format!("Transaction logging error: {}", e));
    }

    // Commit transaction
    conn.execute("COMMIT", [])
        .map_err(|e| format!("Transaction commit error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_low_stock_items(
    pool: State<'_, Arc<DbPool>>,
    threshold: Option<i32>,
) -> Result<Vec<InventoryStatus>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let threshold_value = threshold.unwrap_or(10);

    let low_stock_items = conn.prepare(
        "SELECT p.id, p.name, p.quantity_in_stock, p.reorder_level FROM products p WHERE p.quantity_in_stock <= ? ORDER BY p.quantity_in_stock ASC"
    )
    .map_err(|e| format!("Query preparation error: {}", e))?
    .query_map([threshold_value], |row| {
        let id: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let stock: i32 = row.get(2)?;
        let reorder_level: i32 = row.get(3)?;

        let status = if stock <= 0 {
            "out_of_stock"
        } else {
            "low_stock"
        };

        Ok(InventoryStatus {
            product_id: id,
            product_name: name,
            current_stock: stock,
            reorder_level,
            status: status.to_string(),
        })
    })
    .map_err(|e| format!("Query execution error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(low_stock_items)
}

#[tauri::command]
pub async fn get_inventory_transactions(
    pool: State<'_, Arc<DbPool>>,
    product_id: Option<i32>,
    limit: Option<i32>,
) -> Result<Vec<crate::models::InventoryTransaction>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let mut query = "SELECT id, product_id, transaction_type, quantity, reference_id, notes, created_by, created_at FROM inventory_transactions".to_string();
    let mut params: Vec<String> = Vec::new();

    if let Some(pid) = product_id {
        query.push_str(" WHERE product_id = ?");
        params.push(pid.to_string());
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(l) = limit {
        query.push_str(&format!(" LIMIT {}", l));
    }

    let param_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
    let transactions = conn.prepare(&query)
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map(rusqlite::params_from_iter(param_refs), |row| {
            Ok(crate::models::InventoryTransaction {
                id: row.get(0)?,
                product_id: row.get(1)?,
                transaction_type: row.get(2)?,
                quantity: row.get(3)?,
                reference_id: row.get(4)?,
                notes: row.get(5)?,
                created_by: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(transactions)
}
