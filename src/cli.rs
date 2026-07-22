use std::{
    sync::{Arc, atomic::AtomicUsize},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use futures::{StreamExt, future::Either, stream};
use reqwest::Client;
use tokio::runtime::Builder;

use crate::metrics::{
    basic::GlobalBasicMetrics, interface::GlobalMetricExt, llm::llamacpp::GlobalLlamaCppMetrics,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum HttpMethod {
    Get,
    Post,
}

// TODO:
// - header
// - std, confidence interval
// - log bench data to file for offline analysis

/// Server benchmarking tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// URL of the endpoint to benchmark.
    pub url: String,

    /// Number of worker threads (one more "display thread" is spawned for metrics).
    #[arg(short, long, default_value_t = 1)]
    pub threads: usize,

    /// Number of concurrent requests per thread.
    #[arg(short, long, default_value_t = 10)]
    pub concurrent_requests: u64,

    /// Total number of requests sent by each thread. If not specified, requests are sent indefinitely.
    #[arg(short, long)]
    pub requests: Option<u64>,

    /// HTTP method.
    #[arg(short, long, value_enum, default_value_t = HttpMethod::Get)]
    pub method: HttpMethod,

    /// JSON payload to send as the request body. Sets 'Content-Type: application/json' header.
    #[arg(long)]
    pub json: Option<String>,

    /// How often metric are updated (in seconds).
    #[arg(long, default_value_t = 1.0)]
    pub metrics_interval_seconds: f64,

    /// How often metrics are displayed to stdout (in seconds).
    #[arg(long, default_value_t = 1.0)]
    pub display_interval_seconds: f64,

    /// Metrics plugins (e.g., basic,llamacpp).
    #[arg(long, value_delimiter = ',', default_values=["basic"])]
    pub plugins: Vec<String>,
}

pub struct Panca;

impl Panca {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let cli = Arc::new(Cli::parse());
        let mut worker_threads = vec![];

        let mut global_metrics: Vec<Box<dyn GlobalMetricExt>> = vec![];

        for plugin in &cli.plugins {
            match plugin.to_ascii_lowercase().as_str() {
                "basic" => global_metrics.push(Box::new(GlobalBasicMetrics::new(cli.threads))),
                "llamacpp" => {
                    global_metrics.push(Box::new(GlobalLlamaCppMetrics::new(cli.threads)))
                }
                other => return Err(anyhow!("Plugin {} not available.", other)),
            }
        }
        let global_metrics = Arc::new(global_metrics);
        let done_counter = Arc::new(AtomicUsize::new(0));

        // Display thread
        let global_metrics_ref = global_metrics.clone();
        let cli_ref = Arc::clone(&cli);
        let done_counter_ref = Arc::clone(&done_counter);
        let display_thread = thread::spawn(move || {
            let mut display_snapshot: u64 = 0;
            let fn_display_snapshot = |display_snapshot: &mut u64| {
                *display_snapshot += 1;
                println!("----Snapshot {display_snapshot}----");
                for global_metric in global_metrics_ref.iter() {
                    global_metric.display();
                }
                println!("----Snapshot {display_snapshot}----");
            };
            loop {
                fn_display_snapshot(&mut display_snapshot);
                if done_counter_ref.load(std::sync::atomic::Ordering::Acquire) == cli_ref.threads {
                    // Final display, at this point we are sure that all metrics have been updated by worker threads
                    fn_display_snapshot(&mut display_snapshot);
                    break;
                }
                sleep(Duration::from_secs_f64(cli_ref.display_interval_seconds));
            }
        });

        // Worker threads
        for thread_id in 0..cli.threads {
            let global_metrics_ref = global_metrics.clone();

            let cli_ref = Arc::clone(&cli);
            let done_counter_ref = Arc::clone(&done_counter);
            let worker_thread = thread::spawn(move || {
                // Each thread can just use its current thread for async runtime
                let runtime = Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to build with current thread scheduler.");

                let mut local_metrics = global_metrics_ref
                    .iter()
                    .map(|gm| gm.create_local_metric(thread_id))
                    .collect::<Vec<_>>();
                let mut last_snapshot = Instant::now();
                let mut requests_sent = 0;

                runtime.block_on(async {
                    let client = Client::new();

                    let stream_iter = if let Some(requests) = cli_ref.requests {
                        Either::Left(stream::iter(0..requests))
                    } else {
                        Either::Right(stream::iter(0..))
                    };

                    let mut response = stream_iter
                        .map(async |_| {
                            let mut request = match cli_ref.method {
                                HttpMethod::Get => client.get(&cli_ref.url),
                                HttpMethod::Post => client.post(&cli_ref.url),
                            };

                            if let Some(json) = &cli_ref.json {
                                request = request
                                    .header("Content-Type", "application/json")
                                    .body(json.clone());
                            }

                            let request_time = Instant::now();

                            let response = request.send().await;
                            let response_time = Instant::now();

                            match response {
                                Ok(response) if !response.status().is_server_error() => {
                                    Ok((response_time - request_time, response))
                                }
                                Ok(response) if response.status().is_server_error() => {
                                    Err(anyhow!("Server Error."))
                                }
                                Err(e) => Err(anyhow!(e)),
                                _ => unreachable!(),
                            }
                        })
                        // Handle at most these number of requests concurrently (on the current thread)
                        .buffer_unordered(cli_ref.concurrent_requests as usize);

                    while let Some(bytes) = response.next().await {
                        match bytes {
                            Ok((response_time, response)) => {
                                let response_bytes = response.bytes().await.ok();
                                let json_response = response_bytes
                                    .as_ref()
                                    .and_then(|bytes| std::str::from_utf8(bytes).ok())
                                    .and_then(|str| json::parse(str).ok());
                                for metric in local_metrics.iter_mut() {
                                    metric.on_success_response(
                                        &json_response,
                                        response_time.as_secs_f64(),
                                    );
                                }
                            }
                            Err(_) => {
                                for metric in local_metrics.iter_mut() {
                                    metric.on_error_response();
                                }
                            }
                        }

                        requests_sent += 1;
                        let is_thread_done = cli_ref
                            .requests
                            .as_ref()
                            .is_some_and(|cli_requests| *cli_requests == requests_sent);
                        let curr_snapshot = Instant::now();

                        if (curr_snapshot - last_snapshot
                            > Duration::from_secs_f64(cli_ref.metrics_interval_seconds))
                            || is_thread_done
                        {
                            for metric in local_metrics.iter_mut() {
                                metric.on_snapshot(curr_snapshot - last_snapshot);
                            }

                            for local_metric in local_metrics.iter() {
                                local_metric.update_global_slot();
                            }

                            if is_thread_done {
                                // Note: we signal done after updating global metrics
                                // And make sure that updated metrics are visible to the display thread before the final display (Release + Acquire)
                                done_counter_ref.fetch_add(1, std::sync::atomic::Ordering::Release);
                            }

                            last_snapshot = Instant::now();
                        }
                    }
                });
            });
            worker_threads.push(worker_thread);
        }

        for t in worker_threads {
            t.join().unwrap();
        }

        if cli.requests.is_some() {
            display_thread.join().unwrap();
        }

        Ok(())
    }
}

impl Default for Panca {
    fn default() -> Self {
        Self::new()
    }
}
