use crate::app::App;
use crate::error::Result;
use crate::rate_limiters::{RateLimitResponse, RateLimiter};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
struct Bucket {
    tokens: f64,
    last_update: Instant,
}

pub struct LocalRateLimiter {
    buckets: Arc<DashMap<String, Mutex<Bucket>>>,
}

impl Default for LocalRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalRateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
        }
    }

    fn consume(&self, key: String, points: u64, limit: u64) -> RateLimitResponse {
        if limit == 0 {
            return RateLimitResponse {
                can_continue: true,
                headers: HashMap::new(),
            };
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

        b.tokens = (b.tokens + elapsed * refill_rate).min(capacity);
        b.last_update = now;

        let cost = points as f64;
        let can_continue = b.tokens >= cost;

        if can_continue {
            b.tokens -= cost;
        }

        let remaining = b.tokens.max(0.0) as u64;

        let mut headers = HashMap::new();
        headers.insert("X-RateLimit-Limit".to_string(), limit.to_string());
        headers.insert("X-RateLimit-Remaining".to_string(), remaining.to_string());
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
}

#[async_trait]
impl RateLimiter for LocalRateLimiter {
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
        // No resources to clean up for local rate limiter
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_rate_limiter_allows() {
        let limiter = LocalRateLimiter::new();
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10); // 10 per second

        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "9");
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks() {
        let limiter = LocalRateLimiter::new();
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        // Consume all
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
    async fn test_rate_limiter_refills() {
        let limiter = LocalRateLimiter::new();
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
    async fn test_rate_limiter_frontend_per_socket() {
        let limiter = LocalRateLimiter::new();
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
    async fn test_rate_limiter_read_requests() {
        let limiter = LocalRateLimiter::new();
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_read_requests_per_second = Some(20);

        let res = limiter.consume_read_request_points(1, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "20");
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "19");
    }

    #[tokio::test]
    async fn test_rate_limiter_no_limit() {
        let limiter = LocalRateLimiter::new();
        let app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        // No rate limits set

        let res = limiter
            .consume_backend_event_points(1000, &app)
            .await
            .unwrap();
        assert!(res.can_continue);
        assert!(res.headers.is_empty());
    }

    #[tokio::test]
    async fn test_rate_limiter_headers() {
        let limiter = LocalRateLimiter::new();
        let mut app = App::new("id".to_string(), "key".to_string(), "secret".to_string());
        app.max_backend_events_per_second = Some(10);

        let res = limiter.consume_backend_event_points(5, &app).await.unwrap();
        assert!(res.can_continue);
        assert_eq!(res.headers.get("X-RateLimit-Limit").unwrap(), "10");
        assert_eq!(res.headers.get("X-RateLimit-Remaining").unwrap(), "5");
        assert!(!res.headers.contains_key("Retry-After"));

        // Consume remaining and try one more
        let _ = limiter.consume_backend_event_points(5, &app).await.unwrap();
        let res = limiter.consume_backend_event_points(1, &app).await.unwrap();
        assert!(!res.can_continue);
        assert!(res.headers.contains_key("Retry-After"));
    }
}
