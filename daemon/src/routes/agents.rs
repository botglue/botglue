use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::agent::{self, Agent, CreateAgent};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub env_id: Option<String>,
}

pub async fn list(
    State(db): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    agent::list_agents(&db, query.env_id.as_deref())
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, StatusCode> {
    match agent::get_agent(&db, &id) {
        Ok(Some(a)) => Ok(Json(a)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateAgent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    agent::create_agent(&db, input)
        .map(|a| (StatusCode::CREATED, Json(a)))
        .map_err(|e| {
            tracing::error!("Failed to create agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
