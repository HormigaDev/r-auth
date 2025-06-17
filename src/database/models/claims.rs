use axum::{Json, http::StatusCode};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    database::models::entities::user::User,
    utils::{Permissions, errors::HttpError},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
    pub iat: usize,
    user: Option<User>,
}

impl Claims {
    #[allow(unused)]
    pub fn new(user_id: String, expiration_minutes: i64) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::minutes(expiration_minutes);

        Claims {
            user_id,
            exp: exp.timestamp() as usize,
            iat: iat.timestamp() as usize,
            user: None,
        }
    }

    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    pub fn get_user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn require_permission(
        &self,
        perm: Permissions,
    ) -> Result<bool, (StatusCode, Json<HttpError>)> {
        let user = match self.get_user() {
            Some(u) => u,
            None => return Err(HttpError::unauthorized("Usuario no encontrado")),
        };
        let perms = user.permissions;
        let bitperms = Permissions::from_bits_retain(perms);
        let has_perms = bitperms.contains(perm) || bitperms.contains(Permissions::ADMIN);

        if !has_perms {
            return Err(HttpError::forbbiden("Permisos de usuario insuficientes"));
        }

        Ok(has_perms)
    }
}
