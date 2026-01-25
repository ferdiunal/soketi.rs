// Standalone test for timestamp validation functionality
// This test file only tests the timestamp validation without depending on other modules

use std::time::{SystemTime, UNIX_EPOCH};

const MAX_TIMESTAMP_AGE_SECONDS: u64 = 600;

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn validate_timestamp(timestamp: u64) -> bool {
    let current_time = get_current_timestamp();
    let time_diff = if current_time >= timestamp {
        current_time - timestamp
    } else {
        timestamp - current_time
    };

    time_diff <= MAX_TIMESTAMP_AGE_SECONDS
}

fn validate_timestamp_str(timestamp_str: &str) -> Result<bool, std::num::ParseIntError> {
    let timestamp = timestamp_str.parse::<u64>()?;
    Ok(validate_timestamp(timestamp))
}

fn extract_timestamp_from_query(query_string: &str) -> Result<u64, String> {
    for param in query_string.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            if key == "auth_timestamp" {
                return value
                    .parse::<u64>()
                    .map_err(|e| format!("Failed to parse auth_timestamp: {}", e));
            }
        }
    }

    Err("auth_timestamp not found in query string".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_current_timestamp() {
        let current_time = get_current_timestamp();
        assert!(validate_timestamp(current_time));
    }

    #[test]
    fn test_validate_recent_timestamp() {
        // Timestamp from 5 minutes ago should be valid
        let current_time = get_current_timestamp();
        let five_minutes_ago = current_time - 300;
        assert!(validate_timestamp(five_minutes_ago));
    }

    #[test]
    fn test_validate_future_timestamp() {
        // Timestamp from 5 minutes in the future should be valid (clock skew)
        let current_time = get_current_timestamp();
        let five_minutes_future = current_time + 300;
        assert!(validate_timestamp(five_minutes_future));
    }

    #[test]
    fn test_reject_old_timestamp() {
        // Timestamp from 15 minutes ago should be invalid
        let current_time = get_current_timestamp();
        let fifteen_minutes_ago = current_time - 900;
        assert!(!validate_timestamp(fifteen_minutes_ago));
    }

    #[test]
    fn test_reject_far_future_timestamp() {
        // Timestamp from 15 minutes in the future should be invalid
        let current_time = get_current_timestamp();
        let fifteen_minutes_future = current_time + 900;
        assert!(!validate_timestamp(fifteen_minutes_future));
    }

    #[test]
    fn test_validate_timestamp_at_boundary() {
        // Timestamp exactly 600 seconds ago should be valid
        let current_time = get_current_timestamp();
        let boundary_timestamp = current_time - 600;
        assert!(validate_timestamp(boundary_timestamp));
    }

    #[test]
    fn test_reject_timestamp_just_outside_boundary() {
        // Timestamp 601 seconds ago should be invalid
        let current_time = get_current_timestamp();
        let outside_boundary = current_time - 601;
        assert!(!validate_timestamp(outside_boundary));
    }

    #[test]
    fn test_validate_timestamp_str_valid() {
        let current_time = get_current_timestamp();
        let timestamp_str = current_time.to_string();
        assert!(validate_timestamp_str(&timestamp_str).unwrap());
    }

    #[test]
    fn test_validate_timestamp_str_invalid_format() {
        let result = validate_timestamp_str("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_timestamp_str_old() {
        let current_time = get_current_timestamp();
        let old_timestamp = current_time - 900;
        let timestamp_str = old_timestamp.to_string();
        assert!(!validate_timestamp_str(&timestamp_str).unwrap());
    }

    #[test]
    fn test_extract_timestamp_from_query_success() {
        let query = "auth_key=key&auth_timestamp=1234567890&auth_version=1.0";
        let timestamp = extract_timestamp_from_query(query).unwrap();
        assert_eq!(timestamp, 1234567890);
    }

    #[test]
    fn test_extract_timestamp_from_query_missing() {
        let query = "auth_key=key&auth_version=1.0";
        let result = extract_timestamp_from_query(query);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "auth_timestamp not found in query string"
        );
    }

    #[test]
    fn test_extract_timestamp_from_query_invalid_format() {
        let query = "auth_key=key&auth_timestamp=invalid&auth_version=1.0";
        let result = extract_timestamp_from_query(query);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Failed to parse auth_timestamp")
        );
    }

    #[test]
    fn test_extract_timestamp_from_query_first_param() {
        let query = "auth_timestamp=1234567890&auth_key=key&auth_version=1.0";
        let timestamp = extract_timestamp_from_query(query).unwrap();
        assert_eq!(timestamp, 1234567890);
    }

    #[test]
    fn test_extract_timestamp_from_query_last_param() {
        let query = "auth_key=key&auth_version=1.0&auth_timestamp=1234567890";
        let timestamp = extract_timestamp_from_query(query).unwrap();
        assert_eq!(timestamp, 1234567890);
    }

    #[test]
    fn test_timestamp_validation_window() {
        // Test the 600 second window (10 minutes)
        let current_time = get_current_timestamp();

        // Test various points within the window
        for offset in [0, 100, 200, 300, 400, 500, 599, 600] {
            assert!(
                validate_timestamp(current_time - offset),
                "Timestamp {} seconds ago should be valid",
                offset
            );
            assert!(
                validate_timestamp(current_time + offset),
                "Timestamp {} seconds in future should be valid",
                offset
            );
        }

        // Test just outside the window
        assert!(
            !validate_timestamp(current_time - 601),
            "Timestamp 601 seconds ago should be invalid"
        );
        assert!(
            !validate_timestamp(current_time + 601),
            "Timestamp 601 seconds in future should be invalid"
        );
    }

    #[test]
    fn test_zero_timestamp() {
        // Timestamp of 0 (Unix epoch) should be invalid
        assert!(!validate_timestamp(0));
    }

    #[test]
    fn test_max_u64_timestamp() {
        // Very far future timestamp should be invalid
        assert!(!validate_timestamp(u64::MAX));
    }
}
