use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS words (
            id TEXT PRIMARY KEY,
            word TEXT NOT NULL,
            translation TEXT NOT NULL,
            phonetic TEXT,
            source_lang TEXT DEFAULT 'EN',
            target_lang TEXT DEFAULT 'ZH',
            status TEXT DEFAULT 'new',
            tags TEXT DEFAULT '',
            notes TEXT DEFAULT '',
            audio_url TEXT,
            ease_factor REAL DEFAULT 2.5,
            interval INTEGER DEFAULT 0,
            repetitions INTEGER DEFAULT 0,
            next_review_at INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS review_records (
            id TEXT PRIMARY KEY,
            word_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            reviewed_at INTEGER NOT NULL,
            next_review_at INTEGER NOT NULL,
            FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS word_details (
            word_id TEXT PRIMARY KEY,
            part_of_speech_json TEXT NOT NULL DEFAULT '[]',
            definitions_json TEXT NOT NULL DEFAULT '[]',
            examples_json TEXT NOT NULL DEFAULT '[]',
            memory_tip TEXT NOT NULL DEFAULT '',
            detail TEXT NOT NULL DEFAULT '',
            provider TEXT NOT NULL DEFAULT '',
            raw_json TEXT NOT NULL DEFAULT '{}',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}
