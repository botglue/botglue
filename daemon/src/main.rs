mod db;
mod models;
pub mod podman;
mod routes;

use axum::{routing::{get, post}, Json, Router};
use db::Db;
use podman::PodmanConfig;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

pub struct AppStateInner {
    pub db: Db,
    pub podman: PodmanConfig,
}

pub type AppState = Arc<AppStateInner>;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Db::open("botglue.db").expect("Failed to open database");
    let state = Arc::new(AppStateInner {
        db,
        podman: PodmanConfig::default(),
    });

    let api_routes = Router::new()
        .route("/api/health", get(health))
        .route("/api/projects", get(routes::projects::list).post(routes::projects::create))
        .route("/api/projects/{id}", get(routes::projects::get).delete(routes::projects::delete))
        .route("/api/environments", get(routes::environments::list).post(routes::environments::create))
        .route("/api/environments/{id}", get(routes::environments::get).delete(routes::environments::delete))
        .route("/api/environments/{id}/pause", post(routes::environments::pause))
        .route("/api/environments/{id}/resume", post(routes::environments::resume))
        .route("/api/environments/{id}/exec", post(routes::environments::exec))
        .route("/api/agents", get(routes::agents::list).post(routes::agents::create))
        .route("/api/agents/{id}", get(routes::agents::get))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));

    let static_files = ServeDir::new("../web/dist").fallback(
        tower_http::services::ServeFile::new("../web/dist/index.html"),
    );

    let app = api_routes.fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("BotGlue daemon listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
