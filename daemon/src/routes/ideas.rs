use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::idea::{self, CreateIdea, Idea};
use crate::models::project;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub project_id: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    error: String,
}

type ApiError = (StatusCode, Json<ErrorResponse>);

fn internal_err(msg: String) -> ApiError {
    tracing::error!("Internal error: {}", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: msg }),
    )
}

fn not_found_err(msg: &str) -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: msg.to_string(),
        }),
    )
}

fn bad_request_err(msg: String) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse { error: msg }),
    )
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Idea>>, StatusCode> {
    idea::list_ideas(&state.db, &query.project_id)
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list ideas: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Idea>, StatusCode> {
    match idea::get_idea(&state.db, &id) {
        Ok(Some(i)) => Ok(Json(i)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get idea: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateIdea>,
) -> Result<(StatusCode, Json<Idea>), StatusCode> {
    idea::create_idea(&state.db, input)
        .map(|i| (StatusCode::CREATED, Json(i)))
        .map_err(|e| {
            tracing::error!("Failed to create idea: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[derive(Debug, Deserialize)]
pub struct UpdateIdeaInput {
    pub title: String,
    pub description: String,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateIdeaInput>,
) -> Result<StatusCode, StatusCode> {
    match idea::update_idea(&state.db, &id, &input.title, &input.description) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update idea: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusInput {
    pub status: String,
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateStatusInput>,
) -> Result<StatusCode, StatusCode> {
    match idea::update_idea_status(&state.db, &id, &input.status) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update idea status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match idea::delete_idea(&state.db, &id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete idea: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GraduateInput {
    pub name: String,
    pub repo_url: String,
}

pub async fn graduate(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<GraduateInput>,
) -> Result<(StatusCode, Json<project::Project>), ApiError> {
    // Get the idea
    let idea = idea::get_idea(&state.db, &id)
        .map_err(|e| internal_err(format!("Failed to get idea: {}", e)))?
        .ok_or_else(|| not_found_err("idea not found"))?;

    // Get the project to verify it's an incubator
    let proj = project::get_project(&state.db, &idea.project_id)
        .map_err(|e| internal_err(format!("Failed to get project: {}", e)))?
        .ok_or_else(|| not_found_err("project not found"))?;

    if proj.project_type != "incubator" {
        return Err(bad_request_err(
            "can only graduate ideas from incubator projects".to_string(),
        ));
    }

    // Create new standard project
    let new_project = project::create_project(
        &state.db,
        project::CreateProject {
            name: input.name,
            repo_url: input.repo_url,
            default_branch: Some(proj.default_branch.clone()),
            notification_prefs: None,
            project_type: Some("standard".to_string()),
        },
    )
    .map_err(|e| internal_err(format!("Failed to create project: {}", e)))?;

    // Mark idea as completed
    idea::update_idea_status(&state.db, &id, "completed")
        .map_err(|e| internal_err(format!("Failed to update idea status: {}", e)))?;

    Ok((StatusCode::CREATED, Json(new_project)))
}
