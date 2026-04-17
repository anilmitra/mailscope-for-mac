use anyhow::{Context, Result};
use mailscope_core::models::{ConnectedAccount, SeedGroup, SeedGroupMember};

use crate::db::Database;

// ── Connected accounts ────────────────────────────────────────────────────────

pub fn list_accounts(db: &Database) -> Result<Vec<ConnectedAccount>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, provider, display_name, seed_email, auth_type, keychain_ref, status,
                last_seen_at, created_at FROM connected_accounts ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], map_account)?;
    rows.collect::<Result<Vec<_>, _>>().context("list_accounts")
}

pub fn get_account(db: &Database, id: i64) -> Result<ConnectedAccount> {
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, provider, display_name, seed_email, auth_type, keychain_ref, status,
                last_seen_at, created_at FROM connected_accounts WHERE id = ?1",
        [id],
        map_account,
    )
    .context("get_account")
}

pub fn add_imap_account(
    db: &Database,
    display_name: &str,
    seed_email: &str,
    host: &str,
    port: u16,
    auth_type: &str,
    password: &str,
) -> Result<ConnectedAccount> {
    let conn = db.conn()?;

    // Store password reference (in production this would go to macOS Keychain)
    let keychain_ref = format!("imap:{}:{}", seed_email, host);
    // TODO: actually store `password` in macOS Keychain via keychain_ref

    conn.execute(
        "INSERT INTO connected_accounts (provider, display_name, seed_email, auth_type, keychain_ref, status)
         VALUES ('imap', ?1, ?2, ?3, ?4, 'active')",
        rusqlite::params![display_name, seed_email, auth_type, keychain_ref],
    )?;
    let id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO imap_configs (connected_account_id, host, port) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, host, port],
    )?;

    // Suppress unused var warning until keychain integration is added
    let _ = password;

    get_account(db, id)
}

pub fn add_oauth_account(
    db: &Database,
    provider: &str,
    display_name: &str,
    seed_email: &str,
    auth_type: &str,
    refresh_token: &str,
) -> Result<ConnectedAccount> {
    let conn = db.conn()?;
    let keychain_ref = format!("oauth:{}:{}", provider, seed_email);
    // TODO: store refresh_token in macOS Keychain
    let _ = refresh_token;

    conn.execute(
        "INSERT INTO connected_accounts (provider, display_name, seed_email, auth_type, keychain_ref, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'active')",
        rusqlite::params![provider, display_name, seed_email, auth_type, keychain_ref],
    )?;
    let id = conn.last_insert_rowid();
    get_account(db, id)
}

pub fn remove_account(db: &Database, id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM connected_accounts WHERE id = ?1", [id])?;
    Ok(())
}

pub async fn check_and_update_health(db: &Database, id: i64) -> Result<ConnectedAccount> {
    // TODO: actually attempt to connect and verify credentials
    let conn = db.conn()?;
    conn.execute(
        "UPDATE connected_accounts SET status = 'active', last_seen_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    drop(conn);
    get_account(db, id)
}

fn map_account(row: &rusqlite::Row<'_>) -> rusqlite::Result<ConnectedAccount> {
    Ok(ConnectedAccount {
        id: row.get(0)?,
        provider: row.get(1)?,
        display_name: row.get(2)?,
        seed_email: row.get(3)?,
        auth_type: row.get(4)?,
        keychain_ref: row.get(5)?,
        status: row.get(6)?,
        last_seen_at: row.get(7)?,
        created_at: row.get(8)?,
    })
}

// ── Seed groups ───────────────────────────────────────────────────────────────

pub fn list_seed_groups(db: &Database) -> Result<Vec<SeedGroup>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT sg.id, sg.name, sg.description, sg.created_at,
                COUNT(m.id) as member_count
         FROM seed_groups sg
         LEFT JOIN seed_group_members m ON m.seed_group_id = sg.id
         GROUP BY sg.id
         ORDER BY sg.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SeedGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            member_count: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().context("list_seed_groups")
}

pub fn create_seed_group(db: &Database, name: &str, description: Option<&str>) -> Result<SeedGroup> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT INTO seed_groups (name, description) VALUES (?1, ?2)",
        rusqlite::params![name, description],
    )?;
    let id = conn.last_insert_rowid();
    drop(conn);
    let conn = db.conn()?;
    conn.query_row(
        "SELECT id, name, description, created_at, 0 FROM seed_groups WHERE id = ?1",
        [id],
        |row| {
            Ok(SeedGroup {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                member_count: Some(0),
            })
        },
    )
    .context("create_seed_group")
}

pub fn delete_seed_group(db: &Database, id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM seed_groups WHERE id = ?1", [id])?;
    Ok(())
}

pub fn add_account_to_group(
    db: &Database,
    seed_group_id: i64,
    account_id: i64,
) -> Result<SeedGroupMember> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT OR IGNORE INTO seed_group_members (seed_group_id, connected_account_id) VALUES (?1, ?2)",
        rusqlite::params![seed_group_id, account_id],
    )?;
    let member_id = conn.last_insert_rowid();
    Ok(SeedGroupMember {
        id: member_id,
        seed_group_id,
        connected_account_id: account_id,
        account: None,
    })
}

pub fn remove_account_from_group(db: &Database, seed_group_id: i64, account_id: i64) -> Result<()> {
    let conn = db.conn()?;
    conn.execute(
        "DELETE FROM seed_group_members WHERE seed_group_id = ?1 AND connected_account_id = ?2",
        rusqlite::params![seed_group_id, account_id],
    )?;
    Ok(())
}

pub fn get_seed_group_members(db: &Database, seed_group_id: i64) -> Result<Vec<SeedGroupMember>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT m.id, m.seed_group_id, m.connected_account_id
         FROM seed_group_members m
         WHERE m.seed_group_id = ?1",
    )?;
    let rows = stmt.query_map([seed_group_id], |row| {
        Ok(SeedGroupMember {
            id: row.get(0)?,
            seed_group_id: row.get(1)?,
            connected_account_id: row.get(2)?,
            account: None,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().context("get_seed_group_members")
}
