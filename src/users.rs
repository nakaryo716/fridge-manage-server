use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, prelude::FromRow, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::util::HashFunc;

pub mod repo;

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
pub struct UserId(pub(crate) String);

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self(value.to_string())
    }
}

impl From<UserId> for String {
    fn from(value: UserId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct UserName(String);

impl From<String> for UserName {
    fn from(value: String) -> Self {
        Self(value.to_string())
    }
}

impl From<UserName> for String {
    fn from(value: UserName) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, FromRow, Deserialize, PartialEq)]
pub struct Mail(String);

impl From<String> for Mail {
    fn from(value: String) -> Self {
        Self(value.to_string())
    }
}

impl From<Mail> for String {
    fn from(value: Mail) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Deserialize, FromRow, PartialEq)]
pub struct Password(String);

impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value.to_string())
    }
}

impl From<Password> for String {
    fn from(value: Password) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CreateUserPayload {
    pub user_name: UserName,
    pub mail: Mail,
    pub password: Password,
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    user_id: UserId,
    user_name: UserName,
    mail: Mail,
    password: Password,
}

impl User {
    pub(crate) fn new(
        payload: CreateUserPayload,
        hasher: Box<dyn HashFunc>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_id: UserId::from(Uuid::new_v4().to_string()),
            user_name: payload.user_name,
            mail: payload.mail,
            password: Password::from(hasher.call(&payload.password.0)?),
        })
    }
}

impl FromRow<'_, MySqlRow> for User {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(User {
            user_id: UserId(row.try_get("user_id")?),
            user_name: UserName(row.try_get("user_name")?),
            mail: Mail(row.try_get("mail")?),
            password: Password(row.try_get("password")?),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PubUserInfo {
    pub user_id: UserId,
    pub user_name: UserName,
}

impl FromRow<'_, MySqlRow> for PubUserInfo {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(PubUserInfo {
            user_id: UserId(row.try_get("user_id")?),
            user_name: UserName(row.try_get("user_name")?),
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("error")]
    NotFound,
}
