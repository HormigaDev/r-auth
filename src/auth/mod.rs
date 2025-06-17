use crate::{
    config::get_config,
    database::{
        connection::GLOBAL_DB_POOL,
        models::{claims::Claims, entities::user::User},
    },
    utils::errors::HttpError,
};
use argon2::{self, Config, Variant, Version};
use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use fancy_regex::Regex;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{RngCore, rng};
use tracing::error;

pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let config = get_config();

    let mut salt = [0u8; 16];
    rng().fill_bytes(&mut salt);

    let pwd_config = Config {
        variant: Variant::Argon2i,
        version: Version::Version13,
        mem_cost: config.password.mem_cost,
        time_cost: config.password.time_cost,
        lanes: config.password.lanes,
        secret: &[],
        ad: &[],
        hash_length: config.password.hash_length,
    };

    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &pwd_config)?;
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match argon2::verify_encoded(&hash, password.as_bytes()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            false
        }
    }
}

pub fn generate_jwt(claims: Claims) -> Result<String, jsonwebtoken::errors::Error> {
    let config = get_config();

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(config.auth.secret.as_ref());
    encode(&header, &claims, &encoding_key)
}

pub fn decode_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let config = get_config();

    let decoding_key = DecodingKey::from_secret(config.auth.secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

pub struct AuthenticatedClaims(pub Claims);

impl<S> FromRequestParts<S> for AuthenticatedClaims
where
    S: Send + Sync + 'static,
{
    type Rejection = (StatusCode, Json<HttpError>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .filter(|s| s.starts_with("Bearer "))
            .map(|s| s.trim_start_matches("Bearer ").to_string())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Token de autorización faltante o inválido",
            ))
            .map_err(|_| HttpError::unauthorized("Token faltante o inválido"))?;

        match decode_jwt(&auth_header) {
            Ok(mut claims) => {
                let pool = match GLOBAL_DB_POOL.get() {
                    Some(p) => p,
                    None => return Err(HttpError::internal_server_error()),
                };
                let client = pool
                    .get()
                    .await
                    .map_err(|_| HttpError::internal_server_error())?;

                let sql = r#"
                    SELECT
                        id,
                        username,
                        email,
                        permissions,
                        status,
                        created_at,
                        updated_at
                    FROM users WHERE id = $1
                "#;
                let id: i64 = claims
                    .user_id
                    .parse()
                    .map_err(|_| HttpError::bad_request("Id de usuario inválido"))?;
                let row = match client.query_opt(sql, &[&id]).await {
                    Ok(r) => match r {
                        Some(r) => r,
                        None => {
                            return Err(HttpError::not_found("Usuario no encontrado"));
                        }
                    },
                    Err(e) => {
                        error!(error = %e, "Error al obtener el usuario");
                        return Err(HttpError::internal_server_error());
                    }
                };
                let user = match User::from_row(&row) {
                    Ok(u) => u,
                    Err(e) => {
                        error!("Error al intentar crear el usuario: {}", e);
                        return Err(HttpError::internal_server_error());
                    }
                };
                let user = match user.status {
                    1 => user,
                    2 => return Err(HttpError::forbbiden("Usuario inactivo")),
                    _ => return Err(HttpError::not_found("Usuario no encontrado")),
                };

                claims.set_user(user);

                Ok(AuthenticatedClaims(claims))
            }
            Err(e) => {
                error!("Error verificando el TOKEN: {}", e);
                return Err(HttpError::unauthorized("Token inválido"));
            }
        }
    }
}

pub fn validate_password(password: &str) -> Result<bool, (StatusCode, Json<HttpError>)> {
    let pattern = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).+$";
    let re = Regex::new(pattern).unwrap();
    let is_valid = re.is_match(password).unwrap_or(false);
    if !is_valid {
        return Err(HttpError::bad_request(
            "La contraseña debe tener al menos: 1 minúscula, 1 mayúscula, 1 número y 1 caracter especial",
        ));
    }

    Ok(is_valid)
}
