pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;
pub mod services;
pub mod swagger;
pub mod utils;

use std::process::exit;

use axum::{Json, Router, http::StatusCode, routing::get};
use colored::Colorize;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    database::connection::{GLOBAL_DB_POOL, initialize_global_db_pool},
    services::UsersService,
};

#[derive(Clone)]
pub struct AppState {
    pub users_service: Arc<UsersService>,
}

pub async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    config::init_config()?;
    let cfg = config::get_config();
    let database_url = &cfg.db.database_url;

    println!("{}", "Conectando a la base de datos...".yellow());
    initialize_global_db_pool(&database_url).await?;
    println!("{}", "Conectado a la base de datos.".green());

    let pool = match GLOBAL_DB_POOL.get() {
        Some(p) => p,
        None => {
            eprintln!("{}", "Pool de conexiones no configurado.".red());
            exit(1);
        }
    };

    let users_service = Arc::new(UsersService::new(&pool));

    let state = AppState { users_service };
    let openapi = swagger::ApiDoc::openapi();

    let app = Router::new()
        .route("/", get(root))
        .layer(TraceLayer::new_for_http())
        .merge(SwaggerUi::new("/docs").url("/openapi.json", openapi.clone()))
        .nest("/api", handlers::api_routes(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3032").await.unwrap();
    println!(
        "{}",
        format!("Servidor corriendo en el puerto {}", "3032".yellow()).green()
    );

    Ok(axum::serve(listener, app).await.unwrap())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiInfo {
    name: String,
    version: String,
    author: String,
}

async fn root() -> (StatusCode, Json<ApiInfo>) {
    let info = ApiInfo {
        name: "R-AUTH API".to_string(),
        version: "1.0.0".to_string(),
        author: "<HormigaDev hormigadev7@gmail.com>".to_string(),
    };

    (StatusCode::OK, Json(info))
}
