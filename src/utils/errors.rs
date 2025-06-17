use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use validator::ValidationErrors;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HttpError {
    pub errors: HashMap<String, Vec<String>>,
}

impl HttpError {
    // Crea un HttpError para errores 500 genéricos
    pub fn internal_server_error() -> (StatusCode, Json<Self>) {
        let mut map = HashMap::new();
        map.insert(
            "server".to_string(),
            vec!["Internal Server Error".to_string()],
        );
        let error = HttpError { errors: map };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error))
    }

    pub fn bad_request(message: &str) -> (StatusCode, Json<Self>) {
        Self::error("client", StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: &str) -> (StatusCode, Json<Self>) {
        Self::error("server", StatusCode::NOT_FOUND, message)
    }

    pub fn unauthorized(message: &str) -> (StatusCode, Json<Self>) {
        Self::error("client", StatusCode::UNAUTHORIZED, message)
    }

    pub fn forbbiden(message: &str) -> (StatusCode, Json<Self>) {
        Self::error("client", StatusCode::FORBIDDEN, message)
    }

    pub fn conflict(message: &str) -> (StatusCode, Json<Self>) {
        Self::error("client", StatusCode::CONFLICT, message)
    }

    fn error(key: &str, code: StatusCode, message: &str) -> (StatusCode, Json<Self>) {
        let mut map = HashMap::new();
        map.insert(key.to_string(), vec![message.to_string()]);
        let error = HttpError { errors: map };
        (code, Json(error))
    }

    pub fn errors(e: ValidationErrors) -> (StatusCode, Json<Self>) {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        map.insert("validation".to_string(), vec![format!("{}", e)]);
        let http_err = HttpError { errors: map };
        return (StatusCode::BAD_REQUEST, Json(http_err));
    }
}
