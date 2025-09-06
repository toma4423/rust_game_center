use crate::{
    game_api::{broadcast_handler, enable_action_handler, next_phase_handler},
    room::{GameInfo, GameRoom},
    websocket::handle_socket,
};
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// モジュール宣言
pub mod game_api;
pub mod realtime;
pub mod room;
pub mod websocket;

// アプリケーション全体で共有する状態
#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<DashMap<String, GameRoom>>,
    pub available_games: Arc<Vec<GameInfo>>,
}

#[tokio::main]
async fn main() {
    // ロギング設定
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "game_engine=debug,tower_http=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 利用可能なゲームリストを初期化（ハードコード）
    let available_games = Arc::new(vec![GameInfo {
        id: "quiz".to_string(),
        title: "早押しクイズ".to_string(),
        description: "シンプルな早押しクイズゲームです。".to_string(),
        min_players: 2,
        max_players: 8,
        recommended_players: 4,
        api_endpoint: "http://localhost:5001".to_string(),
        settings_schema: serde_json::json!({}),
    }]);

    // 共有状態の初期化
    let state = AppState {
        rooms: Arc::new(DashMap::new()),
        available_games,
    };

    // CORS設定
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // アプリケーションのルーティング設定
    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(ws_handler))
        // Game API routes
        .route("/realtime/broadcast", post(broadcast_handler))
        .route("/realtime/enable_action", post(enable_action_handler))
        .route("/game/next_phase", post(next_phase_handler))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(cors));

    // サーバーのアドレスとポート
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    // サーバーを起動
    axum::serve(listener, app).await.unwrap();
}

// ルートパスへのリクエストを処理するハンドラー
async fn root() -> &'static str {
    "Hello, World! This is the GameCenter."
}

// WebSocketへのアップグレードを処理するハンドラー
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}
