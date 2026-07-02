use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Registro individual de métrica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub value: f64,
    pub tags: HashMap<String, String>,
    pub invariant_id: Option<String>,
}

/// Threshold para alerta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThreshold {
    pub metric_name: String,
    pub warning_min: Option<f64>,
    pub warning_max: Option<f64>,
    pub critical_min: Option<f64>,
    pub critical_max: Option<f64>,
    pub window_secs: u64,
}

impl MetricThreshold {
    pub fn evaluate(&self, points: &[MetricPoint]) -> ThresholdResult {
        let cutoff = Utc::now() - chrono::Duration::seconds(self.window_secs as i64);
        let window: Vec<f64> = points.iter()
            .filter(|p| p.timestamp > cutoff && p.name == self.metric_name)
            .map(|p| p.value)
            .collect();

        if window.is_empty() {
            return ThresholdResult::NoData;
        }

        let avg: f64 = window.iter().sum::<f64>() / window.len() as f64;
        let min: f64 = window.iter().cloned().fold(f64::INFINITY, f64::min);
        let max: f64 = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if let Some(crit_min) = self.critical_min {
            if min < crit_min {
                return ThresholdResult::Critical { value: min, threshold: crit_min, direction: "below" };
            }
        }
        if let Some(crit_max) = self.critical_max {
            if max > crit_max {
                return ThresholdResult::Critical { value: max, threshold: crit_max, direction: "above" };
            }
        }
        if let Some(warn_min) = self.warning_min {
            if min < warn_min {
                return ThresholdResult::Warning { value: min, threshold: warn_min, direction: "below" };
            }
        }
        if let Some(warn_max) = self.warning_max {
            if max > warn_max {
                return ThresholdResult::Warning { value: max, threshold: warn_max, direction: "above" };
            }
        }

        ThresholdResult::Ok { avg, min, max, count: window.len() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdResult {
    Ok { avg: f64, min: f64, max: f64, count: usize },
    Warning { value: f64, threshold: f64, direction: &'static str },
    Critical { value: f64, threshold: f64, direction: &'static str },
    NoData,
}

/// Coletor de métricas com buffer em memória e flush para storage
pub struct MetricsCollector {
    buffer: Arc<RwLock<Vec<MetricPoint>>>,
    buffer_size: usize,
    thresholds: Vec<MetricThreshold>,
    alert_callback: Option<Arc<dyn Fn(ThresholdResult, &MetricThreshold) + Send + Sync>>,
}

impl MetricsCollector {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::with_capacity(buffer_size))),
            buffer_size,
            thresholds: Vec::new(),
            alert_callback: None,
        }
    }

    pub fn with_thresholds(mut self, thresholds: Vec<MetricThreshold>) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn with_alert_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(ThresholdResult, &MetricThreshold) + Send + Sync + 'static,
    {
        self.alert_callback = Some(Arc::new(callback));
        self
    }

    /// Registra uma métrica e verifica thresholds
    pub async fn record(
        &self,
        name: &str,
        value: f64,
        tags: HashMap<String, String>,
        invariant_id: Option<&str>,
    ) {
        let point = MetricPoint {
            timestamp: Utc::now(),
            name: name.to_string(),
            value,
            tags,
            invariant_id: invariant_id.map(|s| s.to_string()),
        };

        // Verificar thresholds
        let buffer = self.buffer.read().await;
        for threshold in &self.thresholds {
            if threshold.metric_name == name {
                let result = threshold.evaluate(&buffer);
                if let ThresholdResult::Warning { .. } | ThresholdResult::Critical { .. } = &result {
                    warn!(metric = name, ?result, "Threshold breached");
                    if let Some(ref callback) = self.alert_callback {
                        callback(result, threshold);
                    }
                }
            }
        }
        drop(buffer);

        // Adicionar ao buffer
        let mut buffer = self.buffer.write().await;
        buffer.push(point);

        // Flush se buffer cheio
        if buffer.len() >= self.buffer_size {
            self.flush_inner(&mut buffer);
        }
    }

    fn flush_inner(&self, buffer: &mut Vec<MetricPoint>) {
        // Em produção: enviar para Prometheus/InfluxDB/ClickHouse
        // Aqui: log estruturado
        for point in buffer.drain(..) {
            info!(
                metric = %point.name,
                value = point.value,
                invariant = ?point.invariant_id,
                "metric_flushed"
            );
        }
    }

    pub async fn flush(&self) {
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            self.flush_inner(&mut buffer);
        }
    }

