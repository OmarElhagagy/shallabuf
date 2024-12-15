use axum::{
    extract::{
        ws::{self, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use redis::AsyncCommands;
use sqlx::{pool::PoolConnection, Postgres};
use tokio::sync::broadcast;
use tracing::{debug, error};
use uuid::Uuid;

use crate::app_state::{
    Broadcast, DatabaseConnection, RedisConnection, RemoveEditorParticipant, WsAction,
    WsActionPayload,
};

pub async fn ws_events(
    ws: WebSocketUpgrade,
    DatabaseConnection(conn): DatabaseConnection,
    RedisConnection(redis): RedisConnection,
    State(sender): State<Broadcast>,
) -> impl IntoResponse {
    let sender_id = Uuid::new_v4(); // Assign unique ID for this connection
    ws.on_upgrade(move |socket| handle_ws_events(socket, conn, redis, sender.0, sender_id))
}

#[allow(clippy::too_many_lines)]
async fn handle_ws_events(
    mut socket: WebSocket,
    conn: PoolConnection<Postgres>,
    mut redis: redis::aio::ConnectionManager,
    sender: broadcast::Sender<WsAction>,
    sender_id: Uuid,
) {
    let mut receiver = sender.subscribe();

    loop {
        tokio::select! {
            res = socket.recv() => {
                match res {
                    Some(Ok(ws::Message::Text(message))) => {
                        debug!("Received message: {message}");

                        if let Ok(mut message) = serde_json::from_str::<WsAction>(&message) {
                            match message {
                                WsAction::UpdateNode(ref mut update) => {
                                    update.sender_id = Some(sender_id);

                                    if let Err(error) = sender.send(WsAction::UpdateNode(update.clone())) {
                                        error!("Failed to broadcast message: {error:?}");
                                    } else {
                                        debug!("Broadcasted message: {update:?}");
                                    }
                                }
                                WsAction::AddEditorParticipant(ref mut update) => {
                                    update.sender_id = Some(sender_id);

                                    debug!("Updating participant: {update:?}");

                                    match redis.hset(
                                        "pipeline:1:participants".to_string(),
                                        sender_id.to_string(),
                                        update.payload.username.clone()
                                    ).await {
                                        Ok(()) => {
                                            debug!("Updated participant: {update:?}");
                                        }
                                        Err(error) => {
                                            error!("Failed to update participant: {error:?}");
                                        }
                                    };

                                    if let Err(error) = sender.send(WsAction::AddEditorParticipant(update.clone())) {
                                        error!("Failed to broadcast message: {error:?}");
                                    } else {
                                        debug!("Broadcasted message: {update:?}");
                                    }
                                }
                                WsAction::RemoveEditorParticipant(ref mut update) => {
                                    update.sender_id = Some(sender_id);

                                    debug!("Removing participant: {update:?}");

                                    match redis.hdel(
                                        "pipeline:1:participants".to_string(),
                                        sender_id.to_string()
                                    ).await {
                                        Ok(()) => {
                                            debug!("Removed participant: {update:?}");
                                        }
                                        Err(error) => {
                                            error!("Failed to remove participant: {error:?}");
                                        }
                                    };

                                    if let Err(error) = sender.send(WsAction::RemoveEditorParticipant(update.clone())) {
                                        error!("Failed to broadcast message: {error:?}");
                                    } else {
                                        debug!("Broadcasted message: {update:?}");
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => tracing::debug!("client disconnected abruptly: {error}"),
                    None => {
                        debug!("WebSocket connection closed for sender_id: {sender_id}");

                        let update = WsAction::RemoveEditorParticipant(WsActionPayload {
                            sender_id: None,
                            payload: RemoveEditorParticipant {
                                pipeline_id: 1.to_string(),
                                user_id: sender_id.to_string(),
                            },
                        });

                        if let Err(error) = sender.send(update.clone()) {
                            error!("Failed to broadcast message: {error:?}");
                        } else {
                            debug!("Broadcasted message: {update:?}");
                        }

                        match redis.hdel("pipeline:1:participants".to_string(), sender_id.to_string()).await {
                            Ok(()) => {
                                debug!("Removed participant: {update:?}");
                            }
                            Err(error) => {
                                error!("Failed to remove participant: {error:?}");
                            }
                        };

                        break;
                    },
                }
            },
            res = receiver.recv() => {
                match res {
                    Ok(action) => {
                        match action {
                            WsAction::AddEditorParticipant(ref update) => {
                                if let Some(update_sender_id) = update.sender_id {
                                    if update_sender_id == sender_id {
                                        continue;
                                    }
                                }

                                if let Ok(text) = serde_json::to_string(&WsAction::AddEditorParticipant(update.clone())) {
                                    if let Err(error) = socket.send(Message::Text(text)).await {
                                        error!("Failed to send message to WebSocket: {error:?}");
                                        break;
                                    }

                                    debug!("Sent message to WebSocket: {update:?}");
                                }
                            }
                            WsAction::RemoveEditorParticipant(ref update) => {
                                if let Some(update_sender_id) = update.sender_id {
                                    if update_sender_id == sender_id {
                                        continue;
                                    }
                                }

                                if let Ok(text) = serde_json::to_string(&WsAction::RemoveEditorParticipant(update.clone())) {
                                    if let Err(error) = socket.send(Message::Text(text)).await {
                                        error!("Failed to send message to WebSocket: {error:?}");
                                        break;
                                    }
                                }

                                debug!("Sent message to WebSocket: {update:?}");
                            }
                            WsAction::UpdateNode(ref update) => {
                                if let Some(update_sender_id) = update.sender_id {
                                    if update_sender_id == sender_id {
                                        continue;
                                    }
                                }

                                if let Ok(text) = serde_json::to_string(&WsAction::UpdateNode(update.clone())) {
                                    if let Err(error) = socket.send(Message::Text(text)).await {
                                        error!("Failed to send message to WebSocket: {error:?}");
                                        break;
                                    }
                                }

                                debug!("Sent message to WebSocket: {update:?}");
                            }
                        }
                    }
                    Err(error) => {
                        error!("Failed to receive broadcast message: {error:?}");
                        break;
                    }
                }
            }
        }
    }
}
