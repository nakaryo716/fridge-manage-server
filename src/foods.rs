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
    async fn execute_insert_query(&self, payload: &Food) -> Result<(), FoodsError> {
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

    async fn execute_query(&self, id: &str) -> Result<Food, FoodsError> {
        query_as::<_, Food>(
            r#"
                SELECT food_id, food_name, exp, user_id
                FROM food_table
                WHERE food_id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)
    }

    async fn execute_query_all(&self, user_id: &str) -> Result<Vec<Food>, FoodsError> {
        query_as::<_, Food>(
            r#"
                SELECT food_id, food_name, exp, user_id
                FROM food_table
                WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)
    }

    async fn execute_update_query(&self, payload: &Food) -> Result<(), FoodsError> {
        query(
            r#"
                UPDATE food_table
                SET
                food_name = ?, exp = ?
                WHERE food_id = ?
            "#,
        )
        .bind(&payload.food_name)
        .bind(&payload.exp)
        .bind(&payload.food_id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }

    async fn execute_delete_query(&self, id: &str) -> Result<(), FoodsError> {
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
        self.execute_insert_query(payload).await?;
        let query_res = self.read(&payload.food_id).await?;
        Ok(query_res)
    }

    async fn update(&self, _id: &'a String, payload: &Food) -> Result<Self::Output, Self::Error> {
        self.execute_update_query(payload).await?;
        let query_res = self.read(&payload.food_id).await.map_err(|e| e)?;
        Ok(query_res)
    }

    async fn delete(&self, id: &'a String) -> Result<(), Self::Error> {
        self.execute_delete_query(id).await?;
        Ok(())
    }
}

#[async_trait]
impl RepositoryTargetReader<String> for FoodsRepository {
    type QueryRes = Food;
    type QueryErr = FoodsError;

    async fn read(&self, id: &String) -> Result<Self::QueryRes, Self::QueryErr> {
        self.execute_query(id).await
    }
}

#[async_trait]
impl RepositoryAllReader<String> for FoodsRepository {
    type QueryRes = AllFoods;
    type QueryErr = FoodsError;

    async fn read_all(&self, id: String) -> Result<Self::QueryRes, Self::QueryErr> {
        let foods = self.execute_query_all(&id).await?;
        Ok(AllFoods { foods })
    }
}

// CAUTION: Before running these tests, ensure the `user_table` in your Docker container's MySQL database contains a user with the following credentials:
//
// - `user_id`: `test_user_id`
// - `user_name`: `test_user_name`
//
// You'll need to manually insert this user into the `user_table` using a SQL query like this:
//
// ```sql
// INSERT INTO user_table (user_id, user_name, mail, password) VALUES ('test_user_id', 'test_user_name', 'mail', 'pass');
// ```
#[cfg(test)]
mod test {
    use chrono::NaiveDate;
    use sqlx::{query_as, MySql, MySqlPool, Pool};

    use crate::{foods::Food, users::PubUserInfo};

    use super::{CreateFoodPayload, FoodsRepository};

    static USER_ID: &str = "test_user_id";
    static USER_NAME: &str = "test_user_name";

    async fn set_up_db() -> Pool<MySql> {
        let db_url = dotenvy::var("DATABASE_URL").unwrap();
        MySqlPool::connect(&db_url).await.unwrap()
    }

    fn foodsrepo_new(pool: Pool<MySql>) -> FoodsRepository {
        FoodsRepository { pool }
    }

    fn pub_user_info() -> PubUserInfo {
        PubUserInfo {
            user_id: USER_ID.to_string(),
            user_name: USER_NAME.to_string(),
        }
    }

    fn create_food() -> CreateFoodPayload {
        CreateFoodPayload {
            food_name: "test_food".to_string(),
            exp: NaiveDate::from_ymd_opt(2025, 4, 8).unwrap_or_default(),
        }
    }

    fn new_update_food(old_food: &Food) -> Food {
        Food {
            food_id: old_food.food_id.to_owned(),
            food_name: format!("updated_{}", old_food.food_name),
            exp: old_food.exp,
            user_id: old_food.user_id.to_string(),
        }
    }

    async fn query_full_data(id: &str) -> Result<Food, Box<dyn std::error::Error>> {
        let pool = set_up_db().await;
        let repo = FoodsRepository { pool };

        let res = query_as(
            r#"
                SELECT food_id, food_name, exp, user_id FROM food_table
                WHERE food_id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&repo.pool)
        .await?;
        Ok(res)
    }

    #[tokio::test]
    async fn test_insert_food() {
        let repo = foodsrepo_new(set_up_db().await);

        let user = pub_user_info();
        let food = Food::new(create_food(), user.clone());
        repo.execute_insert_query(&food).await.unwrap();

        let db_food = query_full_data(&food.food_id).await.unwrap();
        assert_eq!(db_food.food_id, food.food_id);
        assert_eq!(db_food.food_name, food.food_name);
        assert_eq!(db_food.exp, food.exp);
        assert_eq!(db_food.user_id, food.user_id);
    }

    #[tokio::test]
    async fn test_query_food() {
        let repo = foodsrepo_new(set_up_db().await);

        let user = pub_user_info();
        let food = Food::new(create_food(), user);

        repo.execute_insert_query(&food).await.unwrap();

        println!("{:?}", food.food_id);
        let query_food = repo.execute_query(&food.food_id).await.unwrap();

        assert_eq!(query_food.food_id, food.food_id);
        assert_eq!(query_food.food_name, food.food_name);
        assert_eq!(query_food.exp, food.exp);
        assert_eq!(query_food.user_id, food.user_id);
    }

    #[tokio::test]
    async fn test_update_food() {
        let repo = foodsrepo_new(set_up_db().await);

        let user = pub_user_info();
        let food = Food::new(create_food(), user.clone());
        repo.execute_insert_query(&food).await.unwrap();

        let update_food = new_update_food(&food);
        repo.execute_update_query(&update_food).await.unwrap();

        let db_food = query_full_data(&update_food.food_id).await.unwrap();
        assert_eq!(db_food.food_id, update_food.food_id);
        assert_eq!(db_food.food_name, update_food.food_name);
        assert_eq!(db_food.exp, update_food.exp);
        assert_eq!(db_food.user_id, update_food.user_id);
    }

    #[tokio::test]
    async fn test_delete_food() {
        let repo = foodsrepo_new(set_up_db().await);

        let user = pub_user_info();
        let food = Food::new(create_food(), user.clone());
        repo.execute_insert_query(&food).await.unwrap();

        repo.execute_delete_query(&food.food_id).await.unwrap();

        if let Ok(_) = query_full_data(&food.food_id).await {
            panic!("food should deleted but exists");
        }
    }
}
