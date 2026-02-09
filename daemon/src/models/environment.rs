use std::collections::HashSet;

use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::Db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortMapping {
    pub name: String,
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub branch: String,
    pub status: String,
    pub container_id: String,
    pub ports: Vec<PortMapping>,
    pub created_at: String,
    pub last_active: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvironment {
    pub project_id: String,
    pub branch: String,
    pub container_id: Option<String>,
    pub ports: Option<Vec<PortMapping>>,
}

impl Environment {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        let ports_json: String = row.get("ports")?;
        let ports: Vec<PortMapping> = serde_json::from_str(&ports_json).unwrap_or_default();
        Ok(Environment {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            branch: row.get("branch")?,
            status: row.get("status")?,
            container_id: row.get("container_id")?,
            ports,
            created_at: row.get("created_at")?,
            last_active: row.get("last_active")?,
        })
    }
}

pub fn list_environments(db: &Db, project_id: &str) -> Result<Vec<Environment>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch, status, container_id, ports, created_at, last_active \
         FROM environments WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;
    let envs = stmt
        .query_map(params![project_id], |row| Environment::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(envs)
}

pub fn get_environment(db: &Db, id: &str) -> Result<Option<Environment>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, project_id, branch, status, container_id, ports, created_at, last_active \
         FROM environments WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Environment::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_environment(
    db: &Db,
    input: CreateEnvironment,
) -> Result<Environment, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let container_id = input.container_id.unwrap_or_default();
    let ports = input.ports.unwrap_or_default();
    let ports_json = serde_json::to_string(&ports).unwrap();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO environments (id, project_id, branch, status, container_id, ports, created_at, last_active) \
         VALUES (?1, ?2, ?3, 'creating', ?4, ?5, ?6, ?7)",
        params![id, input.project_id, input.branch, container_id, ports_json, now, now],
    )?;

    Ok(Environment {
        id,
        project_id: input.project_id,
        branch: input.branch,
        status: "creating".to_string(),
        container_id,
        ports,
        created_at: now.clone(),
        last_active: now,
    })
}

pub fn update_environment_status(
    db: &Db,
    id: &str,
    status: &str,
) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE environments SET status = ?1, last_active = ?2 WHERE id = ?3",
        params![status, now, id],
    )?;
    Ok(rows > 0)
}

pub fn delete_environment(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

pub fn get_used_ports(db: &Db) -> Result<HashSet<u16>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT ports FROM environments WHERE status != 'destroyed'",
    )?;
    let rows = stmt.query_map([], |row| {
        let ports_json: String = row.get(0)?;
        Ok(ports_json)
    })?;

    let mut used = HashSet::new();
    for row in rows {
        let ports_json = row?;
        if let Ok(ports) = serde_json::from_str::<Vec<PortMapping>>(&ports_json) {
            for mapping in ports {
                if let Some(hp) = mapping.host_port {
                    used.insert(hp);
                }
            }
        }
    }
    Ok(used)
}

