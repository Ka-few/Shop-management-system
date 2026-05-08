use crate::db::DbPool;
use crate::models::{
    Account, JournalEntry, JournalLine, NewAccount, NewJournalEntry, NewJournalLine, UpdateAccount,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use std::sync::Arc;
use tauri::State;

const ACCOUNT_TYPES: [&str; 5] = ["Asset", "Liability", "Equity", "Revenue", "Expense"];
const EPSILON: f64 = 0.005;

#[tauri::command]
pub async fn get_accounts(pool: State<'_, Arc<DbPool>>) -> Result<Vec<Account>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;
    fetch_accounts(&conn)
}

#[tauri::command]
pub async fn create_account(
    pool: State<'_, Arc<DbPool>>,
    account: NewAccount,
) -> Result<Account, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    validate_account_type(&account.account_type)?;
    validate_parent(&conn, None, account.parent_id, &account.account_type)?;
    validate_numeric_code(&account.code)?;

    conn.execute(
        "INSERT INTO accounts (code, name, type, parent_id, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, 1, ?, ?)",
        params![
            account.code.trim(),
            account.name.trim(),
            account.account_type,
            account.parent_id,
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339(),
        ],
    ).map_err(|e| format!("Account creation error: {}", e))?;

    get_account_by_id(&conn, conn.last_insert_rowid() as i32)
}

#[tauri::command]
pub async fn update_account(
    pool: State<'_, Arc<DbPool>>,
    id: i32,
    account: UpdateAccount,
) -> Result<Account, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    validate_account_type(&account.account_type)?;
    validate_parent(&conn, Some(id), account.parent_id, &account.account_type)?;
    validate_numeric_code(&account.code)?;

    conn.execute(
        "UPDATE accounts SET code = ?, name = ?, type = ?, parent_id = ?, is_active = ?, updated_at = ? WHERE id = ?",
        params![
            account.code.trim(),
            account.name.trim(),
            account.account_type,
            account.parent_id,
            if account.is_active { 1 } else { 0 },
            Utc::now().to_rfc3339(),
            id,
        ],
    ).map_err(|e| format!("Account update error: {}", e))?;

    get_account_by_id(&conn, id)
}

