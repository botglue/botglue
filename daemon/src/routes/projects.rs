use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::project::{self, CreateProject, Project};
use crate::AppState;

pub async fn list(State(db): State<AppState>) -> Result<Json<Vec<Project>>, StatusCode> {
    project::list_projects(&db).map(Json).map_err(|e| {
        tracing::error!("Failed to list projects: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

pub async fn get(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, StatusCode> {
    match project::get_project(&db, &id) {
        Ok(Some(p)) => Ok(Json(p)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(db): State<AppState>,
    Json(input): Json<CreateProject>,
) -> Result<(StatusCode, Json<Project>), StatusCode> {
    project::create_project(&db, input)
        .map(|p| (StatusCode::CREATED, Json(p)))
        .map_err(|e| {
            tracing::error!("Failed to create project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn delete(
    State(db): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match project::delete_project(&db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
