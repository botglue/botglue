mod db;

use axum::{routing::get, Json, Router};
use db::Db;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub type AppState = Arc<Db>;

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

    let db = Arc::new(Db::open("botglue.db").expect("Failed to open database"));

    let api_routes = Router::new()
        .route("/api/health", get(health))
        .with_state(db);

    let static_files = ServeDir::new("../web/dist").fallback(
        tower_http::services::ServeFile::new("../web/dist/index.html"),
    );

    let app = api_routes.fallback_service(static_files);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("BotGlue daemon listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
