use std::error::Error;

use async_trait::async_trait;
use serde::Serialize;
use sqlx::{mysql::MySqlRow, FromRow};

pub mod auth;
pub mod foods;
pub mod users;
pub mod util;

#[async_trait]
trait RepositoryWriter<'a, 'r, Payload, Target>: RepositoryTargetReader<'a, Target> {
    type Output: Serialize + FromRow<'r, MySqlRow>;
    type Error: Error;

    async fn insert(&self, payload: &Payload) -> Result<Self::Output, Self::Error>;
    async fn update(&self, id: &'a Target, payload: &Payload) -> Result<Self::Output, Self::Error>;
    async fn delete(&self, id: &'a Target) -> Result<(), Self::Error>;
}

#[async_trait]
trait RepositoryTargetReader<'a, Target> {
    type QueryRes: Serialize;
    type QueryErr: Error;

    async fn read(&self, id: &'a Target) -> Result<Self::QueryRes, Self::QueryErr>;
}

#[async_trait]
trait RepositoryAllReader<Id> {
    type QueryRes: Serialize;
    type QueryErr: Error;

    async fn read_all(&self, id: Id) -> Result<Self::QueryRes, Self::QueryErr>;
}
