use crate::domain::repositories::todo_repository::{MockTodoRepository, TodoRepository};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_repository() {
        let mut mock_repo = MockTodoRepository::new();

        // find_allメソッドが空のベクターを返すように設定
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|| Box::pin(async { Ok(vec![]) }));

        // テスト実行
        let result = mock_repo.find_all().await.unwrap();

        // 空のベクターが返されることを確認
        assert_eq!(result.len(), 0);
    }
}
