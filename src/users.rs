use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, prelude::FromRow, MySql, Pool, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::{util::HashFunc, RepositoryTargetReader, RepositoryWriter};

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
pub struct UserId(String);

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

pub struct UserRepository {
    pool: Pool<MySql>,
}

impl UserRepository {
    pub(crate) fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    async fn execute_insert_query(&self, payload: &User) -> Result<(), UserError> {
        sqlx::query(
            r#"
                INSERT INTO user_table
                (user_id, user_name, mail, password) VALUES (?, ?, ?, ?)
            "#,
        )
        .bind::<String>(payload.user_id.clone().into())
        .bind::<String>(payload.user_name.clone().into())
        .bind::<String>(payload.mail.clone().into())
        .bind::<String>(payload.password.clone().into())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(())
    }

    async fn execute_query(&self, id: &UserId) -> Result<PubUserInfo, UserError> {
        println!("{:?}", id);
        let query_res = sqlx::query_as(
            r#"
                SELECT user_id, user_name
                FROM user_table
                WHERE user_id = ?
            "#,
        )
        .bind::<String>(id.clone().into())
        .fetch_one(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(query_res)
    }

    async fn execute_update_query(&self, payload: &User) -> Result<(), UserError> {
        sqlx::query(
            r#"
                UPDATE user_table
                SET
                user_name = ?,
                mail = ?,
                password = ?
                WHERE user_id = ?
            "#,
        )
        .bind::<String>(payload.user_name.clone().into())
        .bind::<String>(payload.mail.clone().into())
        .bind::<String>(payload.password.clone().into())
        .bind::<String>(payload.user_id.clone().into())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(())
    }

    async fn execute_delete_query(&self, id: &UserId) -> Result<(), UserError> {
        sqlx::query(
            r#"
                DELETE FROM user_table
                WHERE user_id = ?
            "#,
        )
        .bind::<String>(id.clone().into())
        .execute(&self.pool)
        .await
        .map_err(|_e| UserError::NotFound)?;
        Ok(())
    }
}

#[async_trait]
impl RepositoryTargetReader<UserId> for UserRepository {
    type QueryRes = PubUserInfo;
    type QueryErr = UserError;

    async fn read(&self, id: &UserId) -> Result<Self::QueryRes, Self::QueryErr> {
        self.execute_query(id).await
    }
}

#[async_trait]
impl<'a> RepositoryWriter<'a, '_, User, UserId> for UserRepository {
    type Output = PubUserInfo;
    type Error = UserError;

    async fn insert(&self, payload: &User) -> Result<Self::Output, Self::Error> {
        self.execute_insert_query(payload).await?;
        let query_res = self.read(&payload.user_id).await?;
        Ok(query_res)
    }

    async fn update(&self, _id: &'a UserId, payload: &User) -> Result<Self::Output, Self::Error> {
        self.execute_update_query(payload).await?;
        let query_res = self.read(&payload.user_id).await?;
        Ok(query_res)
    }
    async fn delete(&self, id: &'a UserId) -> Result<(), Self::Error> {
        self.execute_delete_query(id).await
    }
}

#[cfg(test)]
mod test {
    use rand::random;
    use sqlx::{query_as, MySqlPool};

    use crate::{
        users::{CreateUserPayload, User},
        util::default_hash_password,
        RepositoryTargetReader, RepositoryWriter,
    };

    use super::{Mail, Password, UserId, UserName, UserRepository};

    async fn set_up_db() -> UserRepository {
        let db_url = dotenvy::var("DATABASE_URL").unwrap();
        let pool = MySqlPool::connect(&db_url).await.unwrap();
        UserRepository { pool }
    }

    fn user_provider() -> User {
        let num = random::<i32>();
        let payload = CreateUserPayload {
            user_name: UserName::from(format!("test_user_name_{}", num)),
            mail: Mail::from(format!("test_user_mail_{}@mail.com", num)),
            password: Password::from(format!("test_user_pass_{}", num)),
        };

        let hasher = Box::new(default_hash_password);
        User::new(payload, hasher).unwrap()
    }

    fn update_user(user: User) -> User {
        let num = random::<i32>();
        User {
            user_id: user.user_id,
            user_name: UserName::from(format!("test_user_name_{}", num)),
            mail: Mail::from(format!("test_user_mail_{}@mail.com", num)),
            password: Password::from(format!("test_user_pass_{}", num)),
        }
    }

    async fn query_full_data(id: &UserId) -> Result<User, Box<dyn std::error::Error>> {
        let repo = set_up_db().await;
        let res = query_as(
            r#"
                SELECT user_id, user_name, mail, password
                FROM user_table
                WHERE user_id = ?
            "#,
        )
        .bind::<String>(id.clone().into())
        .fetch_one(&repo.pool)
        .await
        .map_err(|e| e)?;
        Ok(res)
    }

    #[tokio::test]
    async fn test_insert_user() {
        let repo = set_up_db().await;
        let new_user = user_provider();

        repo.insert(&new_user).await.unwrap();
        let user_info = query_full_data(&new_user.user_id).await.unwrap();

        assert_eq!(user_info.user_id, new_user.user_id);
        assert_eq!(user_info.user_name, new_user.user_name);

        let inserted_full_data = query_full_data(&new_user.user_id).await.unwrap();
        assert_eq!(inserted_full_data, new_user);
    }

    #[tokio::test]
    async fn test_update_user() {
        let repo = set_up_db().await;
        let new_user = user_provider();
        repo.insert(&new_user).await.unwrap();

        let update_user = update_user(new_user);
        repo.update(&update_user.user_id, &update_user)
            .await
            .unwrap();

        let modified_full_data = query_full_data(&update_user.user_id).await.unwrap();
        assert_eq!(modified_full_data, update_user);
    }

    #[tokio::test]
    async fn test_delete_user() {
        let repo = set_up_db().await;
        let new_user = user_provider();
        repo.insert(&new_user).await.unwrap();

        repo.delete(&new_user.user_id).await.unwrap();
        if let Ok(_) = query_full_data(&new_user.user_id).await {
            panic!("Expected user is deleted, but found user");
        }
    }

    #[tokio::test]
    async fn test_read_user() {
        let repo = set_up_db().await;
        let new_user = user_provider();
        repo.insert(&new_user).await.unwrap();

        let user_info = repo.read(&new_user.user_id).await.unwrap();
        assert_eq!(user_info.user_id, new_user.user_id);
        assert_eq!(user_info.user_name, new_user.user_name);
    }
}
