use deadpool_postgres::{Client, Object, Transaction};
use tokio_postgres::types::ToSql;
use tracing::error;

use crate::{
    database::connection::PgPool,
    utils::{ApiError, errors::HttpError},
};
use axum::{Json, http::StatusCode};
use validator::Validate;

pub async fn get_pg_client(pool: &PgPool) -> Result<Object, (StatusCode, Json<HttpError>)> {
    pool.get().await.map_err(|e| {
        error!(error = %e, "Error obteniendo conexión a Postgres");
        HttpError::internal_server_error()
    })
}

pub async fn get_transaction(
    client: &mut Object,
) -> Result<Transaction<'_>, (StatusCode, Json<HttpError>)> {
    client.transaction().await.map_err(|e| {
        error!(error = %e, "Error iniciando transacción");
        HttpError::internal_server_error()
    })
}

pub fn validate_dto<T: Validate>(dto: &T) -> Result<(), (StatusCode, Json<HttpError>)> {
    dto.validate().map_err(|e| HttpError::errors(e))
}

pub async fn check_duplicate(
    tx: &Transaction<'_>,
    column: &str,
    value: &(dyn ToSql + Sync),
    exclude_id: i64,
    error_msg: &str,
) -> Result<(), (StatusCode, Json<HttpError>)> {
    let query = format!("SELECT 1 FROM users WHERE {} = $1 AND id != $2", column);
    let exists = tx
        .query_opt(&query, &[value, &exclude_id])
        .await
        .map_err(|e| {
            error!(error = %e, "Error verificando duplicado en {}", column);
            HttpError::internal_server_error()
        })?;

    if exists.is_some() {
        return Err(HttpError::conflict(error_msg));
    }

    Ok(())
}

pub async fn commit_transaction(
    tx: Transaction<'_>,
    context: &str,
) -> Result<(), (StatusCode, Json<HttpError>)> {
    tx.commit().await.map_err(|e| {
        error!(error = %e, "{}", context);
        HttpError::internal_server_error()
    })
}

pub fn map_db_error(message: &str, e: impl std::error::Error) -> (StatusCode, Json<HttpError>) {
    error!(error = %e, message);
    HttpError::internal_server_error()
}

pub async fn ensure_row_exists(
    client: &Client,
    table: &str,
    column: &str,
    value: &(dyn ToSql + Sync),
    not_found_msg: &str,
) -> Result<(), ApiError> {
    let statement = format!(
        r#"
            SELECT 1 FROM {} WHERE {} = $1
        "#,
        table, column
    );

    let row = client
        .query_opt(&statement, &[value])
        .await
        .map_err(|e| map_db_error("Error al verificar la existencia del registro", e))?;
    match row {
        Some(_) => Ok(()),
        None => return Err(HttpError::not_found(not_found_msg)),
    }
}

pub fn validate_query_key(allowed_keys: &[&str], key: &str) -> Result<(), ApiError> {
    if !allowed_keys.contains(&key) {
        return Err(HttpError::bad_request(&format!("{}: Opción inválida", key)));
    }

    Ok(())
}
