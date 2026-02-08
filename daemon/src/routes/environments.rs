use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::environment::{self, CreateEnvironment, Environment};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub project_id: String,
}

pub async fn list(
    State(db): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Environment>>, StatusCode> {
    environment::list_environments(&db, &query.project_id)
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list environments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Environment>, StatusCode> {
    match environment::get_environment(&db, &id) {
        Ok(Some(env)) => Ok(Json(env)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateEnvironment>,
) -> Result<(StatusCode, Json<Environment>), StatusCode> {
    environment::create_environment(&db, input)
        .map(|env| (StatusCode::CREATED, Json(env)))
        .map_err(|e| {
            tracing::error!("Failed to create environment: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn pause(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::update_environment_status(&db, &id, "paused") {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to pause environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn resume(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::update_environment_status(&db, &id, "running") {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to resume environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match environment::delete_environment(&db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
