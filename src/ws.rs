use crate::pusher::{ConnectionEstablishedData, PusherMessage};
use crate::state::AppState;
use axum::{
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(app_key): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("New connection attempt for app_key: {}", app_key);
    ws.on_upgrade(move |socket| handle_socket(socket, state, app_key))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, app_key: String) {
    let socket_id = format!(
        "{}.{}",
        Uuid::new_v4().as_u128() % 1000000000,
        Uuid::new_v4().as_u128() % 1000000000
    );

    // Find app by key to get the app_id and secret
    let (app_id, app_secret) = match state.app_manager.find_by_key(&app_key).await {
        Ok(Some(app)) => {
            tracing::debug!("Found app by key '{}': app_id = '{}'", app_key, app.id);
            (app.id.clone(), app.secret.clone())
        }
        Ok(None) => {
            warn!("App not found for key: {}", app_key);
            return;
        }
        Err(e) => {
            warn!("Error finding app: {}", e);
            return;
        }
    };

    let (mut sender, mut receiver) = socket.split();

    let conn_data = ConnectionEstablishedData {
        socket_id: socket_id.clone(),
        activity_timeout: 120,
    };

    let msg = PusherMessage {
        event: "pusher:connection_established".to_string(),
        data: Some(serde_json::to_value(&conn_data).unwrap()),
        channel: None,
    };

    if let Ok(json) = serde_json::to_string(&msg)
        && sender.send(Message::Text(json.into())).await.is_err()
    {
        return;
    }

    info!("Socket established: {}", socket_id);

    // Channel to receive messages from subscriptions to send to client
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(100);

    // Add socket to adapter
    let socket_obj = crate::namespace::Socket {
        id: socket_id.clone(),
        sender: tx.clone(),
    };
    
    if let Err(e) = state.adapter.add_socket(&app_id, socket_obj).await {
        warn!("Failed to add socket to adapter: {}", e);
        return;
    }
    
    tracing::debug!("Socket added to adapter: {}", socket_id);

    // Task to forward messages from tx to sender
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Main loop for receiving messages from client
    let socket_id_clone = socket_id.clone();
    let recv_task = tokio::spawn(async move {
        let socket_id = socket_id_clone;
        let subscriptions: Vec<(String, tokio::task::JoinHandle<()>)> = Vec::new();

        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        info!("Received: {}", text);
                        if let Ok(pusher_msg) = serde_json::from_str::<PusherMessage>(&text) {
                            match pusher_msg.event.as_str() {
                                "pusher:ping" => {
                                    info!("Handling ping");
                                    let pong = PusherMessage {
                                        event: "pusher:pong".to_string(),
                                        data: Some(serde_json::json!({})),
                                        channel: None,
                                    };
                                    let _ = tx
                                        .send(Message::Text(
                                            serde_json::to_string(&pong).unwrap().into(),
                                        ))
                                        .await;
                                }
                                "pusher:subscribe" => {
                                    info!("Handling subscribe event");
                                    // Handle data being Option
                                    if let Some(data) = &pusher_msg.data {
                                        info!("Data present: {:?}", data);
                                        if let Some(channel_name) =
                                            data.get("channel").and_then(|v| v.as_str())
                                        {
                                            let channel_name = channel_name.to_string();
                                            // Auth check for private/presence channels
                                            if channel_name.starts_with("private-")
                                                || channel_name.starts_with("presence-")
                                            {
                                                let submitted_auth = pusher_msg
                                                    .data
                                                    .as_ref()
                                                    .and_then(|d| d.get("auth"))
                                                    .and_then(|v| v.as_str());

                                                if let Some(signature) = submitted_auth {
                                                    // Signature format: app_key:signature
                                                    let parts: Vec<&str> =
                                                        signature.split(':').collect();
                                                    if parts.len() != 2 || parts[0] != app_key {
                                                        warn!("Invalid auth key in signature");
                                                        let error_msg = PusherMessage {
                                                            event: "pusher:subscription_error"
                                                                .to_string(),
                                                            data: Some(serde_json::json!({
                                                                "type": "AuthError",
                                                                "error": "Invalid auth key",
                                                                "status": 401
                                                            })),
                                                            channel: Some(channel_name.clone()),
                                                        };
                                                        let _ = tx
                                                            .send(Message::Text(
                                                                serde_json::to_string(&error_msg)
                                                                    .unwrap()
                                                                    .into(),
                                                            ))
                                                            .await;
                                                        continue;
                                                    }

                                                    let to_sign = if channel_name
                                                        .starts_with("presence-")
                                                    {
                                                        let channel_data = pusher_msg
                                                            .data
                                                            .as_ref()
                                                            .and_then(|d| d.get("channel_data"))
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("{}");
                                                        format!(
                                                            "{}:{}:{}",
                                                            socket_id, channel_name, channel_data
                                                        )
                                                    } else {
                                                        format!("{}:{}", socket_id, channel_name)
                                                    };

                                                    if !crate::auth::verify_pusher_signature(
                                                        parts[1],
                                                        &app_secret,
                                                        &to_sign,
                                                    ) {
                                                        warn!(
                                                            "Invalid signature for channel {}",
                                                            channel_name
                                                        );
                                                        // ... error handling
                                                        let error_msg = PusherMessage {
                                                            event: "pusher:subscription_error"
                                                                .to_string(),
                                                            data: Some(serde_json::json!({
                                                                "type": "AuthError",
                                                                "error": "Invalid signature",
                                                                "status": 401
                                                            })),
                                                            channel: Some(channel_name.clone()),
                                                        };
                                                        let _ = tx
                                                            .send(Message::Text(
                                                                serde_json::to_string(&error_msg)
                                                                    .unwrap()
                                                                    .into(),
                                                            ))
                                                            .await;
                                                        continue;
                                                    }

                                                    // If presence, add member
                                                    if channel_name.starts_with("presence-") {
                                                        let channel_data_str = pusher_msg
                                                            .data
                                                            .as_ref()
                                                            .and_then(|d| d.get("channel_data"))
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("{}");

                                                        let channel_data: serde_json::Value =
                                                            serde_json::from_str(channel_data_str)
                                                                .unwrap_or(serde_json::json!({}));
                                                        let user_id = channel_data
                                                            .get("user_id")
                                                            .and_then(|v| v.as_str())
                                                            .unwrap_or("unknown")
                                                            .to_string();
                                                        let user_info = channel_data
                                                            .get("user_info")
                                                            .cloned()
                                                            .unwrap_or(serde_json::Value::Null);

                                                        let member = crate::app::PresenceMember {
                                                            user_id: user_id.clone(),
                                                            user_info: user_info.clone(),
                                                        };

                                                        let _ = state
                                                            .adapter
                                                            .add_member(
                                                                &app_id,
                                                                &channel_name,
                                                                &socket_id,
                                                                member,
                                                            )
                                                            .await;

                                                        // Broadcast member_added to other subscribers
                                                        let member_added_msg = PusherMessage {
                                                            event: "pusher_internal:member_added".to_string(),
                                                            data: Some(serde_json::json!({
                                                                "user_id": user_id,
                                                                "user_info": user_info
                                                            })),
                                                            channel: Some(channel_name.clone()),
                                                        };
                                                        
                                                        if let Ok(msg_str) = serde_json::to_string(&member_added_msg) {
                                                            let _ = state.adapter.send(
                                                                &app_id,
                                                                &channel_name,
                                                                &msg_str,
                                                                Some(&socket_id)
                                                            ).await;
                                                        }
                                                    }
                                                } else {
                                                    warn!(
                                                        "Missing auth for private/presence channel {}",
                                                        channel_name
                                                    );
                                                    let error_msg = PusherMessage {
                                                        event: "pusher:subscription_error"
                                                            .to_string(),
                                                        data: Some(serde_json::json!({
                                                            "type": "AuthError",
                                                            "error": "Auth missing",
                                                            "status": 401
                                                        })),
                                                        channel: Some(channel_name.clone()),
                                                    };
                                                    let _ = tx
                                                        .send(Message::Text(
                                                            serde_json::to_string(&error_msg)
                                                                .unwrap()
                                                                .into(),
                                                        ))
                                                        .await;
                                                    continue;
                                                }
                                            }

                                            // Subscribe to channel via adapter
                                            if let Err(e) = state
                                                .adapter
                                                .add_to_channel(&app_id, &channel_name, socket_id.to_string())
                                                .await
                                            {
                                                warn!("Failed to subscribe to channel {}: {}", channel_name, e);
                                            } else {
                                                info!("Successfully subscribed to channel: {}", channel_name);
                                            }

                                            // Send subscription_succeeded
                                            let mut data = None;
                                            if channel_name.starts_with("presence-") {
                                                // Prepare presence data (ids, hash, count)
                                                if let Ok(members) = state
                                                    .adapter
                                                    .get_channel_members(&app_id, &channel_name)
                                                    .await
                                                {
                                                    let mut ids = Vec::new();
                                                    let mut hash = std::collections::HashMap::new();

                                                    for (_, member) in members {
                                                        let uid = &member.user_id;
                                                        if !ids.contains(uid) {
                                                            ids.push(uid.clone());
                                                        }
                                                        hash.insert(
                                                            uid.clone(),
                                                            member.user_info.clone(),
                                                        );
                                                    }

                                                    data = Some(serde_json::json!({
                                                        "presence": {
                                                            "ids": ids,
                                                            "hash": hash,
                                                            "count": ids.len()
                                                        }
                                                    }));
                                                }
                                            }

                                            let success_msg = PusherMessage {
                                                event: "pusher_internal:subscription_succeeded"
                                                    .to_string(),
                                                data,
                                                channel: Some(channel_name.clone()),
                                            };
                                            info!("Sending success message");
                                            let _ = tx
                                                .send(Message::Text(
                                                    serde_json::to_string(&success_msg)
                                                        .unwrap()
                                                        .into(),
                                                ))
                                                .await;
                                        } else {
                                            info!("Channel field missing in data or not a string");
                                        }
                                    } else {
                                        info!("Data field missing in subscribe message");
                                    }
                                }
                                _ => {
                                    info!("Ignoring event: {}", pusher_msg.event);
                                }
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            } else {
                break;
            }
        }
        // Cleanup subscriptions
        for (channel_name, sub) in subscriptions {
            sub.abort();
            if channel_name.starts_with("presence-") {
                let _ = state
                    .adapter
                    .remove_member(&app_id, &channel_name, &socket_id)
                    .await;
                // TODO: Broadcast member_removed
            }
        }
    });

    // Wait for either to finish
    let _ = recv_task.await;
    send_task.abort();
}
