use crate::db::DbPool;
use crate::models::{LoginRequest, LoginResponse, NewUser, UpdateUser, User};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rusqlite::{params, OptionalExtension, Result};
use std::sync::Arc;
use tauri::State;

const JWT_SECRET: &str = "your-secret-key-change-this-in-production";
const ALLOWED_ROLES: [&str; 4] = ["admin", "user", "staff", "teller"];

fn map_user_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        username: row.get(1)?,
        email: row.get(2)?,
        role: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn normalize_role(role: Option<String>) -> Result<String, String> {
    let value = role
        .unwrap_or_else(|| "user".to_string())
        .trim()
        .to_lowercase();
    if ALLOWED_ROLES.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err("Role must be admin, user, staff, or teller".to_string())
    }
}

fn require_admin(conn: &rusqlite::Connection, admin_user_id: i32) -> Result<(), String> {
    let role: Option<String> = conn
        .query_row(
            "SELECT role FROM users WHERE id = ?",
            [admin_user_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Admin lookup failed: {}", e))?;

    match role.as_deref() {
        Some("admin") => Ok(()),
        Some(_) => Err("Admin access is required".to_string()),
        None => Err("Admin user not found".to_string()),
    }
}

#[tauri::command]
pub async fn login(
    pool: State<'_, Arc<DbPool>>,
    request: LoginRequest,
) -> Result<LoginResponse, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // Find user by username
    let user = conn.query_row(
        "SELECT id, username, email, role, created_at, updated_at FROM users WHERE username = ?",
        [&request.username],
        map_user_row
    ).map_err(|e| format!("User not found: {}", e))?;

    // Verify password
    let password_hash: String = conn
        .query_row(
            "SELECT password FROM users WHERE id = ?",
            [user.id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Password retrieval error: {}", e))?;

    let is_valid = verify(&request.password, &password_hash)
        .map_err(|e| format!("Password verification error: {}", e))?;

    if !is_valid {
        return Err("Invalid credentials".to_string());
    }

    // Generate JWT token
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = serde_json::json!({
        "sub": user.id.to_string(),
        "username": user.username,
        "role": user.role,
        "exp": expiration
    });

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|e| format!("Token generation error: {}", e))?;

    Ok(LoginResponse { token, user })
}

#[tauri::command]
pub async fn get_current_user(pool: State<'_, Arc<DbPool>>, user_id: i32) -> Result<User, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let user = conn
        .query_row(
            "SELECT id, username, email, role, created_at, updated_at FROM users WHERE id = ?",
            [user_id],
            map_user_row,
        )
        .map_err(|e| format!("User retrieval error: {}", e))?;

    Ok(user)
}

#[tauri::command]
pub async fn get_users(
    pool: State<'_, Arc<DbPool>>,
    admin_user_id: i32,
) -> Result<Vec<User>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;
    require_admin(&conn, admin_user_id)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, username, email, role, created_at, updated_at FROM users ORDER BY username",
        )
        .map_err(|e| format!("User query error: {}", e))?;

    let users = stmt
        .query_map([], map_user_row)
        .map_err(|e| format!("User query error: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("User row error: {}", e))?;

    Ok(users)
}

#[tauri::command]
pub async fn create_user(
    pool: State<'_, Arc<DbPool>>,
    admin_user_id: i32,
    user: NewUser,
) -> Result<User, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;
    require_admin(&conn, admin_user_id)?;

    let username = user.username.trim();
    let email = user.email.trim();
    if username.is_empty() {
        return Err("Username is required".to_string());
    }
    if email.is_empty() {
        return Err("Email is required".to_string());
    }
    if user.password.len() < 6 {
        return Err("Password must be at least 6 characters".to_string());
    }

    let role = normalize_role(user.role)?;
    let hashed_password =
        hash(user.password, DEFAULT_COST).map_err(|e| format!("Password hashing error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO users (username, password, email, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![username, hashed_password, email, role, now, now],
    ).map_err(|e| format!("User creation error: {}", e))?;

    let id = conn.last_insert_rowid() as i32;
    get_user_by_id(&conn, id)
}

#[tauri::command]
pub async fn update_user(
    pool: State<'_, Arc<DbPool>>,
    admin_user_id: i32,
    user_id: i32,
    updates: UpdateUser,
) -> Result<User, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;
    require_admin(&conn, admin_user_id)?;

    let existing = get_user_by_id(&conn, user_id)?;
    let username = updates
        .username
        .unwrap_or(existing.username)
        .trim()
        .to_string();
    let email = updates.email.unwrap_or(existing.email).trim().to_string();
    let role = normalize_role(updates.role.or(Some(existing.role)))?;

    if username.is_empty() {
        return Err("Username is required".to_string());
    }
    if email.is_empty() {
        return Err("Email is required".to_string());
    }
    if user_id == admin_user_id && role != "admin" {
        return Err("You cannot remove admin access from your own account".to_string());
    }
    let current_role: String = conn
        .query_row("SELECT role FROM users WHERE id = ?", [user_id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("User lookup error: {}", e))?;
    if current_role == "admin" && role != "admin" {
        let admin_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'admin'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Admin count error: {}", e))?;
        if admin_count <= 1 {
            return Err("At least one admin account is required".to_string());
        }
    }

    if let Some(password) = updates.password {
        if !password.trim().is_empty() {
            if password.len() < 6 {
                return Err("Password must be at least 6 characters".to_string());
            }
            let hashed_password = hash(password, DEFAULT_COST)
                .map_err(|e| format!("Password hashing error: {}", e))?;
            conn.execute(
                "UPDATE users SET username = ?, email = ?, role = ?, password = ?, updated_at = ? WHERE id = ?",
                params![username, email, role, hashed_password, Utc::now().to_rfc3339(), user_id],
            ).map_err(|e| format!("User update error: {}", e))?;
            return get_user_by_id(&conn, user_id);
        }
    }

    conn.execute(
        "UPDATE users SET username = ?, email = ?, role = ?, updated_at = ? WHERE id = ?",
        params![username, email, role, Utc::now().to_rfc3339(), user_id],
    )
    .map_err(|e| format!("User update error: {}", e))?;

    get_user_by_id(&conn, user_id)
}

#[tauri::command]
pub async fn delete_user(
    pool: State<'_, Arc<DbPool>>,
    admin_user_id: i32,
    user_id: i32,
) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;
    require_admin(&conn, admin_user_id)?;

    if admin_user_id == user_id {
        return Err("You cannot delete your own admin account".to_string());
    }

    let admin_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM users WHERE role = 'admin'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Admin count error: {}", e))?;
    let deleted_user_role: String = conn
        .query_row("SELECT role FROM users WHERE id = ?", [user_id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("User lookup error: {}", e))?;

    if deleted_user_role == "admin" && admin_count <= 1 {
        return Err("At least one admin account is required".to_string());
    }

    for query in [
        "UPDATE sales SET created_by = NULL WHERE created_by = ?",
        "UPDATE inventory_transactions SET created_by = NULL WHERE created_by = ?",
        "UPDATE barcode_scans SET user_id = NULL WHERE user_id = ?",
        "UPDATE purchase_orders SET created_by = NULL WHERE created_by = ?",
        "UPDATE journal_entries SET created_by = NULL WHERE created_by = ?",
    ] {
        conn.execute(query, [user_id])
            .map_err(|e| format!("User reference cleanup error: {}", e))?;
    }

    conn.execute("DELETE FROM users WHERE id = ?", [user_id])
        .map_err(|e| format!("User delete error: {}", e))?;

    Ok(())
}

fn get_user_by_id(conn: &rusqlite::Connection, user_id: i32) -> Result<User, String> {
    conn.query_row(
        "SELECT id, username, email, role, created_at, updated_at FROM users WHERE id = ?",
        [user_id],
        map_user_row,
    )
    .map_err(|e| format!("User retrieval error: {}", e))
}
