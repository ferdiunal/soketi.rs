use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use soketi_rs::app::App;
use soketi_rs::config::{
    AdapterDriver, AppManagerDriver, CacheDriver, QueueDriver, RateLimiterDriver, ServerConfig,
};
use soketi_rs::server::Server;
use std::env;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const APP_ID: &str = "payload_matrix_app";
const APP_KEY: &str = "payload_matrix_key";
const APP_SECRET: &str = "payload_matrix_secret";
const DEFAULT_REPORT_PATH: &str = "target/benchmarks/payload-matrix.json";
const REPORT_PATH_ENV: &str = "SOKETI_BENCH_REPORT";
const SCALE_ENV: &str = "SOKETI_BENCH_MATRIX_SCALE";

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsWrite = SplitSink<WsStream, Message>;
type WsRead = SplitStream<WsStream>;

#[derive(Clone)]
struct BenchServer {
    port: u16,
}

impl BenchServer {
    fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/app/{}", self.port, APP_KEY)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PayloadScenario {
    name: &'static str,
    payload_bytes: usize,
    messages: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct LatencyStats {
    p50_us: u64,
    p95_us: u64,
    p99_us: u64,
    p999_us: u64,
    max_us: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScenarioReport {
    scenario: String,
    attempted: usize,
    delivered: usize,
    duration_ms: u128,
    throughput_per_second: f64,
    delivery_rate: f64,
    latency: LatencyStats,
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    runtime.block_on(async {
        let report = run_payload_matrix().await;
        write_report(&report);
    });
}

fn payload_scenarios() -> [PayloadScenario; 4] {
    [
        PayloadScenario {
            name: "ws_client_event_1_to_1_payload_100b",
            payload_bytes: 100,
            messages: 100_000,
        },
        PayloadScenario {
            name: "ws_client_event_1_to_1_payload_1kb",
            payload_bytes: 1024,
            messages: 100_000,
        },
        PayloadScenario {
            name: "ws_client_event_1_to_1_payload_10kb",
            payload_bytes: 10 * 1024,
            messages: 50_000,
        },
        PayloadScenario {
            name: "ws_client_event_1_to_1_payload_100kb",
            payload_bytes: 100 * 1024,
            messages: 5_000,
        },
    ]
}

fn scaled_messages(messages: usize) -> usize {
    env::var(SCALE_ENV)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|scale| *scale > 0.0 && *scale <= 1.0)
        .map(|scale| ((messages as f64) * scale).ceil() as usize)
        .unwrap_or(messages)
        .max(1)
}

fn payload_string(bytes: usize) -> String {
    "x".repeat(bytes)
}

fn client_event_message(
    event: &str,
    channel: &str,
    sequence: usize,
    sent_at_us: u64,
    payload: &str,
) -> String {
    format!(
        r#"{{"event":"{}","channel":"{}","data":{{"sequence":{},"sent_at_us":{},"payload":"{}"}}}}"#,
        event, channel, sequence, sent_at_us, payload
    )
}

async fn run_payload_matrix() -> Vec<ScenarioReport> {
    let server = start_bench_server().await;
    let ws_url = server.ws_url();
    let mut reports = Vec::with_capacity(payload_scenarios().len());

    for scenario in payload_scenarios() {
        eprintln!(
            "running {}: {} messages, {} byte payload",
            scenario.name,
            scaled_messages(scenario.messages),
            scenario.payload_bytes
        );
        reports.push(run_ws_client_event_payload(ws_url.clone(), scenario).await);
        sleep(Duration::from_millis(250)).await;
    }

    // Keep the server state alive until every scenario has finished.
    drop(server);
    reports
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind ephemeral port");
    listener
        .local_addr()
        .expect("failed to read ephemeral port")
        .port()
}

fn create_bench_config(port: u16) -> ServerConfig {
    let mut config = ServerConfig::default();
    config.host = "127.0.0.1".to_string();
    config.port = port;
    config.adapter.driver = AdapterDriver::Local;
    config.app_manager.driver = AppManagerDriver::Array;
    config.cache.driver = CacheDriver::Memory;
    config.rate_limiter.driver = RateLimiterDriver::Local;
    config.queue.driver = QueueDriver::Sync;
    config.metrics.enabled = false;
    config.debug = false;
    config.http_api.max_request_size_in_kb = 256.0;

    let mut app = App::new(
        APP_ID.to_string(),
        APP_KEY.to_string(),
        APP_SECRET.to_string(),
    );
    app.enable_client_messages = true;
    app.max_backend_events_per_second = Some(1_000_000);
    app.max_client_events_per_second = Some(1_000_000);
    app.max_read_requests_per_second = Some(1_000_000);
    app.max_event_payload_in_kb = Some(256.0);

    config.app_manager.array.apps = vec![app];
    config
}

async fn start_bench_server() -> BenchServer {
    let port = free_port();
    let config = create_bench_config(port);
    let mut server = Server::new(config);
    server
        .initialize()
        .await
        .expect("failed to initialize benchmark server");
    tokio::spawn(async move {
        if let Err(error) = server.start().await {
            eprintln!("payload matrix server failed: {}", error);
        }
    });

    wait_for_server(port).await;
    BenchServer { port }
}

async fn wait_for_server(port: u16) {
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return;
        }

