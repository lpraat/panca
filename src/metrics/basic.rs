use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use json::JsonValue;

use crate::metrics::{
    interface::{GlobalMetricExt, LocalMetricExt},
    utils::update_running_mean,
};

pub struct GlobalBasicMetrics {
    pub locals: Vec<Arc<Mutex<BasicMetricsState>>>,
}

impl GlobalBasicMetrics {
    pub fn new(n_threads: usize) -> Self {
        Self {
            locals: (0..n_threads)
                .map(|_| Arc::new(Mutex::new(BasicMetricsState::default())))
                .collect(),
        }
    }
}

impl GlobalMetricExt for GlobalBasicMetrics {
    fn name(&self) -> &str {
        "basic"
    }

    fn create_local_metric(&self, thread_idx: usize) -> Box<dyn LocalMetricExt> {
        Box::new(LocalBasicMetrics {
            state: BasicMetricsState::default(),
            global_slot: Arc::clone(&self.locals[thread_idx]),
        })
    }

    fn display(&self) {
        let mut g_n_successes: u64 = 0;
        let mut g_n_failures: u64 = 0;
        let mut g_mean_requests_per_second: f64 = 0.0;
        let mut g_mean_response_time: f64 = 0.0;
        let mut g_n_snapshots: u64 = 0;

        for (thread_idx, local_mutex) in self.locals.iter().enumerate() {
            {
                let local = local_mutex.lock().unwrap();
                self.display_local_stat(
                    thread_idx,
                    &format!("Mean Req/s: {:.04}", local.mean_requests_per_second),
                );
                self.display_local_stat(
                    thread_idx,
                    &format!(
                        "Mean Response time (ms): {:.04}",
                        local.mean_response_time * 1e3
                    ),
                );
                self.display_local_stat(
                    thread_idx,
                    &format!(
                        "Total requests sent: {}",
                        local.n_successes + local.n_failures
                    ),
                );
                self.display_local_stat(
                    thread_idx,
                    &format!(
                        "Errors: {}% ({}/{})",
                        local.err_pct(),
                        local.n_failures,
                        local.n_successes + local.n_failures
                    ),
                );
                g_mean_requests_per_second +=
                    local.mean_requests_per_second * local.n_snapshot as f64;
                g_n_successes += local.n_successes;
                g_n_failures += local.n_failures;
                g_n_snapshots += local.n_snapshot;
                g_mean_response_time += local.mean_response_time * local.n_successes as f64;
            }
        }
        g_mean_response_time /= g_n_successes as f64;
        g_mean_requests_per_second /= g_n_snapshots as f64;

        self.display_global_stat(&format!(
            "Mean Req/s: {:.04}",
            g_mean_requests_per_second * self.locals.len() as f64
        ));
        self.display_global_stat(&format!(
            "Mean Response Time (ms): {:.04}",
            g_mean_response_time * 1e3
        ));
        self.display_global_stat(&format!(
            "Total requests sent: {}",
            g_n_successes + g_n_failures
        ));
        self.display_global_stat(&format!(
            "Errors: {:.02}% ({}/{})",
            (g_n_failures as f64 / (g_n_successes as f64 + g_n_failures as f64)) * 100.0,
            g_n_failures,
            g_n_successes + g_n_failures
        ));
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BasicMetricsState {
    n_requests: u64,
    n_successes: u64,
    n_failures: u64,
    n_snapshot: u64,
    mean_requests_per_second: f64,
    mean_response_time: f64,
}

impl BasicMetricsState {
    pub fn err_pct(&self) -> f64 {
        (self.n_failures as f64 / (self.n_successes as f64 + self.n_failures as f64)) * 100.0
    }
}

#[derive(Debug)]
pub struct LocalBasicMetrics {
    state: BasicMetricsState,
    global_slot: Arc<Mutex<BasicMetricsState>>,
}

impl LocalMetricExt for LocalBasicMetrics {
    fn on_success_response(&mut self, _: &Option<JsonValue>, response_time: f64) {
        update_running_mean(
            &mut self.state.mean_response_time,
            &mut self.state.n_requests,
            response_time,
        );
        self.state.n_successes += 1;
    }

    fn on_error_response(&mut self) {
        self.state.n_failures += 1;
    }

    fn on_snapshot(&mut self, interval: Duration) {
        let requests_per_second = self.requests_per_second(interval);
        update_running_mean(
            &mut self.state.mean_requests_per_second,
            &mut self.state.n_snapshot,
            requests_per_second,
        );
        self.state.n_requests = 0;
    }

    fn update_global_slot(&self) {
        *self.global_slot.lock().unwrap() = self.state;
    }
}

impl LocalBasicMetrics {
    fn requests_per_second(&self, interval: Duration) -> f64 {
        self.state.n_requests as f64 / interval.as_secs_f64()
    }
}
