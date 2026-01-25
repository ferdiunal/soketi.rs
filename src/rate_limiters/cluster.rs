use crate::app::App;
use crate::error::Result;
use crate::rate_limiters::{RateLimitResponse, RateLimiter};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Bucket for token bucket rate limiting algorithm
#[derive(Clone)]
struct Bucket {
    tokens: f64,
    last_update: Instant,
}

/// ClusterRateLimiter extends LocalRateLimiter with cluster synchronization capabilities.
///
/// In a cluster deployment, rate limits need to be synchronized across multiple server instances
/// to ensure accurate rate limiting. This implementation uses a local token bucket algorithm
/// similar to LocalRateLimiter, but is designed to integrate with a cluster discovery mechanism
/// for synchronizing consumption across nodes.
///
/// # Cluster Synchronization
///
/// When cluster discovery is available, the ClusterRateLimiter will:
/// - Broadcast token consumption to other nodes when limits are consumed
/// - Receive consumption messages from other nodes and update local buckets
/// - Handle master election where the master node maintains authoritative state
/// - Transfer state to new master on shutdown/demotion
///
/// # Current Implementation
///
/// The current implementation provides the core rate limiting logic using local buckets.
/// Cluster synchronization hooks are prepared but require the cluster discovery mechanism
/// to be fully implemented (see Task 17.1 - Implement ClusterAdapter).
///
/// # Algorithm
///
/// Uses a token bucket algorithm where:
/// - Each bucket has a capacity equal to the rate limit
/// - Tokens refill at a rate equal to the limit per second
/// - Consuming points deducts tokens from the bucket
/// - Requests are rejected when insufficient tokens are available
pub struct ClusterRateLimiter {
    /// Local buckets for rate limiting, keyed by app_id:type:identifier
    buckets: Arc<DashMap<String, Mutex<Bucket>>>,

    /// Flag indicating if this node is the cluster master
    /// The master node maintains authoritative state and coordinates with other nodes
    is_master: Arc<Mutex<bool>>,

    /// Optional cluster synchronization channel for broadcasting consumption
    /// This will be used when cluster discovery is implemented
    #[allow(dead_code)]
    cluster_tx: Option<tokio::sync::broadcast::Sender<ClusterMessage>>,
}

/// Message format for cluster synchronization
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClusterMessage {
    /// The app ID
    app_id: String,
    /// The rate limit key (e.g., "backend:events", "frontend:events:socket123")
    event_key: String,
    /// Number of points consumed
    points: u64,
    /// Maximum points allowed
    max_points: u64,
}

