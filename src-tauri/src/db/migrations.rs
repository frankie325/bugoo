use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS words (
            id TEXT PRIMARY KEY,
            word TEXT NOT NULL,
            translation TEXT NOT NULL,
            source_lang TEXT NOT NULL DEFAULT 'EN',
            target_lang TEXT NOT NULL DEFAULT 'ZH',
            status TEXT NOT NULL DEFAULT 'learning',
            tags TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            audio_url TEXT NOT NULL DEFAULT '',
            ease_factor REAL NOT NULL DEFAULT 2.5,
            interval INTEGER NOT NULL DEFAULT 1,
            repetitions INTEGER NOT NULL DEFAULT 0,
            next_review_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now', '+1 day')),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6b7280',
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS review_log (
            id TEXT PRIMARY KEY,
            word_id TEXT NOT NULL,
            result TEXT NOT NULL,
            reviewed_at INTEGER NOT NULL,
            FOREIGN KEY (word_id) REFERENCES words(id)
        );

        CREATE INDEX IF NOT EXISTS idx_words_next_review ON words(next_review_at);
        CREATE INDEX IF NOT EXISTS idx_words_status ON words(status);
        "
    )
}
