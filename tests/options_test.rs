use soketi_rs::config::{AdapterDriver, CacheDriver, ServerConfig, ServerMode};
use soketi_rs::options::Options;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test that default Options can be created
#[test]
fn test_options_default() {
    let options = Options::default();

    assert!(options.host.is_none());
    assert!(options.port.is_none());
    assert!(options.debug.is_none());
}

/// Test loading configuration from JSON file
#[test]
fn test_load_from_json_file() {
    // This test is simplified - in practice, config files should include all required fields
    // or use partial updates via Options::load() which merges with defaults

    // For now, we'll just test that the file loading mechanism works
    // by testing with a minimal valid config that has all required fields
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Create a default config and serialize it
    let default_config = ServerConfig::default();
    let json_config = serde_json::to_string_pretty(&default_config).unwrap();

    fs::write(&config_path, json_config).unwrap();

    // Load the config file
    let config = Options::load_from_file(&config_path).unwrap();

    // Verify it loaded successfully
    assert_eq!(config.host, default_config.host);
    assert_eq!(config.port, default_config.port);
}

/// Test loading configuration from YAML file
#[test]
fn test_load_from_yaml_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");

    // Create a default config and serialize it
    let default_config = ServerConfig::default();
    let yaml_config = serde_yaml::to_string(&default_config).unwrap();

    fs::write(&config_path, yaml_config).unwrap();

    let config = Options::load_from_file(&config_path).unwrap();

    assert_eq!(config.host, default_config.host);
    assert_eq!(config.port, default_config.port);
}

/// Test loading configuration from TOML file
#[test]
fn test_load_from_toml_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create a default config and serialize it
    let default_config = ServerConfig::default();
    let toml_config = toml::to_string_pretty(&default_config).unwrap();

    fs::write(&config_path, toml_config).unwrap();

    let config = Options::load_from_file(&config_path).unwrap();

    assert_eq!(config.host, default_config.host);
    assert_eq!(config.port, default_config.port);
}

/// Test that unsupported file format returns error
#[test]
fn test_load_from_unsupported_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.txt");

    fs::write(&config_path, "some content").unwrap();

    let result = Options::load_from_file(&config_path);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unsupported config file format")
    );
}

/// Test that missing file returns error
#[test]
fn test_load_from_missing_file() {
    let config_path = PathBuf::from("/nonexistent/config.json");

    let result = Options::load_from_file(&config_path);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to read config file")
    );
}

/// Test that invalid JSON returns error
#[test]
fn test_load_from_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    fs::write(&config_path, "{ invalid json }").unwrap();

    let result = Options::load_from_file(&config_path);

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse JSON config")
    );
}

