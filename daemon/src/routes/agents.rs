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
    pub idea_id: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    let result = if let Some(idea_id) = query.idea_id.as_deref() {
        agent::list_agents_by_idea(&state.db, idea_id)
    } else {
        agent::list_agents(&state.db, query.env_id.as_deref())
    };
    result.map(Json).map_err(|e| {
        tracing::error!("Failed to list agents: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Agent>, StatusCode> {
    match agent::get_agent(&state.db, &id) {
        Ok(Some(a)) => Ok(Json(a)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateAgent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    agent::create_agent(&state.db, input)
        .map(|a| (StatusCode::CREATED, Json(a)))
        .map_err(|e| {
            tracing::error!("Failed to create agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentInput {
    pub status: String,
    pub blocker: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateAgentInput>,
) -> Result<StatusCode, StatusCode> {
    match agent::update_agent_status(&state.db, &id, &input.status, input.blocker.as_deref()) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match agent::delete_agent(&state.db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete agent: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
