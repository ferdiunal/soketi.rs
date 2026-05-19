use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use soketi_rs::app::App;
use soketi_rs::auth::{generate_api_auth_signature, generate_md5_hash_bytes};
use soketi_rs::config::{
    AdapterDriver, AppManagerDriver, CacheDriver, QueueDriver, RateLimiterDriver, ServerConfig,
};
use soketi_rs::server::Server;
use soketi_rs::state::AppState;
use std::env;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const APP_ID: &str = "perf_test_app";
const APP_KEY: &str = "perf_test_key";
const APP_SECRET: &str = "perf_test_secret";
const CONNECT_BATCH: usize = 50;
const SUBSCRIBE_BATCH: usize = 50;
const WS_MESSAGES: usize = 1_000;
const FANOUT_RECEIVERS: usize = 100;
const CHURN_CONNECTIONS: usize = 1_000;
const HTTP_MESSAGES: usize = 500;
const DEFAULT_REPORT_PATH: &str = "target/benchmarks/local_e2e.json";
const REPORT_PATH_ENV: &str = "SOKETI_BENCH_REPORT";

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsWrite = SplitSink<WsStream, Message>;
type WsRead = SplitStream<WsStream>;

#[derive(Clone)]
struct BenchServer {
    port: u16,
    state: Arc<AppState>,
}

impl BenchServer {
    fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/app/{}", self.port, APP_KEY)
    }

    fn http_url(&self, path: &str, body: &[u8]) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_secs();
        let body_md5 = generate_md5_hash_bytes(body);
        let query = format!(
            "auth_key={}&auth_timestamp={}&auth_version=1.0&body_md5={}",
            APP_KEY, timestamp, body_md5
        );
        let signature = generate_api_auth_signature(APP_SECRET, "POST", path, &query);

        format!(
            "http://127.0.0.1:{}{}?{}&auth_signature={}",
            self.port, path, query, signature
        )
    }
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

fn benchmark_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(10))
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
    let state = server
        .state()
        .expect("server state missing after initialize");

    tokio::spawn(async move {
        if let Err(error) = server.start().await {
            eprintln!("benchmark server failed: {}", error);
        }
    });

    wait_for_server(port).await;
    BenchServer { port, state }
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
    let (stream, _) = connect_async(ws_url)
        .await
        .expect("failed to connect websocket client");
    let (write, mut read) = stream.split();
    let message = next_json_message(&mut read).await;
    assert_eq!(message["event"], "pusher:connection_established");

    (write, read)
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
    let subscribe = json!({
        "event": "pusher:subscribe",
        "data": {
            "channel": channel
        }
    });

    write
        .send(Message::Text(subscribe.to_string().into()))
        .await
        .expect("failed to send subscribe message");

    let response = next_json_message(read).await;
    assert_eq!(response["event"], "pusher_internal:subscription_succeeded");
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
    let deadline = Instant::now() + Duration::from_secs(30);

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

fn reset_report() {
    let path = report_path();
    if path.exists() {
        fs::remove_file(path).expect("failed to reset benchmark report");
    }
}

fn write_report(report: &ScenarioReport) {
    let path = report_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create benchmark report directory");
    }

    let mut reports: Vec<ScenarioReport> = fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();

    reports.retain(|existing| existing.scenario != report.scenario);
    reports.push(report.clone());
    reports.sort_by(|left, right| left.scenario.cmp(&right.scenario));

    let encoded = serde_json::to_vec_pretty(&reports).expect("failed to encode benchmark report");
    fs::write(path, encoded).expect("failed to write benchmark report");
}

async fn run_ws_connect_handshake(ws_url: String) -> ScenarioReport {
    let started_at = Instant::now();
    let mut delivered = 0;

    for _ in 0..CONNECT_BATCH {
        let (mut write, _read) = connect_client(&ws_url).await;
        delivered += 1;
        let _ = write.close().await;
    }

    let histogram = Histogram::<u64>::new(3).expect("histogram init failed");
    let report = scenario_report(
        "ws_connect_handshake",
        CONNECT_BATCH,
        delivered,
        started_at,
        &histogram,
    );
    write_report(&report);
    report
}

