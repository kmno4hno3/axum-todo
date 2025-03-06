use chrono::{Datetime, FixedOffset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
// トレイトを実装して、データベースの行から直接構造体にマッピングできるようにする
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: Datetime<Utc>,
    pub updated_at: Datetime<Utc>,
}

impl Todo {
    pub fn new(title: String, description: String) -> Self {
        let jst = FixedOffset::east_opt(9 * 3600).unwrap();
        let now_jst = jst.from_utc_datatime(&Utc::now().naive_utc());
        let now_utc = now_jst.with_timezone(&Utc);
        Self {
            id: Uuid::now_v7(),
            title,
            description: Some(description),
            completed: false,
            created_at: now_utc,
            updated_at: now_utc,
        }
    }
}
