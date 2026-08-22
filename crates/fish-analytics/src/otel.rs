use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpanKind {
    Internal = 1,
    Server = 2,
    Client = 3,
    Producer = 4,
    Consumer = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    Unset = 0,
    Ok = 1,
    Error = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanStatus {
    pub code: StatusCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub time_unix_nano: u128,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelSpan {
    pub trace_id: String,
    pub span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_span_id: Option<String>,
    pub name: String,
    pub kind: SpanKind,
    pub start_time_unix_nano: u128,
    pub end_time_unix_nano: u128,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

impl OtelSpan {
    pub fn duration(&self) -> Duration {
        let nanos = self
            .end_time_unix_nano
            .saturating_sub(self.start_time_unix_nano);
        Duration::from_nanos(nanos as u64)
    }

    pub fn duration_ms(&self) -> f64 {
        self.duration().as_secs_f64() * 1000.0
    }
}

static SPAN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct ActiveSpanBuilder {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    name: String,
    kind: SpanKind,
    start_time_unix_nano: u128,
    attributes: HashMap<String, AttributeValue>,
    events: Vec<SpanEvent>,
}

impl ActiveSpanBuilder {
    pub fn new(trace_id: impl Into<String>, name: impl Into<String>) -> Self {
        let id_val = SPAN_COUNTER.fetch_add(1, Ordering::Relaxed);
        let span_id = format!(
            "{:016x}",
            id_val
                ^ SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64
        );
        let start_time_unix_nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        Self {
            trace_id: trace_id.into(),
            span_id,
            parent_span_id: None,
            name: name.into(),
            kind: SpanKind::Internal,
            start_time_unix_nano,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }

    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<AttributeValue>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn add_event(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, AttributeValue>,
    ) {
        let time_unix_nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        self.events.push(SpanEvent {
            name: name.into(),
            time_unix_nano,
            attributes,
        });
    }

    pub fn finish(self, success: bool, message: Option<String>) -> OtelSpan {
        let end_time_unix_nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let status = if success {
            SpanStatus {
                code: StatusCode::Ok,
                message: None,
            }
        } else {
            SpanStatus {
                code: StatusCode::Error,
                message,
            }
        };

        OtelSpan {
            trace_id: self.trace_id,
            span_id: self.span_id,
            parent_span_id: self.parent_span_id,
            name: self.name,
            kind: self.kind,
            start_time_unix_nano: self.start_time_unix_nano,
            end_time_unix_nano,
            attributes: self.attributes,
            events: self.events,
            status,
        }
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<i32> for AttributeValue {
    fn from(i: i32) -> Self {
        AttributeValue::Int(i as i64)
    }
}

impl From<u32> for AttributeValue {
    fn from(u: u32) -> Self {
        AttributeValue::Int(u as i64)
    }
}

impl From<u64> for AttributeValue {
    fn from(u: u64) -> Self {
        AttributeValue::Int(u as i64)
    }
}

impl From<usize> for AttributeValue {
    fn from(u: usize) -> Self {
        AttributeValue::Int(u as i64)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Float(f)
    }
}

#[derive(Debug, Clone)]
pub struct OtelTracer {
    service_name: String,
    trace_id: String,
    spans: Arc<Mutex<Vec<OtelSpan>>>,
}

impl OtelTracer {
    pub fn new(service_name: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let trace_id = format!("{:032x}", now);

        Self {
            service_name: service_name.into(),
            trace_id,
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_trace_id(service_name: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            trace_id: trace_id.into(),
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn start_span(&self, name: impl Into<String>) -> ActiveSpanBuilder {
        ActiveSpanBuilder::new(&self.trace_id, name)
    }

    pub fn record_span(&self, span: OtelSpan) {
        if let Ok(mut lock) = self.spans.lock() {
            lock.push(span);
        }
    }

    pub fn record_spans(&self, spans: impl IntoIterator<Item = OtelSpan>) {
        if let Ok(mut lock) = self.spans.lock() {
            lock.extend(spans);
        }
    }

    pub fn spans(&self) -> Vec<OtelSpan> {
        self.spans.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn span_count(&self) -> usize {
        self.spans.lock().map(|s| s.len()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.spans.lock() {
            lock.clear();
        }
    }

    pub fn to_otlp_json(&self) -> serde_json::Value {
        let spans = self.spans();
        let otlp_spans: Vec<serde_json::Value> = spans
            .iter()
            .map(|s| {
                let attrs: Vec<serde_json::Value> = s
                    .attributes
                    .iter()
                    .map(|(k, v)| match v {
                        AttributeValue::String(val) => serde_json::json!({
                            "key": k,
                            "value": { "stringValue": val }
                        }),
                        AttributeValue::Bool(val) => serde_json::json!({
                            "key": k,
                            "value": { "boolValue": val }
                        }),
                        AttributeValue::Int(val) => serde_json::json!({
                            "key": k,
                            "value": { "intValue": val.to_string() }
                        }),
                        AttributeValue::Float(val) => serde_json::json!({
                            "key": k,
                            "value": { "doubleValue": val }
                        }),
                    })
                    .collect();

                let events: Vec<serde_json::Value> = s
                    .events
                    .iter()
                    .map(|e| {
                        let ev_attrs: Vec<serde_json::Value> = e
                            .attributes
                            .iter()
                            .map(|(k, v)| match v {
                                AttributeValue::String(val) => serde_json::json!({
                                    "key": k,
                                    "value": { "stringValue": val }
                                }),
                                AttributeValue::Bool(val) => serde_json::json!({
                                    "key": k,
                                    "value": { "boolValue": val }
                                }),
                                AttributeValue::Int(val) => serde_json::json!({
                                    "key": k,
                                    "value": { "intValue": val.to_string() }
                                }),
                                AttributeValue::Float(val) => serde_json::json!({
                                    "key": k,
                                    "value": { "doubleValue": val }
                                }),
                            })
                            .collect();

                        serde_json::json!({
                            "timeUnixNano": e.time_unix_nano.to_string(),
                            "name": e.name,
                            "attributes": ev_attrs
                        })
                    })
                    .collect();

                let mut span_map = serde_json::json!({
                    "traceId": s.trace_id,
                    "spanId": s.span_id,
                    "name": s.name,
                    "kind": s.kind as u8,
                    "startTimeUnixNano": s.start_time_unix_nano.to_string(),
                    "endTimeUnixNano": s.end_time_unix_nano.to_string(),
                    "attributes": attrs,
                    "events": events,
                    "status": {
                        "code": s.status.code as u8,
                        "message": s.status.message
                    }
                });

                if let Some(parent) = &s.parent_span_id {
                    span_map["parentSpanId"] = serde_json::Value::String(parent.clone());
                }

                span_map
            })
            .collect();

        serde_json::json!({
            "resourceSpans": [
                {
                    "resource": {
                        "attributes": [
                            {
                                "key": "service.name",
                                "value": { "stringValue": self.service_name }
                            },
                            {
                                "key": "telemetry.sdk.language",
                                "value": { "stringValue": "rust" }
                            },
                            {
                                "key": "telemetry.sdk.name",
                                "value": { "stringValue": "fish-otel" }
                            }
                        ]
                    },
                    "scopeSpans": [
                        {
                            "scope": {
                                "name": "fish-orchestrator",
                                "version": env!("CARGO_PKG_VERSION")
                            },
                            "spans": otlp_spans
                        }
                    ]
                }
            ]
        })
    }

    pub fn to_chrome_trace_json(&self) -> serde_json::Value {
        let spans = self.spans();
        let events: Vec<serde_json::Value> = spans
            .iter()
            .map(|s| {
                let start_us = (s.start_time_unix_nano / 1000) as f64;
                let dur_us = ((s.end_time_unix_nano - s.start_time_unix_nano) / 1000) as f64;
                let mut args = serde_json::Map::new();
                for (k, v) in &s.attributes {
                    match v {
                        AttributeValue::String(str_v) => {
                            args.insert(k.clone(), serde_json::Value::String(str_v.clone()));
                        }
                        AttributeValue::Bool(bool_v) => {
                            args.insert(k.clone(), serde_json::Value::Bool(*bool_v));
                        }
                        AttributeValue::Int(int_v) => {
                            args.insert(k.clone(), serde_json::json!(*int_v));
                        }
                        AttributeValue::Float(flt_v) => {
                            args.insert(k.clone(), serde_json::json!(*flt_v));
                        }
                    }
                }

                serde_json::json!({
                    "name": s.name,
                    "cat": "build,task",
                    "ph": "X",
                    "ts": start_us,
                    "dur": dur_us,
                    "pid": 1,
                    "tid": s.span_id,
                    "args": args
                })
            })
            .collect();

        serde_json::json!({
            "traceEvents": events,
            "displayTimeUnit": "ms"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_span_creation_and_otlp_serialization() {
        let tracer = OtelTracer::new("fish-build");
        assert_eq!(tracer.service_name(), "fish-build");
        assert_eq!(tracer.trace_id().len(), 32);

        let mut root_builder = tracer.start_span("build_dag");
        root_builder = root_builder.with_attribute("workspace.packages", 35);
        root_builder = root_builder.with_attribute("build.mode", "release");

        let mut task_builder = tracer.start_span("compile_fish_core");
        task_builder = task_builder.with_parent(&root_builder.span_id);
        task_builder = task_builder.with_attribute("task.status", "executed");
        task_builder = task_builder.with_attribute("cache.hit", false);

        task_builder.add_event("jobserver_token_acquired", HashMap::new());
        let task_span = task_builder.finish(true, None);
        assert!(task_span.duration_ms() >= 0.0);
        tracer.record_span(task_span);

        let root_span = root_builder.finish(true, None);
        tracer.record_span(root_span);

        let spans = tracer.spans();
        assert_eq!(spans.len(), 2);

        let otlp_json = tracer.to_otlp_json();
        assert!(otlp_json["resourceSpans"].is_array());
        let scope_spans = &otlp_json["resourceSpans"][0]["scopeSpans"][0]["spans"];
        assert_eq!(scope_spans.as_array().unwrap().len(), 2);

        let chrome_json = tracer.to_chrome_trace_json();
        assert!(chrome_json["traceEvents"].is_array());
        assert_eq!(chrome_json["traceEvents"].as_array().unwrap().len(), 2);
    }
}
