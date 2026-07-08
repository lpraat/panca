use num::{Float, Unsigned, cast::AsPrimitive};

/// Computes the new running mean when a new sample is added.
pub fn running_mean<T: Float + 'static, N: Unsigned + AsPrimitive<T>>(
    mean: T,
    n: N,
    new_value: T,
) -> T {
    let n_t: T = n.as_();
    (mean * n_t) / (n_t + T::one()) + new_value / (n_t + T::one())
}

/// Updates a running mean in-place and increments the sample counter.
pub fn update_running_mean<T: Float + 'static, N: Unsigned + AsPrimitive<T>>(
    mean: &mut T,
    n: &mut N,
    new_value: T,
) {
    let n_t: T = n.as_();
    let new_running_mean = (*mean * n_t) / (n_t + T::one()) + new_value / (n_t + T::one());
    *mean = new_running_mean;
    *n = *n + N::one();
}

#[cfg(test)]
mod tests {
    use crate::metrics::utils::{running_mean, update_running_mean};

    #[test]
    fn test_running_mean() {
        let mut mean = 0_f64;
        let n = 0_u64;

        mean = running_mean(mean, n, 10_f64);
        assert_eq!(mean, 10.0);

        mean = running_mean(mean, n + 1, 5_f64);
        assert_eq!(mean, 7.5);
    }

    #[test]
    fn test_update_running_mean() {
        let mut mean = 0_f64;
        let mut n = 0_u64;

        update_running_mean(&mut mean, &mut n, 10_f64);
        assert_eq!(mean, 10.0);
        assert_eq!(n, 1);

        update_running_mean(&mut mean, &mut n, 5_f64);
        assert_eq!(mean, 7.5);
        assert_eq!(n, 2);
    }
}
