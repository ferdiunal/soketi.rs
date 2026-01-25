use crate::app::App;
use crate::error::Result;
use crate::rate_limiters::{RateLimitResponse, RateLimiter};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// RedisRateLimiter implements distributed rate limiting using Redis.
///
/// This implementation uses Redis sorted sets to implement a sliding window
/// rate limiting algorithm. Each request is recorded with a timestamp, and
/// old entries outside the window are automatically removed.
///
/// # Algorithm
///
/// The sliding window algorithm works as follows:
/// 1. Remove all entries older than 1 second from the sorted set
/// 2. Count the remaining entries in the sorted set
/// 3. If count < limit, add a new entry with current timestamp
/// 4. Return whether the request can continue and rate limit headers
///
/// # Redis Data Structure
///
/// Uses Redis sorted sets where:
/// - Key: `ratelimit:{app_id}:{type}:{identifier}`
/// - Score: Unix timestamp in milliseconds
/// - Member: Unique identifier for each request (timestamp + random)
///
/// # Benefits
///
/// - Distributed: Works across multiple server instances
/// - Accurate: Sliding window provides precise rate limiting
/// - Automatic cleanup: Old entries are removed on each check
/// - Scalable: Redis handles concurrent access efficiently
pub struct RedisRateLimiter {
    /// Redis client for creating connections
    #[allow(dead_code)]
    client: redis::Client,
    /// Shared connection pool (using `Arc<Mutex>` for async access)
    conn: Arc<Mutex<redis::aio::MultiplexedConnection>>,
}

impl RedisRateLimiter {
    /// Create a new RedisRateLimiter
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    ///
    /// # Returns
    ///
    /// Result containing the RedisRateLimiter or an error
    ///
    /// # Example
    ///
    /// ```no_run
    /// use soketi_rs::rate_limiters::redis::RedisRateLimiter;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let limiter = RedisRateLimiter::new("redis://127.0.0.1:6379").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;

        let conn = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Consume points using Redis sliding window algorithm
    ///
    /// This method implements the core rate limiting logic using Redis sorted sets.
    /// It uses a Lua script for atomic operations to ensure consistency.
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
    async fn consume(&self, key: String, points: u64, limit: u64) -> Result<RateLimitResponse> {
        // If limit is 0, allow unlimited requests
        if limit == 0 {
            return Ok(RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            });
        }

        let mut conn = self.conn.lock().await;

        // Get current timestamp in milliseconds
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // Window is 1 second (1000 milliseconds)
        let window_start = now - 1000;

        // Lua script for atomic rate limiting
        // This script:
        // 1. Removes old entries outside the window
        // 2. Counts current entries
        // 3. If under limit, adds new entries for the consumed points
        // 4. Returns the count and whether the request can continue
        let script = redis::Script::new(
            r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window_start = tonumber(ARGV[2])
            local limit = tonumber(ARGV[3])
            local points = tonumber(ARGV[4])
            
            -- Remove old entries outside the window
            redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)
            
            -- Count current entries
            local current = redis.call('ZCARD', key)
            
            -- Check if we can consume the points
            if current + points <= limit then
                -- Add entries for each point consumed
                for i = 1, points do
                    local member = now .. ':' .. i
                    redis.call('ZADD', key, now, member)
                end
                
                -- Set expiration to 2 seconds (window + buffer)
                redis.call('EXPIRE', key, 2)
                
