use crate::domain::models::todo::Todo;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TodoRepositories {
    async fn find_all(&self) -> Result<Vec<Todo>, sqlx::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Todo>, sqlx::Error>;
    async fn create(&self, todo: Todo) -> Result<Todo, sqlx::Error>;
    async fn update(&self, todo: Todo) -> Result<Todo, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}
