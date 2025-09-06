use crate::{
    websocket::ServerMsg, // websocketモジュールはトップレベル
    AppState,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Deserialize;

// APIリクエストに共通で含まれるルームID
#[derive(Deserialize, Debug)]
pub struct RoomRequest {
    pub room_id: String,
}

#[derive(Deserialize, Debug)]
pub struct BroadcastRequest {
    #[serde(flatten)]
    pub room: RoomRequest,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize, Debug)]
pub struct EnableActionRequest {
    #[serde(flatten)]
    pub room: RoomRequest,
    pub action_type: String, // e.g., "first_press"
    pub enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct NextPhaseRequest {
    #[serde(flatten)]
    pub room: RoomRequest,
    pub phase: String,
    pub ui_data: serde_json::Value,
}

pub async fn broadcast_handler(
    State(state): State<AppState>,
    Json(payload): Json<BroadcastRequest>,
) -> StatusCode {
    if let Some(room) = state.rooms.get(&payload.room.room_id) {
        let broadcast_msg = ServerMsg::BroadcastEvent {
            event: payload.event_type,
            data: payload.data,
        };
        let broadcast_str = serde_json::to_string(&broadcast_msg).unwrap();
        if let Some(tx) = &room.tx {
            tx.send(broadcast_str).ok();
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn enable_action_handler(
    State(state): State<AppState>,
    Json(payload): Json<EnableActionRequest>,
) -> StatusCode {
    if let Some(room) = state.rooms.get(&payload.room.room_id) {
        // TODO: 実際にアクションを有効/無効にするロジックを room.realtime_state に実装する必要がある
        tracing::info!(
            "Action '{}' in room {} set to {}",
            payload.action_type,
            payload.room.room_id,
            payload.enabled
        );
        
        // 状態変化をブロードキャスト
        let broadcast_msg = ServerMsg::RoomUpdate {
            room: room.value().clone(),
        };
        let broadcast_str = serde_json::to_string(&broadcast_msg).unwrap();
        if let Some(tx) = &room.tx {
            tx.send(broadcast_str).ok();
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn next_phase_handler(
    State(state): State<AppState>,
    Json(payload): Json<NextPhaseRequest>,
) -> StatusCode {
    if let Some(mut room) = state.rooms.get_mut(&payload.room.room_id) {
        room.realtime_state.game_phase = payload.phase.clone();

        let broadcast_msg = ServerMsg::GameState {
            phase: payload.phase,
            data: payload.ui_data,
        };
        let broadcast_str = serde_json::to_string(&broadcast_msg).unwrap();
        if let Some(tx) = &room.tx {
            tx.send(broadcast_str).ok();
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}