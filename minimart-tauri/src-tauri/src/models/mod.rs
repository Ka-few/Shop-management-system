use serde::{Deserialize, Serialize};

// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

// Category model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCategory {
    pub name: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

// Product model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub category_id: i32,
    pub sku: String,
    pub barcode: Option<String>,
    pub barcode_format: String,
    pub description: Option<String>,
    pub unit_price: f64,
    pub cost_price: Option<f64>,
    pub quantity_in_stock: i32,
    pub reorder_level: i32,
    pub expiry_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewProduct {
    pub name: String,
    pub category_id: i32,
    pub sku: String,
    pub barcode: Option<String>,
    pub barcode_format: Option<String>,
    pub description: Option<String>,
    pub unit_price: f64,
    pub cost_price: Option<f64>,
    pub quantity_in_stock: Option<i32>,
    pub reorder_level: Option<i32>,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub category_id: Option<i32>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub barcode_format: Option<String>,
    pub description: Option<String>,
    pub unit_price: Option<f64>,
    pub cost_price: Option<f64>,
    pub quantity_in_stock: Option<i32>,
    pub reorder_level: Option<i32>,
    pub expiry_date: Option<String>,
}

// Customer model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: i32,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewCustomer {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

// Sale models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    pub id: i32,
    pub customer_id: Option<i32>,
    pub total_amount: f64,
    pub payment_method: String,
    pub payment_amount: f64,
    pub change_amount: f64,
    pub status: String,
    pub created_by: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewSale {
    pub customer_id: Option<i32>,
    pub payment_method: Option<String>,
    pub payment_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    pub id: i32,
    pub sale_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: f64,
    pub barcode_scanned: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewSaleItem {
    pub product_id: i32,
    pub quantity: i32,
    pub barcode_scanned: Option<String>,
}

// Inventory models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: i32,
    pub product_id: i32,
    pub transaction_type: String,
    pub quantity: i32,
    pub reference_id: Option<i32>,
    pub notes: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewInventoryTransaction {
    pub product_id: i32,
    pub transaction_type: String,
    pub quantity: i32,
    pub reference_id: Option<i32>,
    pub notes: Option<String>,
}

// Barcode models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarcodeScan {
    pub id: i32,
    pub product_id: i32,
    pub barcode: String,
    pub scan_time: String,
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct BarcodeInfo {
    pub barcode: String,
    pub format: String,
    pub image_data: String, // Base64 encoded image
}

// Settings model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: i32,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSetting {
    pub value: String,
}

// Dashboard models
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_sales: f64,
    pub total_products: i32,
    pub low_stock_items: i32,
    pub todays_sales: f64,
    pub weekly_sales: f64,
    pub monthly_sales: f64,
}

#[derive(Debug, Serialize)]
pub struct InventoryStatus {
    pub product_id: i32,
    pub product_name: String,
    pub current_stock: i32,
    pub reorder_level: i32,
    pub status: String, // "in_stock", "low_stock", "out_of_stock"
}

// Report models
#[derive(Debug, Serialize)]
pub struct SalesReport {
    pub period: String,
    pub total_sales: f64,
    pub total_items: i32,
    pub average_sale: f64,
    pub top_products: Vec<ProductSales>,
}

#[derive(Debug, Serialize)]
pub struct ProductSales {
    pub product_id: i32,
    pub product_name: String,
    pub quantity_sold: i32,
    pub revenue: f64,
}

#[derive(Debug, Serialize)]
pub struct InventoryReport {
    pub total_products: i32,
    pub total_value: f64,
    pub low_stock_count: i32,
    pub out_of_stock_count: i32,
    pub items: Vec<InventoryItem>,
}

#[derive(Debug, Serialize)]
pub struct InventoryItem {
    pub product_id: i32,
    pub product_name: String,
    pub category: String,
    pub current_stock: i32,
    pub unit_cost: f64,
    pub total_value: f64,
}

// Accounting models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewAccount {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccount {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLine {
    pub id: Option<i32>,
    pub journal_entry_id: Option<i32>,
    pub account_id: i32,
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub account_type: Option<String>,
    pub debit: f64,
    pub credit: f64,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i32,
    pub date: String,
    pub reference: Option<String>,
    pub description: String,
    pub status: String,
    pub source_type: Option<String>,
    pub source_id: Option<i32>,
    pub reversed_entry_id: Option<i32>,
    pub is_system_generated: bool,
    pub total_debit: f64,
    pub total_credit: f64,
    pub lines: Vec<JournalLine>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NewJournalEntry {
    pub date: String,
    pub reference: Option<String>,
    pub description: String,
    pub lines: Vec<NewJournalLine>,
}

#[derive(Debug, Deserialize)]
pub struct NewJournalLine {
    pub account_id: i32,
    pub debit: f64,
    pub credit: f64,
    pub memo: Option<String>,
}
