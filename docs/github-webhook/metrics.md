# 指标监控

> [返回目录](README.md)

## 1. 指标定义

```rust
// src/webhooks/github/metrics.rs

use prometheus::{
    CounterVec, HistogramVec, IntCounterVec, IntGauge, Registry,
    opts, register_counter_vec_with_registry, register_histogram_vec_with_registry,
    register_int_counter_vec_with_registry, register_int_gauge_with_registry,
};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
}

pub struct WebhookMetrics {
    /// 接收的 Webhook 总数
    pub webhook_received: IntCounterVec,

    /// 处理成功的 Webhook 数
    pub webhook_success: IntCounterVec,

    /// 处理失败的 Webhook 数
    pub webhook_errors: CounterVec,

    /// Webhook 处理延迟
    pub webhook_duration: HistogramVec,

    /// 当前处理中的 Webhook 数
    pub webhook_inflight: IntGauge,
}

impl WebhookMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let webhook_received = register_int_counter_vec_with_registry!(
            opts!(
                "github_webhook_received_total",
                "Total received GitHub webhooks"
            ),
            &["event"],
            REGISTRY
        )?;

        let webhook_success = register_int_counter_vec_with_registry!(
            opts!(
                "github_webhook_success_total",
                "Total successfully processed GitHub webhooks"
            ),
            &["event"],
            REGISTRY
        )?;

        let webhook_errors = register_counter_vec_with_registry!(
            opts!(
                "github_webhook_errors_total",
                "Total failed GitHub webhooks"
            ),
            &["event", "error_type"],
            REGISTRY
        )?;

        let webhook_duration = register_histogram_vec_with_registry!(
            opts!(
                "github_webhook_duration_seconds",
                "GitHub webhook processing duration"
            ),
            &["event"],
            vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0],
            REGISTRY
        )?;

        let webhook_inflight = register_int_gauge_with_registry!(
            opts!(
                "github_webhook_inflight",
                "Currently processing GitHub webhooks"
            ),
            REGISTRY
        )?;

        Ok(Self {
            webhook_received,
            webhook_success,
            webhook_errors,
            webhook_duration,
            webhook_inflight,
        })
    }

    pub fn registry(&self) -> &Registry {
        &REGISTRY
    }
}
```

## 2. 指标记录

```rust
// src/webhooks/github/handler.rs

impl WebhookMetrics {
    pub fn record_received(&self, event: &str) {
        self.webhook_received.with_label_values(&[event]).inc();
        self.webhook_inflight.inc();
    }

    pub fn record_success(&self, event: &str, duration: f64) {
        self.webhook_success.with_label_values(&[event]).inc();
        self.webhook_duration.with_label_values(&[event]).observe(duration);
        self.webhook_inflight.dec();
    }

    pub fn record_error(&self, event: &str, error_type: &str) {
        self.webhook_errors
            .with_label_values(&[event, error_type])
            .inc();
        self.webhook_inflight.dec();
    }
}
```

## 3. 指标说明

| 指标 | 类型 | 标签 | 说明 |
|------|------|------|------|
| `github_webhook_received_total` | Counter | `event` | 接收的 Webhook 总数 |
| `github_webhook_success_total` | Counter | `event` | 处理成功的 Webhook 数 |
| `github_webhook_errors_total` | Counter | `event`, `error_type` | 处理失败的 Webhook 数 |
| `github_webhook_duration_seconds` | Histogram | `event` | Webhook 处理延迟分布 |
| `github_webhook_inflight` | Gauge | - | 当前处理中的 Webhook 数 |

## 4. Prometheus 告警规则

```yaml
# prometheus/rules/github-webhook.yaml
groups:
  - name: github_webhook
    interval: 30s
    rules:
      # 错误率告警
      - alert: GitHubWebhookHighErrorRate
        expr: |
          rate(github_webhook_errors_total[5m]) / rate(github_webhook_received_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "GitHub webhook error rate > 5%"
          description: "{{ $labels.event }} event error rate is {{ $value | humanizePercentage }}"

      # 处理延迟告警
      - alert: GitHubWebhookHighLatency
        expr: |
          histogram_quantile(0.95, rate(github_webhook_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "GitHub webhook P95 latency > 1s"
          description: "{{ $labels.event }} event P95 latency is {{ $value }}s"

      # 堆积告警
      - alert: GitHubWebhookBacklog
        expr: github_webhook_inflight > 100
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "GitHub webhook backlog detected"
          description: "{{ $value }} webhooks are currently in-flight"
```

## 5. Grafana 面板查询

```promql
# Webhook 接收速率 (按事件类型)
rate(github_webhook_received_total[5m])

# Webhook 成功率
rate(github_webhook_success_total[5m]) / rate(github_webhook_received_total[5m])

# P95 处理延迟
histogram_quantile(0.95, rate(github_webhook_duration_seconds_bucket[5m]))

# 当前堆积
github_webhook_inflight

# 错误分布
rate(github_webhook_errors_total[5m])
```
