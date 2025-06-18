use axum::{Json, http::StatusCode};
use r_auth_api::{
    database::models::dto::{ChangePasswordDto, CreateUserDto},
    services::UsersService,
};

use crate::common;

/// ---
///
/// ## Test Case 1: Cambio exitoso de contraseña
///
#[tokio::test]
async fn test_change_password_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "change_pwd".to_string(),
        email: "change_pwd@example.com".to_string(),
        password: "OldPassword@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let change_pwd_dto = ChangePasswordDto {
        previous_password: "OldPassword@123".to_string(),
        new_password: "NewPassword@456".to_string(),
    };

    let result = users_service
        .change_password(created_user.id.to_string(), change_pwd_dto)
        .await;

    assert!(
        result.is_ok(),
        "El cambio de contraseña debería ser exitoso"
    );
}

/// ---
///
/// ## Test Case 2: Fallo por ID de usuario inválido (parseo fallido)
///
#[tokio::test]
async fn test_change_password_invalid_user_id() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let change_pwd_dto = ChangePasswordDto {
        previous_password: "AnyPassword@123".to_string(),
        new_password: "NewPassword@456".to_string(),
    };

    let result = users_service
        .change_password("invalid_id".to_string(), change_pwd_dto)
        .await;

    assert!(result.is_err(), "Debería fallar por ID inválido");
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Id de usuario inválido"
    );
}

/// ---
///
/// ## Test Case 3: Fallo cuando la nueva contraseña es igual a la antigua
///
#[tokio::test]
async fn test_change_password_new_equals_old() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "same_pwd".to_string(),
        email: "same_pwd@example.com".to_string(),
        password: "Password@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let change_pwd_dto = ChangePasswordDto {
        previous_password: "Password@123".to_string(),
        new_password: "Password@123".to_string(),
    };

    let result = users_service
        .change_password(created_user.id.to_string(), change_pwd_dto)
        .await;

    assert!(
        result.is_err(),
        "Debería fallar si la nueva contraseña es igual a la antigua"
    );
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "La contraseña nueva no puede ser igual a la contraseña antigua"
    );
}

/// ---
///
/// ## Test Case 4: Fallo por contraseña anterior incorrecta
///
#[tokio::test]
async fn test_change_password_wrong_previous_password() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "wrong_prev".to_string(),
        email: "wrong_prev@example.com".to_string(),
        password: "CorrectPassword@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let change_pwd_dto = ChangePasswordDto {
        previous_password: "WrongPassword@123".to_string(),
        new_password: "NewPassword@456".to_string(),
    };

    let result = users_service
        .change_password(created_user.id.to_string(), change_pwd_dto)
        .await;

    assert!(
        result.is_err(),
        "Debería fallar por contraseña anterior incorrecta"
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
/// ## Test Case 5: Fallo por nueva contraseña débil o inválida (validation falla)
///
#[tokio::test]
async fn test_change_password_weak_new_password() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let user_dto = CreateUserDto {
        username: "weak_new_pwd".to_string(),
        email: "weak_new_pwd@example.com".to_string(),
        password: "OldStrongPwd@123".to_string(),
    };

    let created_user = users_service
        .create(user_dto)
        .await
        .expect("Error creando usuario");

    let change_pwd_dto = ChangePasswordDto {
        previous_password: "OldStrongPwd@123".to_string(),
        new_password: "weakpassword".to_string(),
    };

    let result = users_service
        .change_password(created_user.id.to_string(), change_pwd_dto)
        .await;

    assert!(result.is_err(), "Debería fallar por nueva contraseña débil");
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "La contraseña debe tener al menos: 1 minúscula, 1 mayúscula, 1 número y 1 caracter especial"
    );
}
