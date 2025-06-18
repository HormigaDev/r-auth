use axum::{Json, http::StatusCode};
use r_auth_api::{database::models::dto::CreateUserDto, services::UsersService};

use crate::common;

/// ---
///
/// ## Test Case 1: Búsqueda exitosa por ID existente
///
#[tokio::test]
async fn test_find_by_id_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "findbyid_success".to_string(),
        email: "findbyid_success@example.com".to_string(),
        password: "StrongPassword@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let result = users_service.find_by_id(created_user.id).await;

    assert!(
        result.is_ok(),
        "La búsqueda por ID debería ser exitosa. Error: {:?}",
        result.unwrap_err()
    );

    let found_user = result.unwrap();
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.username, "findbyid_success");
    assert_eq!(found_user.email, "findbyid_success@example.com");
}

/// ---
///
/// ## Test Case 2: Error al buscar ID inexistente (Not Found)
///
#[tokio::test]
async fn test_find_by_id_not_found() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let result = users_service.find_by_id(99999).await;

    assert!(
        result.is_err(),
        "La búsqueda de un ID inexistente debería fallar con NotFound."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Usuario no encontrado"
    );
}
