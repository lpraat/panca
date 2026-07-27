use std::fmt::Display;

use num::{Float, Unsigned, cast::AsPrimitive};

/// Container to compute running statistics
#[derive(Clone, Copy, Debug)]
pub struct RunningStats<T: Float + 'static, N: Unsigned + AsPrimitive<T>> {
    mean: T,
    n: N,
    m2: T,
    min: T,
    max: T,
}

impl<T: Float + 'static + Display, N: Unsigned + AsPrimitive<T> + PartialOrd> RunningStats<T, N> {
    pub fn new() -> Self {
        Self {
            mean: T::zero(),
            n: N::zero(),
            m2: T::zero(),
            max: T::min_value(),
            min: T::max_value(),
        }
    }

    pub fn add_sample(&mut self, new_value: T) {
        self.n = self.n + N::one();
        let delta = new_value - self.mean;
        self.mean = self.mean + delta / self.n.as_();
        self.m2 = self.m2 + delta * (new_value - self.mean);
        self.min = self.min.min(new_value);
        self.max = self.max.max(new_value);
    }

    pub fn mean(&self) -> T {
        self.mean
    }

    pub fn min(&self) -> T {
        self.min
    }

    pub fn max(&self) -> T {
        self.max
    }

    pub fn n_samples(&self) -> N {
        self.n
    }

    pub fn variance(&self) -> T {
        if self.n > N::one() {
            self.m2 / (self.n.as_() - T::one())
        } else {
            T::zero()
        }
    }

    pub fn stddev(&self) -> T {
        self.variance().sqrt()
    }

    pub fn ci_limit(&self, confidence_level: ConfidenceLevel) -> T {
        let z = T::from(confidence_level.zscore()).unwrap();
        z * self.stddev() / self.n.as_().sqrt()
    }

    /// Merges another [`RunningStats`] into this one in-place.
    ///
    /// This method can be used to aggregate local trhread metrics.
    /// See <https://en.wikipedia.org/wiki/Algorithms_for_calculating_variance#Parallel_algorithm>
    pub fn merge_with(&mut self, other: &RunningStats<T, N>) {
        let new_count = self.n_samples() + other.n_samples();
        let new_delta = other.mean() - self.mean();
        let new_mean = (self.n_samples().as_() * self.mean()
            + other.n_samples().as_() * other.mean())
            / new_count.as_();
        let new_m2 = self.m2
            + other.m2
            + new_delta.powi(2) * self.n_samples().as_() * other.n_samples().as_()
                / new_count.as_();
        self.n = new_count;
        self.mean = new_mean;
        self.m2 = new_m2;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn display_str(&self, name: &str, scale: T, precision: usize) -> String {
        format!(
            "{name}: {mean:.precision$} ± {limit:.precision$} [min={min:.precision$}, max={max:.precision$}]",
            name = name,
            mean = self.mean() * scale,
            precision = precision,
            limit = self.ci_limit(ConfidenceLevel::P95) * scale,
            max = self.max() * scale,
            min = self.min() * scale
        )
    }

    /// Builds a string to display global sum statistics across `n_threads`.
    ///
    /// Assumes threads are independent and identically distributed, scaling the pooled statistics (this object, pooled using the [`Self::merge_with`] method) up by the thread count.
    pub fn display_str_global_sum(
        &self,
        name: &str,
        n_threads: usize,
        scale: T,
        precision: usize,
    ) -> String {
        let n_threads = T::from(n_threads as f64).unwrap();
        format!(
            "{name}: {mean:.precision$} ± {limit:.precision$}",
            name = name,
            mean = self.mean() * scale * n_threads,
            precision = precision,
            limit = self.ci_limit(ConfidenceLevel::P95) * scale * n_threads,
        )
    }
}

impl<T: Float + 'static + Display, N: Unsigned + AsPrimitive<T> + PartialOrd> Default
    for RunningStats<T, N>
{
    fn default() -> Self {
        Self::new()
    }
}

pub enum ConfidenceLevel {
    P95,
    P99,
}

impl ConfidenceLevel {
    fn zscore(&self) -> f32 {
        match self {
            ConfidenceLevel::P95 => 1.96,
            ConfidenceLevel::P99 => 2.58,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::utils::RunningStats;

    #[test]
    fn test_running_stats() {
        let mut stats: RunningStats<f64, u64> = RunningStats::default();

        stats.add_sample(10_f64);
        assert_eq!(stats.mean(), 10.0);
        assert_eq!(stats.variance(), 0.0);

        stats.add_sample(5_f64);
        assert_eq!(stats.mean(), 7.5);
        assert_eq!(stats.variance(), 12.5);
        assert_eq!(stats.stddev(), 3.5355339059327378);
    }
}
