use std::sync::Arc;

use std::sync::Mutex;

use crate::metrics::interface::{GlobalMetricExt, LocalMetricExt};
use crate::metrics::utils::RunningStats;
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
        let mut g_prompt_tokens_per_second = RunningStats::new();
        let mut g_completion_tokens_per_second = RunningStats::new();
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
                        &local
                            .prompt_tokens_per_second
                            .display_str("Prompt Tok/s", 1.0, 4),
                    );
                    self.display_local_stat(
                        thread_idx,
                        &local
                            .completion_tokens_per_second
                            .display_str("Completion Tok/s", 1.0, 4),
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
                g_prompt_tokens_per_second.merge_with(&local.prompt_tokens_per_second);
                g_completion_tokens_per_second.merge_with(&local.completion_tokens_per_second);
            }
        }
        self.display_global_stat(&g_prompt_tokens_per_second.display_str_global_sum(
            "Prompt Tok/s",
            self.locals.len(),
            1.0,
            4,
        ));
        self.display_global_stat(&g_completion_tokens_per_second.display_str_global_sum(
            "Completion Tok/s",
            self.locals.len(),
            1.0,
            4,
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
    completion_tokens_per_second: RunningStats<f64, u64>,
    prompt_tokens_per_second: RunningStats<f64, u64>,
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

        self.state
            .prompt_tokens_per_second
            .add_sample(timings["prompt_per_second"].as_f64().unwrap());
        self.state
            .completion_tokens_per_second
            .add_sample(timings["predicted_per_second"].as_f64().unwrap());
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
