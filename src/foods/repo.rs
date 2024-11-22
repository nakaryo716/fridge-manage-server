use async_trait::async_trait;
use sqlx::{query, query_as, MySql, Pool};

use crate::{users::UserId, RepositoryAllReader, RepositoryTargetReader, RepositoryWriter};

use super::{AllFoods, Food, FoodId, FoodsError};

pub struct FoodsRepository {
    pool: Pool<MySql>,
}

impl FoodsRepository {
    pub(crate) fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> RepositoryWriter<'a, '_, Food, FoodId> for FoodsRepository {
    type Output = ();
    type Error = FoodsError;

    async fn insert(&self, payload: &Food) -> Result<Self::Output, Self::Error> {
        query(
            r#"
                INSERT INTO food_table
                (food_id, food_name, exp, user_id)
                VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&payload.food_id)
        .bind(&payload.food_name)
        .bind(&payload.exp)
        .bind(&payload.user_id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }

    async fn update(&self, id: &'a FoodId, payload: &Food) -> Result<Self::Output, Self::Error> {
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
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
        Ok(())
    }

    async fn delete(&self, id: &'a FoodId) -> Result<(), Self::Error> {
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
impl<'a> RepositoryTargetReader<'a, FoodId> for FoodsRepository {
    type QueryRes = Food;
    type QueryErr = FoodsError;

    async fn read(&self, id: &'a FoodId) -> Result<Self::QueryRes, Self::QueryErr> {
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
}

#[async_trait]
impl<T> RepositoryAllReader<T> for FoodsRepository 
where 
    T: Into<UserId> + Clone + Send + Sync + 'static,
{
    type QueryRes = AllFoods;
    type QueryErr = FoodsError;

    async fn read_all(&self, id: T) -> Result<Self::QueryRes, Self::QueryErr> {
        let foods = query_as::<_, Food>(
            r#"
                SELECT food_id, food_name, exp, user_id
                FROM food_table
                WHERE user_id = ?
            "#,
        )
        .bind::<UserId>(id.clone().into())
        .fetch_all(&self.pool)
        .await
        .map_err(|_e| FoodsError::NotFound)?;
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

    use crate::{
        foods::{CreateFoodPayload, Food, FoodId, FoodName},
        users::{PubUserInfo, UserId, UserName},
        RepositoryTargetReader, RepositoryWriter,
    };

    use super::FoodsRepository;

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
            user_id: UserId::from(USER_ID.to_string()),
            user_name: UserName::from(USER_NAME.to_string()),
        }
    }

    fn create_food() -> CreateFoodPayload {
        CreateFoodPayload {
            food_name: FoodName::from("test_food"),
            exp: NaiveDate::from_ymd_opt(2025, 4, 8).unwrap_or_default(),
        }
    }

    fn new_update_food(old_food: &Food) -> Food {
        let updated_food_name = format!(
            "updated_{}",
            <FoodName as Into<String>>::into(old_food.food_name.clone())
        );

        Food {
            food_id: old_food.food_id.to_owned(),
            food_name: FoodName::from(&updated_food_name),
            exp: old_food.exp,
            user_id: old_food.user_id.clone(),
        }
    }

    async fn query_full_data(id: &FoodId) -> Result<Food, Box<dyn std::error::Error>> {
        let pool = set_up_db().await;
        let repo = FoodsRepository { pool };

        let res = query_as(
            r#"
                SELECT food_id, food_name, exp, user_id FROM food_table
                WHERE food_id = ?
            "#,
        )
        .bind::<String>(id.clone().into())
        .fetch_one(&repo.pool)
        .await?;
        Ok(res)
    }

    #[tokio::test]
    async fn test_insert_food() {
        let repo = foodsrepo_new(set_up_db().await);

        let user = pub_user_info();
        let food = Food::new(create_food(), user.clone());
        repo.insert(&food).await.unwrap();

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

        repo.insert(&food).await.unwrap();

        println!("{:?}", food.food_id);
        let query_food = repo.read(&food.food_id).await.unwrap();

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
        repo.insert(&food).await.unwrap();

        let update_food = new_update_food(&food);
        repo.update(&update_food.food_id, &update_food)
            .await
            .unwrap();

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
        repo.insert(&food).await.unwrap();

        repo.delete(&food.food_id).await.unwrap();

        if let Ok(_) = query_full_data(&food.food_id).await {
            panic!("food should deleted but exists");
        }
    }
}
