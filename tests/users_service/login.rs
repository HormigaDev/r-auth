use axum::{Json, http::StatusCode};
use r_auth_api::{
    database::models::dto::{CreateUserDto, LoginRequest},
    services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Login exitoso con credenciales válidas
///
#[tokio::test]
async fn test_login_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let password = "StrongPassword@123".to_string();

    let user_dto = CreateUserDto {
        username: "login_success".to_string(),
        email: "login_success@example.com".to_string(),
        password: password.clone(),
    };

    users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let login_request = LoginRequest {
        email: "login_success@example.com".to_string(),
        password: password.clone(),
    };

    let result = users_service.login(login_request).await;

    assert!(
        result.is_ok(),
        "El login debería ser exitoso. Error: {:?}",
        result.unwrap_err()
    );

    let token = result.unwrap();
    assert!(
        !token.is_empty(),
        "El token JWT devuelto no debería estar vacío"
    );
}

/// ---
///
/// ## Test Case 2: Error por email inexistente (Credenciales inválidas)
///
#[tokio::test]
async fn test_login_email_not_found() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "AnyPassword123!".to_string(),
    };

    let result = users_service.login(login_request).await;

    assert!(
        result.is_err(),
        "El login con email inexistente debería fallar"
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Credenciales inválidas"
    );
}

/// ---
///
/// ## Test Case 3: Error por contraseña incorrecta
///
#[tokio::test]
async fn test_login_wrong_password() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "login_wrong_pwd".to_string(),
        email: "login_wrong_pwd@example.com".to_string(),
        password: "CorrectPassword@123".to_string(),
    };

    users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let login_request = LoginRequest {
        email: "login_wrong_pwd@example.com".to_string(),
        password: "WrongPassword!".to_string(),
    };

    let result = users_service.login(login_request).await;

    assert!(
        result.is_err(),
        "El login con contraseña incorrecta debería fallar"
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Credenciales inválidas"
    );
}
