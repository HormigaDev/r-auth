use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,

    #[serde(skip_serializing)]
    pub password: Option<String>,

    #[serde(skip_serializing)]
    pub permissions: i64,

    pub status: i32,
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
            status: 1,
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
            status: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn from_row(row: &tokio_postgres::Row) -> Result<Self, Box<dyn std::error::Error>> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

        let user = Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            permissions: row.try_get("permissions")?,
            password: None,
            status: row.try_get("status")?,
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

        let user = Self {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            email: row.try_get("email")?,
            permissions: row.try_get("permissions")?,
            password: Some(password),
            status: row.try_get("status")?,
            created_at,
            updated_at,
        };
        Ok(user)
    }
}
