use utoipa::OpenApi;

use crate::{
    ApiInfo,
    database::models::{
        FindQuery, FindResult, OneResult,
        dto::{ChangePasswordDto, CreateUserDto, LoginRequest, LoginResponse, UpdateUserDto},
        entities::user::User,
    },
    utils::{MessageResponse, errors::HttpError},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::users_handler::login,
        crate::handlers::users_handler::create_user,
        crate::handlers::users_handler::get_users,
        crate::handlers::users_handler::get_user,
        crate::handlers::users_handler::get_myinfo,
        crate::handlers::users_handler::update_user,
        crate::handlers::users_handler::update_myself,
        crate::handlers::users_handler::change_password,
        crate::handlers::users_handler::inactive_user,
        crate::handlers::users_handler::inactive_myself,
        crate::handlers::users_handler::delete_user,
        crate::handlers::users_handler::delete_myself,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        CreateUserDto,
        UpdateUserDto,
        ChangePasswordDto,
        FindQuery,
        FindResult<User>,
        OneResult<User>,
        User,
        MessageResponse,
        HttpError,
        ApiInfo
    )),
    tags(
        (name = "Users", description = "Operaciones relacionadas con usuarios")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Define el esquema de seguridad Bearer (JWT)
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

        let mut api_key = ApiKeyValue::new("Authorization");
        api_key.description =
            Some("JWT Authorization header usando el esquema Bearer.".to_string());

        openapi
            .components
            .as_mut()
            .unwrap()
            .security_schemes
            .insert(
                "bearerAuth".to_string(),
                SecurityScheme::ApiKey(ApiKey::Header(api_key)),
            );
    }
}
