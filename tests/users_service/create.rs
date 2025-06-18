use crate::common;
use axum::{Json, http::StatusCode};
use r_auth_api::{
    auth::verify_password, database::models::dto::CreateUserDto, services::UsersService,
    utils::get_pg_client,
};

/// ---
///
/// ## Test Case 1: Creación exitosa de un nuevo usuario
///
#[tokio::test]
async fn test_create_user_success() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let password = "SecurePassword@123".to_string();

    let new_user_dto = CreateUserDto {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
    };

    let result = users_service.create(new_user_dto.clone()).await;

    assert!(
        result.is_ok(),
        "La creación del usuario debería ser exitosa. Error: {:?}",
        result.unwrap_err()
    );

    let created_user = result.unwrap();
    assert_eq!(created_user.username, username.clone());
    assert_eq!(created_user.email, email.clone());
    assert!(
        created_user.id > 0,
        "El ID del usuario debería ser mayor que 0"
    );
    assert!(
        created_user.password.is_none(),
        "La contraseña no debería estar presente en la respuesta al usuario (por seguridad)"
    );

    let client = get_pg_client(pool)
        .await
        .expect("Fallo al obtener el cliente de la base de pruebas.");
    let row = client
        .query_one(
            r#"
            SELECT username, email, password FROM users WHERE id = $1
        "#,
            &[&created_user.id],
        )
        .await
        .expect("Fallo al consultar el usuario creado en la base de datos de pruebas");

    assert_eq!(row.get::<_, String>("username"), username.clone());
    assert_eq!(row.get::<_, String>("email"), email.clone());

    let password_hash: String = row.get("password");
    assert!(
        !password_hash.is_empty(),
        "El hash de la contraseña no debería estar vacío en la base de datos."
    );
    assert!(
        verify_password(&password, &password_hash),
        "La contraseña almacenada no coincide con la original."
    )
}

/// ---
///
/// ## Test Case 2: Intentar crear un usuario con un nombre de usuario duplicado
///
#[tokio::test]
async fn test_create_user_duplicate_username() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let username = "testuser_existing".to_string();
    let email = "testexisting@example.com".to_string();
    let password = "SecurePassword@123!".to_string();

    let first_user_dto = CreateUserDto {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
    };

    users_service
        .create(first_user_dto.clone())
        .await
        .expect("Fallo al crear usuario de pruebas");

    let duplicate_user_dto = CreateUserDto {
        username: username.clone(),
        email: "anotheremail@example.com".to_string(),
        password: password.clone(),
    };

    let result = users_service.create(duplicate_user_dto).await;

    assert!(
        result.is_err(),
        "La creación del usuario debería fallar debido a un nombre de usuario duplicado."
    );
    let (status, Json(http_error)) = result.unwrap_err();

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Ya existe un usuario con ese nombre de usuario o email"
    );
}

/// ---
///
/// ## Test Case 3: Intentar crear un usuario con un email duplicado
///
#[tokio::test]
async fn test_create_user_duplicate_email() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let username = "testuser_existing2".to_string();
    let email = "testexisting2@example.com".to_string();
    let password = "SecurePassword@123!".to_string();

    let first_user_dto = CreateUserDto {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
    };

    users_service
        .create(first_user_dto.clone())
        .await
        .expect("Fallo al crear usuario de pruebas");

    let duplicate_user_dto = CreateUserDto {
        username: "anotheruser2".to_string(),
        email: email.clone(),
        password: password.clone(),
    };

    let result = users_service.create(duplicate_user_dto).await;

    assert!(
        result.is_err(),
        "La creación del usuario debería fallar debido a un eail duplicado."
    );
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "Ya existe un usuario con ese nombre de usuario o email"
    );
}

/// ---
///
/// ## Test Case 4: Intentar crear un usuario con una contraseña débil
///
#[tokio::test]
async fn test_create_user_weak_password() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let username = "weakpwduser".to_string();
    let email = "weaktest@example.com".to_string();
    let password = "shortpwd".to_string();

    let user_dto = CreateUserDto {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
    };

    let result = users_service.create(user_dto).await;

    assert!(
        result.is_err(),
        "La creación del usuario debería fallar por contraseña débil."
    );
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    println!("{:?}", http_error);
    assert_eq!(
        http_error.errors.get("client").unwrap().first().unwrap(),
        "La contraseña debe tener al menos: 1 minúscula, 1 mayúscula, 1 número y 1 caracter especial"
    );
}

/// ---
///
/// ## Test Case 5: Intentar crear un usuario con DTO inválido (ej. email mal formateado)
///
#[tokio::test]
async fn test_create_user_invalid_dto() {
    let pool = common::get_test_pool();
    common::setup_test_environment(pool).await;
    let users_service = UsersService::new(pool);

    let username = "invalidemail".to_string();
    let email = "invalid-email".to_string();
    let password = "ValidPassword@123".to_string();

    let user_dto = CreateUserDto {
        username: username.clone(),
        email: email.clone(),
        password: password.clone(),
    };

    let result = users_service.create(user_dto).await;

    assert!(
        result.is_err(),
        "La creación del usuario debería fallar por DTO inválido."
    );
    let (status, Json(http_error)) = result.unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        http_error
            .errors
            .get("validation")
            .unwrap()
            .first()
            .unwrap(),
        "email: El correo electrónico es obligatorio"
    );
}