                return {1, current + points}
            else
                return {0, current}
            end
        "#,
        );

        let result: Vec<i64> = script
            .key(&key)
            .arg(now)
            .arg(window_start)
            .arg(limit as i64)
            .arg(points as i64)
            .invoke_async(&mut *conn)
            .await?;

        let can_continue = result[0] == 1;
        let current_count = result[1] as u64;

        // Calculate remaining tokens (same calculation regardless of can_continue)
        let remaining = limit.saturating_sub(current_count);

        // Build response headers
        let mut headers = HashMap::new();
        headers.insert("X-RateLimit-Limit".to_string(), limit.to_string());
        headers.insert("X-RateLimit-Remaining".to_string(), remaining.to_string());

        // Add Retry-After header if rate limited
        if !can_continue {
            // Estimate retry after based on window (1 second)
            headers.insert("Retry-After".to_string(), "1".to_string());
        }

        Ok(RateLimitResponse {
            can_continue,
            headers,
        })
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn consume_backend_event_points(
        &self,
        points: u64,
        app: &App,
    ) -> Result<RateLimitResponse> {
        if let Some(limit) = app.max_backend_events_per_second {
            self.consume(
                format!("ratelimit:{}:backend:events", app.id),
                points,
                limit,
            )
            .await
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
            let key = format!("ratelimit:{}:frontend:events:{}", app.id, socket_id);
            self.consume(key, points, limit).await
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
            self.consume(format!("ratelimit:{}:backend:read", app.id), points, limit)
                .await
        } else {
            Ok(RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            })
        }
    }

    async fn disconnect(&self) -> Result<()> {
        // Connection is automatically closed when dropped
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    // Helper function to get Redis URL from environment or use default
    fn get_redis_url() -> String {
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string())
    }

    // Helper function to check if Redis is available
    async fn is_redis_available() -> bool {
        match redis::Client::open(get_redis_url().as_str()) {
            Ok(client) => {
                match client.get_multiplexed_async_connection().await {
                    Ok(mut conn) => {
                        // Try a simple ping
                        redis::cmd("PING")
                            .query_async::<String>(&mut conn)
                            .await
                            .is_ok()
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    // Helper function to generate unique app ID for tests
    fn unique_app_id(prefix: &str) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!(
            "{}_{}_{}",
            prefix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            counter
        )
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_allows() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_allows"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_backend_events_per_second = Some(10);

        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "10");

        // Remaining should be 9 or less (due to timing)
        let remaining: u64 = res
            .headers
            .get("X-RateLimit-Remaining")
            .unwrap()
            .parse()
            .unwrap();
        assert!(remaining <= 9);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_blocks() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_blocks"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_backend_events_per_second = Some(5);

        // Consume all tokens
        let res = limiter.consume_backend_event_points(5, &app).await.unwrap();
        assert!(res.can_continue);

        // Next should fail
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);
        assert!(res.headers.contains_key("Retry-After"));
        assert_eq!(res.headers.get("Retry-After").unwrap(), "1");
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_refills() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_refills"),
            "key".to_string(),
            "secret".to_string(),
        );
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

        // Wait for sliding window to move (1.1 seconds to be safe)
        sleep(Duration::from_millis(1100)).await;

        // Should now be able to consume tokens again
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_frontend_per_socket() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_frontend"),
            "key".to_string(),
            "secret".to_string(),
        );
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
    async fn test_redis_rate_limiter_read_requests() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_read"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_read_requests_per_second = Some(20);

        let res = limiter.consume_read_request_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "20");

        let remaining: u64 = res
            .headers
            .get("X-RateLimit-Remaining")
            .unwrap()
            .parse()
            .unwrap();
        assert!(remaining <= 19);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_no_limit() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let app = App::new(
            unique_app_id("test_no_limit"),
            "key".to_string(),
            "secret".to_string(),
        );
        // No rate limits set

        let res = limiter
            .consume_backend_event_points(1000, &app)
            .await
            .unwrap();
        assert!(res.can_continue);
        assert!(res.headers.is_empty());
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_headers() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_headers"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_backend_events_per_second = Some(10);

        let res = limiter.consume_backend_event_points(5, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "10");

        let remaining: u64 = res
            .headers
            .get("X-RateLimit-Remaining")
            .unwrap()
            .parse()
            .unwrap();
        assert!(remaining <= 5);
        assert!(!res.headers.contains_key("Retry-After"));

        // Small delay to ensure requests are processed separately
        sleep(Duration::from_millis(10)).await;

        // Consume remaining and try one more
        let res2 = limiter.consume_backend_event_points(5, &app).await.unwrap();
        assert!(res2.can_continue);

        // Small delay to ensure the second request is fully processed
        sleep(Duration::from_millis(10)).await;

        // This should fail since we've consumed all 10 tokens
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);
        assert!(res.headers.contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_distributed() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        // Create two separate limiters (simulating two server instances)
        let limiter1 = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let limiter2 = RedisRateLimiter::new(&get_redis_url()).await.unwrap();

        let mut app = App::new(
            unique_app_id("test_distributed"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_backend_events_per_second = Some(10);

        // Consume 5 tokens from limiter1
        let res = limiter1
            .consume_backend_event_points(5, &app)
            .await
            .unwrap();
        assert!(res.can_continue);

        // Small delay to ensure the first request is processed
        sleep(Duration::from_millis(10)).await;

        // Consume 5 tokens from limiter2
        let res = limiter2
            .consume_backend_event_points(5, &app)
            .await
            .unwrap();
        assert!(res.can_continue);

        // Small delay to ensure the second request is processed
        sleep(Duration::from_millis(10)).await;

        // Both limiters should now see the limit reached
        let res = limiter1
            .consume_backend_event_points(1, &app)
            .await
            .unwrap();
        assert!(!res.can_continue);

        let res = limiter2
            .consume_backend_event_points(1, &app)
            .await
            .unwrap();
        assert!(!res.can_continue);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_sliding_window() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let mut app = App::new(
            unique_app_id("test_sliding"),
            "key".to_string(),
            "secret".to_string(),
        );
        app.max_backend_events_per_second = Some(10);

        // Consume 10 tokens
        let res = limiter
            .consume_backend_event_points(10, &app)
            .await
            .unwrap();
        assert!(res.can_continue);

        // Should be blocked
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);

        // Wait 600ms (more than half the window)
        sleep(Duration::from_millis(600)).await;

        // Still blocked (window hasn't fully passed)
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);

        // Wait another 500ms (total 1.1 seconds)
        sleep(Duration::from_millis(500)).await;

        // Now should be allowed (old entries expired)
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_connection_error() {
        // Try to connect to invalid Redis URL
        let result = RedisRateLimiter::new("redis://invalid-host:9999").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_redis_rate_limiter_disconnect() {
        if !is_redis_available().await {
            eprintln!("Skipping test: Redis not available");
            return;
        }

        let limiter = RedisRateLimiter::new(&get_redis_url()).await.unwrap();
        let result = limiter.disconnect().await;
        assert!(result.is_ok());
    }
}