#[tauri::command]
pub async fn delete_account(pool: State<'_, Arc<DbPool>>, id: i32) -> Result<(), String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let used: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_lines WHERE account_id = ?",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Journal usage check error: {}", e))?;
    if used > 0 {
        return Err("Account is used in journal entries and cannot be deleted.".to_string());
    }

    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM accounts WHERE parent_id = ? AND is_active = 1",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Child account check error: {}", e))?;
    if child_count > 0 {
        return Err("Account has active child accounts and cannot be deleted.".to_string());
    }

    conn.execute(
        "UPDATE accounts SET is_active = 0, updated_at = ? WHERE id = ?",
        params![Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| format!("Account delete error: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_journal_entries(
    pool: State<'_, Arc<DbPool>>,
    date_from: Option<String>,
    date_to: Option<String>,
    account_id: Option<i32>,
    reference: Option<String>,
    search: Option<String>,
) -> Result<Vec<JournalEntry>, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let mut entries =
        fetch_journal_entries(&conn, date_from, date_to, account_id, reference, search)?;
    for entry in &mut entries {
        entry.lines = fetch_journal_lines(&conn, entry.id)?;
    }
    Ok(entries)
}

#[tauri::command]
pub async fn create_manual_journal_entry(
    pool: State<'_, Arc<DbPool>>,
    entry: NewJournalEntry,
) -> Result<JournalEntry, String> {
    let mut conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let id = create_journal_entry_tx(
        &mut conn,
        &entry.date,
        entry.reference.as_deref(),
        &entry.description,
        "manual",
        None,
        false,
        &entry.lines,
    )?;
    fetch_journal_entry(&conn, id)
}

#[tauri::command]
pub async fn reverse_journal_entry(
    pool: State<'_, Arc<DbPool>>,
    id: i32,
    description: Option<String>,
) -> Result<JournalEntry, String> {
    let mut conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let existing = fetch_journal_entry(&conn, id)?;
    if existing.status == "reversed" {
        return Err("Journal entry is already reversed.".to_string());
    }

    let reversal_lines = existing
        .lines
        .iter()
        .map(|line| NewJournalLine {
            account_id: line.account_id,
            debit: line.credit,
            credit: line.debit,
            memo: Some(format!("Reversal of journal entry #{}", id)),
        })
        .collect::<Vec<_>>();

    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin error: {}", e))?;
    tx.execute(
        "UPDATE journal_entries SET status = 'reversed', updated_at = ? WHERE id = ?",
        params![Utc::now().to_rfc3339(), id],
    )
    .map_err(|e| format!("Journal reversal update error: {}", e))?;

    let reversal_id = insert_journal_entry(
        &tx,
        &Utc::now().date_naive().to_string(),
        existing.reference.as_deref(),
        description.as_deref().unwrap_or("Reversal entry"),
        existing.source_type.as_deref().unwrap_or("manual"),
        existing.source_id,
        existing.is_system_generated,
        Some(id),
        &reversal_lines,
    )?;
    tx.commit()
        .map_err(|e| format!("Transaction commit error: {}", e))?;

    fetch_journal_entry(&conn, reversal_id)
}

#[tauri::command]
pub async fn get_profit_and_loss(
    pool: State<'_, Arc<DbPool>>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<serde_json::Value, String> {
    let conn = crate::db::get_connection(&pool)
        .map_err(|e| format!("Database connection error: {}", e))?;

    let date_from = date_from.unwrap_or_else(|| "0000-01-01".to_string());
    let date_to = date_to.unwrap_or_else(|| "9999-12-31".to_string());

    let rows = conn.prepare(
        "SELECT a.id, a.code, a.name, a.type, COALESCE(SUM(jl.credit - jl.debit), 0) AS revenue_balance,
                COALESCE(SUM(jl.debit - jl.credit), 0) AS expense_balance
         FROM accounts a
         JOIN journal_lines jl ON jl.account_id = a.id
         JOIN journal_entries je ON je.id = jl.journal_entry_id
         WHERE je.status = 'posted'
           AND je.date >= ?
           AND je.date <= ?
           AND a.type IN ('Revenue', 'Expense')
         GROUP BY a.id, a.code, a.name, a.type
         ORDER BY a.code"
    ).map_err(|e| format!("P&L query preparation error: {}", e))?
    .query_map(params![date_from, date_to], |row| {
        let account_type: String = row.get(3)?;
        let amount: f64 = if account_type == "Revenue" { row.get(4)? } else { row.get(5)? };
        Ok(serde_json::json!({
            "account_id": row.get::<_, i32>(0)?,
            "code": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "type": account_type,
            "amount": amount,
        }))
    }).map_err(|e| format!("P&L query error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("P&L result error: {}", e))?;

    let mut revenue = Vec::new();
    let mut cogs = Vec::new();
    let mut expenses = Vec::new();
    let mut total_revenue = 0.0;
    let mut total_cogs = 0.0;
    let mut total_expenses = 0.0;

    for row in rows {
        let amount = row["amount"].as_f64().unwrap_or(0.0);
        let name = row["name"].as_str().unwrap_or("");
        let code = row["code"].as_str().unwrap_or("");
        if row["type"] == "Revenue" {
            total_revenue += amount;
            revenue.push(row);
        } else if code == "5100" || name.to_lowercase().contains("cost of goods") {
            total_cogs += amount;
            cogs.push(row);
        } else {
            total_expenses += amount;
            expenses.push(row);
        }
    }

    Ok(serde_json::json!({
        "revenue": revenue,
        "cogs": cogs,
        "expenses": expenses,
        "total_revenue": total_revenue,
        "total_cogs": total_cogs,
        "gross_profit": total_revenue - total_cogs,
        "total_expenses": total_expenses,
        "net_profit": total_revenue - total_cogs - total_expenses,
    }))
}

pub fn post_sale_accounting(
    conn: &mut Connection,
    sale_id: i32,
    payment_method: &str,
    total: f64,
    vat: f64,
    cogs: f64,
) -> Result<(), String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE source_type = 'sale' AND source_id = ?",
            [sale_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Sale journal lookup error: {}", e))?;
    if existing > 0 {
        return Ok(());
    }

    let payment_account = match payment_method {
        "mpesa" => account_id_by_code(conn, "1110")?,
        "card" => account_id_by_code(conn, "1120")?,
        _ => account_id_by_code(conn, "1100")?,
    };
    let revenue_account = account_id_by_code(conn, "4100")?;
    let vat_account = account_id_by_code(conn, "2100")?;
    let inventory_account = account_id_by_code(conn, "1200")?;
    let cogs_account = account_id_by_code(conn, "5100")?;
    let net_revenue = (total - vat).max(0.0);

    let mut lines = vec![
        NewJournalLine {
            account_id: payment_account,
            debit: total,
            credit: 0.0,
            memo: Some("Customer payment".to_string()),
        },
        NewJournalLine {
            account_id: revenue_account,
            debit: 0.0,
            credit: net_revenue,
            memo: Some("Sale revenue net of VAT".to_string()),
        },
        NewJournalLine {
            account_id: vat_account,
            debit: 0.0,
            credit: vat,
            memo: Some("VAT payable".to_string()),
        },
    ];

    if cogs > EPSILON {
        lines.push(NewJournalLine {
            account_id: cogs_account,
            debit: cogs,
            credit: 0.0,
            memo: Some("Cost of goods sold".to_string()),
        });
        lines.push(NewJournalLine {
            account_id: inventory_account,
            debit: 0.0,
            credit: cogs,
            memo: Some("Inventory relieved".to_string()),
        });
    }

    create_journal_entry_tx(
        conn,
        &Utc::now().date_naive().to_string(),
        Some(&format!("sale:{}", sale_id)),
        &format!("POS sale #{}", sale_id),
        "sale",
        Some(sale_id),
        true,
        &lines,
    )?;

    Ok(())
}

pub fn post_inventory_adjustment_accounting(
    conn: &mut Connection,
    transaction_id: i32,
    product_id: i32,
    quantity: f64,
    reason: &str,
) -> Result<(), String> {
    if quantity.abs() <= EPSILON {
        return Ok(());
    }

    let unit_cost: f64 = conn
        .query_row(
            "SELECT COALESCE(cost_price, unit_price, 0) FROM products WHERE id = ?",
            [product_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Product cost lookup error: {}", e))?;
    let value = (unit_cost * quantity.abs()).max(0.0);
    if value <= EPSILON {
        return Ok(());
    }

    let inventory_account = account_id_by_code(conn, "1200")?;
    let adjustment_account = account_id_by_code(conn, "5200")?;
    let lines = if quantity > 0.0 {
        vec![
            NewJournalLine {
                account_id: inventory_account,
                debit: value,
                credit: 0.0,
                memo: Some(reason.to_string()),
            },
            NewJournalLine {
                account_id: adjustment_account,
                debit: 0.0,
                credit: value,
                memo: Some("Inventory gain offset".to_string()),
            },
        ]
    } else {
        vec![
            NewJournalLine {
                account_id: adjustment_account,
                debit: value,
                credit: 0.0,
                memo: Some(reason.to_string()),
            },
            NewJournalLine {
                account_id: inventory_account,
                debit: 0.0,
                credit: value,
                memo: Some("Inventory reduction".to_string()),
            },
        ]
    };

    create_journal_entry_tx(
        conn,
        &Utc::now().date_naive().to_string(),
        Some(&format!("inventory:{}", transaction_id)),
        &format!("Inventory adjustment for product #{}", product_id),
        "inventory",
        Some(transaction_id),
        true,
        &lines,
    )?;

    Ok(())
}

fn fetch_accounts(conn: &Connection) -> Result<Vec<Account>, String> {
    conn.prepare(
        "SELECT id, code, name, type, parent_id, is_active, created_at, updated_at FROM accounts ORDER BY code"
    ).map_err(|e| format!("Accounts query preparation error: {}", e))?
    .query_map([], account_from_row)
    .map_err(|e| format!("Accounts query error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Accounts result error: {}", e))
}

fn get_account_by_id(conn: &Connection, id: i32) -> Result<Account, String> {
    conn.query_row(
        "SELECT id, code, name, type, parent_id, is_active, created_at, updated_at FROM accounts WHERE id = ?",
        [id],
        account_from_row,
    ).map_err(|e| format!("Account lookup error: {}", e))
}

fn account_from_row(row: &rusqlite::Row<'_>) -> Result<Account> {
    let is_active: i32 = row.get(5)?;
    Ok(Account {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        account_type: row.get(3)?,
        parent_id: row.get(4)?,
        is_active: is_active == 1,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn validate_numeric_code(code: &str) -> Result<(), String> {
    if code.trim().is_empty() || !code.chars().all(|char| char.is_ascii_digit()) {
        return Err("Account code must be numeric.".to_string());
    }
    Ok(())
}

fn validate_account_type(account_type: &str) -> Result<(), String> {
    if !ACCOUNT_TYPES.contains(&account_type) {
        return Err("Invalid account type.".to_string());
    }
    Ok(())
}

fn validate_parent(
    conn: &Connection,
    account_id: Option<i32>,
    parent_id: Option<i32>,
    account_type: &str,
) -> Result<(), String> {
    if parent_id == account_id {
        return Err("An account cannot be its own parent.".to_string());
    }

    if let Some(parent_id) = parent_id {
        let parent_type: String = conn
            .query_row(
                "SELECT type FROM accounts WHERE id = ? AND is_active = 1",
                [parent_id],
                |row| row.get(0),
            )
            .map_err(|_| "Parent account must be active and valid.".to_string())?;
        if parent_type != account_type {
            return Err("Parent account must have the same account type.".to_string());
        }

        let mut current = Some(parent_id);
        while let Some(id) = current {
            if Some(id) == account_id {
                return Err("Parent selection would create a cycle.".to_string());
            }
            current = conn
                .query_row("SELECT parent_id FROM accounts WHERE id = ?", [id], |row| {
                    row.get(0)
                })
                .optional()
                .map_err(|e| format!("Parent validation error: {}", e))?
                .flatten();
        }
    }

    Ok(())
}

fn validate_journal_lines(
    conn: &Connection,
    lines: &[NewJournalLine],
) -> Result<(f64, f64), String> {
    if lines.len() < 2 {
        return Err("At least two journal lines are required.".to_string());
    }

    let mut total_debit = 0.0;
    let mut total_credit = 0.0;
    for line in lines {
        if line.debit < 0.0 || line.credit < 0.0 {
            return Err("Debits and credits cannot be negative.".to_string());
        }
        if line.debit > EPSILON && line.credit > EPSILON {
            return Err("A journal line cannot contain both debit and credit.".to_string());
        }
        if line.debit <= EPSILON && line.credit <= EPSILON {
            return Err("Each journal line must contain a debit or a credit.".to_string());
        }
        let active: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM accounts WHERE id = ? AND is_active = 1",
                [line.account_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Account validation error: {}", e))?;
        if active == 0 {
            return Err("Journal lines must use active accounts.".to_string());
        }
        total_debit += line.debit;
        total_credit += line.credit;
    }

    if (total_debit - total_credit).abs() > EPSILON {
        return Err("Total debit must equal total credit.".to_string());
    }

    Ok((total_debit, total_credit))
}

fn create_journal_entry_tx(
    conn: &mut Connection,
    date: &str,
    reference: Option<&str>,
    description: &str,
    source_type: &str,
    source_id: Option<i32>,
    is_system_generated: bool,
    lines: &[NewJournalLine],
) -> Result<i32, String> {
    validate_journal_lines(conn, lines)?;
    let tx = conn
        .transaction()
        .map_err(|e| format!("Transaction begin error: {}", e))?;
    let id = insert_journal_entry(
        &tx,
        date,
        reference,
        description,
        source_type,
        source_id,
        is_system_generated,
        None,
        lines,
    )?;
    tx.commit()
        .map_err(|e| format!("Transaction commit error: {}", e))?;
    Ok(id)
}

fn insert_journal_entry(
    conn: &Connection,
    date: &str,
    reference: Option<&str>,
    description: &str,
    source_type: &str,
    source_id: Option<i32>,
    is_system_generated: bool,
    reversed_entry_id: Option<i32>,
    lines: &[NewJournalLine],
) -> Result<i32, String> {
    validate_journal_lines(conn, lines)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO journal_entries (date, reference, description, status, source_type, source_id, reversed_entry_id, is_system_generated, created_at, updated_at)
         VALUES (?, ?, ?, 'posted', ?, ?, ?, ?, ?, ?)",
        params![
            date,
            reference,
            description,
            source_type,
            source_id,
            reversed_entry_id,
            if is_system_generated { 1 } else { 0 },
            now,
            now,
        ],
    ).map_err(|e| format!("Journal entry creation error: {}", e))?;

    let entry_id = conn.last_insert_rowid() as i32;
    for line in lines {
        conn.execute(
            "INSERT INTO journal_lines (journal_entry_id, account_id, debit, credit, memo, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![entry_id, line.account_id, line.debit, line.credit, line.memo, Utc::now().to_rfc3339()],
        ).map_err(|e| format!("Journal line creation error: {}", e))?;
    }

    Ok(entry_id)
}

fn fetch_journal_entries(
    conn: &Connection,
    date_from: Option<String>,
    date_to: Option<String>,
    account_id: Option<i32>,
    reference: Option<String>,
    search: Option<String>,
) -> Result<Vec<JournalEntry>, String> {
    let mut conditions = vec!["1 = 1".to_string()];
    let mut params_vec = Vec::<String>::new();

    if let Some(value) = date_from {
        conditions.push("je.date >= ?".to_string());
        params_vec.push(value);
    }
    if let Some(value) = date_to {
        conditions.push("je.date <= ?".to_string());
        params_vec.push(value);
    }
    if let Some(value) = account_id {
        conditions.push("EXISTS (SELECT 1 FROM journal_lines fl WHERE fl.journal_entry_id = je.id AND fl.account_id = ?)".to_string());
        params_vec.push(value.to_string());
    }
    if let Some(value) = reference {
        if !value.trim().is_empty() {
            conditions.push("COALESCE(je.reference, '') LIKE ?".to_string());
            params_vec.push(format!("%{}%", value.trim()));
        }
    }
    if let Some(value) = search {
        if !value.trim().is_empty() {
            conditions
                .push("(je.description LIKE ? OR COALESCE(je.reference, '') LIKE ?)".to_string());
            params_vec.push(format!("%{}%", value.trim()));
            params_vec.push(format!("%{}%", value.trim()));
        }
    }

    let query = format!(
        "SELECT je.id, je.date, je.reference, je.description, je.status, je.source_type, je.source_id,
                je.reversed_entry_id, je.is_system_generated, COALESCE(SUM(jl.debit), 0), COALESCE(SUM(jl.credit), 0),
                je.created_at, je.updated_at
         FROM journal_entries je
         LEFT JOIN journal_lines jl ON jl.journal_entry_id = je.id
         WHERE {}
         GROUP BY je.id
         ORDER BY je.date DESC, je.id DESC
         LIMIT 500",
        conditions.join(" AND ")
    );
    let refs = params_vec
        .iter()
        .map(|value| value.as_str())
        .collect::<Vec<_>>();

    conn.prepare(&query)
        .map_err(|e| format!("Journal query preparation error: {}", e))?
        .query_map(rusqlite::params_from_iter(refs), journal_entry_from_row)
        .map_err(|e| format!("Journal query error: {}", e))?
        .collect::<Result<Vec<_>>>()
        .map_err(|e| format!("Journal result error: {}", e))
}

fn fetch_journal_entry(conn: &Connection, id: i32) -> Result<JournalEntry, String> {
    let mut entry = conn.query_row(
        "SELECT je.id, je.date, je.reference, je.description, je.status, je.source_type, je.source_id,
                je.reversed_entry_id, je.is_system_generated, COALESCE(SUM(jl.debit), 0), COALESCE(SUM(jl.credit), 0),
                je.created_at, je.updated_at
         FROM journal_entries je
         LEFT JOIN journal_lines jl ON jl.journal_entry_id = je.id
         WHERE je.id = ?
         GROUP BY je.id",
        [id],
        journal_entry_from_row,
    ).map_err(|e| format!("Journal lookup error: {}", e))?;
    entry.lines = fetch_journal_lines(conn, id)?;
    Ok(entry)
}

fn journal_entry_from_row(row: &rusqlite::Row<'_>) -> Result<JournalEntry> {
    let is_system_generated: i32 = row.get(8)?;
    Ok(JournalEntry {
        id: row.get(0)?,
        date: row.get(1)?,
        reference: row.get(2)?,
        description: row.get(3)?,
        status: row.get(4)?,
        source_type: row.get(5)?,
        source_id: row.get(6)?,
        reversed_entry_id: row.get(7)?,
        is_system_generated: is_system_generated == 1,
        total_debit: row.get(9)?,
        total_credit: row.get(10)?,
        lines: Vec::new(),
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn fetch_journal_lines(conn: &Connection, entry_id: i32) -> Result<Vec<JournalLine>, String> {
    conn.prepare(
        "SELECT jl.id, jl.journal_entry_id, jl.account_id, a.code, a.name, a.type, jl.debit, jl.credit, jl.memo
         FROM journal_lines jl
         JOIN accounts a ON a.id = jl.account_id
         WHERE jl.journal_entry_id = ?
         ORDER BY jl.id"
    ).map_err(|e| format!("Journal line query preparation error: {}", e))?
    .query_map([entry_id], |row| {
        Ok(JournalLine {
            id: row.get(0)?,
            journal_entry_id: row.get(1)?,
            account_id: row.get(2)?,
            account_code: row.get(3)?,
            account_name: row.get(4)?,
            account_type: row.get(5)?,
            debit: row.get(6)?,
            credit: row.get(7)?,
            memo: row.get(8)?,
        })
    }).map_err(|e| format!("Journal line query error: {}", e))?
    .collect::<Result<Vec<_>>>()
    .map_err(|e| format!("Journal line result error: {}", e))
}

fn account_id_by_code(conn: &Connection, code: &str) -> Result<i32, String> {
    conn.query_row(
        "SELECT id FROM accounts WHERE code = ? AND is_active = 1",
        [code],
        |row| row.get(0),
    )
    .map_err(|_| {
        format!(
            "Required accounting account {} is missing or inactive.",
            code
        )
    })
}
