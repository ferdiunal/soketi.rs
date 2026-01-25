use crate::app::App;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from rate limiter indicating whether the request can continue
/// and any rate limit headers to include in the response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    /// Whether the request can continue (true) or should be rate limited (false)
    pub can_continue: bool,
    /// HTTP headers to include in the response (e.g., X-RateLimit-Limit, X-RateLimit-Remaining)
    pub headers: HashMap<String, String>,
}

/// Trait for rate limiting implementations
///
/// Rate limiters enforce per-app rate limits for backend events, frontend events,
/// and read requests. They return a RateLimitResponse indicating whether the
/// request should be allowed and any headers to include in the response.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Consume points for backend events (HTTP API triggered events)
    ///
    /// # Arguments
    /// * `points` - Number of points to consume (typically 1 per event)
    /// * `app` - The app configuration containing rate limit settings
    ///
    /// # Returns
    /// RateLimitResponse indicating if the request can continue and headers
    async fn consume_backend_event_points(
        &self,
        points: u64,
        app: &App,
    ) -> Result<RateLimitResponse>;

    /// Consume points for frontend events (WebSocket client events)
    ///
    /// # Arguments
    /// * `points` - Number of points to consume (typically 1 per event)
    /// * `app` - The app configuration containing rate limit settings
    /// * `socket_id` - The socket ID sending the event (for per-socket rate limiting)
    ///
    /// # Returns
    /// RateLimitResponse indicating if the request can continue and headers
    async fn consume_frontend_event_points(
        &self,
        points: u64,
        app: &App,
        socket_id: &str,
    ) -> Result<RateLimitResponse>;

    /// Consume points for read requests (HTTP API channel queries)
    ///
    /// # Arguments
    /// * `points` - Number of points to consume (typically 1 per request)
    /// * `app` - The app configuration containing rate limit settings
    ///
    /// # Returns
    /// RateLimitResponse indicating if the request can continue and headers
    async fn consume_read_request_points(
        &self,
        points: u64,
        app: &App,
    ) -> Result<RateLimitResponse>;

    /// Disconnect and clean up resources
    async fn disconnect(&self) -> Result<()>;
}

pub mod cluster;
pub mod local;
pub mod redis;
