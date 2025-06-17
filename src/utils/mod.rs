mod db_utils;
pub mod errors;
use axum::{Json, http::StatusCode};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Permissions: i64 {
        const READ_MYSELF = 1 <<0;
        const UPDATE_MYSELF = 1 << 1;
        const DELETE_MYSELF = 1 << 2;

        const ADMIN = 1 << 3;

        const CREATE_USERS = 1 << 4;
        const READ_USERS = 1 << 5;
        const UPDATE_USERS = 1 << 6;
        const DELETE_USERS = 1 << 7;

    }
}

pub const USER_PERMISSIONS: Permissions = Permissions::from_bits_truncate(
    Permissions::READ_MYSELF.bits()
        | Permissions::UPDATE_MYSELF.bits()
        | Permissions::DELETE_MYSELF.bits(),
);

pub use db_utils::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::errors::HttpError;

pub type ApiResult<T> = Result<(StatusCode, Json<T>), ApiError>;
pub type ApiError = (StatusCode, Json<HttpError>);

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}
