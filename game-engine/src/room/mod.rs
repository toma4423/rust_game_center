use crate::realtime::{QueuedRealtimeAction, Timer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use tokio::sync::broadcast;

// 設計書 4.3. ゲーム情報
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub min_players: u8,
    pub max_players: u8,
    pub recommended_players: u8,
    pub api_endpoint: String,
    pub settings_schema: serde_json::Value,
}

// 設計書 4.2. リアルタイム状態
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RealtimeState {
    pub current_turn: Option<String>,
    pub turn_order: Vec<String>,
    pub active_timers: HashMap<String, Timer>,
    pub first_press_winner: Option<String>,
    #[serde(skip)]
    pub pending_actions: VecDeque<QueuedRealtimeAction>,
    pub game_phase: String,
}


// ユーザー情報
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: String, // セッションID (UUID)
    pub display_name: String,
}

// ルーム設定
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomSettings {
    pub max_players: u8,
    pub room_liberation_time: u64, // 分単位
    pub progression_rule: String, // "全員一致", "過半数", "単独進行"
}

// ルームの状態
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RoomState {
    Waiting,
    InGame,
    Finished,
}

// 設計書 4.1. ルーム情報
#[derive(Serialize, Clone, Debug)]
pub struct GameRoom {
    pub id: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub settings: RoomSettings,
    pub selected_game: Option<GameInfo>,
    pub state: RoomState,
    pub realtime_state: RealtimeState,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    #[serde(skip)]
    pub tx: Option<broadcast::Sender<String>>,
}

impl GameRoom {
    pub fn new(host_id: String, host_display_name: String) -> Self {
        let host = Player {
            id: host_id.clone(),
            display_name: host_display_name,
        };
        let (tx, _rx) = broadcast::channel(100);

        Self {
            id: generate_room_id(),
            host_id,
            players: vec![host],
            settings: RoomSettings {
                max_players: 8, // デフォルト値
                room_liberation_time: 15,
                progression_rule: "単独進行".to_string(),
            },
            selected_game: None,
            state: RoomState::Waiting,
            realtime_state: RealtimeState {
                current_turn: None,
                turn_order: vec![],
                active_timers: HashMap::new(),
                first_press_winner: None,
                pending_actions: VecDeque::new(),
                game_phase: "lobby".to_string(),
            },
            created_at: Utc::now(),
            last_activity: Utc::now(),
            tx: Some(tx),
        }
    }
}

// 5桁のランダムな数字のルームIDを生成
fn generate_room_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..=99999).to_string()
}
