use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::Db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Idea {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateIdea {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
}

impl Idea {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Idea {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            status: row.get("status")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

pub fn list_ideas(db: &Db, project_id: &str) -> Result<Vec<Idea>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, description, status, created_at, updated_at \
         FROM ideas WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;
    let ideas = stmt
        .query_map(params![project_id], |row| Idea::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ideas)
}

pub fn get_idea(db: &Db, id: &str) -> Result<Option<Idea>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, description, status, created_at, updated_at \
         FROM ideas WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Idea::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_idea(db: &Db, input: CreateIdea) -> Result<Idea, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let description = input.description.unwrap_or_default();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO ideas (id, project_id, title, description, status, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, 'draft', ?5, ?6)",
        params![id, input.project_id, input.title, description, now, now],
    )?;

    Ok(Idea {
        id,
        project_id: input.project_id,
        title: input.title,
        description,
        status: "draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_idea(
    db: &Db,
    id: &str,
    title: &str,
    description: &str,
) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE ideas SET title = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, description, now, id],
    )?;
    Ok(rows > 0)
}

pub fn update_idea_status(db: &Db, id: &str, status: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE ideas SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now, id],
    )?;
    Ok(rows > 0)
}

pub fn delete_idea(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM ideas WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::{create_project, CreateProject};

    fn test_db() -> Db {
        Db::open_in_memory().expect("Failed to create test database")
    }

    fn create_test_project(db: &Db) -> crate::models::project::Project {
        create_project(
            db,
            CreateProject {
                name: "test-project".to_string(),
                repo_url: "https://github.com/example/test".to_string(),
                default_branch: None,
                notification_prefs: None,
                project_type: None,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_get_idea() {
        let db = test_db();
        let project = create_test_project(&db);

        let idea = create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Add login page".to_string(),
                description: Some("Build a login page with OAuth support".to_string()),
            },
        )
        .unwrap();

        assert_eq!(idea.project_id, project.id);
        assert_eq!(idea.title, "Add login page");
        assert_eq!(idea.description, "Build a login page with OAuth support");
        assert_eq!(idea.status, "draft");

        let fetched = get_idea(&db, &idea.id).unwrap().unwrap();
        assert_eq!(fetched.id, idea.id);
        assert_eq!(fetched.title, "Add login page");
    }

    #[test]
    fn test_create_idea_default_description() {
        let db = test_db();
        let project = create_test_project(&db);

        let idea = create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Quick idea".to_string(),
                description: None,
            },
        )
        .unwrap();

        assert_eq!(idea.description, "");
    }

    #[test]
    fn test_list_ideas() {
        let db = test_db();
        let project = create_test_project(&db);

        create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Idea A".to_string(),
                description: None,
            },
        )
        .unwrap();
        create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Idea B".to_string(),
                description: None,
            },
        )
        .unwrap();

        let ideas = list_ideas(&db, &project.id).unwrap();
        assert_eq!(ideas.len(), 2);
    }

    #[test]
    fn test_update_idea() {
        let db = test_db();
        let project = create_test_project(&db);

        let idea = create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Original".to_string(),
                description: None,
            },
        )
        .unwrap();

        let updated = update_idea(&db, &idea.id, "Updated Title", "New description").unwrap();
        assert!(updated);

        let fetched = get_idea(&db, &idea.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Title");
        assert_eq!(fetched.description, "New description");
    }

    #[test]
    fn test_update_idea_status() {
        let db = test_db();
        let project = create_test_project(&db);

        let idea = create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "Test".to_string(),
                description: None,
            },
        )
        .unwrap();

        assert_eq!(idea.status, "draft");

        let updated = update_idea_status(&db, &idea.id, "active").unwrap();
        assert!(updated);

        let fetched = get_idea(&db, &idea.id).unwrap().unwrap();
        assert_eq!(fetched.status, "active");

        let not_found = update_idea_status(&db, "nonexistent", "active").unwrap();
        assert!(!not_found);
    }

    #[test]
    fn test_delete_idea() {
        let db = test_db();
        let project = create_test_project(&db);

        let idea = create_idea(
            &db,
            CreateIdea {
                project_id: project.id.clone(),
                title: "To delete".to_string(),
                description: None,
            },
        )
        .unwrap();

        assert!(delete_idea(&db, &idea.id).unwrap());
        assert!(get_idea(&db, &idea.id).unwrap().is_none());
        assert!(!delete_idea(&db, &idea.id).unwrap());
    }

    #[test]
    fn test_get_nonexistent_idea() {
        let db = test_db();
        assert!(get_idea(&db, "nonexistent").unwrap().is_none());
    }
}
