use serde::{Deserialize, Serialize};
use std::time::Duration;

fn default_max_retries() -> Option<u32> {
    None
}

fn default_backoff_base() -> f64 {
    1.0
}

fn default_backoff_max() -> f64 {
    300.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum RestartPolicy {
    Always {
        #[serde(default = "default_max_retries")]
        max_retries: Option<u32>,
        #[serde(default = "default_backoff_base")]
        backoff_base_secs: f64,
        #[serde(default = "default_backoff_max")]
        backoff_max_secs: f64,
    },
    OnFailure {
        #[serde(default = "default_max_retries")]
        max_retries: Option<u32>,
        #[serde(default = "default_backoff_base")]
        backoff_base_secs: f64,
        #[serde(default = "default_backoff_max")]
        backoff_max_secs: f64,
    },
    Never,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        RestartPolicy::Never
    }
}

pub fn compute_backoff(attempt: u32, base_secs: f64, max_secs: f64) -> Duration {
    let exp = base_secs * 2.0f64.powi(attempt as i32);
    let capped = exp.min(max_secs);
    // Add jitter: random between 0 and 10% of capped
    let jitter = rand::random::<f64>() * capped * 0.1;
    Duration::from_secs_f64(capped + jitter)
}

pub struct RestartEvaluator;

impl RestartEvaluator {
    pub fn should_restart(
        policy: &RestartPolicy,
        exit_code: Option<i32>,
        restart_count: u32,
    ) -> bool {
        match policy {
            RestartPolicy::Never => false,
            RestartPolicy::Always { max_retries, .. } => {
                max_retries.map_or(true, |max| restart_count < max)
            }
            RestartPolicy::OnFailure { max_retries, .. } => {
                let failed = exit_code.map_or(true, |c| c != 0);
                failed && max_retries.map_or(true, |max| restart_count < max)
            }
        }
    }

    pub fn backoff_duration(policy: &RestartPolicy, restart_count: u32) -> Duration {
        match policy {
            RestartPolicy::Always {
                backoff_base_secs,
                backoff_max_secs,
                ..
            }
            | RestartPolicy::OnFailure {
                backoff_base_secs,
                backoff_max_secs,
                ..
            } => compute_backoff(restart_count, *backoff_base_secs, *backoff_max_secs),
            RestartPolicy::Never => Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_policy_never_restarts() {
        let policy = RestartPolicy::Never;
        assert!(!RestartEvaluator::should_restart(&policy, Some(1), 0));
        assert!(!RestartEvaluator::should_restart(&policy, Some(0), 0));
        assert!(!RestartEvaluator::should_restart(&policy, None, 0));
    }

    #[test]
    fn always_policy_restarts_regardless_of_exit_code() {
        let policy = RestartPolicy::Always {
            max_retries: None,
            backoff_base_secs: 1.0,
            backoff_max_secs: 300.0,
        };
        assert!(RestartEvaluator::should_restart(&policy, Some(0), 0));
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 0));
        assert!(RestartEvaluator::should_restart(&policy, None, 0));
        assert!(RestartEvaluator::should_restart(&policy, Some(0), 100));
    }

    #[test]
    fn always_policy_respects_max_retries() {
        let policy = RestartPolicy::Always {
            max_retries: Some(3),
            backoff_base_secs: 1.0,
            backoff_max_secs: 300.0,
        };
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 0));
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 2));
        assert!(!RestartEvaluator::should_restart(&policy, Some(1), 3));
        assert!(!RestartEvaluator::should_restart(&policy, Some(1), 10));
    }

    #[test]
    fn on_failure_restarts_only_on_failure() {
        let policy = RestartPolicy::OnFailure {
            max_retries: None,
            backoff_base_secs: 1.0,
            backoff_max_secs: 300.0,
        };
        // exit_code 0 => success => no restart
        assert!(!RestartEvaluator::should_restart(&policy, Some(0), 0));
        // exit_code != 0 => failure => restart
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 0));
        assert!(RestartEvaluator::should_restart(&policy, Some(137), 0));
        // None exit_code (e.g., signal) => treat as failure
        assert!(RestartEvaluator::should_restart(&policy, None, 0));
    }

    #[test]
    fn on_failure_respects_max_retries() {
        let policy = RestartPolicy::OnFailure {
            max_retries: Some(2),
            backoff_base_secs: 1.0,
            backoff_max_secs: 300.0,
        };
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 0));
        assert!(RestartEvaluator::should_restart(&policy, Some(1), 1));
        assert!(!RestartEvaluator::should_restart(&policy, Some(1), 2));
    }

    #[test]
    fn backoff_exponential_growth() {
        let d0 = compute_backoff(0, 1.0, 300.0);
        let d1 = compute_backoff(1, 1.0, 300.0);
        let d2 = compute_backoff(2, 1.0, 300.0);

        // Base case: 1.0 * 2^0 = 1.0s (+ up to 10% jitter)
        assert!(d0.as_secs_f64() >= 1.0);
        assert!(d0.as_secs_f64() <= 1.1);

        // 1.0 * 2^1 = 2.0s (+ up to 10% jitter)
        assert!(d1.as_secs_f64() >= 2.0);
        assert!(d1.as_secs_f64() <= 2.2);

        // 1.0 * 2^2 = 4.0s (+ up to 10% jitter)
        assert!(d2.as_secs_f64() >= 4.0);
        assert!(d2.as_secs_f64() <= 4.4);
    }

    #[test]
    fn backoff_caps_at_max() {
        let d = compute_backoff(20, 1.0, 300.0);
        // 2^20 = 1048576 >> 300, so should be capped at 300 + up to 10% jitter
        assert!(d.as_secs_f64() >= 300.0);
        assert!(d.as_secs_f64() <= 330.0);
    }

    #[test]
    fn backoff_never_policy_returns_zero() {
        let policy = RestartPolicy::Never;
        let d = RestartEvaluator::backoff_duration(&policy, 5);
        assert_eq!(d, Duration::ZERO);
    }

    #[test]
    fn default_restart_policy_is_never() {
        match RestartPolicy::default() {
            RestartPolicy::Never => {}
            _ => panic!("Default should be Never"),
        }
    }
}
