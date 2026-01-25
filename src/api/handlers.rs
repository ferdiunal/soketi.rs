use crate::api::middleware::AuthenticatedApp;
use crate::state::AppState;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use sysinfo::System;

#[derive(Deserialize, Debug)]
pub struct TriggerEventRequest {
    name: String,
    data: Value, // Changed to Value to accept JSON directly or String
    channels: Option<Vec<String>>,
    channel: Option<String>,
    socket_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BatchEventRequest {
    batch: Vec<TriggerEventRequest>,
}

/// Trigger event endpoint
///
/// POST /apps/:appId/events
///
/// Triggers a single event to one or more channels. This endpoint:
/// 1. Validates the message structure and limits
/// 2. Checks rate limits (handled by middleware)
/// 3. Broadcasts the message to channels via adapter
/// 4. Caches the event if channel is cache-enabled (TODO)
/// 5. Returns success response
///
/// Request body format:
/// ```json
/// {
///   "name": "event-name",
///   "data": "event-data",
///   "channels": ["channel1", "channel2"],
///   "channel": "channel3",
///   "socket_id": "optional-socket-id"
/// }
/// ```
///
/// Note: Either `channels` (array) or `channel` (string) can be provided, or both.
///
/// Requirements: 3.4
pub async fn trigger_event(
    State(state): State<Arc<AppState>>,
    Path(_app_id): Path<String>,
    request: Request,
) -> impl IntoResponse {
    use crate::config::ServerConfig;
    use crate::validation::{
        validate_channel_count, validate_channel_name_length, validate_event_name_length,
        validate_payload_size,
    };

    // Get the authenticated app from middleware
    let app = match request.extensions().get::<AuthenticatedApp>() {
        Some(AuthenticatedApp(app)) => app.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({"error": "Internal error: app not found"})),
            )
                .into_response();
        }
    };

    // Extract the JSON payload
    let (_parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": format!("Failed to read body: {}", e)})),
            )
                .into_response();
        }
    };

    let payload: TriggerEventRequest = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": format!("Invalid JSON: {}", e)})),
            )
                .into_response();
        }
    };

    // Create a default ServerConfig for validation
    // In a real implementation, this would come from the state
    let config = ServerConfig::default();

    // Validate event name length
    if let Err(e) = validate_event_name_length(&payload.name, Some(&app), &config) {
        let err_msg = format!("{}", e);
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({"error": err_msg})),
        )
            .into_response();
    }

    // Validate payload size
    // Convert data to string for validation
    let data_str = match &payload.data {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };

    if let Err(e) = validate_payload_size(&data_str, Some(&app), &config) {
        let err_msg = format!("{}", e);
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({"error": err_msg})),
        )
            .into_response();
    }

    // Normalize channels - combine both channels array and single channel
    let mut channels = payload.channels.clone().unwrap_or_default();
    if let Some(ch) = &payload.channel {
        channels.push(ch.clone());
    }

    // Validate that at least one channel is specified
    if channels.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({"error": "No channels specified"})),
        )
            .into_response();
    }

    // Validate channel count
    if let Err(e) = validate_channel_count(channels.len(), Some(&app), &config) {
        let err_msg = format!("{}", e);
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({"error": err_msg})),
        )
            .into_response();
    }

    // Validate each channel name length
    for channel in &channels {
        if let Err(e) = validate_channel_name_length(channel, Some(&app), &config) {
            let err_msg = format!("{}", e);
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": err_msg})),
            )
                .into_response();
        }
    }

    // Broadcast to all channels
    for channel in &channels {
        tracing::debug!("Broadcasting to channel: {}", channel);
        
        // Create the message to broadcast
        let msg = serde_json::json!({
            "event": payload.name,
            "data": payload.data,
            "channel": channel
        })
        .to_string();

        tracing::debug!("Message to broadcast: {}", msg);

        // Broadcast to adapter, excluding the socket_id if provided
        if let Err(e) = state
            .adapter
            .send(&app.id, channel, &msg, payload.socket_id.as_deref())
            .await
        {
            // Log the error but continue with other channels
            eprintln!("Failed to send message to channel {}: {}", channel, e);
        } else {
            tracing::debug!("Successfully sent to adapter for channel: {}", channel);
        }

        // TODO: Cache event if channel is cache-enabled
        // This will be implemented when cache functionality is added for channels
        // Requirements: 7.10
        // if channel.starts_with("cache-") {
        //     state.cache_manager.set(&format!("{}:{}", app.id, channel), &msg, Some(ttl)).await;
        // }
    }

    // Return success response
    (StatusCode::OK, JsonResponse(serde_json::json!({}))).into_response()
}

