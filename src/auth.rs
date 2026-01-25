use hex;
use hmac::{Hmac, Mac};
use md5;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Maximum allowed age for authentication timestamps in seconds
/// Pusher protocol requires timestamps to be within 600 seconds (10 minutes) of server time
pub const MAX_TIMESTAMP_AGE_SECONDS: u64 = 600;

/// Generate a Pusher signature using HMAC-SHA256
///
/// This function creates a signature for various Pusher protocol operations:
/// - WebSocket authentication
/// - HTTP API authentication
/// - Channel authentication (private and presence channels)
/// - User authentication (pusher:signin)
///
/// # Arguments
/// * `app_secret` - The application secret key
/// * `to_sign` - The string to sign (format varies by use case)
///
/// # Returns
/// A hex-encoded HMAC-SHA256 signature
///
/// # Examples
///
/// ## WebSocket Channel Authentication
/// For private/presence channels, the string to sign is: `socket_id:channel_name`
/// ```ignore
/// let signature = generate_pusher_signature("app_secret", "123.456:private-channel");
/// ```
///
/// ## HTTP API Authentication
/// For API requests, the string to sign is: `METHOD\nPATH\nQUERY_STRING`
/// ```ignore
/// let to_sign = "POST\n/apps/123/events\nauth_key=key&auth_timestamp=123&auth_version=1.0";
/// let signature = generate_pusher_signature("app_secret", &to_sign);
/// ```
///
/// ## User Authentication (pusher:signin)
/// For user signin, the string to sign is: `socket_id::user::user_data`
/// ```ignore
/// let to_sign = "123.456::user::{\"id\":\"user123\"}";
/// let signature = generate_pusher_signature("app_secret", &to_sign);
/// ```
pub fn generate_pusher_signature(app_secret: &str, to_sign: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(app_secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(to_sign.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes)
}

/// Verify a Pusher signature
///
/// Compares a provided signature against the expected signature for the given data.
///
/// # Arguments
/// * `signature` - The signature to verify
/// * `app_secret` - The application secret key
/// * `to_sign` - The string that was signed
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise
pub fn verify_pusher_signature(signature: &str, app_secret: &str, to_sign: &str) -> bool {
    let expected_signature = generate_pusher_signature(app_secret, to_sign);
    expected_signature == signature
}

/// Generate an auth string for channel authentication
///
/// Creates the full auth string in the format: `app_key:signature`
/// This is used for private and presence channel subscriptions.
///
/// # Arguments
/// * `app_key` - The application key
/// * `app_secret` - The application secret
/// * `socket_id` - The socket ID
/// * `channel_name` - The channel name
/// * `channel_data` - Optional channel data (required for presence channels)
///
/// # Returns
/// The auth string in format `app_key:signature`
pub fn generate_channel_auth(
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel_name: &str,
    channel_data: Option<&str>,
) -> String {
    let to_sign = if let Some(data) = channel_data {
        format!("{}:{}:{}", socket_id, channel_name, data)
    } else {
        format!("{}:{}", socket_id, channel_name)
    };

    let signature = generate_pusher_signature(app_secret, &to_sign);
    format!("{}:{}", app_key, signature)
}

/// Verify channel authentication
///
/// Verifies that the provided auth string is valid for the given channel subscription.
///
/// # Arguments
/// * `auth` - The auth string to verify (format: `app_key:signature`)
/// * `app_key` - The expected application key
/// * `app_secret` - The application secret
/// * `socket_id` - The socket ID
/// * `channel_name` - The channel name
/// * `channel_data` - Optional channel data (required for presence channels)
///
/// # Returns
/// `true` if the auth is valid, `false` otherwise
pub fn verify_channel_auth(
    auth: &str,
    app_key: &str,
    app_secret: &str,
    socket_id: &str,
    channel_name: &str,
    channel_data: Option<&str>,
) -> bool {
    let expected_auth =
        generate_channel_auth(app_key, app_secret, socket_id, channel_name, channel_data);
    expected_auth == auth
}

/// Generate an auth string for user authentication (pusher:signin)
///
/// Creates the auth signature for user authentication.
///
/// # Arguments
/// * `app_secret` - The application secret
/// * `socket_id` - The socket ID
/// * `user_data` - The user data as a JSON string
///
/// # Returns
/// The signature for user authentication
pub fn generate_user_auth(app_secret: &str, socket_id: &str, user_data: &str) -> String {
    let to_sign = format!("{}::user::{}", socket_id, user_data);
    generate_pusher_signature(app_secret, &to_sign)
}

/// Verify user authentication
///
/// Verifies that the provided auth signature is valid for user authentication.
///
/// # Arguments
/// * `auth` - The auth signature to verify
/// * `app_secret` - The application secret
/// * `socket_id` - The socket ID
/// * `user_data` - The user data as a JSON string
///
/// # Returns
/// `true` if the auth is valid, `false` otherwise
pub fn verify_user_auth(auth: &str, app_secret: &str, socket_id: &str, user_data: &str) -> bool {
    let expected_auth = generate_user_auth(app_secret, socket_id, user_data);
    expected_auth == auth
}

/// Generate MD5 hash of a string
///
/// Used for HTTP API request body hashing.
///
/// # Arguments
/// * `data` - The data to hash
///
/// # Returns
/// The hex-encoded MD5 hash
pub fn generate_md5_hash(data: &str) -> String {
    let digest = md5::compute(data.as_bytes());
    format!("{:x}", digest)
}

/// Generate signature for HTTP API requests
///
/// Creates the signature for authenticating HTTP API requests.
///
/// # Arguments
/// * `app_secret` - The application secret
/// * `method` - The HTTP method (e.g., "POST", "GET")
/// * `path` - The request path (e.g., "/apps/123/events")
/// * `query_string` - The query string with auth parameters
///
/// # Returns
/// The signature for the API request
pub fn generate_api_auth_signature(
    app_secret: &str,
    method: &str,
    path: &str,
    query_string: &str,
) -> String {
    let to_sign = format!("{}\n{}\n{}", method, path, query_string);
    generate_pusher_signature(app_secret, &to_sign)
}

/// Verify HTTP API request signature
///
/// Verifies that the provided signature is valid for the API request.
///
/// # Arguments
/// * `signature` - The signature to verify
/// * `app_secret` - The application secret
/// * `method` - The HTTP method
/// * `path` - The request path
/// * `query_string` - The query string with auth parameters
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise
pub fn verify_api_auth_signature(
    signature: &str,
    app_secret: &str,
    method: &str,
    path: &str,
    query_string: &str,
) -> bool {
    let expected_signature = generate_api_auth_signature(app_secret, method, path, query_string);
    expected_signature == signature
}

/// Get the current Unix timestamp in seconds
///
/// # Returns
/// The current time as seconds since Unix epoch
pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Validate that a timestamp is within the acceptable time window
///
/// According to the Pusher protocol, authentication timestamps must be within
/// 600 seconds (10 minutes) of the current server time to prevent replay attacks.
///
/// # Arguments
/// * `timestamp` - The timestamp to validate (Unix timestamp in seconds)
///
/// # Returns
/// `true` if the timestamp is within the acceptable window, `false` otherwise
///
/// # Examples
///
/// ```ignore
/// let current_time = get_current_timestamp();
/// assert!(validate_timestamp(current_time));
///
/// // Timestamp from 5 minutes ago should be valid
/// assert!(validate_timestamp(current_time - 300));
///
/// // Timestamp from 15 minutes ago should be invalid
/// assert!(!validate_timestamp(current_time - 900));
/// ```
pub fn validate_timestamp(timestamp: u64) -> bool {
    let current_time = get_current_timestamp();
    let time_diff = current_time.abs_diff(timestamp);

    time_diff <= MAX_TIMESTAMP_AGE_SECONDS
}

/// Validate timestamp from a string
///
/// Parses a timestamp string and validates it's within the acceptable time window.
///
/// # Arguments
/// * `timestamp_str` - The timestamp as a string (Unix timestamp in seconds)
///
/// # Returns
/// `Ok(true)` if the timestamp is valid and within the window
/// `Ok(false)` if the timestamp is valid but outside the window
/// `Err` if the timestamp string cannot be parsed
///
/// # Examples
///
/// ```ignore
/// let current_time = get_current_timestamp();
/// let timestamp_str = current_time.to_string();
/// assert!(validate_timestamp_str(&timestamp_str).unwrap());
/// ```
pub fn validate_timestamp_str(timestamp_str: &str) -> Result<bool, std::num::ParseIntError> {
    let timestamp = timestamp_str.parse::<u64>()?;
    Ok(validate_timestamp(timestamp))
}

/// Verify HTTP API request signature with timestamp validation
///
/// This is an enhanced version of `verify_api_auth_signature` that also validates
/// the timestamp is within the acceptable time window.
///
/// # Arguments
/// * `signature` - The signature to verify
/// * `app_secret` - The application secret
/// * `method` - The HTTP method
/// * `path` - The request path
/// * `query_string` - The query string with auth parameters (must include auth_timestamp)
///
/// # Returns
/// `Ok(true)` if both signature and timestamp are valid
/// `Ok(false)` if signature is invalid or timestamp is outside the window
/// `Err` if timestamp cannot be parsed from query string
///
/// # Examples
///
/// ```ignore
/// let current_time = get_current_timestamp();
/// let query = format!("auth_key=key&auth_timestamp={}&auth_version=1.0", current_time);
/// let signature = generate_api_auth_signature("secret", "POST", "/apps/123/events", &query);
///
/// let result = verify_api_auth_with_timestamp(&signature, "secret", "POST", "/apps/123/events", &query);
/// assert!(result.unwrap());
/// ```
pub fn verify_api_auth_with_timestamp(
    signature: &str,
    app_secret: &str,
    method: &str,
    path: &str,
    query_string: &str,
) -> Result<bool, String> {
    // First verify the signature
    if !verify_api_auth_signature(signature, app_secret, method, path, query_string) {
        return Ok(false);
    }

    // Extract and validate timestamp from query string
    let timestamp = extract_timestamp_from_query(query_string)?;

    if !validate_timestamp(timestamp) {
        return Ok(false);
    }

    Ok(true)
}

/// Extract auth_timestamp from query string
///
/// # Arguments
/// * `query_string` - The query string containing auth_timestamp parameter
///
/// # Returns
/// `Ok(timestamp)` if auth_timestamp is found and can be parsed
/// `Err` if auth_timestamp is missing or cannot be parsed
fn extract_timestamp_from_query(query_string: &str) -> Result<u64, String> {
    for param in query_string.split('&') {
        if let Some((key, value)) = param.split_once('=')
            && key == "auth_timestamp"
        {
            return value
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse auth_timestamp: {}", e));
        }
    }

    Err("auth_timestamp not found in query string".to_string())
}
