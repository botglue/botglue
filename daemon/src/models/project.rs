use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationPrefs {
    pub blocked: bool,
    pub error: bool,
    pub finished: bool,
    pub progress: bool,
}

impl Default for NotificationPrefs {
    fn default() -> Self {
        Self {
            blocked: true,
            error: true,
            finished: true,
            progress: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub repo_url: String,
    pub default_branch: String,
    pub notification_prefs: NotificationPrefs,
    pub project_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub repo_url: String,
    pub default_branch: Option<String>,
    pub notification_prefs: Option<NotificationPrefs>,
    pub project_type: Option<String>,
}

impl Project {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let prefs_json: String = row.get("notification_prefs")?;
        let notification_prefs: NotificationPrefs =
            serde_json::from_str(&prefs_json).unwrap_or_default();
        let project_type: String = row.get("project_type").unwrap_or_else(|_| "standard".to_string());
        Ok(Project {
            id: row.get("id")?,
            name: row.get("name")?,
            repo_url: row.get("repo_url")?,
            default_branch: row.get("default_branch")?,
            notification_prefs,
            project_type,
            created_at: row.get("created_at")?,
        })
    }
}

use crate::db::Db;

pub fn list_projects(db: &Db) -> Result<Vec<Project>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, repo_url, default_branch, notification_prefs, project_type, created_at FROM projects ORDER BY created_at DESC",
    )?;
    let projects = stmt
        .query_map([], |row| Project::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn get_project(db: &Db, id: &str) -> Result<Option<Project>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, repo_url, default_branch, notification_prefs, project_type, created_at FROM projects WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Project::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_project(db: &Db, input: CreateProject) -> Result<Project, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let default_branch = input.default_branch.unwrap_or_else(|| "main".to_string());
    let prefs = input.notification_prefs.unwrap_or_default();
    let prefs_json = serde_json::to_string(&prefs).unwrap();
    let project_type = input.project_type.unwrap_or_else(|| "standard".to_string());

    let conn = db.conn();
    conn.execute(
        "INSERT INTO projects (id, name, repo_url, default_branch, notification_prefs, project_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, input.name, input.repo_url, default_branch, prefs_json, project_type, now],
    )?;

    Ok(Project {
        id,
        name: input.name,
        repo_url: input.repo_url,
        default_branch,
        notification_prefs: prefs,
        project_type,
        created_at: now,
    })
}

pub fn delete_project(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Db {
        Db::open_in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_create_and_get_project() {
        let db = test_db();
        let project = create_project(
            &db,
            CreateProject {
                name: "test".to_string(),
                repo_url: "https://github.com/example/test".to_string(),
                default_branch: None,
                notification_prefs: None,
                project_type: None,
            },
        )
        .unwrap();

        assert_eq!(project.name, "test");
        assert_eq!(project.default_branch, "main");
        assert!(project.notification_prefs.blocked);

        let fetched = get_project(&db, &project.id).unwrap().unwrap();
        assert_eq!(fetched.id, project.id);
        assert_eq!(fetched.name, "test");
    }

    #[test]
    fn test_list_projects() {
        let db = test_db();
        create_project(
            &db,
            CreateProject {
                name: "a".to_string(),
                repo_url: "https://github.com/a".to_string(),
                default_branch: None,
                notification_prefs: None,
                project_type: None,
            },
        )
        .unwrap();
        create_project(
            &db,
            CreateProject {
                name: "b".to_string(),
                repo_url: "https://github.com/b".to_string(),
                default_branch: None,
                notification_prefs: None,
                project_type: None,
            },
        )
        .unwrap();

        let projects = list_projects(&db).unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_delete_project() {
        let db = test_db();
        let project = create_project(
            &db,
            CreateProject {
                name: "to-delete".to_string(),
                repo_url: "https://github.com/del".to_string(),
                default_branch: None,
                notification_prefs: None,
                project_type: None,
            },
        )
        .unwrap();

        assert!(delete_project(&db, &project.id).unwrap());
        assert!(get_project(&db, &project.id).unwrap().is_none());
        assert!(!delete_project(&db, &project.id).unwrap()); // already deleted
    }

    #[test]
    fn test_get_nonexistent_project() {
        let db = test_db();
        assert!(get_project(&db, "nonexistent").unwrap().is_none());
    }
}
