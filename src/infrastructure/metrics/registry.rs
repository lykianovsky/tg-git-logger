use once_cell::sync::Lazy;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGaugeVec, Opts, Registry, TextEncoder,
};

pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

pub struct Metrics {
    pub registry: Registry,

    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,

    pub webhook_received_total: IntCounterVec,
    pub webhook_signature_invalid_total: IntCounterVec,

    pub telegram_send_total: IntCounterVec,
    pub telegram_send_duration_seconds: HistogramVec,

    pub github_api_requests_total: IntCounterVec,
    pub github_api_request_duration_seconds: HistogramVec,

    pub kaiten_api_requests_total: IntCounterVec,
    pub kaiten_api_request_duration_seconds: HistogramVec,

    pub job_processed_total: IntCounterVec,
    pub job_processing_duration_seconds: HistogramVec,

    pub errors_total: IntCounterVec,

    pub users_total: IntGaugeVec,
    pub users_on_vacation_total: prometheus::IntGauge,
    pub repositories_total: prometheus::IntGauge,
    pub release_plans_active_total: prometheus::IntGauge,
    pub release_plans_today_total: prometheus::IntGauge,
    pub health_ping_status: IntGaugeVec,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();

        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests received"),
            &["method", "route", "status"],
        )
        .expect("metric");

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request handling latency",
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
            &["method", "route"],
        )
        .expect("metric");

        let webhook_received_total = IntCounterVec::new(
            Opts::new("webhook_received_total", "GitHub webhook events received"),
            &["event_type"],
        )
        .expect("metric");

        let webhook_signature_invalid_total = IntCounterVec::new(
            Opts::new(
                "webhook_signature_invalid_total",
                "Webhook events with invalid HMAC signature",
            ),
            &["source"],
        )
        .expect("metric");

        let telegram_send_total = IntCounterVec::new(
            Opts::new(
                "telegram_send_total",
                "Telegram messages sent (success/fail/blocked)",
            ),
            &["status"],
        )
        .expect("metric");

        let telegram_send_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "telegram_send_duration_seconds",
                "Telegram send API latency",
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]),
            &["status"],
        )
        .expect("metric");

        let github_api_requests_total = IntCounterVec::new(
            Opts::new("github_api_requests_total", "GitHub API calls"),
            &["op", "status"],
        )
        .expect("metric");

        let github_api_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "github_api_request_duration_seconds",
                "GitHub API call latency",
            )
            .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["op"],
        )
        .expect("metric");

        let kaiten_api_requests_total = IntCounterVec::new(
            Opts::new("kaiten_api_requests_total", "Kaiten API calls"),
            &["op", "status"],
        )
        .expect("metric");

        let kaiten_api_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "kaiten_api_request_duration_seconds",
                "Kaiten API call latency",
            )
            .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["op"],
        )
        .expect("metric");

        let job_processed_total = IntCounterVec::new(
            Opts::new(
                "job_processed_total",
                "RabbitMQ jobs processed (success/retry/fail)",
            ),
            &["queue", "status"],
        )
        .expect("metric");

        let job_processing_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "job_processing_duration_seconds",
                "RabbitMQ job processing latency",
            )
            .buckets(vec![
                0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
            ]),
            &["queue"],
        )
        .expect("metric");

        let errors_total = IntCounterVec::new(
            Opts::new("errors_total", "Errors by module"),
            &["module", "kind"],
        )
        .expect("metric");

        let users_total = IntGaugeVec::new(
            Opts::new("users_total", "Registered users by status"),
            &["status"],
        )
        .expect("metric");

        let users_on_vacation_total = prometheus::IntGauge::new(
            "users_on_vacation_total",
            "Users currently on vacation",
        )
        .expect("metric");

        let repositories_total = prometheus::IntGauge::new(
            "repositories_total",
            "Repositories registered in the system",
        )
        .expect("metric");

        let release_plans_active_total = prometheus::IntGauge::new(
            "release_plans_active_total",
            "Active (planned) release plans",
        )
        .expect("metric");

        let release_plans_today_total = prometheus::IntGauge::new(
            "release_plans_today_total",
            "Release plans scheduled for today",
        )
        .expect("metric");

        let health_ping_status = IntGaugeVec::new(
            Opts::new("health_ping_status", "Health ping status (1=up, 0=down)"),
            &["service"],
        )
        .expect("metric");

        registry
            .register(Box::new(http_requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .unwrap();
        registry
            .register(Box::new(webhook_received_total.clone()))
            .unwrap();
        registry
            .register(Box::new(webhook_signature_invalid_total.clone()))
            .unwrap();
        registry
            .register(Box::new(telegram_send_total.clone()))
            .unwrap();
        registry
            .register(Box::new(telegram_send_duration_seconds.clone()))
            .unwrap();
        registry
            .register(Box::new(github_api_requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(github_api_request_duration_seconds.clone()))
            .unwrap();
        registry
            .register(Box::new(kaiten_api_requests_total.clone()))
            .unwrap();
        registry
            .register(Box::new(kaiten_api_request_duration_seconds.clone()))
            .unwrap();
        registry
            .register(Box::new(job_processed_total.clone()))
            .unwrap();
        registry
            .register(Box::new(job_processing_duration_seconds.clone()))
            .unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();
        registry.register(Box::new(users_total.clone())).unwrap();
        registry
            .register(Box::new(users_on_vacation_total.clone()))
            .unwrap();
        registry
            .register(Box::new(repositories_total.clone()))
            .unwrap();
        registry
            .register(Box::new(release_plans_active_total.clone()))
            .unwrap();
        registry
            .register(Box::new(release_plans_today_total.clone()))
            .unwrap();
        registry
            .register(Box::new(health_ping_status.clone()))
            .unwrap();

        Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            webhook_received_total,
            webhook_signature_invalid_total,
            telegram_send_total,
            telegram_send_duration_seconds,
            github_api_requests_total,
            github_api_request_duration_seconds,
            kaiten_api_requests_total,
            kaiten_api_request_duration_seconds,
            job_processed_total,
            job_processing_duration_seconds,
            errors_total,
            users_total,
            users_on_vacation_total,
            repositories_total,
            release_plans_active_total,
            release_plans_today_total,
            health_ping_status,
        }
    }

    pub fn render(&self) -> String {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap_or_default();
        String::from_utf8(buffer).unwrap_or_default()
    }
}
