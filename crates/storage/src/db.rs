use anyhow::{Context, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;

pub struct Database {
    pub pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path).with_init(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=NORMAL;",
            )
        });
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .context("failed to build SQLite connection pool")?;

        Ok(Self { pool })
    }

    pub fn conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().context("failed to acquire db connection")
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn()?;
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))
            .context("migration 001 failed")?;
        Ok(())
    }
}
