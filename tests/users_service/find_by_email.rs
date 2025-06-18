use axum::{Json, http::StatusCode};
use r_auth_api::{
    auth::verify_password, database::models::dto::CreateUserDto, services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Búsqueda exitosa por email existente (con password)
///
#[tokio::test]
async fn test_find_by_email_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let password = "StrongPassword@123".to_string();

    let user_dto = CreateUserDto {
        username: "findbyemail_success".to_string(),
        email: "findbyemail_success@example.com".to_string(),
        password: password.clone(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let result = users_service.find_by_email(&created_user.email).await;

    assert!(
        result.is_ok(),
        "La búsqueda por email debería ser exitosa. Error: {:?}",
        result.unwrap_err()
    );

    let found_user = result.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, "findbyemail_success@example.com");
    assert!(
        found_user.password.is_some(),
        "El campo password debería estar presente en el resultado"
    );
    assert!(
        verify_password(&password, &found_user.password.clone().unwrap()),
        "La contraseña no coincide al verificar"
    );
}

/// ---
///
/// ## Test Case 2: Error por email inexistente (Credenciales inválidas)
///
#[tokio::test]
async fn test_find_by_email_not_found() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let result = users_service.find_by_email("nonexistent@example.com").await;

    assert!(
        result.is_err(),
        "La búsqueda con un email inexistente debería fallar."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Credenciales inválidas"
    );
}
