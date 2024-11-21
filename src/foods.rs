use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, types::chrono::NaiveDate, FromRow, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::users::{PubUserInfo, UserId};

mod repo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FoodId(String);

impl From<FoodId> for String {
    fn from(value: FoodId) -> Self {
        value.0
    }
}

impl<T> From<T> for FoodId
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FoodName(String);

impl From<FoodName> for String {
    fn from(value: FoodName) -> Self {
        value.0
    }
}

impl<T> From<T> for FoodName
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFoodPayload {
    food_name: FoodName,
    exp: NaiveDate,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Food {
    food_id: FoodId,
    food_name: FoodName,
    exp: NaiveDate,
    user_id: UserId,
}

impl Food {
    pub fn new(payload: CreateFoodPayload, user: PubUserInfo) -> Self {
        Self {
            food_id: FoodId::from(Uuid::new_v4().to_string().as_str()),
            food_name: payload.food_name,
            exp: payload.exp,
            user_id: user.user_id,
        }
    }
}

impl FromRow<'_, MySqlRow> for Food {
    fn from_row(row: &'_ MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(Food {
            food_id: FoodId(row.try_get("food_id")?),
            food_name: FoodName(row.try_get("food_name")?),
            exp: row.try_get("exp")?,
            user_id: UserId(row.try_get("user_id")?),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AllFoods {
    foods: Vec<Food>,
}

#[derive(Debug, Clone, Error)]
pub enum FoodsError {
    #[error("Not found")]
    NotFound,
}
