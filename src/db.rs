use rusqlite::{Connection, Result, params};

use crate::Note;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Database> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Database { connection: conn })
    }

    pub fn add_note(&self, title: &str, content: &str) -> Result<Note> {
        self.connection.execute(
            "INSERT INTO notes (title, content) VALUES (?1, ?2)",
            params![title, content],
        )?;

        Ok(Note {
            id: self.connection.last_insert_rowid(),
            title: title.to_string(),
            content: content.to_string(),
        })
    }
    pub fn update_note(&self, id: i64, title: &str, content: &str) -> Result<Note> {
        self.connection.execute(
            "UPDATE notes SET title = ?1, content = ?2 WHERE id = ?3",
            params![title, content, id],
        )?;

        Ok(Note {
            id,
            title: title.to_string(),
            content: content.to_string(),
        })
    }
    pub fn delete_note(&self, id: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM notes WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub fn get_all_notes(&self) -> Result<Vec<Note>> {
        let mut query = self
            .connection
            .prepare("SELECT id, title, content FROM notes ORDER BY id")?;

        let notes = query
            .query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<Note>>>()?;

        Ok(notes)
    }
}
