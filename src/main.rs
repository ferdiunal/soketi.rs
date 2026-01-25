use soketi_rs::options::Options;
use soketi_rs::server::Server;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration from all sources (CLI args, env vars, config file)
    let config = match Options::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize structured logging
    soketi_rs::log::init_logging(config.debug);

    // Create and initialize server
    let mut server = Server::new(config.clone());

    if let Err(e) = server.initialize().await {
        tracing::error!("Failed to initialize server: {}", e);
        std::process::exit(1);
    }

    // Start the server
    if let Err(e) = server.start().await {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }
}
