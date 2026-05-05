use crate::db::DbPool;
use crate::models::BarcodeScan;
use rusqlite::Result;
use tauri::State;
use std::sync::Arc;
use chrono::Utc;
// Removed: use barcode::{Barcode, BarcodeFormat, Symbology};

#[tauri::command]
pub async fn generate_barcode(
    pool: State<'_, Arc<DbPool>>,
    product_id: i32,
    format: String,
) -> Result<String, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // Get product info
    let (sku, current_barcode): (String, Option<String>) = conn.query_row(
        "SELECT sku, barcode FROM products WHERE id = ?",
        [product_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).map_err(|e| format!("Product not found: {}", e))?;

    // Use existing barcode or generate new one
    let barcode_data = if current_barcode.is_some() {
        current_barcode.clone().unwrap()
    } else {
        // Generate barcode based on SKU
        generate_barcode_from_sku(&sku, &format)?
    };

    // Generate barcode image (simplified - in real app you'd use a proper barcode library)
    let barcode_image = generate_barcode_image(&barcode_data, &format)?;

    // Update product with barcode if it didn't have one
    if current_barcode.is_none() {
        conn.execute(
            "UPDATE products SET barcode = ?, barcode_format = ? WHERE id = ?",
            [&barcode_data, &format, &product_id.to_string()]
        ).map_err(|e| format!("Barcode update error: {}", e))?;
    }

    Ok(barcode_image)
}

#[tauri::command]
pub async fn log_barcode_scan(
    pool: State<'_, Arc<DbPool>>,
    product_id: i32,
    barcode: String,
) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    conn.execute(
        "INSERT INTO barcode_scans (product_id, barcode, scan_time) VALUES (?, ?, ?)",
        [
            &product_id.to_string(),
            &barcode,
            &Utc::now().to_rfc3339(),
        ]
    ).map_err(|e| format!("Barcode scan logging error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_scan_history(
    pool: State<'_, Arc<DbPool>>,
    product_id: i32,
    limit: Option<i32>,
) -> Result<Vec<BarcodeScan>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let limit_value = limit.unwrap_or(50).clamp(1, 500);

    let scans = conn.prepare(
        "SELECT id, product_id, barcode, scan_time, user_id FROM barcode_scans WHERE product_id = ? ORDER BY scan_time DESC LIMIT ?"
    )
    .map_err(|e| format!("Query preparation error: {}", e))?
    .query_map(rusqlite::params![product_id, limit_value], |row| {
        Ok(BarcodeScan {
            id: row.get(0)?,
            product_id: row.get(1)?,
            barcode: row.get(2)?,
            scan_time: row.get(3)?,
            user_id: row.get(4)?,
        })
    })
    .map_err(|e| format!("Query execution error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Result collection error: {}", e))?;

    Ok(scans)
}

fn generate_barcode_from_sku(sku: &str, format: &str) -> Result<String, String> {
    match format {
        "EAN13" => {
            // Generate a valid EAN13 barcode
            // For simplicity, we'll use the SKU as base and pad/calculate check digit
            let base = format!("{:0>12}", sku.chars().filter(|c| c.is_numeric()).collect::<String>());
            let check_digit = calculate_ean13_check_digit(&base)?;
            Ok(format!("{}{}", &base[..12], check_digit))
        },
        "EAN8" => {
            // Generate EAN8 barcode
            let base = format!("{:0>7}", sku.chars().filter(|c| c.is_numeric()).collect::<String>());
            let check_digit = calculate_ean8_check_digit(&base)?;
            Ok(format!("{}{}", &base[..7], check_digit))
        },
        "CODE128" => {
            // For CODE128, we can use the SKU directly
            Ok(sku.to_string())
        },
        _ => Err(format!("Unsupported barcode format: {}", format))
    }
}

fn calculate_ean13_check_digit(digits: &str) -> Result<char, String> {
    if digits.len() != 12 {
        return Err("EAN13 requires 12 digits".to_string());
    }

    let mut sum = 0;
    for (i, c) in digits.chars().enumerate() {
        let digit = c.to_digit(10).ok_or("Invalid digit")?;
        if i % 2 == 0 {
            sum += digit * 1;
        } else {
            sum += digit * 3;
        }
    }

    let check_digit = (10 - (sum % 10)) % 10;
    Ok(char::from_digit(check_digit, 10).unwrap())
}

fn calculate_ean8_check_digit(digits: &str) -> Result<char, String> {
    if digits.len() != 7 {
        return Err("EAN8 requires 7 digits".to_string());
    }

    let mut sum = 0;
    for (i, c) in digits.chars().enumerate() {
        let digit = c.to_digit(10).ok_or("Invalid digit")?;
        if i % 2 == 0 {
            sum += digit * 3;
        } else {
            sum += digit * 1;
        }
    }

    let check_digit = (10 - (sum % 10)) % 10;
    Ok(char::from_digit(check_digit, 10).unwrap())
}

fn generate_barcode_image(barcode_data: &str, format: &str) -> Result<String, String> {
    // In a real implementation, you'd use a barcode generation library
    // For now, we'll return a placeholder base64 encoded image
    // You would typically use libraries like `barcode` crate or external services

    // Placeholder implementation - return a simple SVG as base64
    let svg_content = format!(
        r#"<svg width="200" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="200" height="100" fill="white"/>
            <text x="100" y="30" text-anchor="middle" font-family="monospace" font-size="12">{}</text>
            <text x="100" y="50" text-anchor="middle" font-family="monospace" font-size="10">Format: {}</text>
            <text x="100" y="80" text-anchor="middle" font-family="monospace" font-size="8">Barcode Placeholder</text>
        </svg>"#,
        barcode_data, format
    );

    // Convert to base64 (simplified)
    use base64::{Engine as _, engine::general_purpose};
    let base64_data = general_purpose::STANDARD.encode(svg_content.as_bytes());
    Ok(format!("data:image/svg+xml;base64,{}", base64_data))
}
