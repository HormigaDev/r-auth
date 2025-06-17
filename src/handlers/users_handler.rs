use std::sync::Arc;

use crate::{
    AppState,
    auth::AuthenticatedClaims,
    database::models::{
        FindQuery, FindResult, OneResult,
        dto::{ChangePasswordDto, CreateUserDto, LoginRequest, LoginResponse, UpdateUserDto},
        entities::user::User,
    },
    services::UsersService,
    utils::{ApiError, ApiResult, MessageResponse, Permissions, errors::HttpError},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
};

pub fn users_routes(state: AppState) -> Router {
    let service = state.users_service.clone();
    Router::new()
        .route("/", post(create_user))
        .route("/", get(get_users))
        .route("/me", get(get_myinfo))
        .route("/{id}", get(get_user))
        .route("/login", post(login))
        .route("/me", patch(update_myself))
        .route("/{id}", patch(update_user))
        .route("/change-password", put(change_password))
        .route("/inactive/me", put(inactive_myself))
        .route("/inactive/{id}", put(inactive_user))
        .route("/me", delete(delete_myself))
        .route("/{id}", delete(delete_user))
        .with_state(service)
}

#[utoipa::path(
    post,
    path = "/users/login",
    tag = "Users",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login exitoso", body = LoginResponse),
        (status = 401, description = "Credenciales inválidas", body = HttpError)
    )
)]
pub async fn login(
    State(service): State<Arc<UsersService>>,
    Json(payload): Json<LoginRequest>,
) -> ApiResult<LoginResponse> {
    let token = service.login(payload).await?;
    Ok((StatusCode::OK, Json(LoginResponse { token })))
}

#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "Usuario creado", body = OneResult<User>),
        (status = 400, description = "Error de validación", body = HttpError),
        (status = 409, description = "Usuario ya existe", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn create_user(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Json(payload): Json<CreateUserDto>,
) -> ApiResult<OneResult<User>> {
    claims.require_permission(Permissions::CREATE_USERS)?;
    let user = service.create(payload).await?;
    Ok((StatusCode::CREATED, Json(OneResult { result: user })))
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    params(FindQuery),
    responses(
        (status = 200, description = "Lista de usuarios", body = FindResult<User>),
        (status = 400, description = "Parámetros de búsqueda inválidos", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_users(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Query(dto): Query<FindQuery>,
) -> ApiResult<FindResult<User>> {
    claims.require_permission(Permissions::READ_USERS)?;
    let users = service.find(dto).await?;
    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "ID del usuario")
    ),
    responses(
        (status = 200, description = "Usuario encontrado", body = OneResult<User>),
        (status = 404, description = "Usuario no encontrado", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_user(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Path(id): Path<i64>,
) -> ApiResult<OneResult<User>> {
    claims.require_permission(Permissions::READ_USERS)?;
    let user = service.find_by_id(id).await?;
    Ok((StatusCode::OK, Json(OneResult { result: user })))
}

#[utoipa::path(
    get,
    path = "/users/me",
    tag = "Users",
    responses(
        (status = 200, description = "Información del usuario autenticado", body = OneResult<User>),
        (status = 400, description = "ID de usuario inválido", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn get_myinfo(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
) -> ApiResult<OneResult<User>> {
    claims.require_permission(Permissions::READ_MYSELF)?;
    let id: i64 = claims
        .user_id
        .parse()
        .map_err(|_| HttpError::bad_request("Id de usuario inválido"))?;
    let user = service.find_by_id(id).await?;
    Ok((StatusCode::OK, Json(OneResult { result: user })))
}

#[utoipa::path(
    patch,
    path = "/users/{id}",
    tag = "Users",
    request_body = UpdateUserDto,
    params(
        ("id" = i64, Path, description = "ID del usuario a actualizar")
    ),
    responses(
        (status = 200, description = "Usuario actualizado", body = OneResult<User>),
        (status = 400, description = "Datos inválidos", body = HttpError),
        (status = 404, description = "Usuario no encontrado", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn update_user(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Path(id): Path<i64>,
    Json(mut payload): Json<UpdateUserDto>,
) -> ApiResult<OneResult<User>> {
    claims.require_permission(Permissions::UPDATE_USERS)?;
    payload.id = Some(id);
    let user = service.update(payload).await?;
    Ok((StatusCode::OK, Json(OneResult { result: user })))
}

#[utoipa::path(
    patch,
    path = "/users/me",
    tag = "Users",
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "Usuario autenticado actualizado", body = OneResult<User>),
        (status = 400, description = "Datos inválidos", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn update_myself(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Json(mut payload): Json<UpdateUserDto>,
) -> ApiResult<OneResult<User>> {
    claims.require_permission(Permissions::UPDATE_MYSELF)?;
    let id: i64 = claims
        .user_id
        .parse()
        .map_err(|_| HttpError::bad_request("Id de usuario inválido"))?;
    payload.id = Some(id);
    let user = service.update(payload).await?;
    Ok((StatusCode::OK, Json(OneResult { result: user })))
}

#[utoipa::path(
    put,
    path = "/users/change-password",
    tag = "Users",
    request_body = ChangePasswordDto,
    responses(
        (status = 200, description = "Contraseña cambiada correctamente", body = MessageResponse),
        (status = 400, description = "Datos inválidos", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn change_password(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Json(payload): Json<ChangePasswordDto>,
) -> ApiResult<MessageResponse> {
    claims.require_permission(Permissions::UPDATE_MYSELF)?;
    service.change_password(claims.user_id, payload).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Contraseña actualizada correctamente".to_string(),
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/users/inactive/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "ID del usuario a inactivar")
    ),
    responses(
        (status = 200, description = "Usuario inactivado", body = MessageResponse),
        (status = 404, description = "Usuario no encontrado", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn inactive_user(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Path(id): Path<i64>,
) -> ApiResult<MessageResponse> {
    claims.require_permission(Permissions::UPDATE_USERS)?;
    service.inactive(id).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Usuario inactivado correctamente".to_string(),
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/users/inactive/me",
    tag = "Users",
    responses(
        (status = 200, description = "Usuario autenticado inactivado", body = MessageResponse),
        (status = 400, description = "ID de usuario inválido", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn inactive_myself(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
) -> ApiResult<MessageResponse> {
    claims.require_permission(Permissions::UPDATE_MYSELF)?;
    let id: i64 = claims
        .user_id
        .parse()
        .map_err(|_| HttpError::bad_request("Id de usuario inválido"))?;
    service.inactive(id).await?;
    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Usuario inactivado correctamente".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "ID del usuario a eliminar")
    ),
    responses(
        (status = 204, description = "Usuario eliminado correctamente"),
        (status = 404, description = "Usuario no encontrado", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_user(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    claims.require_permission(Permissions::DELETE_USERS)?;
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/users/me",
    tag = "Users",
    responses(
        (status = 204, description = "Usuario autenticado eliminado"),
        (status = 400, description = "ID inválido", body = HttpError)
    ),
    security(("bearerAuth" = []))
)]
pub async fn delete_myself(
    AuthenticatedClaims(claims): AuthenticatedClaims,
    State(service): State<Arc<UsersService>>,
) -> Result<StatusCode, ApiError> {
    claims.require_permission(Permissions::DELETE_MYSELF)?;
    let id: i64 = claims
        .user_id
        .parse()
        .map_err(|_| HttpError::bad_request("Id de usuario inválido"))?;
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
