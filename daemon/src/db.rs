use rusqlite::Connection;
use std::sync::Mutex;

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Db {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    fn migrate(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                repo_url TEXT NOT NULL,
                default_branch TEXT NOT NULL DEFAULT 'main',
                notification_prefs TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS environments (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id),
                branch TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'creating',
                container_id TEXT NOT NULL DEFAULT '',
                ports TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                last_active TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                env_id TEXT NOT NULL REFERENCES environments(id),
                type TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'running',
                current_task TEXT NOT NULL DEFAULT '',
                blocker TEXT,
                started_at TEXT NOT NULL,
                last_activity TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ideas (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id),
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'draft',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;

        // Idempotent ALTER TABLE migrations
        let _ = conn.execute("ALTER TABLE projects ADD COLUMN project_type TEXT NOT NULL DEFAULT 'standard'", []);
        let _ = conn.execute("ALTER TABLE agents ADD COLUMN idea_id TEXT REFERENCES ideas(id)", []);

        Ok(())
    }
}
