use axum::{
    extract::{Path, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};

use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::models::todo::Todo;
use crate::usecase::todo_usecase::TodoService;

#[derive(Clone)]
pub struct AppState<T: TodoService> {
    pub todo_service: Arc<T>,
}

pub fn create_todo_router<T: TodoService + Send + Sync + 'static + Clone>(
    todo_service: T,
) -> Router {
    let state = AppState {
        todo_service: Arc::new(todo_service),
    };

    Router::new()
        .route("/todos", get(get_all_todos::<T>).post(create_todo::<T>))
        .route(
            "/todos/{id}",
            get(get_todo_by_id::<T>)
                .put(update_todo::<T>)
                .delete(delete_todo::<T>),
        )
        .with_state(state)
}

#[derive(Deserialize)]
struct CreateTodoRequest {
    title: String,
    description: String,
}

#[derive(Deserialize)]
struct UpdateTodoRequest {
    title: String,
    description: String,
    completed: bool,
}

#[derive(Serialize)]
struct TodoResponse {
    id: Uuid,
    title: String,
    description: Option<String>,
    completed: bool,
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title,
            description: todo.description,
            completed: todo.completed,
        }
    }
}

async fn get_all_todos<T: TodoService>(State(state): State<AppState<T>>) -> impl IntoResponse {
    match state.todo_service.get_all_todos().await {
        Ok(todos) => Json(
            todos
                .into_iter()
                .map(TodoResponse::from)
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch todos").into_response(),
    }
}

async fn get_todo_by_id<T: TodoService>(
    State(state): State<AppState<T>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.todo_service.get_todo_by_id(id).await {
        Ok(Some(todo)) => Json(TodoResponse::from(todo)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "Todo not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch todo").into_response(),
    }
}

async fn create_todo<T: TodoService>(
    State(state): State<AppState<T>>,
    Json(payload): Json<CreateTodoRequest>,
) -> impl IntoResponse {
    match state
        .todo_service
        .create_todo(payload.title, payload.description)
        .await
    {
        Ok(todo) => (StatusCode::CREATED, Json(TodoResponse::from(todo))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create todo").into_response(),
    }
}

async fn update_todo<T: TodoService>(
    State(state): State<AppState<T>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoRequest>,
) -> impl IntoResponse {
    match state
        .todo_service
        .update_todo(id, payload.title, payload.description, payload.completed)
        .await
    {
        Ok(todo) => Json(TodoResponse::from(todo)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Todo not found").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update todo").into_response(),
    }
}

async fn delete_todo<T: TodoService>(
    State(state): State<AppState<T>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.todo_service.delete_todo(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Todo not found").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete todo").into_response(),
    }
}
