use axum::{Json, http::StatusCode};
use r_auth_api::{
    database::models::dto::{CreateUserDto, UpdateUserDto},
    services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Actualización exitosa de username y email
///
#[tokio::test]
async fn test_update_user_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "update_success".to_string(),
        email: "update_success@example.com".to_string(),
        password: "StrongPassword@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario");

    let update_dto = UpdateUserDto {
        id: Some(created_user.id),
        username: Some("updated_username".to_string()),
        email: Some("updated_email@example.com".to_string()),
        permissions: None,
    };

    let result = users_service.update(update_dto).await;

    assert!(
        result.is_ok(),
        "La actualización debería ser exitosa. Error: {:?}",
        result.unwrap_err()
    );

    let updated_user = result.unwrap();
    assert_eq!(updated_user.username, "updated_username");
    assert_eq!(updated_user.email, "updated_email@example.com");
}

/// ---
///
/// ## Test Case 2: Fallo por ID de usuario inválido (None)
///
#[tokio::test]
async fn test_update_user_invalid_id() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let update_dto = UpdateUserDto {
        id: None,
        username: Some("should_fail".to_string()),
        email: Some("should_fail@example.com".to_string()),
        permissions: None,
    };

    let result = users_service.update(update_dto).await;

    assert!(result.is_err(), "La actualización sin ID debería fallar.");

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Id de usuario inválido"
    );
}

/// ---
///
/// ## Test Case 3: Fallo por intento de duplicar username existente
///
#[tokio::test]
async fn test_update_user_duplicate_username() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    // Crear primer usuario
    let _user1 = users_service
        .create(CreateUserDto {
            username: "user1".to_string(),
            email: "user1@example.com".to_string(),
            password: "StrongPwd@123".to_string(),
        })
        .await
        .expect("Fallo al crear user1");

    // Crear segundo usuario
    let user2 = users_service
        .create(CreateUserDto {
            username: "user2".to_string(),
            email: "user2@example.com".to_string(),
            password: "StrongPwd@123".to_string(),
        })
        .await
        .expect("Fallo al crear user2");

    // Intentar actualizar user2 con username de user1
    let update_dto = UpdateUserDto {
        id: Some(user2.id),
        username: Some("user1".to_string()),
        email: None,
        permissions: None,
    };

    let result = users_service.update(update_dto).await;

    assert!(
        result.is_err(),
        "La actualización con username duplicado debería fallar."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Ya existe otro usuario con ese username"
    );
}

/// ---
///
/// ## Test Case 4: Fallo por intento de duplicar email existente
///
#[tokio::test]
async fn test_update_user_duplicate_email() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    // Crear primer usuario
    let _user1 = users_service
        .create(CreateUserDto {
            username: "user1email".to_string(),
            email: "user1email@example.com".to_string(),
            password: "StrongPwd@123".to_string(),
        })
        .await
        .expect("Fallo al crear user1");

    // Crear segundo usuario
    let user2 = users_service
        .create(CreateUserDto {
            username: "user2email".to_string(),
            email: "user2email@example.com".to_string(),
            password: "StrongPwd@123".to_string(),
        })
        .await
        .expect("Fallo al crear user2");

    // Intentar actualizar user2 con email de user1
    let update_dto = UpdateUserDto {
        id: Some(user2.id),
        username: None,
        email: Some("user1email@example.com".to_string()),
        permissions: None,
    };

    let result = users_service.update(update_dto).await;

    assert!(
        result.is_err(),
        "La actualización con email duplicado debería fallar."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Ya existe otro usuario con ese email"
    );
}

/// ---
///
/// ## Test Case 5: Fallo por no enviar campos a actualizar
///
#[tokio::test]
async fn test_update_user_no_fields() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user = users_service
        .create(CreateUserDto {
            username: "nofields".to_string(),
            email: "nofields@example.com".to_string(),
            password: "StrongPwd@123".to_string(),
        })
        .await
        .expect("Fallo al crear usuario");

    let update_dto = UpdateUserDto {
        id: Some(user.id),
        username: None,
        email: None,
        permissions: None,
    };

    let result = users_service.update(update_dto).await;

    assert!(
        result.is_err(),
        "La actualización sin campos debería fallar."
    );

    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "No hay campos para actualizar"
    );
}
