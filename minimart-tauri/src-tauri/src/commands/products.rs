use crate::db::DbPool;
use crate::models::{Product, NewProduct, UpdateProduct};
use rusqlite::{params_from_iter, Result};
use tauri::State;
use std::sync::Arc;
use chrono::Utc;

const PRODUCT_IN_USE_DELETE_ERROR: &str = "Products already used in sales or inventory history cannot be deleted. Set stock to 0 instead.";

#[tauri::command]
pub async fn get_products(
    pool: State<'_, Arc<DbPool>>,
    category_id: Option<i32>,
) -> Result<Vec<Product>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let mut query = "SELECT id, name, category_id, sku, barcode, barcode_format, description, unit_price, cost_price, quantity_in_stock, reorder_level, expiry_date, created_at, updated_at FROM products".to_string();
    let mut params: Vec<String> = Vec::new();

    if let Some(cat_id) = category_id {
        query.push_str(" WHERE category_id = ?");
        params.push(cat_id.to_string());
    }

    query.push_str(" ORDER BY name");

    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("Query preparation error: {}", e))?;

    let param_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
    let product_iter = stmt.query_map(params_from_iter(param_refs), |row| {
        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            category_id: row.get(2)?,
            sku: row.get(3)?,
            barcode: row.get(4)?,
            barcode_format: row.get(5)?,
            description: row.get(6)?,
            unit_price: row.get(7)?,
            cost_price: row.get(8)?,
            quantity_in_stock: row.get(9)?,
            reorder_level: row.get(10)?,
            expiry_date: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })
    .map_err(|e| format!("Query execution error: {}", e))?;

    let mut products = Vec::new();
    for product in product_iter {
        products.push(product.map_err(|e| format!("Result collection error: {}", e))?);
    }

    Ok(products)
}

#[tauri::command]
pub async fn get_product(
    pool: State<'_, Arc<DbPool>>,
    id: i32,
) -> Result<Product, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let product = conn.query_row(
        "SELECT id, name, category_id, sku, barcode, barcode_format, description, unit_price, cost_price, quantity_in_stock, reorder_level, expiry_date, created_at, updated_at FROM products WHERE id = ?",
        [id],
        |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                category_id: row.get(2)?,
                sku: row.get(3)?,
                barcode: row.get(4)?,
                barcode_format: row.get(5)?,
                description: row.get(6)?,
                unit_price: row.get(7)?,
                cost_price: row.get(8)?,
                quantity_in_stock: row.get(9)?,
                reorder_level: row.get(10)?,
                expiry_date: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        }
    ).map_err(|e| format!("Product not found: {}", e))?;

    Ok(product)
}

#[tauri::command]
pub async fn get_product_by_barcode(
    pool: State<'_, Arc<DbPool>>,
    barcode: String,
) -> Result<Product, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let product = conn.query_row(
        "SELECT id, name, category_id, sku, barcode, barcode_format, description, unit_price, cost_price, quantity_in_stock, reorder_level, expiry_date, created_at, updated_at FROM products WHERE barcode = ?",
        [&barcode],
        |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                category_id: row.get(2)?,
                sku: row.get(3)?,
                barcode: row.get(4)?,
                barcode_format: row.get(5)?,
                description: row.get(6)?,
                unit_price: row.get(7)?,
                cost_price: row.get(8)?,
                quantity_in_stock: row.get(9)?,
                reorder_level: row.get(10)?,
                expiry_date: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        }
    ).map_err(|e| format!("Product not found for barcode: {}", e))?;

    Ok(product)
}

#[tauri::command]
pub async fn create_product(
    pool: State<'_, Arc<DbPool>>,
    product: NewProduct,
) -> Result<Product, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO products (name, category_id, sku, barcode, barcode_format, description, unit_price, cost_price, quantity_in_stock, reorder_level, expiry_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            product.name,
            product.category_id,
            product.sku,
            product.barcode,
            product.barcode_format.unwrap_or_else(|| "EAN13".to_string()),
            product.description,
            product.unit_price,
            product.cost_price,
            product.quantity_in_stock.unwrap_or(0),
            product.reorder_level.unwrap_or(10),
            product.expiry_date,
            now,
            now,
        ]
    ).map_err(|e| format!("Product creation error: {}", e))?;

    let product_id = conn.last_insert_rowid() as i32;

    // Return the created product
    get_product(pool, product_id).await
}

