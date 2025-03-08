use crate::domain::models::todo::Todo;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_todo() {
        let title = "テストタイトル";
        let description = "テスト説明";

        let todo = Todo::new(title.to_string(), description.to_string());

        // タイトルと説明が正しく設定されていることを確認
        assert_eq!(todo.title, title);
        assert_eq!(todo.description, Some(description.to_string()));

        // 初期状態では完了していないことを確認
        assert!(!todo.completed);

        // created_atとupdated_atが同じであることを確認
        assert_eq!(todo.created_at, todo.updated_at);

        // UUIDがバージョン7であることを確認
        assert_eq!(todo.id.get_version_num(), 7);

        // 現在時刻との差が小さいことを確認（1秒以内）
        let now = Utc::now();
        let diff = now.signed_duration_since(todo.created_at);
        assert!(diff.num_seconds().abs() < 1);
    }
}
