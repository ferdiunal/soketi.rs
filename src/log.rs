use tracing::{debug, error, info, trace, warn};

/// Logging utility wrapper for structured logging
pub struct Log;

impl Log {
    pub fn info(message: &str) {
        info!("{}", message);
    }

    pub fn error(message: &str) {
        error!("{}", message);
    }

    pub fn warning(message: &str) {
        warn!("{}", message);
    }

    pub fn debug(message: &str) {
        debug!("{}", message);
    }

    pub fn trace(message: &str) {
        trace!("{}", message);
    }

    // Soketi specific titles mapping to simple logs
    pub fn info_title(message: &str) {
        info!("[INFO] {}", message);
    }

    pub fn success_title(message: &str) {
        info!("[SUCCESS] {}", message);
    }

    pub fn error_title(message: &str) {
        error!("[ERROR] {}", message);
    }

    pub fn warning_title(message: &str) {
        warn!("[WARNING] {}", message);
    }

    pub fn debug_title(message: &str) {
        debug!("[DEBUG] {}", message);
    }
}

/// Initialize structured logging with tracing
pub fn init_logging(debug: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_line_number(debug)
        .with_file(debug)
        .init();
}
