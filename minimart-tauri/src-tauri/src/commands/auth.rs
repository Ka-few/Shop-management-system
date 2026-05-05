use crate::db::DbPool;
use crate::models::{LoginRequest, LoginResponse, User};
use rusqlite::Result;
use tauri::State;
use std::sync::Arc;
use jsonwebtoken::{encode, Header, EncodingKey};
use bcrypt::verify;
use chrono::{Utc, Duration};

const JWT_SECRET: &str = "your-secret-key-change-this-in-production";

#[tauri::command]
pub async fn login(
    pool: State<'_, Arc<DbPool>>,
    request: LoginRequest,
) -> Result<LoginResponse, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    // Find user by username
    let user = conn.query_row(
        "SELECT id, username, email, role, created_at FROM users WHERE username = ?",
        [&request.username],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: None,
            })
        }
    ).map_err(|e| format!("User not found: {}", e))?;

    // Verify password
    let password_hash: String = conn.query_row(
        "SELECT password FROM users WHERE id = ?",
        [user.id],
        |row| row.get(0)
    ).map_err(|e| format!("Password retrieval error: {}", e))?;

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
        &EncodingKey::from_secret(JWT_SECRET.as_ref())
    ).map_err(|e| format!("Token generation error: {}", e))?;

    Ok(LoginResponse { token, user })
}

#[tauri::command]
pub async fn get_current_user(
    pool: State<'_, Arc<DbPool>>,
    user_id: i32,
) -> Result<User, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let user = conn.query_row(
        "SELECT id, username, email, role, created_at FROM users WHERE id = ?",
        [user_id],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                role: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: None,
            })
        }
    ).map_err(|e| format!("User retrieval error: {}", e))?;

    Ok(user)
}