use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

// Basic mocked instant for WASM compatibility or when `tokio::time::Instant` isn't suitable in testing
#[derive(Clone, Copy)]
pub struct MockInstant(u64);
impl MockInstant {
    pub fn now() -> Self {
        MockInstant(chrono::Utc::now().timestamp_millis() as u64)
    }
    pub fn elapsed(&self) -> Duration {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Duration::from_millis(now.saturating_sub(self.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,     // Normal: requisições passam
    Open,       // Falha: requisições são bloqueadas
    HalfOpen,   // Teste: uma requisição passa para verificar recuperação
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    config: CircuitBreakerConfig,
    last_state_change: Arc<Mutex<MockInstant>>,
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            failure_count: AtomicUsize::new(self.failure_count.load(Ordering::SeqCst)),
            success_count: AtomicUsize::new(self.success_count.load(Ordering::SeqCst)),
            config: self.config.clone(),
            last_state_change: self.last_state_change.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_secs: 30,
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            config,
            last_state_change: Arc::new(Mutex::new(MockInstant::now())),
        }
    }

    pub async fn call<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Display + From<&'static str>,
    {
        let current_state = *self.state.lock().await;

        match current_state {
            CircuitState::Open => {
                let elapsed = self.last_state_change.lock().await.elapsed();
                if elapsed >= Duration::from_secs(self.config.timeout_secs) {
                    *self.state.lock().await = CircuitState::HalfOpen;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                    *self.last_state_change.lock().await = MockInstant::now();
                } else {
                    return Err("Circuit open".into());
                }
            }
            CircuitState::HalfOpen => {}
            _ => {}
        }

        let result = f().await;

        match result {
            Ok(val) => {
                self.success_count.fetch_add(1, Ordering::SeqCst);
                let current_state = *self.state.lock().await;
                if current_state == CircuitState::HalfOpen {
                    let successes = self.success_count.load(Ordering::SeqCst);
                    if successes >= self.config.success_threshold {
                        *self.state.lock().await = CircuitState::Closed;
                        tracing::info!("✅ Circuit Breaker fechado (recuperado)");
                    }
                }
                if current_state == CircuitState::Closed {
                    self.failure_count.store(0, Ordering::SeqCst);
                }
                Ok(val)
            }
            Err(e) => {
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                let current_state = *self.state.lock().await;
                if current_state == CircuitState::Closed && failures >= self.config.failure_threshold {
                    *self.state.lock().await = CircuitState::Open;
                    *self.last_state_change.lock().await = MockInstant::now();
                    tracing::warn!("🔌 Circuit Breaker aberto (threshold: {})", self.config.failure_threshold);
                } else if current_state == CircuitState::HalfOpen {
                    *self.state.lock().await = CircuitState::Open;
                    *self.last_state_change.lock().await = MockInstant::now();
                    tracing::warn!("🔌 Circuit Breaker reaberto (falha no HalfOpen)");
                }
                Err(e)
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        *self.state.lock().await
    }
}
