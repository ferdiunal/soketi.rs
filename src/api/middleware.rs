use crate::{app::App, auth::verify_api_auth_with_timestamp, state::AppState};
use axum::{
    Json,
    extract::{Request, State},
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;

/// Extension type to store the authenticated app in the request
#[derive(Clone)]
pub struct AuthenticatedApp(pub App);

/// CORS middleware
///
/// Adds CORS headers to responses based on the configured CORS settings.
///
/// Requirements: 1.6
pub async fn cors_middleware(
    State(_state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*")
        .to_string();

    // Handle preflight requests
    if method == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Access-Control-Allow-Origin", origin)
            .header(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
            )
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, X-Requested-With",
            )
            .header("Access-Control-Max-Age", "86400")
            .body(axum::body::Body::empty())
            .unwrap();
    }

    // Process the request
    let mut response = next.run(request).await;

    // Add CORS headers to the response
    let headers = response.headers_mut();
    headers.insert(
        "Access-Control-Allow-Origin",
        origin.parse().unwrap_or_else(|_| "*".parse().unwrap()),
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-Requested-With"
            .parse()
            .unwrap(),
    );

    response
}

/// App lookup middleware
///
/// Extracts the app_id from the request path and looks up the app.
/// Stores the app in request extensions for use by handlers.
///
/// Requirements: 3.12
pub async fn app_lookup_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract app_id from the path
    let path = request.uri().path();
    
    tracing::debug!("app_lookup_middleware: path = {}", path);
    
    let app_id = extract_app_id_from_path(path);

    let app_id = match app_id {
        Some(id) => {
            tracing::debug!("app_lookup_middleware: found app_id = {}", id);
            id
        }
        None => {
            tracing::debug!("app_lookup_middleware: no app_id in path, skipping");
            // No app_id in path, skip this middleware
            return Ok(next.run(request).await);
        }
    };

    // Look up the app
    tracing::debug!("app_lookup_middleware: looking up app_id = {}", app_id);
    let app = match state.app_manager.find_by_id(&app_id).await {
        Ok(Some(app)) => {
            tracing::debug!("app_lookup_middleware: found app = {:?}", app.id);
            app
        }
        Ok(None) => {
            tracing::warn!("app_lookup_middleware: app not found: {}", app_id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "App not found"
                })),
            )
                .into_response());
        }
        Err(e) => {
            println!("[ERROR] app_lookup_middleware: error looking up app: {}", e);
            tracing::error!("app_lookup_middleware: error looking up app: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Failed to lookup app: {}", e)
                })),
            )
                .into_response());
        }
    };

    // Check if app is enabled
    if !app.enabled {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "App is disabled"
            })),
        )
            .into_response());
    }

    // Store the app in request extensions
    request.extensions_mut().insert(AuthenticatedApp(app));

    Ok(next.run(request).await)
}

/// Authentication middleware
///
/// Verifies the API request signature using HMAC-SHA256.
/// Requires the app to be present in request extensions (from app_lookup_middleware).
///
/// Requirements: 3.12, 8.4
pub async fn auth_middleware(
    State(_state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get the app from extensions (set by app_lookup_middleware)
    let app = match request.extensions().get::<AuthenticatedApp>() {
        Some(AuthenticatedApp(app)) => app.clone(),
        None => {
            // No app in extensions, this shouldn't happen if middleware is ordered correctly
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal error: app not found in request"
                })),
            )
                .into_response());
        }
    };

    // Extract authentication parameters from query string
    // Clone the data we need before moving request
    let query = request.uri().query().unwrap_or("").to_string();
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();

    let auth_params = parse_auth_params(&query);

    // Verify required auth parameters are present
    if auth_params.auth_key.is_none()
        || auth_params.auth_timestamp.is_none()
        || auth_params.auth_version.is_none()
        || auth_params.auth_signature.is_none()
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Missing authentication parameters"
            })),
        )
            .into_response());
    }

    let auth_key = auth_params.auth_key.unwrap();
    let auth_signature = auth_params.auth_signature.unwrap();

    // Verify the auth key matches the app key
    if auth_key != app.key {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid auth key"
            })),
        )
            .into_response());
    }

    // For the signature, we need the query string without the auth_signature parameter
    let query_without_signature = remove_auth_signature_from_query(&query);

    // Verify the signature with timestamp validation
    match verify_api_auth_with_timestamp(
        &auth_signature,
        &app.secret,
        &method,
        &path,
        &query_without_signature,
    ) {
        Ok(true) => {
            // Signature is valid, continue
            Ok(next.run(request).await)
        }
        Ok(false) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid signature or timestamp"
            })),
        )
            .into_response()),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Authentication error: {}", e)
            })),
        )
            .into_response()),
    }
}