        sleep(Duration::from_millis(25)).await;
    }

    panic!("benchmark server did not start on port {}", port);
}

async fn connect_client(ws_url: &str) -> (WsWrite, WsRead) {
    let mut last_error = None;
    let mut stream = None;

    for attempt in 0..20 {
        match connect_async(ws_url).await {
            Ok((mut connected, _)) => {
                set_tcp_nodelay(&mut connected);
                stream = Some(connected);
                break;
            }
            Err(error) => {
                last_error = Some(error);
                sleep(Duration::from_millis(25 * (attempt + 1))).await;
            }
        }
    }

    let stream = stream.unwrap_or_else(|| {
        panic!(
            "failed to connect websocket client after retries: {:?}",
            last_error
        )
    });

    let (write, mut read) = stream.split();
    let message = next_json_message(&mut read).await;
    assert_eq!(message["event"], "pusher:connection_established");

    (write, read)
}

fn set_tcp_nodelay(stream: &mut WsStream) {
    if let MaybeTlsStream::Plain(tcp) = stream.get_mut() {
        let _ = tcp.set_nodelay(true);
    }
}

async fn next_json_message(read: &mut WsRead) -> Value {
    loop {
        match read.next().await {
            Some(Ok(Message::Text(text))) => {
                return serde_json::from_str(&text).expect("websocket message was not json");
            }
            Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {}
            Some(Ok(Message::Close(_))) => panic!("websocket closed before expected message"),
            Some(Ok(_)) => {}
            Some(Err(error)) => panic!("websocket read failed: {}", error),
            None => panic!("websocket ended before expected message"),
        }
    }
}

async fn subscribe_public(write: &mut WsWrite, read: &mut WsRead, channel: &str) {
    let message = serde_json::json!({
        "event": "pusher:subscribe",
        "data": {
            "channel": channel
        }
    });

    write
        .send(Message::Text(message.to_string().into()))
        .await
        .expect("failed to send subscribe message");

    let response = next_json_message(read).await;
    assert_eq!(response["event"], "pusher_internal:subscription_succeeded");
    assert_eq!(response["channel"], channel);
}

async fn receive_expected(
    mut read: WsRead,
    event: &'static str,
    expected: usize,
    started_at: Instant,
) -> (usize, Histogram<u64>) {
    let mut delivered = 0;
    let mut histogram =
        Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).expect("histogram init failed");
    let deadline = Instant::now() + Duration::from_secs(60);

    while delivered < expected {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline.saturating_duration_since(now);
        let message = match tokio::time::timeout(remaining, read.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => text,
            Ok(Some(Ok(Message::Ping(_)))) | Ok(Some(Ok(Message::Pong(_)))) => continue,
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => break,
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(_))) | Err(_) => break,
        };

        let value: Value = serde_json::from_str(&message).expect("broadcast message was not json");
        if value["event"] != event {
            continue;
        }

        delivered += 1;

        if let Some(sent_at_us) = value
            .get("data")
            .and_then(|data| data.get("sent_at_us"))
            .and_then(Value::as_u64)
        {
            let now_us = started_at.elapsed().as_micros() as u64;
            let latency_us = now_us.saturating_sub(sent_at_us).max(1);
            let _ = histogram.record(latency_us);
        }
    }

    (delivered, histogram)
}

