pub mod claims;
pub mod dto;
pub mod entities;

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FindResult<T> {
    pub results: Vec<T>,
    pub total: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OneResult<T> {
    pub result: T,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, IntoParams)]
pub struct FindQuery {
    #[serde(rename = "queryKey")]
    #[validate(length(
        min = 1,
        message = "The search key of the query must have at least 1 characters"
    ))]
    pub query_key: Option<String>,

    #[serde(rename = "queryValue")]
    #[validate(length(
        min = 1,
        message = "The search value of the query must have at least 1 characters"
    ))]
    pub query_value: Option<String>,

    #[validate(range(min = 1, message = "The pagination page must be greather than 1"))]
    pub page: Option<i32>,

    #[validate(range(
        min = 1,
        max = 100,
        message = "The pagination limit must between 1 and 100"
    ))]
    pub limit: Option<i32>,
}
