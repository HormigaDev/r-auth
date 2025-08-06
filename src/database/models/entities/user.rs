use std::error::Error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type, private::BytesMut};
use utoipa::ToSchema;

use crate::auth::tfa::Tfa;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Copy)]
#[repr(i32)]
pub enum UserStatus {
    Active = 1,
    Inactive = 2,
    Deleted = 3,
}

impl ToSql for UserStatus {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        (*self as i16).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as ToSql>::accepts(ty)
    }

    tokio_postgres::types::to_sql_checked!();
}

impl<'a> FromSql<'a> for UserStatus {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let value = i16::from_sql(ty, raw)?;
        match value {
            1 => Ok(UserStatus::Active),
            2 => Ok(UserStatus::Inactive),
            3 => Ok(UserStatus::Deleted),
            _ => Err("Valor de status desconocido".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as FromSql>::accepts(ty)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,

    #[serde(skip_serializing)]
    pub password: Option<String>,

    #[serde(skip_serializing)]
    pub permissions: i64,

    #[serde(skip_serializing, skip_deserializing)]
    pub tfa: Option<Tfa>,

    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i64, username: String, email: String) -> Self {
        User {
            id,
            username,
            email,
            password: Some(String::from("fake_password")),
            permissions: 0,
            status: UserStatus::Active,
            tfa: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn empty() -> Self {
        User {
            id: 0,
            username: "".to_string(),
            email: "".to_string(),
            password: Some("".to_string()),
            permissions: 0,
            status: UserStatus::Active,
            tfa: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn from_row(row: &tokio_postgres::Row) -> Result<Self, Box<dyn std::error::Error>> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let use_tfa: bool = row.try_get("use_tfa")?;
        let mut tfa = None;
        if use_tfa {
            let secret: String = row.try_get("tfa_secret")?;
            tfa = Some(Tfa::new(secret.as_str()));
        }

        let user = Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            permissions: row.try_get("permissions")?,
            password: None,
            status: row.try_get("status")?,
            tfa,
            created_at,
            updated_at,
        };
        Ok(user)
    }

    pub fn from_row_without_perms(
        row: &tokio_postgres::Row,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        let user = Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            permissions: 0,
            password: None,
            status: row.try_get("status")?,
            tfa: None,
            created_at,
            updated_at,
        };
        Ok(user)
    }

    pub fn from_row_with_password(
        row: &tokio_postgres::Row,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let password = row.try_get("password").unwrap_or_else(|_| String::new());
        let use_tfa: bool = row.try_get("use_tfa")?;
        let mut tfa = None;
        if use_tfa {
            let secret: String = row.try_get("tfa_secret")?;
            tfa = Some(Tfa::new(secret.as_str()));
        }

        let user = Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            permissions: row.try_get("permissions")?,
            password: Some(password),
            status: row.try_get("status")?,
            tfa,
            created_at,
            updated_at,
        };
        Ok(user)
    }
}
