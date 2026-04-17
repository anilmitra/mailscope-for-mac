PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- Users (single-user MVP; reserved for future multi-user)
CREATE TABLE IF NOT EXISTS users (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL DEFAULT 'Default',
    email      TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default user
INSERT OR IGNORE INTO users (id, name) VALUES (1, 'Default');

-- Connected mailbox accounts (seed mailboxes)
CREATE TABLE IF NOT EXISTS connected_accounts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    provider     TEXT NOT NULL CHECK(provider IN ('gmail','microsoft','imap')),
    display_name TEXT NOT NULL,
    seed_email   TEXT NOT NULL,
    auth_type    TEXT NOT NULL CHECK(auth_type IN ('oauth','password','app_password')),
    keychain_ref TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'unknown' CHECK(status IN ('active','error','expired','unknown')),
    last_seen_at TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- IMAP connection details (non-secret fields; password stored in keychain)
CREATE TABLE IF NOT EXISTS imap_configs (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    connected_account_id INTEGER NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    host               TEXT NOT NULL,
    port               INTEGER NOT NULL DEFAULT 993,
    use_tls            INTEGER NOT NULL DEFAULT 1
);

-- Seed groups
CREATE TABLE IF NOT EXISTS seed_groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Membership: many-to-many between seed_groups and connected_accounts
CREATE TABLE IF NOT EXISTS seed_group_members (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_group_id        INTEGER NOT NULL REFERENCES seed_groups(id) ON DELETE CASCADE,
    connected_account_id INTEGER NOT NULL REFERENCES connected_accounts(id) ON DELETE CASCADE,
    UNIQUE(seed_group_id, connected_account_id)
);

-- Sending profiles
CREATE TABLE IF NOT EXISTS sending_profiles (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT NOT NULL,
    from_name      TEXT NOT NULL,
    from_email     TEXT NOT NULL,
    reply_to       TEXT,
    sending_mode   TEXT NOT NULL DEFAULT 'manual' CHECK(sending_mode IN ('manual','smtp','api')),
    smtp_config_ref TEXT,
    notes          TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Test runs
CREATE TABLE IF NOT EXISTS test_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    run_uuid            TEXT NOT NULL UNIQUE DEFAULT (lower(hex(randomblob(16)))),
    sending_profile_id  INTEGER NOT NULL REFERENCES sending_profiles(id),
    seed_group_id       INTEGER NOT NULL REFERENCES seed_groups(id),
    status              TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','running','complete','failed','cancelled')),
    subject_template    TEXT NOT NULL,
    body_text           TEXT NOT NULL DEFAULT '',
    body_html           TEXT,
    subject_token       TEXT NOT NULL,
    body_token          TEXT NOT NULL,
    started_at          TEXT,
    completed_at        TEXT,
    inbox_rate          REAL,
    spam_rate           REAL,
    missing_rate        REAL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_test_runs_status    ON test_runs(status);
CREATE INDEX IF NOT EXISTS idx_test_runs_started   ON test_runs(started_at DESC);

-- Per-seed placement results
CREATE TABLE IF NOT EXISTS test_results (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    test_run_id          INTEGER NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    connected_account_id INTEGER NOT NULL REFERENCES connected_accounts(id),
    provider             TEXT NOT NULL,
    placement            TEXT NOT NULL DEFAULT 'missing'
                             CHECK(placement IN ('inbox','spam','junk','promotions','social','updates','missing','other')),
    raw_folder           TEXT,
    message_id_remote    TEXT,
    matched_at           TEXT,
    delivery_latency_ms  INTEGER,
    spf_result           TEXT,
    dkim_result          TEXT,
    dmarc_result         TEXT,
    auth_summary         TEXT,
    headers_json         TEXT,   -- JSON object
    notes_json           TEXT,   -- JSON array of strings
    created_at           TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(test_run_id, connected_account_id)
);

CREATE INDEX IF NOT EXISTS idx_test_results_run      ON test_results(test_run_id);
CREATE INDEX IF NOT EXISTS idx_test_results_placement ON test_results(placement);

-- Diagnostic issues per run
CREATE TABLE IF NOT EXISTS diagnostics (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    test_run_id    INTEGER NOT NULL REFERENCES test_runs(id) ON DELETE CASCADE,
    check_type     TEXT NOT NULL,
    severity       TEXT NOT NULL DEFAULT 'info' CHECK(severity IN ('info','warning','error','critical')),
    title          TEXT NOT NULL,
    details        TEXT NOT NULL DEFAULT '',
    recommendation TEXT NOT NULL DEFAULT '',
    created_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_diagnostics_run ON diagnostics(test_run_id);