pub fn update_environment_container(
    db: &Db,
    id: &str,
    container_id: &str,
    ports: &[PortMapping],
    status: &str,
) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let now = chrono::Utc::now().to_rfc3339();
    let ports_json = serde_json::to_string(ports).unwrap_or_else(|_| "[]".to_string());
    let rows = conn.execute(
        "UPDATE environments SET container_id = ?1, ports = ?2, status = ?3, last_active = ?4 WHERE id = ?5",
        params![container_id, ports_json, status, now, id],
    )?;
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
            },
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_get_environment() {
        let db = test_db();
        let project = create_test_project(&db);

        let env = create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "feature/test".to_string(),
                container_id: Some("abc123".to_string()),
                ports: Some(vec![
                    PortMapping {
                        name: "http".to_string(),
                        container_port: 8080,
                        host_port: Some(3000),
                        protocol: Some("tcp".to_string()),
                    },
                    PortMapping {
                        name: "debug".to_string(),
                        container_port: 9229,
                        host_port: None,
                        protocol: None,
                    },
                ]),
            },
        )
        .unwrap();

        assert_eq!(env.project_id, project.id);
        assert_eq!(env.branch, "feature/test");
        assert_eq!(env.status, "creating");
        assert_eq!(env.container_id, "abc123");
        assert_eq!(env.ports.len(), 2);
        assert_eq!(env.ports[0].name, "http");
        assert_eq!(env.ports[0].container_port, 8080);
        assert_eq!(env.ports[0].host_port, Some(3000));
        assert_eq!(env.ports[1].name, "debug");
        assert_eq!(env.ports[1].host_port, None);

        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.id, env.id);
        assert_eq!(fetched.branch, "feature/test");
        assert_eq!(fetched.ports.len(), 2);
        assert_eq!(fetched.ports[0].container_port, 8080);
    }

    #[test]
    fn test_list_environments_by_project() {
        let db = test_db();
        let project1 = create_test_project(&db);
        let project2 = create_project(
            &db,
            CreateProject {
                name: "project-2".to_string(),
                repo_url: "https://github.com/example/p2".to_string(),
                default_branch: None,
                notification_prefs: None,
            },
        )
        .unwrap();

        // 2 environments in project 1
        create_environment(
            &db,
            CreateEnvironment {
                project_id: project1.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();
        create_environment(
            &db,
            CreateEnvironment {
                project_id: project1.id.clone(),
                branch: "dev".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        // 1 environment in project 2
        create_environment(
            &db,
            CreateEnvironment {
                project_id: project2.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        let envs1 = list_environments(&db, &project1.id).unwrap();
        assert_eq!(envs1.len(), 2);

        let envs2 = list_environments(&db, &project2.id).unwrap();
        assert_eq!(envs2.len(), 1);
    }

    #[test]
    fn test_update_environment_status() {
        let db = test_db();
        let project = create_test_project(&db);

        let env = create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        assert_eq!(env.status, "creating");

        let updated = update_environment_status(&db, &env.id, "running").unwrap();
        assert!(updated);

        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");

        // Non-existent environment
        let not_found = update_environment_status(&db, "nonexistent", "running").unwrap();
        assert!(!not_found);
    }

    #[test]
    fn test_update_environment_container() {
        let db = test_db();
        let project = create_test_project(&db);

        let env = create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        assert_eq!(env.status, "creating");
        assert_eq!(env.container_id, "");
        assert!(env.ports.is_empty());

        let ports = vec![PortMapping {
            name: "http".to_string(),
            container_port: 8080,
            host_port: Some(10000),
            protocol: Some("tcp".to_string()),
        }];

        let updated =
            update_environment_container(&db, &env.id, "abc123container", &ports, "running")
                .unwrap();
        assert!(updated);

        let fetched = get_environment(&db, &env.id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");
        assert_eq!(fetched.container_id, "abc123container");
        assert_eq!(fetched.ports.len(), 1);
        assert_eq!(fetched.ports[0].host_port, Some(10000));
    }

    #[test]
    fn test_get_used_ports() {
        let db = test_db();
        let project = create_test_project(&db);

        create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: Some(vec![PortMapping {
                    name: "http".to_string(),
                    container_port: 8080,
                    host_port: Some(10000),
                    protocol: None,
                }]),
            },
        )
        .unwrap();

        create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "dev".to_string(),
                container_id: None,
                ports: Some(vec![PortMapping {
                    name: "api".to_string(),
                    container_port: 3000,
                    host_port: Some(10001),
                    protocol: None,
                }]),
            },
        )
        .unwrap();

        let used = get_used_ports(&db).unwrap();
        assert!(used.contains(&10000));
        assert!(used.contains(&10001));
        assert!(!used.contains(&10002));
    }

    #[test]
    fn test_delete_environment() {
        let db = test_db();
        let project = create_test_project(&db);

        let env = create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        assert!(delete_environment(&db, &env.id).unwrap());
        assert!(get_environment(&db, &env.id).unwrap().is_none());
        assert!(!delete_environment(&db, &env.id).unwrap()); // already deleted
    }
}