#[tauri::command]
pub async fn update_product(
    pool: State<'_, Arc<DbPool>>,
    id: i32,
    updates: UpdateProduct,
) -> Result<Product, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let previous_stock: Option<i32> = if updates.quantity_in_stock.is_some() {
        Some(conn.query_row(
            "SELECT quantity_in_stock FROM products WHERE id = ?",
            [id],
            |row| row.get(0),
        ).map_err(|e| format!("Product stock lookup error: {}", e))?)
    } else {
        None
    };

    let mut query = "UPDATE products SET".to_string();
    let mut params: Vec<String> = Vec::new();
    let mut set_parts: Vec<String> = Vec::new();

    if let Some(name) = &updates.name {
        set_parts.push(" name = ?".to_string());
        params.push(name.clone());
    }
    if let Some(category_id) = updates.category_id {
        set_parts.push(" category_id = ?".to_string());
        params.push(category_id.to_string());
    }
    if let Some(sku) = &updates.sku {
        set_parts.push(" sku = ?".to_string());
        params.push(sku.clone());
    }
    if let Some(barcode) = &updates.barcode {
        set_parts.push(" barcode = ?".to_string());
        params.push(barcode.clone());
    }
    if let Some(barcode_format) = &updates.barcode_format {
        set_parts.push(" barcode_format = ?".to_string());
        params.push(barcode_format.clone());
    }
    if let Some(description) = &updates.description {
        set_parts.push(" description = ?".to_string());
        params.push(description.clone());
    }
    if let Some(unit_price) = updates.unit_price {
        set_parts.push(" unit_price = ?".to_string());
        params.push(unit_price.to_string());
    }
    if let Some(cost_price) = updates.cost_price {
        set_parts.push(" cost_price = ?".to_string());
        params.push(cost_price.to_string());
    }
    if let Some(quantity_in_stock) = updates.quantity_in_stock {
        set_parts.push(" quantity_in_stock = ?".to_string());
        params.push(quantity_in_stock.to_string());
    }
    if let Some(reorder_level) = updates.reorder_level {
        set_parts.push(" reorder_level = ?".to_string());
        params.push(reorder_level.to_string());
    }
    if let Some(expiry_date) = &updates.expiry_date {
        set_parts.push(" expiry_date = ?".to_string());
        params.push(expiry_date.clone());
    }

    if set_parts.is_empty() {
        return Err("No fields to update".to_string());
    }

    set_parts.push(" updated_at = ?".to_string());
    params.push(Utc::now().to_rfc3339());

    query.push_str(&set_parts.join(","));
    query.push_str(" WHERE id = ?");
    params.push(id.to_string());

    let param_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
    conn.execute(&query, rusqlite::params_from_iter(param_refs))
        .map_err(|e| format!("Product update error: {}", e))?;

    if let (Some(old_stock), Some(new_stock)) = (previous_stock, updates.quantity_in_stock) {
        let delta = new_stock - old_stock;
        if delta != 0 {
            conn.execute(
                "INSERT INTO inventory_transactions (product_id, transaction_type, quantity, notes, created_at) VALUES (?, ?, ?, ?, ?)",
                rusqlite::params![
                    id,
                    "adjustment",
                    delta,
                    "Product edit stock adjustment",
                    Utc::now().to_rfc3339(),
                ],
            ).map_err(|e| format!("Inventory transaction logging error: {}", e))?;
        }
    }

    // Return the updated product
    get_product(pool, id).await
}

#[tauri::command]
pub async fn delete_product(
    pool: State<'_, Arc<DbPool>>,
    id: i32,
) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let product_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM products WHERE id = ?)",
        [id],
        |row| row.get(0),
    ).map_err(|e| format!("Product lookup error: {}", e))?;

    if !product_exists {
        return Err("Product not found.".to_string());
    }

    let has_history: bool = conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM sale_items WHERE product_id = ?
            UNION ALL
            SELECT 1 FROM inventory_transactions WHERE product_id = ?
            UNION ALL
            SELECT 1 FROM barcode_scans WHERE product_id = ?
            UNION ALL
            SELECT 1 FROM purchase_order_items WHERE product_id = ?
        )",
        rusqlite::params![id, id, id, id],
        |row| row.get(0),
    ).map_err(|e| format!("Product history lookup error: {}", e))?;

    if has_history {
        return Err(PRODUCT_IN_USE_DELETE_ERROR.to_string());
    }

    conn.execute("DELETE FROM products WHERE id = ?", [id])
        .map_err(|e| {
            if matches!(e, rusqlite::Error::SqliteFailure(ref err, _) if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_FOREIGNKEY) {
                PRODUCT_IN_USE_DELETE_ERROR.to_string()
            } else {
                format!("Product deletion error: {}", e)
            }
        })?;

    Ok(())
}
