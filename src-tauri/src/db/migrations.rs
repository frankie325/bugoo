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
            meanings_json TEXT NOT NULL DEFAULT '[]',
            english_definitions_json TEXT NOT NULL DEFAULT '[]',
            examples_json TEXT NOT NULL DEFAULT '[]',
            word_forms_json TEXT NOT NULL DEFAULT '[]',
            memory_tip TEXT NOT NULL DEFAULT '',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (word_id) REFERENCES words(id) ON DELETE CASCADE
        )",
        [],
    )?;

    ensure_column(
        conn,
        "words",
        "translation",
        "translation TEXT NOT NULL DEFAULT ''",
    )?;
    if column_exists(conn, "words", "summary")? {
        conn.execute(
            "UPDATE words SET translation = summary
             WHERE (translation IS NULL OR TRIM(translation) = '')
               AND summary IS NOT NULL
               AND TRIM(summary) != ''",
            [],
        )?;
    }

    ensure_column(
        conn,
        "word_details",
        "meanings_json",
        "meanings_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "word_details",
        "english_definitions_json",
        "english_definitions_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "word_details",
        "examples_json",
        "examples_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "word_details",
        "word_forms_json",
        "word_forms_json TEXT NOT NULL DEFAULT '[]'",
    )?;
    ensure_column(
        conn,
        "word_details",
        "memory_tip",
        "memory_tip TEXT NOT NULL DEFAULT ''",
    )?;
    ensure_column(
        conn,
        "word_details",
        "created_at",
        "created_at INTEGER NOT NULL DEFAULT 0",
    )?;
    ensure_column(
        conn,
        "word_details",
        "updated_at",
        "updated_at INTEGER NOT NULL DEFAULT 0",
    )?;

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
        [table, column],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), rusqlite::Error> {
    if !column_exists(conn, table, column)? {
        conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {definition}"), [])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn run_creates_words_with_translation_column() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let words_has_translation: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('words') WHERE name = 'translation'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(words_has_translation, 1);

        let details_has_meanings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('word_details') WHERE name = 'meanings_json'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(details_has_meanings, 1);
    }

    #[test]
    fn run_upgrades_old_words_and_word_details_schema() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE words (
                id TEXT PRIMARY KEY,
                word TEXT NOT NULL,
                summary TEXT NOT NULL,
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
        )
        .unwrap();
        conn.execute(
            "INSERT INTO words (id, word, summary, created_at, updated_at)
             VALUES ('word-1', 'hello', '你好', 1, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE word_details (
                word_id TEXT PRIMARY KEY,
                part_of_speech_json TEXT NOT NULL DEFAULT '[]',
                definitions_json TEXT NOT NULL DEFAULT '[]',
                examples_json TEXT NOT NULL DEFAULT '[]',
                detail TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();

        run(&conn).unwrap();

        let translation: String = conn
            .query_row(
                "SELECT translation FROM words WHERE id = 'word-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(translation, "你好");

        for column in [
            "meanings_json",
            "english_definitions_json",
            "examples_json",
            "word_forms_json",
            "memory_tip",
        ] {
            let exists = column_exists(&conn, "word_details", column).unwrap();
            assert!(exists, "{column} should exist");
        }
    }
}
