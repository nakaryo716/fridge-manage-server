use async_trait::async_trait;
use serde::Serialize;
use sqlx::{prelude::FromRow, MySql, Pool};
use thiserror::Error;
use uuid::Uuid;

use crate::{RepositoryTargetReader, RepositoryWriter};

pub struct CreateUserPayload {
    pub user_name: String,
    pub mail: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct User {
    user_id: String,
    user_name: String,
    mail: String,
    password: String,
}

impl User {
    pub fn new(payload: CreateUserPayload) -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            user_name: payload.user_name,
            mail: payload.mail,
            password: payload.password,
        }
    }
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PubUserInfo {
    pub user_id: String,
    pub user_name: String,
}

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("error")]
    NotFound,
}

pub struct UserRepository {
    pool: Pool<MySql>,
}

#[async_trait]
impl RepositoryTargetReader<String> for UserRepository {
    type QueryRes = PubUserInfo;
    type QueryErr = UserError;

    async fn read(&self, id: &String) -> Result<Self::QueryRes, Self::QueryErr> {
        let query_res = sqlx::query_as(
            r#"
                SELECT user_id, user_name
                FROM user_table
                WHERE user_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(query_res)
    }
}

#[async_trait]
impl<'a> RepositoryWriter<'a, '_, User, String> for UserRepository {
    type Output = PubUserInfo;
    type Error = UserError;

    async fn insert(&self, payload: &User) -> Result<Self::Output, Self::Error> {
        sqlx::query(
            r#"
                INSERT INTO user_table
                (user_id, user_name, mail, password) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(payload.user_id.to_owned())
        .bind(payload.user_name.to_owned())
        .bind(payload.mail.to_owned())
        .bind(payload.password.to_owned())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;

        let query_res = self.read(&payload.user_id).await.map_err(|e| e)?;
        Ok(query_res)
    }

    async fn update(&self, _id: &'a String, payload: &User) -> Result<Self::Output, Self::Error> {
        sqlx::query(
            r#"
                UPDATE user_table
                SET
                user_name = $1,
                mail = $2,
                password = $3
                WHERE user_id = $4
            "#,
        )
        .bind(payload.user_name.to_owned())
        .bind(payload.mail.to_owned())
        .bind(payload.password.to_owned())
        .bind(payload.user_id.to_owned())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;

        let query_res = self.read(&payload.user_id).await.map_err(|e| e)?;
        Ok(query_res)
    }
    async fn delete(&self, id: &'a String) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
                DELETE FROM user_table
                WHERE user_id = $1
            "#,
        )
        .bind(id.to_owned())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(())
    }
}
