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
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::{
    app_state::{
        AuthStatePayload, Broadcast, BroadcastEvent, BroadcastEventAction,
        BroadcastEventActionPayload, DatabaseConnection, ExcludePipelineEditorParticipantPayload,
        IncludePipelineEditorParticipantPayload, RedisConnection,
        UpdatePipelineEditorParticipantCursorPositionPayload,
        UpdatePipelineEditorParticipantNodePositionPayload, WsClientAction,
    },
    lib::session::validate_session_token,
};

pub async fn ws_events(
    ws: WebSocketUpgrade,
    DatabaseConnection(conn): DatabaseConnection,
    RedisConnection(redis): RedisConnection,
    State(sender): State<Broadcast>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_events(socket, conn, redis, sender.0))
}

#[allow(clippy::too_many_lines)]
async fn handle_ws_events(
    mut socket: WebSocket,
    mut conn: PoolConnection<Postgres>,
    mut redis: redis::aio::ConnectionManager,
    sender: broadcast::Sender<BroadcastEvent>,
) {
    let mut user_id = None::<Uuid>;
    let mut receiver = sender.subscribe();

    loop {
        tokio::select! {
            res = socket.recv() => {
                match res {
                    Some(Ok(ws::Message::Text(message))) => {
                        debug!("Received message: {message}");

                        if let Ok(mut message) = serde_json::from_str::<WsClientAction>(&message) {
                            match message {
                                WsClientAction::Authenticate(ref mut update) => {
                                    if let Ok(Some(session)) = validate_session_token(redis.clone(), &update.payload.token).await {
                                        debug!("Authenticated WebSocket connection for user_id: {:?}", session.user_id);
                                        user_id = Some(session.user_id);

                                        match serde_json::to_string(&BroadcastEventAction::AuthState(BroadcastEventActionPayload {
                                            payload: AuthStatePayload {
                                                is_authenticated: true,
                                            },
                                        })) {
                                            Ok(text) => {
                                                if let Err(error) = socket.send(Message::Text(text)).await {
                                                    error!("Failed to send message to WebSocket: {error:?}");
                                                    break;
                                                }
                                            }
                                            Err(error) => {
                                                error!("Failed to serialize message: {error:?}");
                                            }
                                        }

                                        continue;
                                    };

                                    match serde_json::to_string(&BroadcastEventAction::AuthState(BroadcastEventActionPayload {
                                        payload: AuthStatePayload {
                                            is_authenticated: false,
                                        },
                                    })) {
                                        Ok(text) => {
                                            if let Err(error) = socket.send(Message::Text(text)).await {
                                                error!("Failed to send message to WebSocket: {error:?}");
                                                break;
                                            }
                                        }
                                        Err(error) => {
                                            error!("Failed to serialize message: {error:?}");
                                        }
                                    }
                                }
                                WsClientAction::UpdateNodePosition(action) => {
                                    if let Some(user_id) = user_id {
                                        let event = BroadcastEvent {
                                            sender_id: user_id,
                                            action: BroadcastEventAction::UpdateNodePosition(BroadcastEventActionPayload {
                                                payload: UpdatePipelineEditorParticipantNodePositionPayload {
                                                    pipeline_id: action.payload.pipeline_id,
                                                    user_id,
                                                    node_id: action.payload.node_id,
                                                    node_position: action.payload.node_position,
                                                },
                                            }),
                                        };

                                        match sender.send(event.clone()) {
                                            Ok(_) => {
                                                debug!("Broadcasted message: {event:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to broadcast message: {error:?}");
                                            }
                                        }
                                    } else {
                                        warn!("WsAction::UpdateNodePosition: User isn't authenticated");
                                    }
                                }
                                WsClientAction::EnterPipelineEditor(action) => {
                                    if let Some(user_id) = user_id {
                                        let user = match sqlx::query!(
                                            r#"
                                            SELECT
                                                name
                                            FROM
                                                users
                                            WHERE
                                                id = $1
                                            "#,
                                            user_id
                                        )
                                        .fetch_one(&mut *conn)
                                        .await {
                                            Ok(user) => user,
                                            Err(error) => {
                                                error!("Failed to fetch user: {error:?}");
                                                continue;
                                            }
                                        };

                                        match redis.hset(
                                            to_pipeline_participant_redis_key(action.payload.pipeline_id),
                                            user_id.to_string(),
                                            user.name.clone(),
                                        ).await {
                                            Ok(()) => {
                                                debug!("Updated participant: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to update participant: {error:?}");
                                            }
                                        };

                                        match redis.sadd(
                                            to_participant_pipelines_redis_key(user_id),
                                            action.payload.pipeline_id.to_string(),
                                        ).await {
                                            Ok(()) => {
                                                debug!("Updated participant: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to update participant: {error:?}");
                                            }
                                        };

                                        let event = BroadcastEvent {
                                            sender_id: user_id,
                                            action: BroadcastEventAction::IncludePipelineEditorParticipant(BroadcastEventActionPayload {
                                                payload: IncludePipelineEditorParticipantPayload {
                                                    pipeline_id: action.payload.pipeline_id,
                                                    user_id,
                                                    username: user.name,
                                                },
                                            }),
                                        };

                                        match sender.send(event.clone()) {
                                            Ok(_) => {
                                                debug!("Broadcasted message: {event:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to broadcast message: {error:?}");
                                            }
                                        }
                                    } else {
                                        warn!("WsAction::EnterPipelineEditor: User isn't authenticated");
                                    }
                                }
                                WsClientAction::LeavePipelineEditor(action) => {
                                    if let Some(user_id) = user_id {
                                        match redis.hdel(
                                            to_pipeline_participant_redis_key(action.payload.pipeline_id),
                                            user_id.to_string(),
                                        ).await {
                                            Ok(()) => {
                                                debug!("Removed participant: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to remove participant: {error:?}");
                                            }
                                        };

                                        match redis.srem(
                                            to_participant_pipelines_redis_key(user_id),
                                            action.payload.pipeline_id.to_string(),
                                        ).await {
                                            Ok(()) => {
                                                debug!("Removed participant: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to remove participant: {error:?}");
                                            }
                                        };

                                        match redis.del(to_cursors_redis_key(action.payload.pipeline_id, user_id)).await {
                                            Ok(()) => {
                                                debug!("Removed cursor: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to remove cursor: {error:?}");
                                            }
                                        };

                                        let event = BroadcastEvent {
                                            sender_id: user_id,
                                            action: BroadcastEventAction::ExcludePipelineEditorParticipant(BroadcastEventActionPayload {
                                                payload: ExcludePipelineEditorParticipantPayload {
                                                    pipeline_id: action.payload.pipeline_id,
                                                    user_id,
                                                },
                                            }),
                                        };

                                        match sender.send(event.clone()) {
                                            Ok(_) => {
                                                debug!("Broadcasted message: {event:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to broadcast message: {error:?}");
                                            }
                                        }
                                    } else {
                                        warn!("WsAction::LeavePipelineEditor: User isn't authenticated");
                                    }
                                }
                                WsClientAction::UpdateCursorPosition(action) => {
                                    if let Some(user_id) = user_id {
                                        let cursor_position = match serde_json::to_string(&action.payload.cursor_position) {
                                            Ok(cursor_position_value) => cursor_position_value,
                                            Err(error) => {
                                                error!("Failed to serialize cursor_position: {error:?}");
                                                continue;
                                            }
                                        };

                                        match redis.set(
                                            to_cursors_redis_key(action.payload.pipeline_id, user_id),
                                            cursor_position
                                        ).await {
                                            Ok(()) => {
                                                debug!("Updated cursor: {action:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to update cursor: {error:?}");
                                            }
                                        };

                                        let event = BroadcastEvent {
                                            sender_id: user_id,
                                            action: BroadcastEventAction::UpdateCursorPosition(BroadcastEventActionPayload {
                                                payload: UpdatePipelineEditorParticipantCursorPositionPayload {
                                                    pipeline_id: action.payload.pipeline_id,
                                                    user_id,
                                                    cursor_position: action.payload.cursor_position,
                                                },
                                            }),
                                        };

                                        match sender.send(event.clone()) {
                                            Ok(_) => {
                                                debug!("Broadcasted message: {event:?}");
                                            }
                                            Err(error) => {
                                                error!("Failed to broadcast message: {error:?}");
                                            }
                                        }
                                    } else {
                                        warn!("WsAction::UpdateCursorPosition: User isn't authenticated");
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(_)) => {
                        warn!("Received non-Text message");
                    }
                    Some(Err(error)) => {
                        error!("Failed to receive message: {error:?}");

                        if let Some(user_id) = user_id {
                            exclude_from_all_pipelines(redis.clone(), sender, user_id).await;
                        }

                        break;
                    },
                    None => {
                        debug!("WebSocket connection closed for sender_id: {user_id:?}");

                        if let Some(user_id) = user_id {
                            exclude_from_all_pipelines(redis.clone(), sender, user_id).await;
                        }

                        break;
                    },
                }
            },
            message = receiver.recv() => {
                match message {
                    Ok(event) => {
                        if let Some(user_id) = user_id {
                            if event.sender_id == user_id {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        match serde_json::to_string(&event.action) {
                            Ok(text) => {
                                if let Err(error) = socket.send(Message::Text(text)).await {
                                    error!("Failed to send message to WebSocket: {error:?}");
                                    break;
                                }

                                debug!("Sent message to WebSocket: {event:?}");
                            }
                            Err(error) => {
                                error!("Failed to serialize message: {error:?}");
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

pub async fn exclude_from_all_pipelines(
    mut redis: redis::aio::ConnectionManager,
    sender: broadcast::Sender<BroadcastEvent>,
    user_id: Uuid,
) {
    let pipeline_ids: Vec<String> = match redis
        .smembers(to_participant_pipelines_redis_key(user_id))
        .await
    {
        Ok(pipeline_ids) => pipeline_ids,
        Err(error) => {
            error!("Failed to fetch pipeline_ids: {error:?}");
            return;
        }
    };

    let pipeline_ids: Vec<Uuid> = pipeline_ids
        .into_iter()
        .filter_map(|pipeline_id_str| Uuid::parse_str(&pipeline_id_str).ok())
        .collect();

    for pipeline_id in pipeline_ids {
        let _: () = match redis
            .hdel(
                to_pipeline_participant_redis_key(pipeline_id),
                user_id.to_string(),
            )
            .await
        {
            Ok(()) => {
                debug!("Removed participant: {pipeline_id:?}");
            }
            Err(error) => {
                error!("Failed to remove participant: {error:?}");
            }
        };

        let _: () = match redis.del(to_cursors_redis_key(pipeline_id, user_id)).await {
            Ok(()) => {
                debug!("Removed cursor: {pipeline_id:?}");
            }
            Err(error) => {
                error!("Failed to remove cursor: {error:?}");
            }
        };

        let event = BroadcastEvent {
            sender_id: user_id,
            action: BroadcastEventAction::ExcludePipelineEditorParticipant(
                BroadcastEventActionPayload {
                    payload: ExcludePipelineEditorParticipantPayload {
                        pipeline_id,
                        user_id,
                    },
                },
            ),
        };

        match sender.send(event.clone()) {
            Ok(_) => {
                debug!("Broadcasted message: {event:?}");
            }
            Err(error) => {
                error!("Failed to broadcast message: {error:?}");
            }
        }
    }

    let _: () = match redis.del(to_participant_pipelines_redis_key(user_id)).await {
        Ok(()) => {
            debug!("Removed participant: {user_id:?}");
        }
        Err(error) => {
            error!("Failed to remove participant: {error:?}");
        }
    };
}

pub fn to_pipeline_participant_redis_key(pipeline_id: Uuid) -> String {
    format!("pipelines:{pipeline_id}:participants")
}

pub fn to_participant_pipelines_redis_key(user_id: Uuid) -> String {
    format!("participants:{user_id}:pipelines")
}

pub fn to_cursors_redis_key(pipeline_id: Uuid, user_id: Uuid) -> String {
    format!("cursors:{pipeline_id}:{user_id}")
}
