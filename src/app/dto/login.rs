use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[validate(
        email(message = "El email es obligatorio"),
        length(
            max = 100,
            message = "La longitud máxima del email es de 100 caracteres"
        )
    )]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 64,
        message = "Password must be between 8 and 64 characters"
    ))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
}
