use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
// layer: 特定の機能を追加
// util: ヘルパー関数を提供
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    // ログレジストリ作成
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_sandbox=debug,tower_http=debug".into()),
        ) // 環境フィルタ設定(ログフィルタレベル設定)
        .with(tracing_subscriber::fmt::layer()) // フォーマットレイヤー追加(ログを見やすい形式にする)
        .init(); // ログシステム初期化(適用)

    // Arc::new(RwLock::new(HashMap::new()));と等価
    // 初期化すると以下のような感じ
    // Arc
    //  └── RwLock
    //       └── HashMap
    //           ├── Uuid1 → Todo { id: Uuid1, text: "買い物", completed: false }
    //           ├── Uuid2 → Todo { id: Uuid2, text: "Rustの勉強", completed: true }
    //           └── ...
    let db = Db::default(); // default: 型のデフォルト値を生成
    println!("{:?}", db);
    // dbの中身を出力すると
    // RwLock { data: {239a1d9c-1041-4758-b9d0-3e00414d6563: Todo { id: 239a1d9c-1041-4758-b9d0-3e00414d6563, text: "Rustの勉強", completed: false }}, poisoned: false, .. }

    let app = Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/{id}", patch(todos_update).delete(todos_delete))
        .layer(
            ServiceBuilder::new() // リクエスト処理パイプラインを追加でサーバー到達前に処理できるようになる(Towerライブラリの一部)
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                })) // HandleErrorLayerのクロージャはエラーを処理する
                .timeout(Duration::from_secs(10)) // 最大10秒のタイムアウト設定
                .layer(TraceLayer::new_for_http()) // リクエストとレスポンスの処理に関するトレース情報を収集し、デバッグや監視のためにログを記録
                .into_inner(), // Routerを適用
        ) // layer: リクエストパイプライン追加する(リクエストやレスポンスにカスタムロジックを追加できる)(tower::Layerトレイトを内部的に使用)
        .with_state(db); // 全てのハンドラーがdbにアクセスできるようになる(axumのState: アプリケーション全体で状態を保持する)

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080") // TCPリスナーのバインド
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap()); // サーバがリッスンしているアドレスとポートのポートの情報をデバッグログに出力
    axum::serve(listener, app).await.unwrap(); // HTTPサーバの起動
}

// 引数はアプリケーションの状態への参照(メモリ内データベース)
// State<T>: アプリケーションの状態をリクエストハンドラに注入(with_stateでaxumルーターに追加される)
//impl IntoResponse:IntoResponseに変換可能な任意の型を返すことを宣言
async fn todos_index(State(db): State<Db>) -> impl IntoResponse {
    let todos = db.read().unwrap(); // 読み取りロックを取得
    let todos = todos.values().cloned().collect::<Vec<_>>(); // HashMapのTodo値のイテレータ(参照)を返す。.collect::<Vec<_>>()でTodo値のベクターを生成(_は要素の型を推論させている)
    Json(todos)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

// Json(input): Json<CreateTodo>: CreateTodoにデシリアライズされる
async fn todos_create(State(db): State<Db>, Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: input.text,
        completed: false,
    };

    // write:書き込みロックを取得
    // insert:挿入する(キー(todo.id)と値(todo.clone()))
    db.write().unwrap().insert(todo.id, todo.clone());

    // HTTPステータスコードと作成されたTodo項目をJson形式で返す
    (StatusCode::CREATED, Json(todo))
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    // 検索(Todo項目)
    let mut todo = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;
    // フィールドの更新
    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    // データベースへの書き込み
    db.write().unwrap().insert(todo.id, todo.clone());

    // レスポンスの返却
    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// HashMap<K, V> 以下の場合は、UuidがkeyでTodoがvalue
type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}
