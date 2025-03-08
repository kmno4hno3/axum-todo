use crate::domain::models::todo::Todo;
use crate::domain::repositories::todo_repository::TodoRepository;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Clone)]
pub struct TodoUsecase<T: TodoRepository + Clone> {
    repository: T,
}

impl<T: TodoRepository + Clone> TodoUsecase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
pub trait TodoService {
    async fn get_all_todos(&self) -> Result<Vec<Todo>, sqlx::Error>;
    async fn get_todo_by_id(&self, id: Uuid) -> Result<Option<Todo>, sqlx::Error>;
    async fn create_todo(&self, title: String, description: String) -> Result<Todo, sqlx::Error>;
    async fn update_todo(
        &self,
        id: Uuid,
        title: String,
        description: String,
        completed: bool,
    ) -> Result<Todo, sqlx::Error>;
    async fn delete_todo(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl<T: TodoRepository + Send + Sync + Clone> TodoService for TodoUsecase<T> {
    async fn get_all_todos(&self) -> Result<Vec<Todo>, sqlx::Error> {
        self.repository.find_all().await
    }

    async fn get_todo_by_id(&self, id: Uuid) -> Result<Option<Todo>, sqlx::Error> {
        self.repository.find_by_id(id).await
    }

    async fn create_todo(&self, title: String, description: String) -> Result<Todo, sqlx::Error> {
        let new_todo = Todo::new(title, description);
        self.repository.create(new_todo).await
    }

    async fn update_todo(
        &self,
        id: Uuid,
        title: String,
        description: String,
        completed: bool,
    ) -> Result<Todo, sqlx::Error> {
        let existing_todo = self.repository.find_by_id(id).await?;
        if let Some(mut todo) = existing_todo {
            todo.title = title;
            todo.description = Some(description);
            todo.completed = completed;
            return self.repository.update(todo).await;
        }

        Err(sqlx::Error::RowNotFound)
    }

    async fn delete_todo(&self, id: Uuid) -> Result<(), sqlx::Error> {
        self.repository.delete(id).await
    }
}
