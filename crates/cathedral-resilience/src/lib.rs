use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use async_trait::async_trait;
use reqwest::{Client, Response, Error};
use tokio::time::sleep;

// 0: Closed, 1: Open, 2: HalfOpen
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_threshold: usize,
    timeout: Duration,
    failure_count: std::sync::atomic::AtomicUsize,
    last_failure_time: std::sync::atomic::AtomicU64,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: AtomicU8::new(0),
            failure_threshold,
            timeout,
            failure_count: std::sync::atomic::AtomicUsize::new(0),
            last_failure_time: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_success(&self) {
        self.state.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if count >= self.failure_threshold {
            self.state.store(1, Ordering::SeqCst);
            self.last_failure_time.store(
                chrono::Utc::now().timestamp() as u64,
                Ordering::SeqCst,
            );
        }
    }

    pub fn is_allowed(&self) -> bool {
        let state = self.state.load(Ordering::SeqCst);
        if state == 0 {
            return true;
        }

        if state == 1 {
            let last_failure = self.last_failure_time.load(Ordering::SeqCst);
            let now = chrono::Utc::now().timestamp() as u64;
            if now > last_failure + self.timeout.as_secs() {
                self.state.store(2, Ordering::SeqCst);
                return true;
            }
            return false;
        }

        // HalfOpen: allows 1 attempt
        true
    }
}

pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub jitter: bool,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, base_delay: Duration, max_delay: Duration, backoff_factor: f64) -> Self {
        Self {
            max_attempts,
            base_delay,
            max_delay,
            backoff_factor,
            jitter: false,
        }
    }

    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    pub fn next_delay(&self, attempt: usize) -> Duration {
        let mut delay = self.base_delay.as_secs_f64() * self.backoff_factor.powi(attempt as i32);
        if delay > self.max_delay.as_secs_f64() {
            delay = self.max_delay.as_secs_f64();
        }

        if self.jitter {
            let jitter_factor = 0.5 + (0.5 * rand::random::<f64>());
            delay *= jitter_factor;
        }

        Duration::from_secs_f64(delay)
    }
}

#[async_trait]
pub trait ResilientHttpClient {
    async fn get_with_retry(&self, url: &str, policy: &RetryPolicy) -> Result<Response, Error>;
}

#[async_trait]
impl ResilientHttpClient for Client {
    async fn get_with_retry(&self, url: &str, policy: &RetryPolicy) -> Result<Response, Error> {
        let mut attempts = 0;
        loop {
            match self.get(url).send().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;
                    if attempts >= policy.max_attempts {
                        return Err(e);
                    }
                    let delay = policy.next_delay(attempts);
                    sleep(delay).await;
                }
            }
        }
    }
}
