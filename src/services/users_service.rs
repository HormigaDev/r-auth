use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Json, http::StatusCode};
use tracing::error;
use validator::Validate;

use crate::{
    app::dto::{ChangePasswordDto, CreateUserDto, LoginRequest, UpdateUserDto},
    auth::{
        generate_jwt, hash_password,
        tfa::{Tfa, TfaStatus},
        validate_password, verify_password,
    },
    database::{
        connection::PgPool,
        models::{FindQuery, FindResult, claims::Claims, entities::user::User},
    },
    utils::{
        ApiError, USER_PERMISSIONS, check_duplicate, commit_transaction, ensure_row_exists,
        errors::HttpError, get_pg_client, get_transaction, map_db_error, validate_dto,
        validate_query_key,
    },
};

pub struct UsersService {
    pool: PgPool,
}

impl UsersService {
    pub fn new(pool: &PgPool) -> Self {
        UsersService { pool: pool.clone() }
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<User, (StatusCode, Json<HttpError>)> {
        validate_dto(&dto)?;
        validate_password(&dto.password)?;

        let mut client = get_pg_client(&self.pool).await?;
        let tx = get_transaction(&mut client).await?;

        let exists = match tx
            .query_opt(
                "SELECT 1 FROM users WHERE username = $1 OR email = $2 LIMIT 1",
                &[&dto.username, &dto.email],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!(error= %e, "Error verificando la existencia previa del usuario");
                return Err(HttpError::internal_server_error());
            }
        };

        if let Some(_) = exists {
            return Err(HttpError::conflict(
                "Ya existe un usuario con ese nombre de usuario o email",
            ));
        }

        let password_hash = hash_password(dto.password.as_str()).map_err(|e| {
            error!("Error hasheando password: {}", e);
            HttpError::internal_server_error()
        })?;

        let row = tx
            .query_one(
                r#"
                    INSERT INTO users (username, email, password, permissions, status)
                    VALUES ($1, $2, $3, $4, 1)
                    RETURNING id;
                "#,
                &[
                    &dto.username,
                    &dto.email,
                    &password_hash,
                    &USER_PERMISSIONS.bits(),
                ],
            )
            .await
            .map_err(|e| map_db_error("Error ejecutando insert de usuario", e))?;

        commit_transaction(tx, "Error haciendo commit de transacción").await?;

        let id: i64 = row.get("id");
        let user = self.find_by_id(id).await?;

        Ok(user)
    }

