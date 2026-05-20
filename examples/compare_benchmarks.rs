use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::process;

#[derive(Debug, Deserialize)]
struct LatencyStats {
    p95_us: u64,
    #[serde(default)]
    under_1ms_count: u64,
    #[serde(default)]
    under_1ms_rate: f64,
}

#[derive(Debug, Default, Deserialize)]
struct ResourceStats {
    #[serde(default)]
    process_cpu_avg_percent: f64,
    #[serde(default)]
    process_cpu_peak_percent: f64,
    #[serde(default)]
    process_memory_peak_mb: f64,
}

#[derive(Debug, Deserialize)]
struct ScenarioReport {
    scenario: String,
    throughput_per_second: f64,
    delivery_rate: f64,
    latency: LatencyStats,
    #[serde(default)]
    resources: ResourceStats,
}

#[derive(Debug)]
struct Options {
    baseline_path: String,
    current_path: String,
    threshold_percent: f64,
    fail_on_regression: bool,
}

fn main() {
    let options = parse_args().unwrap_or_else(|error| {
        eprintln!("{}", error);
        print_usage();
        process::exit(2);
    });

    let baseline = read_report(&options.baseline_path);
    let current = read_report(&options.current_path);
    let threshold = options.threshold_percent / 100.0;
    let mut regressions = 0;

    println!(
        "Comparing benchmark reports with {:.2}% regression threshold\n",
        options.threshold_percent
    );
    println!(
        "{:<36} {:>14} {:>10} {:>10} {:>10} {:>10} {:>9} {:>10} {:>9} {:>9} {:>8}  status",
        "scenario",
        "throughput/s",
        "delta",
        "p95_us",
        "delta",
        "delivery",
        "<1ms",
        "<1ms%",
        "rss_mb",
        "cpu_avg",
        "cpu_pk"
    );
    println!("{}", "-".repeat(168));

    let scenarios = baseline
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    if scenarios.is_empty() {
        eprintln!("No scenarios found in benchmark reports.");
        process::exit(2);
    }

    for scenario in scenarios {
        let baseline_report = baseline.get(&scenario);
        let current_report = current.get(&scenario);

        match (baseline_report, current_report) {
            (Some(left), Some(right)) => {
                let throughput_delta =
                    percent_delta(right.throughput_per_second, left.throughput_per_second);
                let p95_delta =
                    percent_delta(right.latency.p95_us as f64, left.latency.p95_us as f64);
                let throughput_regressed =
                    right.throughput_per_second < left.throughput_per_second * (1.0 - threshold);
                let latency_regressed = left.latency.p95_us > 0
                    && right.latency.p95_us as f64 > left.latency.p95_us as f64 * (1.0 + threshold);
                let delivery_regressed = right.delivery_rate + f64::EPSILON < left.delivery_rate;
                let status = if throughput_regressed || latency_regressed || delivery_regressed {
                    regressions += 1;
                    "regressed"
                } else {
                    "ok"
                };

                println!(
                    "{:<36} {:>14.2} {:>+9.2}% {:>10} {:>+9.2}% {:>9.4} {:>9} {:>9.2}% {:>9.1} {:>8.1}% {:>7.1}%  {}",
                    scenario,
                    right.throughput_per_second,
                    throughput_delta,
                    right.latency.p95_us,
                    p95_delta,
                    right.delivery_rate,
                    right.latency.under_1ms_count,
                    right.latency.under_1ms_rate * 100.0,
                    right.resources.process_memory_peak_mb,
                    right.resources.process_cpu_avg_percent,
                    right.resources.process_cpu_peak_percent,
                    status
                );
            }
            (Some(_), None) => {
                regressions += 1;
                println!(
                    "{:<36} {:>14} {:>10} {:>10} {:>10} {:>10} {:>9} {:>10} {:>9} {:>9} {:>8}  missing",
                    scenario, "-", "-", "-", "-", "-", "-", "-", "-", "-", "-"
                );
            }
            (None, Some(right)) => {
                println!(
                    "{:<36} {:>14.2} {:>10} {:>10} {:>10} {:>9.4} {:>9} {:>9.2}% {:>9.1} {:>8.1}% {:>7.1}%  new",
                    scenario,
                    right.throughput_per_second,
                    "-",
                    right.latency.p95_us,
                    "-",
                    right.delivery_rate,
                    right.latency.under_1ms_count,
                    right.latency.under_1ms_rate * 100.0,
                    right.resources.process_memory_peak_mb,
                    right.resources.process_cpu_avg_percent,
                    right.resources.process_cpu_peak_percent
                );
            }
            (None, None) => unreachable!(),
        }
    }

    println!("\nRegressions: {}", regressions);

    if regressions > 0 && options.fail_on_regression {
        process::exit(1);
    }
}

fn parse_args() -> Result<Options, String> {
    let mut positional = Vec::new();
    let mut threshold_percent = 10.0;
    let mut fail_on_regression = false;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fail" => fail_on_regression = true,
            "--threshold-percent" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--threshold-percent requires a value".to_string())?;
                threshold_percent = value
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid threshold percent: {}", value))?;
                if !(0.0..100.0).contains(&threshold_percent) {
                    return Err("--threshold-percent must be between 0 and 100".to_string());
                }
            }
            "--help" | "-h" => return Err(String::new()),
            value => positional.push(value.to_string()),
        }
    }

    if positional.len() != 2 {
        return Err("Expected baseline and current report paths.".to_string());
    }

    Ok(Options {
        baseline_path: positional.remove(0),
        current_path: positional.remove(0),
        threshold_percent,
        fail_on_regression,
    })
}

fn read_report(path: &str) -> BTreeMap<String, ScenarioReport> {
    let bytes = fs::read(path).unwrap_or_else(|error| {
        eprintln!("Failed to read {}: {}", path, error);
        process::exit(2);
    });
    let reports = serde_json::from_slice::<Vec<ScenarioReport>>(&bytes).unwrap_or_else(|error| {
        eprintln!("Failed to parse {}: {}", path, error);
        process::exit(2);
    });

    reports
        .into_iter()
        .map(|report| (report.scenario.clone(), report))
        .collect()
}

fn percent_delta(current: f64, baseline: f64) -> f64 {
    if baseline == 0.0 {
        if current == 0.0 { 0.0 } else { f64::INFINITY }
    } else {
        ((current - baseline) / baseline) * 100.0
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --example compare_benchmarks -- <baseline.json> <current.json> [--threshold-percent 10] [--fail]"
    );
}