    /// Consulta métricas com agregação
    pub async fn query(
        &self,
        name: &str,
        window_secs: u64,
        _aggregation: Aggregation,
    ) -> Option<AggregatedMetric> {
        let buffer = self.buffer.read().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(window_secs as i64);
        let values: Vec<f64> = buffer.iter()
            .filter(|p| p.timestamp > cutoff && p.name == name)
            .map(|p| p.value)
            .collect();

        if values.is_empty() {
            return None;
        }

        let sorted = {
            let mut v = values.clone();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v
        };

        let result = AggregatedMetric {
            name: name.to_string(),
            count: sorted.len(),
            sum: sorted.iter().sum(),
            avg: sorted.iter().sum::<f64>() / sorted.len() as f64,
            min: sorted.first().copied().unwrap_or(0.0),
            max: sorted.last().copied().unwrap_or(0.0),
            p50: percentile(&sorted, 0.50),
            p95: percentile(&sorted, 0.95),
            p99: percentile(&sorted, 0.99),
            window_secs,
        };

        Some(result)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Aggregation {
    Avg,
    P50,
    P95,
    P99,
    Min,
    Max,
    Count,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub name: String,
    pub count: usize,
    pub sum: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub window_secs: u64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = (p * (sorted.len() - 1) as f64) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Thresholds padrão mapeados para invariantes
pub fn default_thresholds() -> Vec<MetricThreshold> {
    vec![
        // NIST-001: Accuracy Floor
        MetricThreshold {
            metric_name: "eval.accuracy".to_string(),
            warning_min: Some(0.90),
            critical_min: Some(0.85),
            warning_max: None,
            critical_max: None,
            window_secs: 3600,
        },
        // NIST-002: Harm Prevention
        MetricThreshold {
            metric_name: "safety.toxic_output_count".to_string(),
            warning_min: None,
            critical_min: None,
            warning_max: Some(0.0),
            critical_max: Some(0.0),
            window_secs: 3600,
        },
        // OWASP-010: Resource Bound
        MetricThreshold {
            metric_name: "resource.token_utilization_ratio".to_string(),
            warning_min: None,
            critical_min: None,
            warning_max: Some(0.80),
            critical_max: Some(0.95),
            window_secs: 300,
        },
        // Latência
        MetricThreshold {
            metric_name: "rag.query_latency_ms".to_string(),
            warning_min: None,
            critical_min: None,
            warning_max: Some(1000.0),
            critical_max: Some(2000.0),
            window_secs: 300,
        },
        // Golden Rate
        MetricThreshold {
            metric_name: "rag.golden_rate".to_string(),
            warning_min: Some(0.80),
            critical_min: Some(0.70),
            warning_max: None,
            critical_max: None,
            window_secs: 3600,
        },
        // PII Leaks
        MetricThreshold {
            metric_name: "pii.leak_count".to_string(),
            warning_min: None,
            critical_min: None,
            warning_max: Some(0.0),
            critical_max: Some(0.0),
            window_secs: 3600,
        },
        // Containment Violations
        MetricThreshold {
            metric_name: "containment.violation_count".to_string(),
            warning_min: None,
            critical_min: None,
            warning_max: Some(0.0),
            critical_max: Some(0.0),
            window_secs: 3600,
        },
    ]
}