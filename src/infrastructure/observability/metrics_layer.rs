use crate::infrastructure::metrics::registry::METRICS;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::registry::LookupSpan;

/// Tracing layer that increments `errors_total{module, kind}` on each ERROR-level event.
///
/// `module` — the `target` of the event (typically `module_path!()`).
/// `kind` — captured from event field `kind`, falling back to `"error"`.
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        if *event.metadata().level() != tracing::Level::ERROR {
            return;
        }

        let target = event.metadata().target();
        let module = top_module(target);

        let mut visitor = KindVisitor::default();
        event.record(&mut visitor);
        let kind = visitor.kind.unwrap_or_else(|| "error".to_string());

        METRICS
            .errors_total
            .with_label_values(&[&module, &kind])
            .inc();
    }
}

#[derive(Default)]
struct KindVisitor {
    kind: Option<String>,
}

impl Visit for KindVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "kind" {
            self.kind = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "kind" {
            self.kind = Some(format!("{:?}", value));
        }
    }
}

fn top_module(target: &str) -> String {
    let mut parts = target.split("::");
    let head = parts.next().unwrap_or("unknown");
    match parts.next() {
        Some(second) => format!("{}::{}", head, second),
        None => head.to_string(),
    }
}
