use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use json::JsonValue;

use crate::metrics::{
    interface::{GlobalMetricExt, LocalMetricExt},
    utils::RunningStats,
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
        let mut g_requests_per_second = RunningStats::new();
        let mut g_response_time = RunningStats::new();

        for (thread_idx, local_mutex) in self.locals.iter().enumerate() {
            {
                let local = local_mutex.lock().unwrap();
                self.display_local_stat(
                    thread_idx,
                    &local.requests_per_second.display_str("Req/s", 1.0, 4),
                );
                self.display_local_stat(
                    thread_idx,
                    &local.respone_time.display_str("Response time (ms)", 1e3, 4),
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
                g_n_successes += local.n_successes;
                g_n_failures += local.n_failures;
                g_requests_per_second.merge_with(&local.requests_per_second);
                g_response_time.merge_with(&local.respone_time);
            }
        }

        self.display_global_stat(&g_requests_per_second.display_str_global_sum(
            "Req/s",
            self.locals.len(),
            1.0,
            4,
        ));
        self.display_global_stat(&g_response_time.display_str("Response time (ms)", 1e3, 4));
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
    requests_per_second: RunningStats<f64, u64>,
    respone_time: RunningStats<f64, u64>,
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
        self.state.n_requests += 1;
        self.state.n_successes += 1;
        self.state.respone_time.add_sample(response_time);
    }

    fn on_error_response(&mut self) {
        self.state.n_failures += 1;
    }

    fn on_snapshot(&mut self, interval: Duration) {
        let requests_per_second = self.requests_per_second(interval);
        self.state
            .requests_per_second
            .add_sample(requests_per_second);
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
