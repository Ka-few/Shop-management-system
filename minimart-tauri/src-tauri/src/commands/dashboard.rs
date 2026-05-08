use crate::db::DbPool;
use crate::models::DashboardStats;
use chrono::{Duration, Utc};
use rusqlite::Result;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_dashboard_stats(pool: State<'_, Arc<DbPool>>) -> Result<DashboardStats, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let now = Utc::now();

    // Total sales
    let total_sales: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE status = 'completed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    // Total products
    let total_products: i32 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .unwrap_or(0);

    // Low stock items (below reorder level)
    let low_stock_items: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM products WHERE quantity_in_stock <= reorder_level",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Today's sales
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let todays_sales: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE status = 'completed' AND created_at >= ?",
        [today_start.to_string()],
        |row| row.get(0)
    ).unwrap_or(0.0);

    // Weekly sales (last 7 days)
    let week_ago = now - Duration::days(7);
    let weekly_sales: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE status = 'completed' AND created_at >= ?",
        [week_ago.to_string()],
        |row| row.get(0)
    ).unwrap_or(0.0);

    // Monthly sales (last 30 days)
    let month_ago = now - Duration::days(30);
    let monthly_sales: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE status = 'completed' AND created_at >= ?",
        [month_ago.to_string()],
        |row| row.get(0)
    ).unwrap_or(0.0);

    Ok(DashboardStats {
        total_sales,
        total_products,
        low_stock_items,
        todays_sales,
        weekly_sales,
        monthly_sales,
    })
}

#[tauri::command]
pub async fn get_recent_sales(
    pool: State<'_, Arc<DbPool>>,
    limit: Option<i32>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let limit_value = limit.unwrap_or(10);
    let query = format!(
        "SELECT s.id, s.total_amount, s.payment_method, s.created_at, COALESCE(c.name, 'Walk-in Customer') as customer_name FROM sales s LEFT JOIN customers c ON s.customer_id = c.id WHERE s.status = 'completed' ORDER BY s.created_at DESC LIMIT {}",
        limit_value
    );

    let sales = conn
        .prepare(&query)
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map([], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, i32>(0)?,
                "total_amount": row.get::<_, f64>(1)?,
                "payment_method": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
                "customer_name": row.get::<_, String>(4)?
            }))
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(sales)
}

#[tauri::command]
pub async fn get_top_products(
    pool: State<'_, Arc<DbPool>>,
    limit: Option<i32>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let limit_value = limit.unwrap_or(10);
    let query = format!(
        "SELECT p.name, SUM(si.quantity) as total_quantity, SUM(si.quantity * si.unit_price) as total_revenue FROM sale_items si JOIN products p ON si.product_id = p.id JOIN sales s ON si.sale_id = s.id WHERE s.status = 'completed' GROUP BY p.id, p.name ORDER BY total_revenue DESC LIMIT {}",
        limit_value
    );

    let products = conn
        .prepare(&query)
        .map_err(|e| format!("Query preparation error: {}", e))?
        .query_map([], |row| {
            Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "total_quantity": row.get::<_, f64>(1)?,
                "total_revenue": row.get::<_, f64>(2)?
            }))
        })
        .map_err(|e| format!("Query execution error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(products)
}

#[tauri::command]
pub async fn get_sales_by_category(
    pool: State<'_, Arc<DbPool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let categories = conn.prepare(
        "SELECT c.name, c.icon, COUNT(p.id) as product_count, COALESCE(SUM(si.quantity * si.unit_price), 0) as total_revenue FROM categories c LEFT JOIN products p ON c.id = p.category_id LEFT JOIN sale_items si ON p.id = si.product_id LEFT JOIN sales s ON si.sale_id = s.id AND s.status = 'completed' GROUP BY c.id, c.name, c.icon ORDER BY total_revenue DESC"
    )
    .map_err(|e| format!("Query preparation error: {}", e))?
    .query_map([], |row| {
        Ok(serde_json::json!({
            "name": row.get::<_, String>(0)?,
            "icon": row.get::<_, Option<String>>(1)?,
            "product_count": row.get::<_, i32>(2)?,
            "total_revenue": row.get::<_, f64>(3)?
        }))
    })
    .map_err(|e| format!("Query execution error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(categories)
}
