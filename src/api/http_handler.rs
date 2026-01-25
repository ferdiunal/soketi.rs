use crate::state::AppState;
use std::sync::Arc;

/// HttpHandler manages HTTP REST API requests for the Pusher protocol server.
///
/// This handler provides endpoints for:
/// - Triggering events (POST /apps/:appId/events)
/// - Batch event triggering (POST /apps/:appId/batch_events)
/// - Querying channels (GET /apps/:appId/channels)
/// - Getting channel information (GET /apps/:appId/channels/:channelName)
/// - Listing presence channel users (GET /apps/:appId/channels/:channelName/users)
/// - Terminating user connections (POST /apps/:appId/users/:userId/terminate_connections)
/// - Health checks (GET /, GET /ready, GET /accept-traffic, GET /usage)
/// - Metrics (GET /metrics)
///
/// Requirements: 3.1
pub struct HttpHandler {
    state: Arc<AppState>,
}

impl HttpHandler {
    /// Creates a new HttpHandler with the given AppState.
    ///
    /// # Arguments
    ///
    /// * `state` - Shared application state containing adapters, managers, and configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use soketi_rs::api::http_handler::HttpHandler;
    /// use soketi_rs::state::AppState;
    ///
    /// // Create test state (see test_helpers for actual implementation)
    /// let state = create_test_state();
    /// let handler = HttpHandler::new(state);
    /// ```
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Returns a reference to the shared application state.
    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    // Note: Comprehensive tests for HTTP handlers are in the tests/ directory
    // including channels_endpoint_test.rs, events_endpoint_test.rs, etc.
}
