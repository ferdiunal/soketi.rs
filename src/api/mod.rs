use crate::state::AppState;
use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use std::sync::Arc;

mod handlers;
pub mod http_handler;
pub mod middleware;

// Re-export handlers for testing
pub use handlers::{
    accept_traffic, batch_events, get_channel_info, get_channel_users, get_channels, health_check,
    metrics, ready, terminate_user_connections, trigger_event, usage,
};

pub fn routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    // Health check routes (no authentication)
    let health_routes = Router::new()
        .route("/", get(handlers::health_check))
        .route("/ready", get(handlers::ready))
        .route("/accept-traffic", get(handlers::accept_traffic))
        .route("/usage", get(handlers::usage))
        .route("/metrics", get(handlers::metrics));

    // Test route - no middleware
    let test_route = Router::new()
        .route("/test", get(|| async { "OK" }));

    // API routes (with authentication)
    let api_routes = Router::new()
        .route("/apps/{app_id}/events", post(handlers::trigger_event))
        .route("/apps/{app_id}/batch_events", post(handlers::batch_events))
        .route("/apps/{app_id}/channels", get(handlers::get_channels))
        .route(
            "/apps/{app_id}/channels/{channel_name}",
            get(handlers::get_channel_info),
        )
        .route(
            "/apps/{app_id}/channels/{channel_name}/users",
            get(handlers::get_channel_users),
        )
        .route(
            "/apps/{app_id}/users/{user_id}/terminate_connections",
            post(handlers::terminate_user_connections),
        )
        // Apply middleware in REVERSE order (layers execute bottom-to-top):
        // 4. Rate limiting - enforce rate limits (executes first)
        // 3. Authentication - verify API signature
        // 2. App lookup - find and validate the app
        // 1. CORS - handle CORS headers (executes last)
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::app_lookup_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::cors_middleware,
        ));

    // Merge routes
    health_routes.merge(test_route).merge(api_routes)
}
