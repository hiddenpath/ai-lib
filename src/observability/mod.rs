use async_trait::async_trait;

/// Minimal tracing facade trait to decouple from concrete backends (e.g., OpenTelemetry).
#[async_trait]
pub trait Tracer: Send + Sync {
    /// Start a span; return an opaque handle that stops on drop.
    async fn start_span(&self, name: &str) -> Box<dyn Span + Send>;

    /// Record an event with optional key/value attributes.
    async fn event(&self, name: &str, attrs: &[(&str, &str)]);
}

/// Opaque span handle.
pub trait Span: Send {
    /// Set attribute key/value on the span.
    fn set_attr(&mut self, key: &str, value: &str);
}

/// No-op implementations for defaults
pub struct NoopTracer;

#[async_trait]
impl Tracer for NoopTracer {
    async fn start_span(&self, _name: &str) -> Box<dyn Span + Send> { Box::new(NoopSpan) }
    async fn event(&self, _name: &str, _attrs: &[(&str, &str)]) { }
}

pub struct NoopSpan;
impl Span for NoopSpan { fn set_attr(&mut self, _key: &str, _value: &str) {} }

/// Structured audit sink for compliance and diagnostics.
#[async_trait]
pub trait AuditSink: Send + Sync {
    /// Record an audit event with serialized payload (caller decides redaction).
    async fn record(&self, category: &str, payload_json: &str);
}

/// No-op audit sink
pub struct NoopAudit;

#[async_trait]
impl AuditSink for NoopAudit {
    async fn record(&self, _category: &str, _payload_json: &str) {}
}