/// Test applying options to config
#[test]
fn test_apply_to_config() {
    let mut config = ServerConfig::default();

    let options = Options {
        host: Some("custom.host".to_string()),
        port: Some(9999),
        debug: Some(true),
        adapter_driver: Some("redis".to_string()),
        cache_driver: Some("redis".to_string()),
        metrics_enabled: Some(true),
        metrics_prefix: Some("custom_prefix".to_string()),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    assert_eq!(config.host, "custom.host");
    assert_eq!(config.port, 9999);
    assert_eq!(config.debug, true);
    assert_eq!(config.adapter.driver, AdapterDriver::Redis);
    assert_eq!(config.cache.driver, CacheDriver::Redis);
    assert_eq!(config.metrics.enabled, true);
    assert_eq!(config.metrics.prefix, "custom_prefix");
}

/// Test that None values don't override config
#[test]
fn test_apply_to_config_preserves_existing() {
    let mut config = ServerConfig::default();
    config.host = "existing.host".to_string();
    config.port = 5555;

    let options = Options {
        debug: Some(true),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    // Existing values should be preserved
    assert_eq!(config.host, "existing.host");
    assert_eq!(config.port, 5555);
    // New value should be applied
    assert_eq!(config.debug, true);
}

/// Test parsing adapter driver strings
#[test]
fn test_adapter_driver_parsing() {
    let mut config = ServerConfig::default();

    let options = Options {
        adapter_driver: Some("local".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.adapter.driver, AdapterDriver::Local);

    let options = Options {
        adapter_driver: Some("cluster".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.adapter.driver, AdapterDriver::Cluster);

    let options = Options {
        adapter_driver: Some("redis".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.adapter.driver, AdapterDriver::Redis);

    let options = Options {
        adapter_driver: Some("nats".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.adapter.driver, AdapterDriver::Nats);
}

/// Test parsing server mode strings
#[test]
fn test_server_mode_parsing() {
    let mut config = ServerConfig::default();

    let options = Options {
        mode: Some("full".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.mode, ServerMode::Full);

    let options = Options {
        mode: Some("server".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.mode, ServerMode::Server);

    let options = Options {
        mode: Some("worker".to_string()),
        ..Default::default()
    };
    options.apply_to_config(&mut config);
    assert_eq!(config.mode, ServerMode::Worker);
}

/// Test parsing comma-separated values
#[test]
fn test_comma_separated_parsing() {
    let mut config = ServerConfig::default();

    let options = Options {
        adapter_nats_servers: Some("server1:4222,server2:4222,server3:4222".to_string()),
        cors_origins: Some("http://localhost:3000,http://example.com".to_string()),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    assert_eq!(
        config.adapter.nats.servers,
        vec![
            "server1:4222".to_string(),
            "server2:4222".to_string(),
            "server3:4222".to_string()
        ]
    );

    assert_eq!(
        config.cors.origins,
        vec![
            "http://localhost:3000".to_string(),
            "http://example.com".to_string()
        ]
    );
}

/// Test applying database configuration
#[test]
fn test_database_config_application() {
    let mut config = ServerConfig::default();

    let options = Options {
        mysql_host: Some("mysql.example.com".to_string()),
        mysql_port: Some(3307),
        mysql_user: Some("testuser".to_string()),
        mysql_password: Some("testpass".to_string()),
        mysql_database: Some("testdb".to_string()),
        postgres_host: Some("postgres.example.com".to_string()),
        postgres_port: Some(5433),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    assert_eq!(config.app_manager.mysql.host, "mysql.example.com");
    assert_eq!(config.app_manager.mysql.port, 3307);
    assert_eq!(config.app_manager.mysql.user, "testuser");
    assert_eq!(config.app_manager.mysql.password, "testpass");
    assert_eq!(config.app_manager.mysql.database, "testdb");

    assert_eq!(config.app_manager.postgres.host, "postgres.example.com");
    assert_eq!(config.app_manager.postgres.port, 5433);
}

/// Test applying limits configuration
#[test]
fn test_limits_config_application() {
    let mut config = ServerConfig::default();

    let options = Options {
        channel_max_name_length: Some(500),
        event_max_name_length: Some(300),
        event_max_payload_kb: Some(200.0),
        event_max_batch_size: Some(20),
        presence_max_members: Some(200),
        presence_max_member_size_kb: Some(5.0),
        http_max_request_size_kb: Some(150.0),
        http_memory_threshold_mb: Some(1024),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    assert_eq!(config.channel_limits.max_name_length, 500);
    assert_eq!(config.event_limits.max_name_length, 300);
    assert_eq!(config.event_limits.max_payload_in_kb, 200.0);
    assert_eq!(config.event_limits.max_batch_size, 20);
    assert_eq!(config.presence.max_members_per_channel, 200);
    assert_eq!(config.presence.max_member_size_in_kb, 5.0);
    assert_eq!(config.http_api.max_request_size_in_kb, 150.0);
    assert_eq!(config.http_api.accept_traffic_memory_threshold_mb, 1024);
}

/// Test SSL configuration
#[test]
fn test_ssl_config_application() {
    let mut config = ServerConfig::default();

    let options = Options {
        ssl_enabled: Some(true),
        ssl_cert_path: Some("/path/to/cert.pem".to_string()),
        ssl_key_path: Some("/path/to/key.pem".to_string()),
        ..Default::default()
    };

    options.apply_to_config(&mut config);

    assert_eq!(config.ssl.enabled, true);
    assert_eq!(config.ssl.cert_path, "/path/to/cert.pem");
    assert_eq!(config.ssl.key_path, "/path/to/key.pem");
}

/// Test that default values are sensible
#[test]
fn test_default_config_values() {
    let config = ServerConfig::default();

    // Server defaults
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 6001);
    assert_eq!(config.path_prefix, "");
    assert_eq!(config.debug, false);
    assert_eq!(config.mode, ServerMode::Full);
    assert_eq!(config.shutdown_grace_period_ms, 3000);

    // Adapter defaults
    assert_eq!(config.adapter.driver, AdapterDriver::Local);

    // Cache defaults
    assert_eq!(config.cache.driver, CacheDriver::Memory);

    // Metrics defaults
    assert_eq!(config.metrics.enabled, false);
    assert_eq!(config.metrics.port, 9601);
    assert_eq!(config.metrics.prefix, "pusher");

    // SSL defaults
    assert_eq!(config.ssl.enabled, false);

    // CORS defaults
    assert_eq!(config.cors.enabled, true);
    assert_eq!(config.cors.origins, vec!["*"]);

    // Channel limits defaults
    assert_eq!(config.channel_limits.max_name_length, 200);

    // Event limits defaults
    assert_eq!(config.event_limits.max_channels_at_once, 100);
    assert_eq!(config.event_limits.max_name_length, 200);
    assert_eq!(config.event_limits.max_payload_in_kb, 100.0);
    assert_eq!(config.event_limits.max_batch_size, 10);

    // Presence limits defaults
    assert_eq!(config.presence.max_members_per_channel, 100);
    assert_eq!(config.presence.max_member_size_in_kb, 2.0);
}