async fn run_ws_subscribe_public(ws_url: String) -> ScenarioReport {
    let started_at = Instant::now();
    let mut delivered = 0;

    for index in 0..SUBSCRIBE_BATCH {
        let (mut write, mut read) = connect_client(&ws_url).await;
        subscribe_public(&mut write, &mut read, &format!("bench-subscribe-{}", index)).await;
        delivered += 1;
        let _ = write.close().await;
    }

    let histogram = Histogram::<u64>::new(3).expect("histogram init failed");
    let report = scenario_report(
        "ws_subscribe_public",
        SUBSCRIBE_BATCH,
        delivered,
        started_at,
        &histogram,
    );
    write_report(&report);
    report
}

async fn run_ws_client_event(
    ws_url: String,
    scenario: &'static str,
    receivers: usize,
    messages: usize,
) -> ScenarioReport {
    let channel = format!("{}-channel", scenario);
    let event = "client-bench-event";
    let started_at = Instant::now();

    let (mut sender_write, mut sender_read) = connect_client(&ws_url).await;
    subscribe_public(&mut sender_write, &mut sender_read, &channel).await;

    let mut receiver_writes = Vec::with_capacity(receivers);
    let mut receiver_tasks = Vec::with_capacity(receivers);
    for _ in 0..receivers {
        let (mut write, mut read) = connect_client(&ws_url).await;
        subscribe_public(&mut write, &mut read, &channel).await;
        receiver_writes.push(write);
        receiver_tasks.push(tokio::spawn(receive_expected(
            read, event, messages, started_at,
        )));
    }

    for sequence in 0..messages {
        let sent_at_us = started_at.elapsed().as_micros() as u64;
        let message = json!({
            "event": event,
            "channel": channel,
            "data": {
                "sequence": sequence,
                "sent_at_us": sent_at_us,
                "payload": "x".repeat(100)
            }
        });

        sender_write
            .send(Message::Text(message.to_string().into()))
            .await
            .expect("failed to send client event");
    }

    let mut delivered = 0;
    let mut histogram =
        Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).expect("histogram init failed");
    for task in receiver_tasks {
        let (count, partial) = task.await.expect("receiver task failed");
        delivered += count;
        histogram.add(&partial).expect("failed to merge histograms");
    }

    let _ = sender_write.close().await;
    for mut write in receiver_writes {
        let _ = write.close().await;
    }

    let attempted = receivers * messages;
    let report = scenario_report(scenario, attempted, delivered, started_at, &histogram);
    write_report(&report);
    report
}

async fn run_connection_churn(server: BenchServer) -> ScenarioReport {
    let ws_url = server.ws_url();
    let started_at = Instant::now();
    let mut delivered = 0;

    for index in 0..CHURN_CONNECTIONS {
        let (mut write, mut read) = connect_client(&ws_url).await;
        subscribe_public(
            &mut write,
            &mut read,
            &format!("bench-churn-{}", index % 10),
        )
        .await;
        delivered += 1;
        let _ = write.close().await;
    }

    sleep(Duration::from_millis(500)).await;
    assert_eq!(
        server
            .state
            .adapter
            .get_sockets_count(APP_ID)
            .await
            .unwrap(),
        0
    );
    assert!(
        server
            .state
            .adapter
            .get_channels(APP_ID)
            .await
            .unwrap()
            .is_empty()
    );

    let histogram = Histogram::<u64>::new(3).expect("histogram init failed");
    let report = scenario_report(
        "connection_churn",
        CHURN_CONNECTIONS,
        delivered,
        started_at,
        &histogram,
    );
    write_report(&report);
    report
}

