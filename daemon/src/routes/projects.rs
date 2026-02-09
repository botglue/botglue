use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::project::{self, CreateProject, Project};
use crate::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Project>>, StatusCode> {
    project::list_projects(&state.db).map(Json).map_err(|e| {
        tracing::error!("Failed to list projects: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    match project::get_project(&state.db, &id) {
        Ok(Some(p)) => Ok(Json(p)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    project::create_project(&state.db, input)
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(|e| {
            tracing::error!("Failed to create project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match project::delete_project(&state.db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
