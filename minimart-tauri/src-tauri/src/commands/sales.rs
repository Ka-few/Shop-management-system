use crate::commands::accounting;
use crate::db::DbPool;
use crate::models::{NewSale, NewSaleItem, Sale, SaleItem};
use chrono::Utc;
use rusqlite::{params, Result};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn create_sale(pool: State<'_, Arc<DbPool>>, sale: NewSale) -> Result<Sale, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sales (customer_id, total_amount, payment_method, payment_amount, change_amount, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            sale.customer_id,
            0.0, // Will be calculated
            sale.payment_method.unwrap_or_else(|| "cash".to_string()),
            sale.payment_amount,
            0.0, // Will be calculated
            "pending",
            now,
            now,
        ]
    ).map_err(|e| format!("Sale creation error: {}", e))?;

    let sale_id = conn.last_insert_rowid() as i32;

    // Return the created sale
    get_sale(pool, sale_id).await
}

#[tauri::command]
pub async fn get_sales(
    pool: State<'_, Arc<DbPool>>,
    limit: Option<i32>,
) -> Result<Vec<Sale>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

    let query = format!(
        "SELECT id, customer_id, total_amount, payment_method, payment_amount, change_amount, status, created_by, created_at, updated_at FROM sales ORDER BY created_at DESC{}",
        limit_clause
    );

    let sales = conn
        .prepare(&query)
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map([], |row| {
            Ok(Sale {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                total_amount: row.get(2)?,
                payment_method: row.get(3)?,
                payment_amount: row.get(4)?,
                change_amount: row.get(5)?,
                status: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(sales)
}

#[tauri::command]
pub async fn get_sale(pool: State<'_, Arc<DbPool>>, id: i32) -> Result<Sale, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let sale = conn.query_row(
        "SELECT id, customer_id, total_amount, payment_method, payment_amount, change_amount, status, created_by, created_at, updated_at FROM sales WHERE id = ?",
        [id],
        |row| {
            Ok(Sale {
                id: row.get(0)?,
                customer_id: row.get(1)?,
                total_amount: row.get(2)?,
                payment_method: row.get(3)?,
                payment_amount: row.get(4)?,
                change_amount: row.get(5)?,
                status: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        }
    ).map_err(|e| format!("Sale not found: {}", e))?;

    Ok(sale)
}

#[tauri::command]
pub async fn complete_sale(
    pool: State<'_, Arc<DbPool>>,
    sale_id: i32,
    payment_method: String,
) -> Result<Sale, String> {
    let mut conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    {
        let tx = conn
            .transaction()
            .map_err(|e| format!("Transaction begin error: {}", e))?;

        let subtotal: f64 = tx
            .query_row(
                "SELECT COALESCE(SUM(quantity * unit_price), 0) FROM sale_items WHERE sale_id = ?",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Total calculation error: {}", e))?;
        let total = subtotal;

        let payment_amount: f64 = tx
            .query_row(
                "SELECT payment_amount FROM sales WHERE id = ?",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Payment amount retrieval error: {}", e))?;

        if payment_amount < total {
            return Err("Payment amount is less than sale total".to_string());
        }

        let change_amount = payment_amount - total;
        let vat = total * 16.0 / 116.0;
        let cogs: f64 = tx
            .query_row(
                "SELECT COALESCE(SUM(si.quantity * COALESCE(p.cost_price, p.unit_price, 0)), 0)
             FROM sale_items si
             JOIN products p ON p.id = si.product_id
             WHERE si.sale_id = ?",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("COGS calculation error: {}", e))?;

        tx.execute(
            "UPDATE sales SET total_amount = ?, payment_method = ?, change_amount = ?, status = ?, updated_at = ? WHERE id = ?",
            params![total, payment_method.as_str(), change_amount, "completed", Utc::now().to_rfc3339(), sale_id]
        ).map_err(|e| format!("Sale completion error: {}", e))?;

        let sale_items: Vec<(i32, f64)> = {
            let mut stmt = tx
                .prepare("SELECT product_id, quantity FROM sale_items WHERE sale_id = ?")
                .map_err(|e| format!("Sale items query error: {}", e))?;
            let rows = stmt
                .query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| format!("Sale items mapping error: {}", e))?
                .collect::<Result<Vec<_>>>()
                .map_err(|e| format!("Sale items collection error: {}", e))?;
            rows
        };

        for (product_id, quantity) in sale_items {
            let current_stock: f64 = tx
                .query_row(
                    "SELECT quantity_in_stock FROM products WHERE id = ?",
                    [product_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Inventory lookup error: {}", e))?;

            if current_stock < quantity {
                return Err(format!("Insufficient stock for product {}", product_id));
            }

            tx.execute(
                "UPDATE products SET quantity_in_stock = quantity_in_stock - ?, updated_at = ? WHERE id = ?",
                params![quantity, Utc::now().to_rfc3339(), product_id]
            ).map_err(|e| format!("Inventory update error: {}", e))?;

            tx.execute(
                "INSERT INTO inventory_transactions (product_id, transaction_type, quantity, reference_id, notes, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                params![product_id, "sale", -quantity, sale_id, "Sale transaction", Utc::now().to_rfc3339()]
            ).map_err(|e| format!("Inventory transaction logging error: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Transaction commit error: {}", e))?;

        accounting::post_sale_accounting(&mut conn, sale_id, &payment_method, total, vat, cogs)?;
    }

    get_sale(pool, sale_id).await
}

#[tauri::command]
pub async fn add_sale_item(
    pool: State<'_, Arc<DbPool>>,
    sale_id: i32,
    item: NewSaleItem,
) -> Result<SaleItem, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // Get product price
    let unit_price: f64 = conn
        .query_row(
            "SELECT unit_price FROM products WHERE id = ?",
            [item.product_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Product price retrieval error: {}", e))?;

    conn.execute(
        "INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, barcode_scanned, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            sale_id,
            item.product_id,
            item.quantity,
            unit_price,
            item.barcode_scanned,
            Utc::now().to_rfc3339(),
        ]
    ).map_err(|e| format!("Sale item creation error: {}", e))?;

    let item_id = conn.last_insert_rowid() as i32;

    // Return the created sale item
    let sale_item = conn.query_row(
        "SELECT id, sale_id, product_id, quantity, unit_price, barcode_scanned, created_at FROM sale_items WHERE id = ?",
        [item_id],
        |row| {
            Ok(SaleItem {
                id: row.get(0)?,
                sale_id: row.get(1)?,
                product_id: row.get(2)?,
                quantity: row.get(3)?,
                unit_price: row.get(4)?,
                barcode_scanned: row.get(5)?,
                created_at: row.get(6)?,
            })
        }
    ).map_err(|e| format!("Sale item retrieval error: {}", e))?;

    Ok(sale_item)
}

#[tauri::command]
pub async fn create_sale_from_barcode_scan(
    pool: State<'_, Arc<DbPool>>,
    barcode: String,
    quantity: f64,
) -> Result<SaleItem, String> {
    if quantity <= 0.0 {
        return Err("Quantity must be greater than zero".to_string());
    }

    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let (product_id, unit_price, stock): (i32, f64, f64) = conn
        .query_row(
            "SELECT id, unit_price, quantity_in_stock FROM products WHERE barcode = ?",
            [&barcode],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Product not found for barcode: {}", e))?;

    if stock < quantity {
        return Err("Insufficient stock for barcode scan".to_string());
    }

    let now = Utc::now().to_rfc3339();
    let total = unit_price * quantity;

    conn.execute(
        "INSERT INTO sales (total_amount, payment_method, payment_amount, change_amount, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![total, "cash", total, 0.0, "pending", now, now]
    ).map_err(|e| format!("Sale creation error: {}", e))?;

    let sale_id = conn.last_insert_rowid() as i32;

    conn.execute(
        "INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, barcode_scanned, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![sale_id, product_id, quantity, unit_price, barcode, Utc::now().to_rfc3339()]
    ).map_err(|e| format!("Sale item creation error: {}", e))?;

    let item_id = conn.last_insert_rowid() as i32;

    conn.execute(
        "INSERT INTO barcode_scans (product_id, barcode, scan_time) VALUES (?, ?, ?)",
        params![product_id, barcode, Utc::now().to_rfc3339()],
    )
    .map_err(|e| format!("Barcode scan logging error: {}", e))?;

    let sale_item = conn.query_row(
        "SELECT id, sale_id, product_id, quantity, unit_price, barcode_scanned, created_at FROM sale_items WHERE id = ?",
        [item_id],
        |row| {
            Ok(SaleItem {
                id: row.get(0)?,
                sale_id: row.get(1)?,
                product_id: row.get(2)?,
                quantity: row.get(3)?,
                unit_price: row.get(4)?,
                barcode_scanned: row.get(5)?,
                created_at: row.get(6)?,
            })
        }
    ).map_err(|e| format!("Sale item retrieval error: {}", e))?;

    Ok(sale_item)
}

#[tauri::command]
pub async fn get_daily_sales_summary(
    pool: State<'_, Arc<DbPool>>,
) -> Result<serde_json::Value, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| "Failed to compute day start".to_string())?
        .to_string();

    let (sale_count, total_sales, items_sold): (i32, f64, f64) = conn.query_row(
        "SELECT COUNT(DISTINCT s.id), COALESCE(SUM(s.total_amount), 0), COALESCE(SUM(si.quantity), 0)
         FROM sales s
         LEFT JOIN sale_items si ON si.sale_id = s.id
         WHERE s.status = 'completed' AND s.created_at >= ?",
        [&today_start],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(|e| format!("Daily summary error: {}", e))?;

    Ok(serde_json::json!({
        "date": today_start,
        "sale_count": sale_count,
        "total_sales": total_sales,
        "items_sold": items_sold,
    }))
}
