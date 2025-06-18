use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateUserDto {
    #[validate(
        length(
            min = 3,
            max = 100,
            message = "The username must be between 3 and 100 characters"
        ),
        regex(
            path = "*USERNAME_RE",
            message = "Username contains invalid characters"
        )
    )]
    pub username: String,

    #[validate(email(message = "El correo electrónico es obligatorio"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 64,
        message = "Password must be between 8 and 64 characters"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpdateUserDto {
    #[serde(skip_deserializing)]
    pub id: Option<i64>,

    #[validate(
        length(
            min = 3,
            max = 100,
            message = "The username must be between 3 and 100 characters"
        ),
        regex(
            path = "*USERNAME_RE",
            message = "Username contains invalid characters"
        )
    )]
    pub username: Option<String>,

    #[validate(email(message = "The user mail is required"))]
    pub email: Option<String>,

    pub permissions: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordDto {
    #[serde(rename = "previousPassword")]
    pub previous_password: String,

    #[serde(rename = "newPassword")]
    #[validate(length(
        min = 8,
        max = 64,
        message = "La nueva contraseña debe tener entre 8 y 64 caracteres"
    ))]
    pub new_password: String,
}
