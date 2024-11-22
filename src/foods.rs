use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, prelude::Type, types::chrono::NaiveDate, FromRow, Row};
use thiserror::Error;
use uuid::Uuid;

use crate::users::{PubUserInfo, UserId};

mod repo;

static FOOD_ID_COLUMN: &'static str = "food_id";
static FOOD_NAME_COLUMN: &'static str = "food_name";
static FOOD_EXP_COLUMN: &'static str = "exp";
static USER_ID_COLUMN: &'static str = "user_id";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[sqlx(transparent)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[sqlx(transparent)]
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
            food_id: FoodId(row.try_get(FOOD_ID_COLUMN)?),
            food_name: FoodName(row.try_get(FOOD_NAME_COLUMN)?),
            exp: row.try_get(FOOD_EXP_COLUMN)?,
            user_id: UserId(row.try_get(USER_ID_COLUMN)?),
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