/// Get channels endpoint
///
/// Returns a list of channels with their subscription counts.
/// Supports filtering by prefix using the `filter_by_prefix` query parameter.
/// Only returns channels with at least one subscriber.
///
/// Query Parameters:
/// - filter_by_prefix: Optional prefix to filter channel names
///
/// Response format:
/// ```json
/// {
///   "channels": {
///     "channel-name": {
///       "subscription_count": 5,
///       "occupied": true
///     }
///   }
/// }
/// ```
///
/// Requirements: 3.1
pub async fn get_channels(
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let channels = state
        .adapter
        .get_channels_with_sockets_count(&app_id)
        .await
        .unwrap_or_default();
    let mut response = serde_json::Map::new();

    // Get the optional filter_by_prefix query parameter
    let filter_prefix = params.get("filter_by_prefix");

    for (channel, count) in channels {
        // Skip channels with no subscribers
        if count == 0 {
            continue;
        }

        // Apply prefix filter if specified
        if let Some(prefix) = filter_prefix
            && !channel.starts_with(prefix)
        {
            continue;
        }

        response.insert(
            channel,
            serde_json::json!({
                "subscription_count": count,
                "occupied": true
            }),
        );
    }

    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({ "channels": response })),
    )
        .into_response()
}

pub async fn get_channel_info(
    State(state): State<Arc<AppState>>,
    Path((app_id, channel_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let count = state
        .adapter
        .get_channel_sockets_count(&app_id, &channel_name)
        .await
        .unwrap_or(0);

    let mut response = serde_json::json!({
        "subscription_count": count,
        "occupied": count > 0
    });

    if channel_name.starts_with("presence-") {
        let member_count = state
            .adapter
            .get_channel_members_count(&app_id, &channel_name)
            .await
            .unwrap_or(0);
        if let Some(obj) = response.as_object_mut() {
            obj.insert("user_count".to_string(), serde_json::json!(member_count));
        }
    }

    (StatusCode::OK, JsonResponse(response)).into_response()
}

/// Get channel users endpoint
///
/// Returns a list of users in a presence channel with their user_info.
/// Only works for presence channels (channels starting with "presence-").
///
/// Response format:
/// ```json
/// {
///   "users": [
///     {
///       "id": "user-1"
///     },
///     {
///       "id": "user-2",
///       "user_info": {
///         "name": "John Doe"
///       }
///     }
///   ]
/// }
/// ```
///
/// Requirements: 3.3
pub async fn get_channel_users(
    State(state): State<Arc<AppState>>,
    Path((app_id, channel_name)): Path<(String, String)>,
) -> impl IntoResponse {
    // Validate that the channel is a presence channel
    if !channel_name.starts_with("presence-") {
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({
                "error": "Channel must be a presence channel"
            })),
        )
            .into_response();
    }

    // Query adapter for channel members
    let members = state
        .adapter
        .get_channel_members(&app_id, &channel_name)
        .await
        .unwrap_or_default();

    // Build user list with optional user_info
    let mut users = Vec::new();
    for (user_id, member) in members {
        let mut user_obj = serde_json::Map::new();
        user_obj.insert("id".to_string(), serde_json::json!(user_id));

        // Include user_info if it's not null
        if !member.user_info.is_null() {
            user_obj.insert("user_info".to_string(), member.user_info);
        }

        users.push(serde_json::Value::Object(user_obj));
    }

    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({
            "users": users
        })),
    )
        .into_response()
}

/// Health check endpoint
///
/// Returns a simple OK response to indicate the server is running.
///
/// Requirements: 3.7
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({ "ok": true })),
    )
        .into_response()
}

/// Readiness check endpoint
///
/// Returns OK if the server is ready to accept connections.
/// This can be used by orchestration systems to determine if the server is ready.
///
/// Requirements: 3.8
pub async fn ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check if server is closing
    if state.closing.load(std::sync::atomic::Ordering::Relaxed) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            JsonResponse(
                serde_json::json!({ "ready": false, "reason": "server is shutting down" }),
            ),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({ "ready": true })),
    )
        .into_response()
}

