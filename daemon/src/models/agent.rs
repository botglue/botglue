use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::Db;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    pub id: String,
    pub env_id: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub status: String,
    pub current_task: String,
    pub blocker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idea_id: Option<String>,
    pub started_at: String,
    pub last_activity: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgent {
    pub env_id: String,
    #[serde(rename = "type")]
    pub agent_type: String,
    pub current_task: String,
    pub idea_id: Option<String>,
}

impl Agent {
    fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(Agent {
            id: row.get("id")?,
            env_id: row.get("env_id")?,
            agent_type: row.get("type")?,
            status: row.get("status")?,
            current_task: row.get("current_task")?,
            blocker: row.get("blocker")?,
            idea_id: row.get("idea_id").ok(),
            started_at: row.get("started_at")?,
            last_activity: row.get("last_activity")?,
        })
    }
}

pub fn list_agents(db: &Db, env_id: Option<&str>) -> Result<Vec<Agent>, rusqlite::Error> {
    let conn = db.conn();
    match env_id {
        Some(eid) => {
            let mut stmt = conn.prepare(
                "SELECT id, env_id, type, status, current_task, blocker, idea_id, started_at, last_activity \
                 FROM agents WHERE env_id = ?1 ORDER BY started_at DESC",
            )?;
            let agents = stmt
                .query_map(params![eid], |row| Agent::from_row(row))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(agents)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, env_id, type, status, current_task, blocker, idea_id, started_at, last_activity \
                 FROM agents ORDER BY started_at DESC",
            )?;
            let agents = stmt
                .query_map([], |row| Agent::from_row(row))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(agents)
        }
    }
}

pub fn get_agent(db: &Db, id: &str) -> Result<Option<Agent>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, env_id, type, status, current_task, blocker, idea_id, started_at, last_activity \
         FROM agents WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| Agent::from_row(row))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn create_agent(db: &Db, input: CreateAgent) -> Result<Agent, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO agents (id, env_id, type, status, current_task, blocker, idea_id, started_at, last_activity) \
         VALUES (?1, ?2, ?3, 'running', ?4, NULL, ?5, ?6, ?7)",
        params![id, input.env_id, input.agent_type, input.current_task, input.idea_id, now, now],
    )?;

    Ok(Agent {
        id,
        env_id: input.env_id,
        agent_type: input.agent_type,
        status: "running".to_string(),
        current_task: input.current_task,
        blocker: None,
        idea_id: input.idea_id,
        started_at: now.clone(),
        last_activity: now,
    })
}

pub fn list_agents_by_idea(db: &Db, idea_id: &str) -> Result<Vec<Agent>, rusqlite::Error> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, env_id, type, status, current_task, blocker, idea_id, started_at, last_activity \
         FROM agents WHERE idea_id = ?1 ORDER BY started_at DESC",
    )?;
    let agents = stmt
        .query_map(params![idea_id], |row| Agent::from_row(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(agents)
}

pub fn delete_agent(db: &Db, id: &str) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let rows = conn.execute("DELETE FROM agents WHERE id = ?1", params![id])?;
    Ok(rows > 0)
}

pub fn update_agent_status(
    db: &Db,
    id: &str,
    status: &str,
    blocker: Option<&str>,
) -> Result<bool, rusqlite::Error> {
    let conn = db.conn();
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn.execute(
        "UPDATE agents SET status = ?1, blocker = ?2, last_activity = ?3 WHERE id = ?4",
        params![status, blocker, now, id],
    )?;
    Ok(rows > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::environment::{create_environment, CreateEnvironment};
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

    fn create_test_environment(db: &Db, project_id: &str) -> crate::models::environment::Environment {
        create_environment(
            db,
            CreateEnvironment {
                project_id: project_id.to_string(),
                branch: "main".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap()
    }

    #[test]
    fn test_create_and_get_agent() {
        let db = test_db();
        let project = create_test_project(&db);
        let env = create_test_environment(&db, &project.id);

        let agent = create_agent(
            &db,
            CreateAgent {
                env_id: env.id.clone(),
                agent_type: "coder".to_string(),
                current_task: "implement feature X".to_string(),
                idea_id: None,
            },
        )
        .unwrap();

        assert_eq!(agent.env_id, env.id);
        assert_eq!(agent.agent_type, "coder");
        assert_eq!(agent.status, "running");
        assert_eq!(agent.current_task, "implement feature X");
        assert!(agent.blocker.is_none());

        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.id, agent.id);
        assert_eq!(fetched.agent_type, "coder");
        assert_eq!(fetched.current_task, "implement feature X");
    }

    #[test]
    fn test_list_agents_with_filter() {
        let db = test_db();
        let project = create_test_project(&db);
        let env1 = create_test_environment(&db, &project.id);
        let env2 = create_environment(
            &db,
            CreateEnvironment {
                project_id: project.id.clone(),
                branch: "dev".to_string(),
                container_id: None,
                ports: None,
            },
        )
        .unwrap();

        // 2 agents in env1
        create_agent(
            &db,
            CreateAgent {
                env_id: env1.id.clone(),
                agent_type: "coder".to_string(),
                current_task: "task A".to_string(),
                idea_id: None,
            },
        )
        .unwrap();
        create_agent(
            &db,
            CreateAgent {
                env_id: env1.id.clone(),
                agent_type: "reviewer".to_string(),
                current_task: "task B".to_string(),
                idea_id: None,
            },
        )
        .unwrap();

        // 1 agent in env2
        create_agent(
            &db,
            CreateAgent {
                env_id: env2.id.clone(),
                agent_type: "coder".to_string(),
                current_task: "task C".to_string(),
                idea_id: None,
            },
        )
        .unwrap();

        // Filter by env1
        let agents1 = list_agents(&db, Some(&env1.id)).unwrap();
        assert_eq!(agents1.len(), 2);

        // Filter by env2
        let agents2 = list_agents(&db, Some(&env2.id)).unwrap();
        assert_eq!(agents2.len(), 1);

        // No filter - all agents
        let all_agents = list_agents(&db, None).unwrap();
        assert_eq!(all_agents.len(), 3);
    }

    #[test]
    fn test_update_agent_status() {
        let db = test_db();
        let project = create_test_project(&db);
        let env = create_test_environment(&db, &project.id);

        let agent = create_agent(
            &db,
            CreateAgent {
                env_id: env.id.clone(),
                agent_type: "coder".to_string(),
                current_task: "implement feature".to_string(),
                idea_id: None,
            },
        )
        .unwrap();

        assert_eq!(agent.status, "running");
        assert!(agent.blocker.is_none());

        // Set to blocked with blocker text
        let updated = update_agent_status(&db, &agent.id, "blocked", Some("waiting for API key")).unwrap();
        assert!(updated);

        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.status, "blocked");
        assert_eq!(fetched.blocker.as_deref(), Some("waiting for API key"));

        // Set back to running with no blocker
        let updated = update_agent_status(&db, &agent.id, "running", None).unwrap();
        assert!(updated);

        let fetched = get_agent(&db, &agent.id).unwrap().unwrap();
        assert_eq!(fetched.status, "running");
        assert!(fetched.blocker.is_none());

        // Non-existent agent
        let not_found = update_agent_status(&db, "nonexistent", "running", None).unwrap();
        assert!(!not_found);
    }
}
