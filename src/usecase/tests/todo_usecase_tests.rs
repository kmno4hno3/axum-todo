use crate::domain::models::todo::Todo;
use crate::domain::repositories::todo_repository::MockTodoRepository;
use crate::usecase::todo_usecase::{TodoService, TodoUsecase};
use chrono::{FixedOffset, TimeZone, Utc};
use mockall::predicate::*;
use uuid::Uuid;

// テスト用のTodoを作成するヘルパー関数
fn create_test_todo(title: &str, description: Option<&str>) -> Todo {
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    let now_jst = jst.from_utc_datetime(&Utc::now().naive_utc());
    let now_utc = now_jst.with_timezone(&Utc);

    Todo {
        id: Uuid::new_v4(),
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        completed: false,
        created_at: now_utc,
        updated_at: now_utc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_all_todos() {
        // モックリポジトリの作成
        let mut mock_repo = MockTodoRepository::new();

        // テスト用のTodoリスト
        let todos = vec![
            create_test_todo("タスク1", Some("説明1")),
            create_test_todo("タスク2", Some("説明2")),
        ];

        // find_allメソッドのモック設定
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move || Box::pin(async move { Ok(todos.clone()) }));

        // ユースケースの作成
        let usecase = TodoUsecase::new(mock_repo);

        // テスト実行
        let result = usecase.get_all_todos().await.unwrap();

        // 検証
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "タスク1");
        assert_eq!(result[1].title, "タスク2");
    }

    // 他のテストメソッドも同様に実装...
}
