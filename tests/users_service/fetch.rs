use axum::{Json, http::StatusCode};
use r_auth_api::{
    auth::verify_password, database::models::dto::CreateUserDto, services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Fetch exitoso por ID (incluyendo password)
///
#[tokio::test]
async fn test_fetch_user_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let password = "StrongPassword@123".to_string();

    let user_dto = CreateUserDto {
        username: "fetch_success".to_string(),
        email: "fetch_success@example.com".to_string(),
        password: password.clone(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let result = users_service.fetch(created_user.id).await;

    assert!(
        result.is_ok(),
        "El fetch por ID debería ser exitoso. Error: {:?}",
        result.unwrap_err()
    );

    let fetched_user = result.unwrap();
    assert_eq!(fetched_user.id, created_user.id);
    assert_eq!(fetched_user.username, "fetch_success");
    assert_eq!(fetched_user.email, "fetch_success@example.com");
    assert!(
        fetched_user.password.is_some(),
        "El usuario obtenido debe incluir el campo password"
    );
    assert!(
        verify_password(&password, &fetched_user.password.clone().unwrap()),
        "El password almacenado no coincide al verificar"
    );
}

/// ---
///
/// ## Test Case 2: Fetch falla por ID inexistente (Not Found)
///
#[tokio::test]
async fn test_fetch_user_not_found() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let result = users_service.fetch(99999).await;

    assert!(
        result.is_err(),
        "El fetch de un ID inexistente debería fallar."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Usuario no encontrado"
    );
}
