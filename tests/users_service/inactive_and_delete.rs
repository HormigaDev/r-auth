use r_auth_api::{database::models::dto::CreateUserDto, services::UsersService};

use crate::common;

/// ---
///
/// ## Test Case 1: Inactivar usuario exitosamente (status = 2)
///
#[tokio::test]
async fn test_inactive_user_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "inactive_user".to_string(),
        email: "inactive_user@example.com".to_string(),
        password: "Password@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let result = users_service.inactive(created_user.id).await;

    assert!(result.is_ok(), "Debería inactivar al usuario exitosamente");
}

/// ---
///
/// ## Test Case 2: Eliminar usuario exitosamente (status = 3)
///
#[tokio::test]
async fn test_delete_user_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "delete_user".to_string(),
        email: "delete_user@example.com".to_string(),
        password: "Password@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let result = users_service.delete(created_user.id).await;

    assert!(
        result.is_ok(),
        "Debería eliminar (marcar como eliminado) al usuario exitosamente"
    );
}
