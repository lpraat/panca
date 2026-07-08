use std::sync::Arc;

use std::sync::Mutex;

use crate::metrics::interface::{GlobalMetricExt, LocalMetricExt};
use crate::metrics::utils::running_mean;
use json::JsonValue;

pub struct GlobalLlamaCppMetrics {
    locals: Vec<Arc<Mutex<Llamacpp>>>,
}

impl GlobalLlamaCppMetrics {
    pub fn new(n_threads: usize) -> Self {
        Self {
            locals: (0..n_threads)
                .map(|_| Arc::new(Mutex::new(Llamacpp::default())))
                .collect(),
        }
    }
}

impl GlobalMetricExt for GlobalLlamaCppMetrics {
    fn name(&self) -> &str {
        "llamacpp"
    }

    fn display(&self) {
        let mut g_mean_prompt_tokens_per_second: f64 = 0.0;
        let mut g_mean_completion_tokens_per_second: f64 = 0.0;
        let mut g_n_successes: u64 = 0;
        let mut g_n_prompt_tokens: u64 = 0;
        let mut g_n_completion_tokens: u64 = 0;
        let mut g_n_total_tokens: u64 = 0;

        for (thread_idx, local_mutex) in self.locals.iter().enumerate() {
            {
                let local = local_mutex.lock().unwrap();
                if local.n_successes > 0 {
                    self.display_local_stat(
                        thread_idx,
                        &format!(
                            "Mean Prompt Tok/s: {:.04}",
                            local.mean_prompt_tokens_per_second
                        ),
                    );
                    self.display_local_stat(
                        thread_idx,
                        &format!(
                            "Mean Completion Tok/s: {:.04}",
                            local.mean_completion_tokens_per_second
                        ),
                    );
                    self.display_local_stat(
                        thread_idx,
                        &format!(
                            "Prompt Tok/req: {:.04}",
                            local.n_prompt_tokens as f64 / local.n_successes as f64
                        ),
                    );
                    self.display_local_stat(
                        thread_idx,
                        &format!(
                            "Completion Tok/req: {:.04}",
                            local.n_completion_tokens as f64 / local.n_successes as f64
                        ),
                    );
                }
                self.display_local_stat(
                    thread_idx,
                    &format!("Prompt tokens: {}", local.n_prompt_tokens),
                );
                self.display_local_stat(
                    thread_idx,
                    &format!("Completion tokens: {}", local.n_completion_tokens),
                );
                self.display_local_stat(
                    thread_idx,
                    &format!("Total tokens: {}", local.n_total_tokens),
                );
                g_n_successes += local.n_successes;
                g_n_prompt_tokens += local.n_prompt_tokens;
                g_n_completion_tokens += local.n_completion_tokens;
                g_n_total_tokens += local.n_total_tokens;
                g_mean_prompt_tokens_per_second +=
                    local.mean_prompt_tokens_per_second * local.n_successes as f64;
                g_mean_completion_tokens_per_second +=
                    local.mean_completion_tokens_per_second * local.n_successes as f64;
            }
        }
        if g_n_successes > 0 {
            g_mean_prompt_tokens_per_second /= g_n_successes as f64;
            g_mean_completion_tokens_per_second /= g_n_successes as f64;
        }

        self.display_global_stat(&format!(
            "Mean Prompt Tok/s: {:.04}",
            g_mean_prompt_tokens_per_second * self.locals.len() as f64
        ));
        self.display_global_stat(&format!(
            "Mean Completion Tok/s: {:.04}",
            g_mean_completion_tokens_per_second * self.locals.len() as f64
        ));
        if g_n_successes > 0 {
            self.display_global_stat(&format!(
                "Prompt Tok/req: {:.04}",
                g_n_prompt_tokens as f64 / g_n_successes as f64
            ));
            self.display_global_stat(&format!(
                "Completion Tok/req: {:.04}",
                g_n_completion_tokens as f64 / g_n_successes as f64
            ));
        }
        self.display_global_stat(&format!("Prompt tokens: {}", g_n_prompt_tokens));
        self.display_global_stat(&format!("Completion tokens: {}", g_n_completion_tokens));
        self.display_global_stat(&format!("Total tokens: {}", g_n_total_tokens));
    }

    fn create_local_metric(&self, thread_idx: usize) -> Box<dyn LocalMetricExt> {
        Box::new(LocalLlamacppMetrics {
            state: Llamacpp::default(),
            global_slot: Arc::clone(&self.locals[thread_idx]),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Llamacpp {
    n_successes: u64,
    n_total_tokens: u64,
    n_completion_tokens: u64,
    n_prompt_tokens: u64,
    mean_completion_tokens_per_second: f64,
    mean_prompt_tokens_per_second: f64,
}

struct LocalLlamacppMetrics {
    state: Llamacpp,
    global_slot: Arc<Mutex<Llamacpp>>,
}

impl LocalMetricExt for LocalLlamacppMetrics {
    fn on_success_response(&mut self, json_response: &Option<JsonValue>, _: f64) {
        if json_response.is_none() {
            return;
        }
        let usage = &json_response.as_ref().unwrap()["usage"];
        let prompt_tokens = usage["prompt_tokens"].as_u64().unwrap();
        let completion_tokens = usage["completion_tokens"].as_u64().unwrap();

        let timings = &json_response.as_ref().unwrap()["timings"];
        self.state.mean_prompt_tokens_per_second = running_mean(
            self.state.mean_prompt_tokens_per_second,
            self.state.n_successes,
            timings["prompt_per_second"].as_f64().unwrap(),
        );
        self.state.mean_completion_tokens_per_second = running_mean(
            self.state.mean_completion_tokens_per_second,
            self.state.n_successes,
            timings["predicted_per_second"].as_f64().unwrap(),
        );
        self.state.n_successes += 1;
        self.state.n_total_tokens += prompt_tokens + completion_tokens;
        self.state.n_prompt_tokens += prompt_tokens;
        self.state.n_completion_tokens += completion_tokens;
    }

    fn on_error_response(&mut self) {}

    fn on_snapshot(&mut self, _: std::time::Duration) {}

    fn update_global_slot(&self) {
        *self.global_slot.lock().unwrap() = self.state;
    }
}
