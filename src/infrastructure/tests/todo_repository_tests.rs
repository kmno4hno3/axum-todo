use crate::domain::models::todo::Todo;
use crate::domain::repositories::todo_repository::TodoRepository;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::todo_repository::TodoRepositoryImpl;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

async fn setup_test_db() -> DbPool {
    // .envファイルを読み込む
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

#[tokio::test]
#[ignore = "Requires DATABASE_URL to be set"]
async fn test_create_and_find_by_id() {
    let pool = setup_test_db().await;
    let repo = TodoRepositoryImpl::new(pool);

    // 新しいTodoを作成
    let title = "テストタスク".to_string();
    let description = "テスト説明".to_string();
    let todo = Todo::new(title.clone(), description.clone());

    // 作成したTodoをデータベースに保存
    let created_todo = repo.create(todo).await.unwrap();

    // IDで検索
    let found_todo = repo.find_by_id(created_todo.id).await.unwrap();

    // 検証
    assert!(found_todo.is_some());
    let found_todo = found_todo.unwrap();
    assert_eq!(found_todo.id, created_todo.id);
    assert_eq!(found_todo.title, title);
    assert_eq!(found_todo.description, Some(description));
    assert!(!found_todo.completed);

    // 後処理：作成したTodoを削除
    repo.delete(created_todo.id).await.unwrap();
}

// 他のテストメソッドも同様に実装...
