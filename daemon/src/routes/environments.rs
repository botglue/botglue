use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::environment::{self, CreateEnvironment, Environment};
use crate::podman::{self, PodmanError};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub project_id: String,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    error: String,
}

pub(crate) type ApiError = (StatusCode, Json<ErrorResponse>);

fn podman_err(e: PodmanError) -> ApiError {
    tracing::error!("Podman error: {}", e);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: e.to_string(),
        }),
    )
}

fn internal_err(msg: String) -> ApiError {
    tracing::error!("Internal error: {}", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: msg }),
    )
}

fn conflict_err(msg: String) -> ApiError {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse { error: msg }),
    )
}

fn not_found_err() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "environment not found".to_string(),
        }),
    )
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Environment>>, StatusCode> {
    environment::list_environments(&state.db, &query.project_id)
        .map(Json)
        .map_err(|e| {
            tracing::error!("Failed to list environments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Environment>, StatusCode> {
    match environment::get_environment(&state.db, &id) {
        Ok(Some(env)) => Ok(Json(env)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get environment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateEnvironment>,
) -> Result<(StatusCode, Json<Environment>), ApiError> {
    let requested_ports = input.ports.clone().unwrap_or_default();

    // 1. Insert DB record with status "creating"
    let env = environment::create_environment(&state.db, input)
        .map_err(|e| internal_err(format!("Failed to create environment: {}", e)))?;

    // 2. Allocate ports
    let used_ports = environment::get_used_ports(&state.db)
        .map_err(|e| internal_err(format!("Failed to get used ports: {}", e)))?;

    let allocated_ports =
        podman::allocate_ports(&state.podman, &used_ports, &requested_ports).map_err(|e| {
            let _ = environment::update_environment_status(&state.db, &env.id, "destroyed");
            podman_err(e)
        })?;

    // 3. Create container
    let name = podman::container_name(&env.id);
    let container_id =
        podman::create_container(&state.podman, &name, None, &allocated_ports)
            .await
            .map_err(|e| {
                let _ = environment::update_environment_status(&state.db, &env.id, "destroyed");
                podman_err(e)
            })?;

    // 4. Update DB with container_id, allocated ports, status "running"
    environment::update_environment_container(
        &state.db,
        &env.id,
        &container_id,
        &allocated_ports,
        "running",
    )
    .map_err(|e| internal_err(format!("Failed to update environment: {}", e)))?;

    // 5. Return the updated environment
    let updated = environment::get_environment(&state.db, &env.id)
        .map_err(|e| internal_err(format!("Failed to get environment: {}", e)))?
        .ok_or_else(|| internal_err("Environment not found after creation".to_string()))?;

    Ok((StatusCode::CREATED, Json(updated)))
}

pub async fn pause(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let env = match environment::get_environment(&state.db, &id) {
        Ok(Some(env)) => env,
        Ok(None) => return Err(not_found_err()),
        Err(e) => return Err(internal_err(format!("Failed to get environment: {}", e))),
    };

    if env.status != "running" {
        return Err(conflict_err(format!(
            "cannot pause environment with status '{}'",
            env.status
        )));
    }

    if !env.container_id.is_empty() {
        podman::stop_container(&state.podman, &env.container_id)
            .await
            .map_err(podman_err)?;
    }

    environment::update_environment_status(&state.db, &id, "paused")
        .map_err(|e| internal_err(format!("Failed to update status: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn resume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let env = match environment::get_environment(&state.db, &id) {
        Ok(Some(env)) => env,
        Ok(None) => return Err(not_found_err()),
        Err(e) => return Err(internal_err(format!("Failed to get environment: {}", e))),
    };

    if env.status != "paused" {
        return Err(conflict_err(format!(
            "cannot resume environment with status '{}'",
            env.status
        )));
    }

    if !env.container_id.is_empty() {
        podman::start_container(&state.podman, &env.container_id)
            .await
            .map_err(podman_err)?;
    }

    environment::update_environment_status(&state.db, &id, "running")
        .map_err(|e| internal_err(format!("Failed to update status: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let env = match environment::get_environment(&state.db, &id) {
        Ok(Some(env)) => env,
        Ok(None) => return Err(not_found_err()),
        Err(e) => return Err(internal_err(format!("Failed to get environment: {}", e))),
    };

    // Best-effort container removal
    if !env.container_id.is_empty() {
        if let Err(e) = podman::remove_container(&state.podman, &env.container_id).await {
            tracing::warn!("Failed to remove container {}: {}", env.container_id, e);
        }
    }

    environment::delete_environment(&state.db, &id)
        .map_err(|e| internal_err(format!("Failed to delete environment: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ExecRequest {
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct ExecResponse {
    pub output: String,
    pub exit_code: i32,
}

pub async fn exec(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ExecRequest>,
) -> Result<Json<ExecResponse>, ApiError> {
    let env = match environment::get_environment(&state.db, &id) {
        Ok(Some(env)) => env,
        Ok(None) => return Err(not_found_err()),
        Err(e) => return Err(internal_err(format!("Failed to get environment: {}", e))),
    };

    if env.status != "running" {
        return Err(conflict_err(format!(
            "cannot exec in environment with status '{}'",
            env.status
        )));
    }

    if env.container_id.is_empty() {
        return Err(conflict_err(
            "environment has no container".to_string(),
        ));
    }

    let result = podman::exec_in_container(&state.podman, &env.container_id, &input.command)
        .await
        .map_err(podman_err)?;

    Ok(Json(ExecResponse {
        output: result.output,
        exit_code: result.exit_code,
    }))
}