/// Accept traffic endpoint for load balancer health checks
///
/// Returns OK if the server can accept traffic based on memory usage.
/// If memory usage exceeds the configured threshold, returns 503.
///
/// Requirements: 3.9
pub async fn accept_traffic(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Check if server is closing
    if state.closing.load(std::sync::atomic::Ordering::Relaxed) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            JsonResponse(serde_json::json!({
                "accept_traffic": false,
                "reason": "server is shutting down"
            })),
        )
            .into_response();
    }

    // Get memory usage
    let mut sys = System::new_all();
    sys.refresh_memory();

    let used_memory_mb = sys.used_memory() / 1024 / 1024;
    let memory_threshold_mb = state.config.http_api.accept_traffic_memory_threshold_mb;

    if used_memory_mb > memory_threshold_mb {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            JsonResponse(serde_json::json!({
                "accept_traffic": false,
                "reason": "memory threshold exceeded",
                "used_memory_mb": used_memory_mb,
                "threshold_mb": memory_threshold_mb
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({
            "accept_traffic": true,
            "used_memory_mb": used_memory_mb,
            "threshold_mb": memory_threshold_mb
        })),
    )
        .into_response()
}

/// Memory usage information endpoint
///
/// Returns detailed memory usage information for monitoring.
///
/// Requirements: 3.10
pub async fn usage() -> impl IntoResponse {
    let mut sys = System::new_all();
    sys.refresh_memory();

    let total_memory_mb = sys.total_memory() / 1024 / 1024;
    let used_memory_mb = sys.used_memory() / 1024 / 1024;
    let free_memory_mb = sys.free_memory() / 1024 / 1024;
    let available_memory_mb = sys.available_memory() / 1024 / 1024;

    (
        StatusCode::OK,
        JsonResponse(serde_json::json!({
            "memory": {
                "total_mb": total_memory_mb,
                "used_mb": used_memory_mb,
                "free_mb": free_memory_mb,
                "available_mb": available_memory_mb,
                "usage_percent": (used_memory_mb as f64 / total_memory_mb as f64 * 100.0)
            }
        })),
    )
        .into_response()
}

/// Batch events endpoint
///
/// POST /apps/:appId/batch_events
///
/// Triggers multiple events in a single request. This endpoint:
/// 1. Validates the batch size
/// 2. Validates each message in the batch
/// 3. Checks rate limits (handled by middleware)
/// 4. Broadcasts all messages to their respective channels
/// 5. Returns success response
///
/// Request body format:
/// ```json
/// {
///   "batch": [
///     {
///       "name": "event-name-1",
///       "data": "event-data-1",
///       "channel": "channel1"
///     },
///     {
///       "name": "event-name-2",
///       "data": "event-data-2",
///       "channels": ["channel2", "channel3"]
///     }
///   ]
/// }
/// ```
///
/// Requirements: 3.5
pub async fn batch_events(
    State(state): State<Arc<AppState>>,
    Path(_app_id): Path<String>,
    request: Request,
) -> impl IntoResponse {
    use crate::config::ServerConfig;
    use crate::validation::{
        validate_batch_size, validate_channel_count, validate_channel_name_length,
        validate_event_name_length, validate_payload_size,
    };

    // Get the authenticated app from middleware
    let app = match request.extensions().get::<AuthenticatedApp>() {
        Some(AuthenticatedApp(app)) => app.clone(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({"error": "Internal error: app not found"})),
            )
                .into_response();
        }
    };

    // Extract the JSON payload
    let (_parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": format!("Failed to read body: {}", e)})),
            )
                .into_response();
        }
    };

    let payload: BatchEventRequest = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": format!("Invalid JSON: {}", e)})),
            )
                .into_response();
        }
    };

    // Create a default ServerConfig for validation
    let config = ServerConfig::default();

    // Validate batch size
    if let Err(e) = validate_batch_size(payload.batch.len(), Some(&app), &config) {
        let err_msg = format!("{}", e);
        return (
            StatusCode::BAD_REQUEST,
            JsonResponse(serde_json::json!({"error": err_msg})),
        )
            .into_response();
    }

    // Validate each event in the batch
    for event in &payload.batch {
        // Validate event name length
        if let Err(e) = validate_event_name_length(&event.name, Some(&app), &config) {
            let err_msg = format!("{}", e);
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": err_msg})),
            )
                .into_response();
        }

        // Validate payload size
        let data_str = match &event.data {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        if let Err(e) = validate_payload_size(&data_str, Some(&app), &config) {
            let err_msg = format!("{}", e);
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": err_msg})),
            )
                .into_response();
        }

        // Normalize channels - combine both channels array and single channel
        let mut channels = event.channels.clone().unwrap_or_default();
        if let Some(ch) = &event.channel {
            channels.push(ch.clone());
        }

        // Validate that at least one channel is specified
        if channels.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": "No channels specified for event"})),
            )
                .into_response();
        }

        // Validate channel count
        if let Err(e) = validate_channel_count(channels.len(), Some(&app), &config) {
            let err_msg = format!("{}", e);
            return (
                StatusCode::BAD_REQUEST,
                JsonResponse(serde_json::json!({"error": err_msg})),
            )
                .into_response();
        }

        // Validate each channel name length
        for channel in &channels {
            if let Err(e) = validate_channel_name_length(channel, Some(&app), &config) {
                let err_msg = format!("{}", e);
                return (
                    StatusCode::BAD_REQUEST,
                    JsonResponse(serde_json::json!({"error": err_msg})),
                )
                    .into_response();
            }
        }
    }

    // Broadcast all events
    for event in &payload.batch {
        // Normalize channels
        let mut channels = event.channels.clone().unwrap_or_default();
        if let Some(ch) = &event.channel {
            channels.push(ch.clone());
        }

        // Broadcast to all channels for this event
        for channel in &channels {
            // Create the message to broadcast
            let msg = serde_json::json!({
                "event": event.name,
                "data": event.data,
                "channel": channel
            })
            .to_string();

            // Broadcast to adapter, excluding the socket_id if provided
            if let Err(e) = state
                .adapter
                .send(&app.id, channel, &msg, event.socket_id.as_deref())
                .await
            {
                // Log the error but continue with other channels
                eprintln!("Failed to send message to channel {}: {}", channel, e);
            }

            // TODO: Cache event if channel is cache-enabled
            // This will be implemented when cache functionality is added for channels
        }
    }

    // Return success response
    (StatusCode::OK, JsonResponse(serde_json::json!({}))).into_response()
}

