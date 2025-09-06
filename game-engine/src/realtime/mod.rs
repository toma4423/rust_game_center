use crate::room::GameRoom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::serde_as;
use std::time::{Duration, Instant};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Timer {
    #[serde_as(as = "serde_with::DurationSeconds<f64>")]
    pub duration: Duration,
    #[serde(skip)]
    pub started_at: Option<Instant>,
    pub timer_type: String,
    pub auto_action: Option<String>,
}

// クライアントから受信するリアルタイムアクション
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "action")]
pub enum ClientRealtimeAction {
    #[serde(rename = "first_press")]
    FirstPress,
    #[serde(rename = "turn_action")]
    TurnAction {
        action_type: String,
        data: serde_json::Value,
    },
}

// サーバー内部でキューに積むためのアクション
#[derive(Clone, Debug)]
pub enum QueuedRealtimeAction {
    FirstPress {
        player_id: String,
        timestamp: Instant,
    },
    TurnAction {
        player_id: String,
        action_type: String,
        data: serde_json::Value,
    },
}

pub async fn process_action(room: &mut GameRoom, action: ClientRealtimeAction, player_id: &str) {
    let room_id = room.id.clone();
    // TODO: ゲームソフトのAPIエンドポイントをroom.selected_gameから取得する
    let game_api_endpoint = "http://localhost:5001".to_string(); // 仮

    match action {
        ClientRealtimeAction::FirstPress => {
            if room.realtime_state.first_press_winner.is_some() {
                tracing::warn!(
                    "First press action ignored, winner already decided in room {}",
                    room.id
                );
                return;
            }

            room.realtime_state.first_press_winner = Some(player_id.to_string());
            tracing::info!("First press by {} in room {}", player_id, room.id);

            // ゲームソフトにHTTP APIで通知
            let player_id_clone = player_id.to_string();
            tokio::spawn(async move {
                let client = reqwest::Client::new();
                let endpoint = format!("{}/game/event", game_api_endpoint);
                let payload = json!({
                    "room_id": room_id,
                    "data": {
                        "action": "first_press",
                        "winner": player_id_clone,
                    }
                });

                tracing::info!("Notifying game software at {}: {}", endpoint, payload);
                match client.post(&endpoint).json(&payload).send().await {
                    Ok(resp) => {
                        tracing::info!("Game software notified, status: {}", resp.status())
                    }
                    Err(e) => tracing::error!("Failed to notify game software: {}", e),
                }
            });
        }
        ClientRealtimeAction::TurnAction { action_type, data: _data } => {
            if room.realtime_state.current_turn.as_deref() != Some(player_id) {
                tracing::warn!(
                    "Turn action from non-turn player {} ignored in room {}",
                    player_id, room.id
                );
                return;
            }

            tracing::info!(
                "Turn action '{}' by {} in room {}",
                action_type, player_id, room.id
            );
            // TODO: アクションを処理し、ゲームソフトに通知
        }
    }
}
