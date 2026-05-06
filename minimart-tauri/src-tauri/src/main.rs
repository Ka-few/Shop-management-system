// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod models;
mod commands;

use std::sync::Arc;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database
            let db_path = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("minimart.db");

            let db_path_str = db_path.to_string_lossy().to_string();

            match db::init_db_pool(&db_path_str) {
                Ok(pool) => {
                    println!("Database initialized successfully at: {}", db_path_str);

                    // Initialize with default data if needed
                    if let Err(e) = db::initialize_database(&pool) {
                        eprintln!("Failed to initialize database: {}", e);
                    }

                    // Store database pool in app state
                    app.manage(Arc::new(pool));
                }
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                    std::process::exit(1);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            commands::auth::login,
            commands::auth::get_current_user,
            commands::auth::get_users,
            commands::auth::create_user,
            commands::auth::update_user,
            commands::auth::delete_user,

            // Product commands
            commands::products::get_products,
            commands::products::get_product,
            commands::products::create_product,
            commands::products::update_product,
            commands::products::delete_product,
            commands::products::get_product_by_barcode,

            // Category commands
            commands::categories::get_categories,
            commands::categories::create_category,

            // Sale commands
            commands::sales::create_sale,
            commands::sales::get_sales,
            commands::sales::get_sale,
            commands::sales::add_sale_item,
            commands::sales::create_sale_from_barcode_scan,
            commands::sales::complete_sale,
            commands::sales::get_daily_sales_summary,

            // Inventory commands
            commands::inventory::get_inventory_status,
            commands::inventory::adjust_inventory,
            commands::inventory::get_low_stock_items,
            commands::inventory::get_inventory_transactions,

            // Barcode commands
            commands::barcode::generate_barcode,
            commands::barcode::log_barcode_scan,
            commands::barcode::get_scan_history,

            // Dashboard commands
            commands::dashboard::get_dashboard_stats,
            commands::dashboard::get_recent_sales,
            commands::dashboard::get_top_products,
            commands::dashboard::get_sales_by_category,

            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_setting_value,

            // Database commands
            commands::database::backup_database_cmd,
            commands::database::get_database_stats,
            commands::database::validate_database,

            // Accounting commands
            commands::accounting::get_accounts,
            commands::accounting::create_account,
            commands::accounting::update_account,
            commands::accounting::delete_account,
            commands::accounting::get_journal_entries,
            commands::accounting::create_manual_journal_entry,
            commands::accounting::reverse_journal_entry,
            commands::accounting::get_profit_and_loss
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