    pub async fn find(
        &self,
        dto: FindQuery,
    ) -> Result<FindResult<User>, (StatusCode, Json<HttpError>)> {
        validate_dto(&dto)?;

        let client = get_pg_client(&self.pool).await?;

        let key = dto.query_key.as_deref().unwrap_or("id");
        let value = dto.query_value.as_deref().unwrap_or("");
        validate_query_key(&["id", "username", "email"], key)?;

        let limit = dto.limit.unwrap_or(100);
        let page = dto.page.unwrap_or(1);
        let offset = limit * (page - 1);
        let filter_value = format!("%{}%", value);

        let count_query = format!("SELECT COUNT(*) FROM users WHERE {}::text ILIKE $1", key);
        let count_row = client
            .query_one(&count_query, &[&filter_value])
            .await
            .map_err(|e| map_db_error("Error ejecutando query de conteo", e))?;
        let total_count: i64 = count_row.get(0);

        let data_query = format!(
            r#"
                SELECT 
                    id,
                    username,
                    email,
                    status,
                    created_at,
                    updated_at
                FROM users
                WHERE {}::text ILIKE $1
                LIMIT {} OFFSET {}
            "#,
            key, limit, offset
        );

        let result = client
            .query(&data_query, &[&filter_value])
            .await
            .map_err(|e| map_db_error("Error ejecutando query de búsqueda", e))?;

        let users: Vec<User> = result
            .iter()
            .map(User::from_row_without_perms)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!("Error mapeando resultados: {}", e);
                HttpError::internal_server_error()
            })?;

        Ok(FindResult {
            results: users,
            total: total_count as u64,
        })
    }

    pub async fn find_by_id(&self, id: i64) -> Result<User, (StatusCode, Json<HttpError>)> {
        let client = get_pg_client(&self.pool).await?;

        let row_opt = match client
            .query_opt(
                r#"
                    SELECT
                        id,
                        username,
                        email,
                        permissions,
                        status,
                        use_tfa,
                        tfa_secret,
                        created_at,
                        updated_at
                    FROM users WHERE id = $1 LIMIT 1
                "#,
                &[&id],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Error consultando el usuario por id");
                return Err(HttpError::internal_server_error());
            }
        };

        let row = match row_opt {
            Some(r) => r,
            None => return Err(HttpError::not_found("Usuario no encontrado")),
        };

        User::from_row(&row)
            .map_err(|e| map_db_error("Error parseando usuario desde row", e.as_ref()))
    }

    pub async fn fetch(&self, id: i64) -> Result<User, (StatusCode, Json<HttpError>)> {
        let client = get_pg_client(&self.pool).await?;

        let rows = client
            .query(
                r#"
                    SELECT 
                        id,
                        username,
                        email,
                        password,
                        permissions,
                        status,
                        use_tfa,
                        tfa_secret,
                        created_at,
                        updated_at
                    FROM users WHERE id = $1 LIMIT 1
                "#,
                &[&id],
            )
            .await
            .map_err(|e| map_db_error("Error ejecutando query de fetch", e))?;

        let row = match rows.first() {
            Some(r) => r,
            None => return Err(HttpError::not_found("Usuario no encontrado")),
        };

        User::from_row_with_password(row)
            .map_err(|e| map_db_error("Error parseando usuario con password", e.as_ref()))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<User, (StatusCode, Json<HttpError>)> {
        let client = get_pg_client(&self.pool).await?;

        let row_opt = match client
            .query_opt(
                r#"
                    SELECT 
                        id,
                        username,
                        email,
                        password,
                        permissions,
                        status,
                        use_tfa,
                        tfa_secret,
                        created_at,
                        updated_at
                    FROM users WHERE email = $1
                "#,
                &[&email],
            )
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Error consultando el usuario por email");
                return Err(HttpError::internal_server_error());
            }
        };

        let row = match row_opt {
            Some(r) => r,
            None => return Err(HttpError::unauthorized("Credenciales inválidas")),
        };

        User::from_row_with_password(&row).map_err(|e| {
            error!(error = %e, "Error parseando usuario con password por email");
            HttpError::unauthorized("Credenciales inválidas")
        })
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<String, (StatusCode, Json<HttpError>)> {
        validate_dto(&dto)?;

        let user = self.find_by_email(dto.email.as_str()).await?;

        let hash = match user.password {
            Some(pwd) => pwd,
            None => return Err(HttpError::unauthorized("Credenciales inválidas")),
        };

        if !verify_password(&dto.password, &hash) {
            error!(
                "Fallo de verificación de password para el usuario: {}",
                user.id
            );
            return Err(HttpError::unauthorized("Credenciales inválidas"));
        }

        let exp_minutes = 60;
        let claims = Claims::new(user.id.to_string(), exp_minutes);
        generate_jwt(claims).map_err(|e| {
            error!("Error generando JWT: {}", e);
            HttpError::internal_server_error()
        })
    }

    pub async fn update(&self, dto: UpdateUserDto) -> Result<User, (StatusCode, Json<HttpError>)> {
        validate_dto(&dto)?;
        let mut client = get_pg_client(&self.pool).await?;
        let id = match &dto.id {
            Some(i) => i,
            None => return Err(HttpError::unauthorized("Id de usuario inválido")),
        };
        ensure_row_exists(&client, "users", "id", &id, "Usuario no encontrado").await?;
        let tx = get_transaction(&mut client).await?;

        if let Some(ref username) = dto.username {
            check_duplicate(
                &tx,
                "username",
                username,
                *id,
                "Ya existe otro usuario con ese username",
            )
            .await?;
        }

        if let Some(ref email) = dto.email {
            check_duplicate(
                &tx,
                "email",
                email,
                *id,
                "Ya existe otro usuario con ese email",
            )
            .await?;
        }

        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut set_clauses = Vec::new();
        let mut idx = 1;

        if let Some(ref username) = dto.username {
            set_clauses.push(format!("username = ${}", idx));
            params.push(username);
            idx += 1;
        }

        if let Some(ref email) = dto.email {
            set_clauses.push(format!("email = ${}", idx));
            params.push(email);
            idx += 1;
        }

        if let Some(permissions) = &dto.permissions {
            set_clauses.push(format!("permissions = ${}", idx));
            params.push(permissions);
            idx += 1;
        }

        if set_clauses.is_empty() {
            return Err(HttpError::bad_request("No hay campos para actualizar"));
        }

        params.push(&dto.id);

        let sql = format!(
            r#"
                UPDATE users SET {} WHERE id = ${}
                RETURNING
                    id,
                    username,
                    email,
                    permissions,
                    status,
                    use_tfa,
                    tfa_secret,
                    created_at,
                    updated_at
            "#,
            set_clauses.join(", "),
            idx
        );

        let row_opt = match tx.query_opt(&sql, &params).await {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Error actualizando el usuario");
                return Err(HttpError::internal_server_error());
            }
        };

        let row = match row_opt {
            Some(r) => r,
            None => return Err(HttpError::not_found("Usuario no encontrado")),
        };

        commit_transaction(tx, "Error haciendo commit").await?;

        User::from_row(&row)
            .map_err(|e| map_db_error("Error mapeando user actualizado", e.as_ref()))
    }

    pub async fn change_password(
        &self,
        user_id: String,
        dto: ChangePasswordDto,
    ) -> Result<(), (StatusCode, Json<HttpError>)> {
        dto.validate().map_err(|e| HttpError::errors(e))?;
        let id: i64 = user_id.parse().map_err(|e| {
            error!(error = %e, "Error al parsear id del usuario");
            HttpError::bad_request("Id de usuario inválido")
        })?;

        if dto.previous_password == dto.new_password {
            return Err(HttpError::bad_request(
                "La contraseña nueva no puede ser igual a la contraseña antigua",
            ));
        }

        let user = self.fetch(id).await?;
        if user.password.is_none() {
            return Err(HttpError::internal_server_error());
        }
        let hash = match user.password {
            Some(pwd) => pwd,
            None => return Err(HttpError::unauthorized("Credenciales inválidas")),
        };

        if !verify_password(&dto.previous_password, &hash) {
            error!(
                "Fallo de verificación de password para el usuario: {}",
                user.id
            );
            return Err(HttpError::unauthorized("Credenciales inválidas"));
        }

        validate_password(&dto.new_password)?;

        let statement = r#"
            UPDATE users
            SET password = $1
            WHERE id = $2
        "#;
        let hash = hash_password(&dto.new_password).map_err(|e| {
            map_db_error(
                format!("Error al hashear la contraseña del usuario {}", &id).as_str(),
                e,
            )
        })?;

        let client = get_pg_client(&self.pool).await?;
        match client.query_opt(statement, &[&hash, &id]).await {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Error al actualizar el usuario");
                return Err(HttpError::internal_server_error());
            }
        };

        Ok(())
    }

    async fn set_user_status(&self, id: i64, status: i32) -> Result<(), ApiError> {
        let client = get_pg_client(&self.pool).await?;
        ensure_row_exists(&client, "users", "id", &id, "Usuario no encontrado").await?;
        let statement = r#"
            UPDATE users
            SET status = $1
            WHERE id = $2
        "#;
        match client.query_opt(statement, &[&status, &id]).await {
            Ok(r) => r,
            Err(e) => {
                error!(error = %e, "Error al cambiar el status del usuario");
                return Err(HttpError::internal_server_error());
            }
        };

        Ok(())
    }

    pub async fn inactive(&self, id: i64) -> Result<(), ApiError> {
        self.set_user_status(id, 2).await
    }

    pub async fn delete(&self, id: i64) -> Result<(), ApiError> {
        self.set_user_status(id, 3).await
    }

    pub async fn enable_tfa(&self, id: i64) -> Result<Tfa, ApiError> {
        let client = get_pg_client(&self.pool).await?;
        ensure_row_exists(&client, "users", "id", &id, "Usuario no encontrado").await?;

        let secret = Tfa::generate_secret().map_err(|e| {
            error!(error = %e, "Error al generar el secreto TFA");
            HttpError::internal_server_error()
        })?;

        let statement = r#"
            UPDATE users
            SET
                tfa_status = $1,
                tfa_secret = $2
            WHERE id = $3
        "#;

        client
            .execute(statement, &[&TfaStatus::Pending, &secret, &id])
            .await
            .map_err(|e| {
                error!(error = %e, "Error al guardar el secreto TFA en base de datos");
                HttpError::internal_server_error()
            })?;

        let tfa = Tfa::new(&secret);

        Ok(tfa)
    }

    pub async fn confirm_tfa(&self, id: i64, code: u32) -> Result<bool, ApiError> {
        let client = get_pg_client(&self.pool).await?;
        ensure_row_exists(&client, "users", "id", &id, "Usuario no encontrado").await?;

        let query = r#"
            SELECT tfa_status, tfa_secret FROM users WHERE id = $1
        "#;

        let row = client.query_opt(query, &[&id]).await.map_err(|e| {
            error!(error = %e, "Error al consultar estado TFA");
            HttpError::internal_server_error()
        })?;

        let row = match row {
            Some(r) => r,
            None => return Err(HttpError::not_found("Usuario no encontrado")),
        };

        let tfa_status: TfaStatus = row.try_get("tfa_status").map_err(|e| {
            error!(error = %e, "Error al leer tfa_status");
            HttpError::internal_server_error()
        })?;

        let tfa_secret: String = row.try_get("tfa_secret").map_err(|e| {
            error!(error = %e, "Error al leer tfa_secret");
            HttpError::internal_server_error()
        })?;

        if tfa_status != TfaStatus::Pending {
            return Err(HttpError::forbbiden(
                "Autenticación de dos factores ya configurada",
            ));
        }

        let tfa = Tfa::new(&tfa_secret);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| {
                error!(error = %e, "Error obteniendo el tiempo actual");
                HttpError::internal_server_error()
            })?
            .as_secs();

        // Verificación TOTP con tolerancia de ±1 paso (30s por paso)
        let confirmed = (-1..=1).any(|offset| {
            let ts = timestamp.wrapping_add((offset * 30) as u64);
            tfa.verify_totp(code, ts).unwrap_or(false)
        });

        if confirmed {
            let update_stmt = r#"
                UPDATE users SET tfa_status = $1 WHERE id = $2
            "#;

            client
                .execute(update_stmt, &[&TfaStatus::Enabled, &id])
                .await
                .map_err(|e| {
                    error!(error = %e, "Error al actualizar estado TFA a 'Enabled'");
                    HttpError::internal_server_error()
                })?;
        }

        Ok(confirmed)
    }
}