impl ClusterRateLimiter {
    /// Create a new ClusterRateLimiter
    ///
    /// # Arguments
    ///
    /// * `is_master` - Whether this node is the cluster master
    pub fn new(is_master: bool) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            is_master: Arc::new(Mutex::new(is_master)),
            cluster_tx: None,
        }
    }

    /// Create a new ClusterRateLimiter with cluster synchronization channel
    ///
    /// This constructor is prepared for future cluster discovery integration.
    ///
    /// # Arguments
    ///
    /// * `is_master` - Whether this node is the cluster master
    /// * `cluster_tx` - Broadcast channel for cluster synchronization
    #[allow(dead_code)]
    pub fn new_with_cluster(
        is_master: bool,
        cluster_tx: tokio::sync::broadcast::Sender<ClusterMessage>,
    ) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            is_master: Arc::new(Mutex::new(is_master)),
            cluster_tx: Some(cluster_tx),
        }
    }

    /// Set whether this node is the cluster master
    ///
    /// This is called during master election/demotion in cluster mode
    #[allow(dead_code)]
    pub fn set_master(&self, is_master: bool) {
        let mut master = self.is_master.lock().unwrap();
        *master = is_master;
    }

    /// Check if this node is the cluster master
    #[allow(dead_code)]
    pub fn is_master(&self) -> bool {
        *self.is_master.lock().unwrap()
    }

    /// Consume points from a rate limit bucket
    ///
    /// This is the core rate limiting logic using a token bucket algorithm.
    /// In cluster mode, successful consumption is broadcast to other nodes.
    ///
    /// # Arguments
    ///
    /// * `key` - The rate limit key (e.g., "app123:backend:events")
    /// * `points` - Number of points to consume
    /// * `limit` - Maximum points allowed per second
    ///
    /// # Returns
    ///
    /// RateLimitResponse indicating if the request can continue and headers
    fn consume(&self, key: String, points: u64, limit: u64) -> RateLimitResponse {
        // If limit is 0, allow unlimited requests
        if limit == 0 {
            return RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            };
        }

        // Get or create bucket for this key
        let bucket = self.buckets.entry(key.clone()).or_insert_with(|| {
            Mutex::new(Bucket {
                tokens: limit as f64,
                last_update: Instant::now(),
            })
        });

        let mut b = bucket.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(b.last_update).as_secs_f64();

        // Token bucket parameters
        let capacity = limit as f64;
        let refill_rate = limit as f64; // Refill at rate of limit per second

        // Refill tokens based on elapsed time
        b.tokens = (b.tokens + elapsed * refill_rate).min(capacity);
        b.last_update = now;

        let cost = points as f64;
        let can_continue = b.tokens >= cost;

        // Deduct tokens if request can continue
        if can_continue {
            b.tokens -= cost;

            // TODO: When cluster discovery is implemented, broadcast consumption to other nodes
            // if let Some(ref tx) = self.cluster_tx {
            //     let _ = tx.send(ClusterMessage {
            //         app_id: extract_app_id_from_key(&key),
            //         event_key: extract_event_key_from_key(&key),
            //         points,
            //         max_points: limit,
            //     });
            // }
        }

        let remaining = b.tokens.max(0.0) as u64;

        // Build response headers
        let mut headers = HashMap::new();
        headers.insert("X-RateLimit-Limit".to_string(), limit.to_string());
        headers.insert("X-RateLimit-Remaining".to_string(), remaining.to_string());

        // Add Retry-After header if rate limited
        if !can_continue {
            let deficit = cost - b.tokens;
            let retry_after = ((deficit / refill_rate).ceil()) as u64;
            headers.insert("Retry-After".to_string(), retry_after.to_string());
        }

        RateLimitResponse {
            can_continue,
            headers,
        }
    }

    /// Handle consumption message from another cluster node
    ///
    /// This method is called when another node in the cluster consumes tokens.
    /// It updates the local bucket to reflect the consumption.
    ///
    /// This is prepared for future cluster discovery integration.
    #[allow(dead_code)]
    pub fn handle_cluster_consumption(&self, key: String, points: u64, limit: u64) {
        if limit == 0 {
            return;
        }

        let bucket = self.buckets.entry(key).or_insert_with(|| {
            Mutex::new(Bucket {
                tokens: limit as f64,
                last_update: Instant::now(),
            })
        });

        let mut b = bucket.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(b.last_update).as_secs_f64();

        let capacity = limit as f64;
        let refill_rate = limit as f64;

        // Refill tokens
        b.tokens = (b.tokens + elapsed * refill_rate).min(capacity);
        b.last_update = now;

        // Deduct consumed points
        let cost = points as f64;
        b.tokens = (b.tokens - cost).max(0.0);
    }

    /// Get current bucket states for transfer to new master
    ///
    /// This is used during master demotion to transfer state to the new master.
    ///
    /// This is prepared for future cluster discovery integration.
    #[allow(dead_code)]
    pub fn get_bucket_states(&self) -> HashMap<String, (f64, Instant)> {
        let mut states = HashMap::new();
        for entry in self.buckets.iter() {
            let key = entry.key().clone();
            let bucket = entry.value().lock().unwrap();
            states.insert(key, (bucket.tokens, bucket.last_update));
        }
        states
    }

    /// Set bucket states from previous master
    ///
    /// This is used during master promotion to receive state from the previous master.
    ///
    /// This is prepared for future cluster discovery integration.
    #[allow(dead_code)]
    pub fn set_bucket_states(&self, states: HashMap<String, (f64, Instant)>) {
        for (key, (tokens, last_update)) in states {
            self.buckets.insert(
                key,
                Mutex::new(Bucket {
                    tokens,
                    last_update,
                }),
            );
        }
    }
}