async fn run_ws_client_event_payload(ws_url: String, scenario: PayloadScenario) -> ScenarioReport {
    let channel = format!("{}-channel", scenario.name);
    let event = "client-bench-event";
    let messages = scaled_messages(scenario.messages);
    let payload = payload_string(scenario.payload_bytes);
    let started_at = Instant::now();

    let (mut sender_write, mut sender_read) = connect_client(&ws_url).await;
    subscribe_public(&mut sender_write, &mut sender_read, &channel).await;

    let (mut receiver_write, mut receiver_read) = connect_client(&ws_url).await;
    subscribe_public(&mut receiver_write, &mut receiver_read, &channel).await;
    let receiver_task = tokio::spawn(receive_expected(receiver_read, event, messages, started_at));

    for sequence in 0..messages {
        let sent_at_us = started_at.elapsed().as_micros() as u64;
        let message = client_event_message(event, &channel, sequence, sent_at_us, &payload);

        sender_write
            .send(Message::Text(message.into()))
            .await
            .expect("failed to send client event");
    }

    let (delivered, histogram) = receiver_task.await.expect("receiver task failed");

    let _ = sender_write.close().await;
    let _ = receiver_write.close().await;

    scenario_report(scenario.name, messages, delivered, started_at, &histogram)
}

fn stats_from_histogram(histogram: &Histogram<u64>) -> LatencyStats {
    if histogram.is_empty() {
        return LatencyStats::default();
    }

    LatencyStats {
        p50_us: histogram.value_at_quantile(0.50),
        p95_us: histogram.value_at_quantile(0.95),
        p99_us: histogram.value_at_quantile(0.99),
        p999_us: histogram.value_at_quantile(0.999),
        max_us: histogram.max(),
    }
}

fn scenario_report(
    scenario: &str,
    attempted: usize,
    delivered: usize,
    started_at: Instant,
    histogram: &Histogram<u64>,
) -> ScenarioReport {
    let duration = started_at.elapsed();
    let throughput_per_second = if duration.is_zero() {
        0.0
    } else {
        delivered as f64 / duration.as_secs_f64()
    };
    let delivery_rate = if attempted == 0 {
        1.0
    } else {
        delivered as f64 / attempted as f64
    };

    ScenarioReport {
        scenario: scenario.to_string(),
        attempted,
        delivered,
        duration_ms: duration.as_millis(),
        throughput_per_second,
        delivery_rate,
        latency: stats_from_histogram(histogram),
    }
}

fn report_path() -> PathBuf {
    env::var_os(REPORT_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_REPORT_PATH))
}

fn write_report(report: &[ScenarioReport]) {
    let path = report_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create benchmark report directory");
    }

    let encoded = serde_json::to_vec_pretty(report).expect("failed to encode benchmark report");
    fs::write(path, encoded).expect("failed to write benchmark report");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_scenarios_match_balanced_profile() {
        let scenarios = payload_scenarios();

        assert_eq!(
            scenarios,
            [
                PayloadScenario {
                    name: "ws_client_event_1_to_1_payload_100b",
                    payload_bytes: 100,
                    messages: 100_000,
                },
                PayloadScenario {
                    name: "ws_client_event_1_to_1_payload_1kb",
                    payload_bytes: 1024,
                    messages: 100_000,
                },
                PayloadScenario {
                    name: "ws_client_event_1_to_1_payload_10kb",
                    payload_bytes: 10 * 1024,
                    messages: 50_000,
                },
                PayloadScenario {
                    name: "ws_client_event_1_to_1_payload_100kb",
                    payload_bytes: 100 * 1024,
                    messages: 5_000,
                },
            ]
        );
    }

    #[test]
    fn client_event_payload_has_requested_payload_bytes() {
        let payload = payload_string(1024);
        let message =
            client_event_message("client-bench-event", "payload-channel", 7, 1234, &payload);
        let value: serde_json::Value = serde_json::from_str(&message).unwrap();

        assert_eq!(value["event"], "client-bench-event");
        assert_eq!(value["channel"], "payload-channel");
        assert_eq!(value["data"]["sequence"], 7);
        assert_eq!(value["data"]["sent_at_us"], 1234);
        assert_eq!(value["data"]["payload"].as_str().unwrap().len(), 1024);
    }
}