async fn run_http_publish_event(server: BenchServer) -> ScenarioReport {
    let ws_url = server.ws_url();
    let channel = "bench-http-publish";
    let event = "http-bench-event";
    let path = format!("/apps/{}/events", APP_ID);
    let started_at = Instant::now();
    let client = reqwest::Client::new();

    let (mut receiver_write, mut receiver_read) = connect_client(&ws_url).await;
    subscribe_public(&mut receiver_write, &mut receiver_read, channel).await;
    let receive_task = tokio::spawn(receive_expected(
        receiver_read,
        event,
        HTTP_MESSAGES,
        started_at,
    ));

    for sequence in 0..HTTP_MESSAGES {
        let sent_at_us = started_at.elapsed().as_micros() as u64;
        let body = json!({
            "name": event,
            "data": {
                "sequence": sequence,
                "sent_at_us": sent_at_us,
                "payload": "x".repeat(100)
            },
            "channel": channel
        })
        .to_string();
        let url = server.http_url(&path, body.as_bytes());

        let response = client
            .post(url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .expect("http publish request failed");
        assert!(
            response.status().is_success(),
            "http publish failed with {}",
            response.status()
        );
    }

    let (delivered, histogram) = receive_task.await.expect("receiver task failed");
    let _ = receiver_write.close().await;

    let report = scenario_report(
        "http_publish_event",
        HTTP_MESSAGES,
        delivered,
        started_at,
        &histogram,
    );
    write_report(&report);
    report
}

fn local_e2e_benchmarks(c: &mut Criterion) {
    let runtime = Runtime::new().expect("failed to create tokio runtime");
    reset_report();

    let connect_server = runtime.block_on(start_bench_server());
    let subscribe_server = runtime.block_on(start_bench_server());
    let direct_server = runtime.block_on(start_bench_server());
    let fanout_server = runtime.block_on(start_bench_server());
    let churn_server = runtime.block_on(start_bench_server());
    let http_server = runtime.block_on(start_bench_server());

    let mut group = c.benchmark_group("local_e2e");

    group.throughput(Throughput::Elements(CONNECT_BATCH as u64));
    group.bench_with_input(
        BenchmarkId::new("ws_connect_handshake", CONNECT_BATCH),
        &connect_server.ws_url(),
        |bench, ws_url| {
            bench.to_async(&runtime).iter(|| {
                let ws_url = ws_url.clone();
                async move { run_ws_connect_handshake(ws_url).await }
            });
        },
    );

    group.throughput(Throughput::Elements(SUBSCRIBE_BATCH as u64));
    group.bench_with_input(
        BenchmarkId::new("ws_subscribe_public", SUBSCRIBE_BATCH),
        &subscribe_server.ws_url(),
        |bench, ws_url| {
            bench.to_async(&runtime).iter(|| {
                let ws_url = ws_url.clone();
                async move { run_ws_subscribe_public(ws_url).await }
            });
        },
    );

    group.throughput(Throughput::Elements(WS_MESSAGES as u64));
    group.bench_function("ws_client_event_1_to_1", |bench| {
        let ws_url = direct_server.ws_url();
        bench.to_async(&runtime).iter(|| {
            let ws_url = ws_url.clone();
            async move { run_ws_client_event(ws_url, "ws_client_event_1_to_1", 1, WS_MESSAGES).await }
        });
    });

    group.throughput(Throughput::Elements(
        (WS_MESSAGES * FANOUT_RECEIVERS) as u64,
    ));
    group.bench_function("ws_client_event_fanout_1_to_100", |bench| {
        let ws_url = fanout_server.ws_url();
        bench.to_async(&runtime).iter(|| {
            let ws_url = ws_url.clone();
            async move {
                run_ws_client_event(
                    ws_url,
                    "ws_client_event_fanout_1_to_100",
                    FANOUT_RECEIVERS,
                    WS_MESSAGES,
                )
                .await
            }
        });
    });

    group.throughput(Throughput::Elements(CHURN_CONNECTIONS as u64));
    group.bench_function("connection_churn", |bench| {
        let server = churn_server.clone();
        bench.to_async(&runtime).iter(|| {
            let server = server.clone();
            async move { run_connection_churn(server).await }
        });
    });

    group.throughput(Throughput::Elements(HTTP_MESSAGES as u64));
    group.bench_function("http_publish_event", |bench| {
        let server = http_server.clone();
        bench.to_async(&runtime).iter(|| {
            let server = server.clone();
            async move { run_http_publish_event(server).await }
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets = local_e2e_benchmarks
}
criterion_main!(benches);