#[async_trait]
impl RateLimiter for ClusterRateLimiter {
    async fn consume_backend_event_points(
        &self,
        points: u64,
        app: &App,
    ) -> Result<RateLimitResponse> {
        if let Some(limit) = app.max_backend_events_per_second {
            Ok(self.consume(format!("{}:backend:events", app.id), points, limit))
        } else {
            Ok(RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            })
        }
    }

    async fn consume_frontend_event_points(
        &self,
        points: u64,
        app: &App,
        socket_id: &str,
    ) -> Result<RateLimitResponse> {
        if let Some(limit) = app.max_client_events_per_second {
            let key = format!("{}:frontend:events:{}", app.id, socket_id);
            Ok(self.consume(key, points, limit))
        } else {
            Ok(RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            })
        }
    }

    async fn consume_read_request_points(
        &self,
        points: u64,
        app: &App,
    ) -> Result<RateLimitResponse> {
        if let Some(limit) = app.max_read_requests_per_second {
            Ok(self.consume(format!("{}:backend:read", app.id), points, limit))
        } else {
            Ok(RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            })
        }
    }

    async fn disconnect(&self) -> Result<()> {
        // TODO: When cluster discovery is implemented:
        // - If this is the master node, demote and send bucket states to new master
        // - Clean up cluster synchronization resources
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_cluster_rate_limiter_allows() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "9");
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_blocks() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        // Consume all tokens
        let _ = limiter
            .consume_backend_event_points(10, &app)
            .await
            .unwrap();

        // Next should fail
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);
        assert!(res.headers.contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_refills() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        // Consume all tokens
        let res = limiter
            .consume_backend_event_points(10, &app)
            .await
            .unwrap();
        assert!(res.can_continue);

        // Should be blocked immediately
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);

        // Wait for tokens to refill (200ms should give us ~2 tokens at 10/sec)
        sleep(Duration::from_millis(200)).await;

        // Should now be able to consume 1-2 tokens
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_frontend_per_socket() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_client_events_per_second = Some(5);

        // Socket 1 consumes all its tokens
        let res = limiter
            .consume_frontend_event_points(5, &app, "socket1")
            .await
            .unwrap();
        assert!(res.can_continue);

        // Socket 1 should be blocked
        let res = limiter
            .consume_frontend_event_points(1, &app, "socket1")
            .await
            .unwrap();
        assert!(!res.can_continue);

        // Socket 2 should still have tokens
        let res = limiter
            .consume_frontend_event_points(1, &app, "socket2")
            .await
            .unwrap();
        assert!(res.can_continue);
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_read_requests() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_read_requests_per_second = Some(20);

        let res = limiter.consume_read_request_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "20");
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "19");
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_no_limit() {
        let limiter = ClusterRateLimiter::new(false);
        let app = App::new("id".to_string(), "key".to_string(), "secret".to_string());

        let res = limiter
            .consume_backend_event_points(1000, &app)
            .await
            .unwrap();
        assert!(res.can_continue);
        assert!(res.headers.is_empty());
    }

    #[tokio::test]
    async fn test_cluster_rate_limiter_master_flag() {
        let limiter = ClusterRateLimiter::new(true);
        assert!(limiter.is_master());

        limiter.set_master(false);
        assert!(!limiter.is_master());
    }

    #[tokio::test]
    async fn test_cluster_consumption_sync() {
        let limiter = ClusterRateLimiter::new(false);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        // Consume 5 tokens locally
        let res = limiter.consume_backend_event_points(5, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "5");

        // Simulate consumption from another node (3 tokens)
        limiter.handle_cluster_consumption(format!("{}:backend:events", app.id), 3, 10);

        // Now we should have approximately 2 tokens remaining
        // (5 remaining - 3 consumed by other node = 2)
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        // Remaining should be around 1 (2 - 1 = 1)
        let remaining: u64 = res
            .headers
            .get("X-RateLimit-Remaining")
            .unwrap()
            .parse()
            .unwrap();
        assert!(remaining <= 2); // Allow some tolerance for timing
    }

    #[tokio::test]
    async fn test_bucket_state_transfer() {
        let limiter1 = ClusterRateLimiter::new(true);
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        // Consume some tokens on limiter1
        let _ = limiter1
            .consume_backend_event_points(7, &app)
            .await
            .unwrap();

        // Get bucket states
        let states = limiter1.get_bucket_states();
        assert!(!states.is_empty());

        // Transfer to new limiter (simulating new master)
        let limiter2 = ClusterRateLimiter::new(true);
        limiter2.set_bucket_states(states);

        // limiter2 should have similar state (approximately 3 tokens remaining)
        let res = limiter2
            .consume_backend_event_points(1, &app)
            .await
            .unwrap();
        assert!(res.can_continue);
        let remaining: u64 = res
            .headers
            .get("X-RateLimit-Remaining")
            .unwrap()
            .parse()
            .unwrap();
        assert!((2..=3).contains(&remaining)); // Allow some tolerance
    }
}
