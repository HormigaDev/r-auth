use axum::{Json, http::StatusCode};
use r_auth_api::{
    database::models::{FindQuery, dto::CreateUserDto},
    services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Búsqueda exitosa por ID
///
#[tokio::test]
async fn test_find_user_by_id_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    // Crear un usuario de prueba
    let user_dto = CreateUserDto {
        username: "findbyiduser".to_string(),
        email: "findbyid@example.com".to_string(),
        password: "StrongPassword@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    // Buscar por ID
    let find_result = users_service
        .find(FindQuery {
            query_key: Some("id".to_string()),
            query_value: Some(created_user.id.to_string()),
            limit: Some(10),
            page: Some(1),
        })
        .await;

    assert!(
        find_result.is_ok(),
        "La búsqueda por ID debería ser exitosa. Error: {:?}",
        find_result.unwrap_err()
    );

    let result_data = find_result.unwrap();
    assert_eq!(result_data.total, 1);
    let found_user = &result_data.results[0];
    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.username, "findbyiduser");
    assert_eq!(found_user.email, "findbyid@example.com");
}

/// ---
///
/// ## Test Case 2: Búsqueda exitosa por username
///
#[tokio::test]
async fn test_find_user_by_username_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "findbyusername".to_string(),
        email: "findbyusername@example.com".to_string(),
        password: "StrongPassword@123".to_string(),
    };

    users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let find_result = users_service
        .find(FindQuery {
            query_key: Some("username".to_string()),
            query_value: Some("findbyusername".to_string()),
            limit: Some(10),
            page: Some(1),
        })
        .await;

    assert!(
        find_result.is_ok(),
        "La búsqueda por username debería ser exitosa. Error: {:?}",
        find_result.unwrap_err()
    );

    let result_data = find_result.unwrap();
    assert_eq!(result_data.total, 1);
    assert_eq!(result_data.results[0].username, "findbyusername");
}

/// ---
///
/// ## Test Case 3: Búsqueda exitosa por email
///
#[tokio::test]
async fn test_find_user_by_email_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "findbyemail".to_string(),
        email: "findbyemail@example.com".to_string(),
        password: "StrongPassword@123".to_string(),
    };

    users_service
        .create(user_dto)
        .await
        .expect("Fallo al crear usuario de prueba");

    let find_result = users_service
        .find(FindQuery {
            query_key: Some("email".to_string()),
            query_value: Some("findbyemail@example.com".to_string()),
            limit: Some(10),
            page: Some(1),
        })
        .await;

    assert!(
        find_result.is_ok(),
        "La búsqueda por email debería ser exitosa. Error: {:?}",
        find_result.unwrap_err()
    );

    let result_data = find_result.unwrap();
    assert_eq!(result_data.total, 1);
    assert_eq!(result_data.results[0].email, "findbyemail@example.com");
}

/// ---
///
/// ## Test Case 4: Fallo por query_key inválida
///
#[tokio::test]
async fn test_find_user_invalid_key() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let find_result = users_service
        .find(FindQuery {
            query_key: Some("invalid_key".to_string()),
            query_value: Some("value".to_string()),
            limit: Some(10),
            page: Some(1),
        })
        .await;

    assert!(
        find_result.is_err(),
        "La búsqueda con key inválida debería fallar."
    );

    let (status, Json(http_error)) = find_result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        http_error
            .errors
            .get("client")
            .unwrap()
            .first()
            .unwrap()
            .contains("invalid_key: Opción inválida"),
        "Mensaje de error inesperado: {:?}",
        http_error
    );
}

/// ---
///
/// ## Test Case 5: Paginación: segunda página de resultados
///
#[tokio::test]
async fn test_find_user_pagination() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    // Crear varios usuarios
    for i in 1..=15 {
        let user_dto = CreateUserDto {
            username: format!("user_pagination_{}", i),
            email: format!("user_pagination_{}@example.com", i),
            password: "StrongPassword@123".to_string(),
        };

        users_service
            .create(user_dto)
            .await
            .expect("Fallo al crear usuario de paginación");
    }

    let find_result = users_service
        .find(FindQuery {
            query_key: Some("username".to_string()),
            query_value: Some("user_pagination".to_string()),
            limit: Some(10),
            page: Some(2),
        })
        .await;

    assert!(
        find_result.is_ok(),
        "La paginación debería funcionar sin errores."
    );

    let result_data = find_result.unwrap();
    assert_eq!(result_data.total, 15);
    assert_eq!(
        result_data.results.len(),
        5,
        "En la página 2 deberían venir 5 resultados (15 en total, 10 en página 1, 5 en página 2)"
    );
}
