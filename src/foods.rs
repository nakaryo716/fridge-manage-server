use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, types::chrono::NaiveDate, FromRow, MySql, Pool};
use thiserror::Error;
use uuid::Uuid;

use crate::{users::PubUserInfo, RepositoryAllReader, RepositoryTargetReader, RepositoryWriter};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFoodPayload {
    food_name: String,
    exp: NaiveDate,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Food {
    food_id: String,
    food_name: String,
    exp: NaiveDate,
    user_id: String,
}

impl Food {
    pub fn new(payload: CreateFoodPayload, user: PubUserInfo) -> Self {
        Self {
            food_id: Uuid::new_v4().to_string(),
            food_name: payload.food_name,
            exp: payload.exp,
            user_id: user.user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AllFoods {
    foods: Vec<Food>,
}

pub struct FoodsRepository {
    pool: Pool<MySql>,
}

impl FoodsRepository {
    async fn excute_insert_query(&self, payload: &Food) -> Result<(), FoodsError> {
        query(
            r#"
                INSERT INTO food_table
                (food_id, food_name, exp, user_id)
                VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&payload.food_id)
        .bind(&payload.food_name)
        .bind(payload.exp)
        .bind(&payload.user_id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }

    async fn excute_update_query(&self, payload: &Food) -> Result<(), FoodsError> {
        query(
            r#"
                UPDATE food_table
                SET
                food_name = ?, exp = ?
                WHERE user_id = ?
            "#,
        )
        .bind(&payload.food_name)
        .bind(&payload.exp)
        .bind(&payload.user_id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Error)]
pub enum FoodsError {
    #[error("Not found")]
    NotFound,
}

#[async_trait]
impl<'a> RepositoryWriter<'a, '_, Food, String> for FoodsRepository {
    type Output = Food;
    type Error = FoodsError;

    async fn insert(&self, payload: &Food) -> Result<Self::Output, Self::Error> {
        self.excute_insert_query(payload).await?;
        let query_res = self.read(&payload.food_id).await?;
        Ok(query_res)
    }

    async fn update(&self, _id: &'a String, payload: &Food) -> Result<Self::Output, Self::Error> {
        self.excute_update_query(payload).await?;
        let query_res = self.read(&payload.food_id).await.map_err(|e| e)?;
        Ok(query_res)
    }

    async fn delete(&self, id: &'a String) -> Result<(), Self::Error> {
        query(
            r#"
                DELETE FROM food_table
                WHERE food_id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }
}

#[async_trait]
impl RepositoryTargetReader<String> for FoodsRepository {
    type QueryRes = Food;
    type QueryErr = FoodsError;

    async fn read(&self, id: &String) -> Result<Self::QueryRes, Self::QueryErr> {
        let query_res = query_as(
            r#"
                SELECT food_id, food_name, exp, user_id
                FROM food_table
                WHERE food_id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(query_res)
    }
}

#[async_trait]
impl RepositoryAllReader<String> for FoodsRepository {
    type QueryRes = AllFoods;
    type QueryErr = FoodsError;

    async fn read_all(&self, id: String) -> Result<Self::QueryRes, Self::QueryErr> {
        let query_res: Vec<Food> = query_as(
            r#"
                SELECT food_id, food_name, exp, user_id
                FROM food_table
                WHERE user_id = ?
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(AllFoods { foods: query_res })
    }
}
