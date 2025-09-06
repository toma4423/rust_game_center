use crate::realtime::{process_action, ClientRealtimeAction};
use crate::room::{GameRoom, Player};
use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

// 5.1.1 クライアント -> サーバー
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMsg {
    #[serde(rename = "create_room")]
    CreateRoom { display_name: String },
    #[serde(rename = "join_room")]
    JoinRoom {
        room_id: String,
        display_name: String,
    },
    #[serde(rename = "select_game")]
    SelectGame { game_id: String },
    #[serde(rename = "start_game")]
    StartGame,
    #[serde(rename = "realtime_action")]
    RealtimeAction {
        #[serde(flatten)]
        action: ClientRealtimeAction,
    },
}

// 5.1.2 サーバー -> クライアント
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMsg {
    #[serde(rename = "room_update")]
    RoomUpdate { room: GameRoom },
    #[serde(rename = "game_state")]
    GameState {
        phase: String,
        data: serde_json::Value,
    },
    #[serde(rename = "broadcast_event")]
    BroadcastEvent {
        event: String,
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

fn error_response(message: &str) -> String {
    serde_json::to_string(&ServerMsg::Error { message: message.to_string() }).unwrap()
}

pub async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let player_id = Uuid::new_v4().to_string();
    tracing::info!("New WebSocket connection: {}", player_id);

    let mut room_rx: Option<broadcast::Receiver<String>> = None;
    let mut current_room_id: Option<String> = None;

    loop {
        tokio::select! {
            biased; // クライアントからのメッセージを優先

            result = socket.next() => {
                let msg = match result {
                    Some(Ok(msg)) => msg,
                    _ => { break; }
                };

                if let Message::Text(text) = msg {
                    match serde_json::from_str::<ClientMsg>(&text) {
                        Ok(client_msg) => {
                            let (response_str, new_rx_opt, new_room_id_opt) = handle_client_message(client_msg, &player_id, &state, &current_room_id).await;
                            if let Some(new_rx) = new_rx_opt { room_rx = Some(new_rx); }
                            if let Some(new_room_id) = new_room_id_opt { current_room_id = Some(new_room_id); }
                            if socket.send(Message::Text(response_str)).await.is_err() { break; }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse msg from {}: {}", player_id, e);
                            let err_msg = error_response(&e.to_string());
                            if socket.send(Message::Text(err_msg)).await.is_err() { break; }
                        }
                    }
                }
            },

            Ok(msg) = async { match room_rx.as_mut() { Some(rx) => rx.recv().await, None => futures_util::future::pending().await } } => {
                if socket.send(Message::Text(msg)).await.is_err() { break; }
            },
        }
    }

    // --- 切断処理 ---
    if let Some(room_id) = current_room_id {
        tracing::info!("Player {} disconnecting from room {}", player_id, room_id);
        if let Some(mut room) = state.rooms.get_mut(&room_id) {
            room.players.retain(|p| p.id != player_id);

            let mut new_host_id = None;
            if room.host_id == player_id {
                if let Some(next_host) = room.players.first() {
                    new_host_id = Some(next_host.id.clone());
                }
            }
            if let Some(id) = new_host_id {
                room.host_id = id.clone();
                tracing::info!("Host changed to {} in room {}", id, room_id);
            }

            let is_empty = room.players.is_empty();
            let room_clone = room.value().clone();
            let response = ServerMsg::RoomUpdate { room: room_clone };
            let response_str = serde_json::to_string(&response).unwrap();
            if let Some(tx) = &room.tx {
                tx.send(response_str).ok();
            }

            if is_empty {
                drop(room);
                state.rooms.remove(&room_id);
                tracing::info!("Room {} is empty, removed.", room_id);
            }
        }
    }
    tracing::info!("WebSocket connection closed: {}", player_id);
}

async fn handle_client_message(
    client_msg: ClientMsg,
    player_id: &str,
    state: &AppState,
    current_room_id: &Option<String>,
) -> (String, Option<broadcast::Receiver<String>>, Option<String>) {
    match client_msg {
        ClientMsg::CreateRoom { display_name } => {
            tracing::info!("Player {} creating room", display_name);
            let room = GameRoom::new(player_id.to_string(), display_name);
            let room_id = room.id.clone();
            let new_rx = room.tx.as_ref().map(|tx| tx.subscribe());

            let response = ServerMsg::RoomUpdate { room: room.clone() };
            state.rooms.insert(room_id.clone(), room);

            (serde_json::to_string(&response).unwrap(), new_rx, Some(room_id))
        }
        ClientMsg::JoinRoom { room_id, display_name } => {
            tracing::info!("Player {} joining room {}", display_name, room_id);
            if let Some(mut room) = state.rooms.get_mut(&room_id) {
                let player = Player {
                    id: player_id.to_string(),
                    display_name,
                };
                room.players.push(player);
                let new_rx = room.tx.as_ref().map(|tx| tx.subscribe());

                let response = ServerMsg::RoomUpdate { room: room.value().clone() };
                let response_str = serde_json::to_string(&response).unwrap();

                if let Some(tx) = &room.tx {
                    tx.send(response_str.clone()).ok();
                }

                (response_str, new_rx, Some(room_id))
            } else {
                (error_response(&format!("Room '{}' not found", room_id)), None, None)
            }
        }
        ClientMsg::SelectGame { game_id } => {
            let room_id = match current_room_id {
                Some(id) => id,
                None => return (error_response("Not in a room"), None, None),
            };

            if let Some(mut room) = state.rooms.get_mut(room_id) {
                if room.host_id != player_id {
                    return (error_response("Only the host can select a game"), None, None);
                }

                if let Some(game_info) = state.available_games.iter().find(|g| g.id == game_id) {
                    room.selected_game = Some(game_info.clone());
                    
                    let response = ServerMsg::RoomUpdate { room: room.value().clone() };
                    let response_str = serde_json::to_string(&response).unwrap();
                    if let Some(tx) = &room.tx {
                        tx.send(response_str.clone()).ok();
                    }
                    (response_str, None, None)
                } else {
                    (error_response("Game not found"), None, None)
                }
            } else {
                (error_response("Room not found"), None, None)
            }
        }
        ClientMsg::StartGame => {
            let room_id = match current_room_id {
                Some(id) => id,
                None => return (error_response("Not in a room"), None, None),
            };

            if let Some(mut room) = state.rooms.get_mut(room_id) {
                if room.host_id != player_id {
                    return (error_response("Only the host can start the game"), None, None);
                }
                if room.selected_game.is_none() {
                    return (error_response("No game selected"), None, None);
                }

                room.state = crate::room::RoomState::InGame;
                
                let room_clone = room.value().clone();
                tokio::spawn(async move {
                    if let Some(game) = room_clone.selected_game {
                        let client = reqwest::Client::new();
                        let endpoint = format!("{}/game/init", game.api_endpoint);
                        let payload = serde_json::json!({
                            "room_id": room_clone.id,
                            "players": room_clone.players,
                            "settings": room_clone.settings,
                        });
                        
                        tracing::info!("Initializing game at {}: {}", endpoint, payload);
                        if let Err(e) = client.post(&endpoint).json(&payload).send().await {
                            tracing::error!("Failed to initialize game: {}", e);
                        }
                    }
                });

                let response = ServerMsg::RoomUpdate { room: room.value().clone() };
                let response_str = serde_json::to_string(&response).unwrap();
                if let Some(tx) = &room.tx {
                    tx.send(response_str.clone()).ok();
                }
                (response_str, None, None)
            } else {
                (error_response("Room not found"), None, None)
            }
        }
        ClientMsg::RealtimeAction { action } => {
            let room_id = match current_room_id {
                Some(id) => id,
                None => return (error_response("Not in a room"), None, None),
            };

            if let Some(mut room) = state.rooms.get_mut(room_id) {
                process_action(&mut room, action, player_id).await;

                let response = ServerMsg::RoomUpdate { room: room.value().clone() };
                let response_str = serde_json::to_string(&response).unwrap();
                if let Some(tx) = &room.tx {
                    tx.send(response_str.clone()).ok();
                }

                (response_str, None, None)
            } else {
                (error_response("Room not found"), None, None)
            }
        }
    }
}
