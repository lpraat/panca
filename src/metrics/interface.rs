use std::time::Duration;

use json::JsonValue;

pub trait GlobalMetricExt: Send + Sync {
    fn name(&self) -> &str;
    fn display(&self);
    fn create_local_metric(&self, thread_idx: usize) -> Box<dyn LocalMetricExt>;
    fn display_local_stat(&self, thread_idx: usize, msg: &str) {
        println!("[{}][thread={}] {}", self.name(), thread_idx, msg);
    }
    fn display_global_stat(&self, msg: &str) {
        println!("[{}][global] {}", self.name(), msg);
    }
}

pub trait LocalMetricExt {
    fn on_success_response(&mut self, json_response: &Option<JsonValue>, response_time: f64);
    fn on_error_response(&mut self);
    fn on_snapshot(&mut self, interval: Duration);
    fn update_global_slot(&self);
}