/// Metrics endpoint for Prometheus metrics
///
/// Returns metrics in Prometheus plaintext format by default.
/// If the query parameter `format=json` is provided, returns metrics as JSON.
///
/// This endpoint is only available if metrics are enabled in the configuration.
/// If metrics are disabled, returns 404 Not Found.
///
/// Requirements: 3.11
pub async fn metrics(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // Check if metrics are enabled
    let metrics_manager = match &state.metrics_manager {
        Some(manager) => manager,
        None => {
            return (
                StatusCode::NOT_FOUND,
                JsonResponse(serde_json::json!({
                    "error": "Metrics are not enabled"
                })),
            )
                .into_response();
        }
    };

    // Check format parameter
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("text");

    match format {
        "json" => {
            // Return metrics as JSON
            match metrics_manager.get_metrics_as_json().await {
                Ok(json) => (StatusCode::OK, JsonResponse(json)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("Failed to get metrics: {}", e)
                    })),
                )
                    .into_response(),
            }
        }
        _ => {
            // Return metrics as Prometheus plaintext (default)
            match metrics_manager.get_metrics_as_plaintext().await {
                Ok(text) => (
                    StatusCode::OK,
                    [(
                        axum::http::header::CONTENT_TYPE,
                        "text/plain; version=0.0.4",
                    )],
                    text,
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    JsonResponse(serde_json::json!({
                        "error": format!("Failed to get metrics: {}", e)
                    })),
                )
                    .into_response(),
            }
        }
    }
}

/// Terminate user connections endpoint
///
/// POST /apps/:appId/users/:userId/terminate_connections
///
/// Terminates all connections for a specific user. This endpoint:
/// 1. Calls the adapter to terminate all user connections
/// 2. Returns success response
///
/// The adapter will send a pusher:error message with code 4009 to all
/// connections associated with the user before disconnecting them.
///
/// Response format:
/// ```json
/// {}
/// ```
///
/// Requirements: 3.6, 16.6
pub async fn terminate_user_connections(
    State(state): State<Arc<AppState>>,
    Path((app_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // Call adapter to terminate all user connections
    match state
        .adapter
        .terminate_user_connections(&app_id, &user_id)
        .await
    {
        Ok(_) => {
            // Return success response
            (StatusCode::OK, JsonResponse(serde_json::json!({}))).into_response()
        }
        Err(e) => {
            // Log the error and return internal server error
            eprintln!(
                "Failed to terminate user connections for user {}: {}",
                user_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                JsonResponse(serde_json::json!({
                    "error": format!("Failed to terminate user connections: {}", e)
                })),
            )
                .into_response()
        }
    }
}