/// Rate limiting middleware
///
/// Enforces rate limits on API requests.
/// Requires the app to be present in request extensions (from app_lookup_middleware).
///
/// Requirements: 8.4, 8.6
pub async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Get the app from extensions
    let app = match request.extensions().get::<AuthenticatedApp>() {
        Some(AuthenticatedApp(app)) => app.clone(),
        None => {
            // No app in extensions, skip rate limiting
            return Ok(next.run(request).await);
        }
    };

    // Determine the type of request and consume appropriate rate limit points
    let method = request.method().clone();
    let _path = request.uri().path().to_string();

    let rate_limit_result = if method == Method::GET {
        // Read request
        state
            .rate_limiter
            .consume_read_request_points(1, &app)
            .await
    } else {
        // Backend event (POST requests)
        state
            .rate_limiter
            .consume_backend_event_points(1, &app)
            .await
    };

    match rate_limit_result {
        Ok(response) => {
            if !response.can_continue {
                // Rate limit exceeded
                let mut resp_builder =
                    axum::response::Response::builder().status(StatusCode::TOO_MANY_REQUESTS);

                for (key, value) in response.headers {
                    if let Ok(header_value) = value.parse::<axum::http::HeaderValue>() {
                        resp_builder = resp_builder.header(key.as_str(), header_value);
                    }
                }

                return Err(resp_builder
                    .body(axum::body::Body::from(
                        serde_json::to_string(&json!({
                            "error": "Rate limit exceeded"
                        }))
                        .unwrap(),
                    ))
                    .unwrap()
                    .into_response());
            }

            // Rate limit OK, add headers to response
            let mut resp = next.run(request).await;
            let resp_headers = resp.headers_mut();
            for (key, value) in response.headers {
                if let Ok(header_value) = value.parse::<axum::http::HeaderValue>() {
                    resp_headers.insert(
                        axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        header_value,
                    );
                }
            }

            Ok(resp)
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Rate limiting error: {}", e)
            })),
        )
            .into_response()),
    }
}

/// Extract app_id from request path
///
/// Looks for patterns like /apps/{app_id}/...
fn extract_app_id_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();

    // Look for /apps/{app_id} pattern
    for i in 0..parts.len() {
        if parts[i] == "apps" && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
        }
    }

    None
}

/// Parse authentication parameters from query string
struct AuthParams {
    auth_key: Option<String>,
    auth_timestamp: Option<String>,
    auth_version: Option<String>,
    auth_signature: Option<String>,
}

fn parse_auth_params(query: &str) -> AuthParams {
    let mut params = AuthParams {
        auth_key: None,
        auth_timestamp: None,
        auth_version: None,
        auth_signature: None,
    };

    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            match key {
                "auth_key" => params.auth_key = Some(value.to_string()),
                "auth_timestamp" => params.auth_timestamp = Some(value.to_string()),
                "auth_version" => params.auth_version = Some(value.to_string()),
                "auth_signature" => params.auth_signature = Some(value.to_string()),
                _ => {}
            }
        }
    }

    params
}

/// Remove auth_signature from query string
///
/// The signature is calculated over the query string without the signature itself.
fn remove_auth_signature_from_query(query: &str) -> String {
    query
        .split('&')
        .filter(|param| !param.starts_with("auth_signature="))
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_id_from_path() {
        assert_eq!(
            extract_app_id_from_path("/apps/123/events"),
            Some("123".to_string())
        );
        assert_eq!(
            extract_app_id_from_path("/apps/my-app/channels"),
            Some("my-app".to_string())
        );
        assert_eq!(extract_app_id_from_path("/health"), None);
    }

    #[test]
    fn test_parse_auth_params() {
        let query =
            "auth_key=key&auth_timestamp=123&auth_version=1.0&auth_signature=sig&other=value";
        let params = parse_auth_params(query);

        assert_eq!(params.auth_key, Some("key".to_string()));
        assert_eq!(params.auth_timestamp, Some("123".to_string()));
        assert_eq!(params.auth_version, Some("1.0".to_string()));
        assert_eq!(params.auth_signature, Some("sig".to_string()));
    }

    #[test]
    fn test_remove_auth_signature_from_query() {
        let query = "auth_key=key&auth_timestamp=123&auth_signature=sig&auth_version=1.0";
        let result = remove_auth_signature_from_query(query);

        assert!(!result.contains("auth_signature"));
        assert!(result.contains("auth_key=key"));
        assert!(result.contains("auth_timestamp=123"));
        assert!(result.contains("auth_version=1.0"));
    }
}
