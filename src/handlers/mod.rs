pub mod users_handler;

use axum::Router;

use crate::AppState;

pub fn api_routes(state: AppState) -> Router {
    Router::new().nest("/users", users_handler::users_routes(state))
}
