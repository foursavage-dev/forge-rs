package telemetry

import (
	"testing"
	"time"
)

func TestOpenTelemetryExporter(t *testing.T) {
	exporter := NewOpenTelemetryExporter()
	now := time.Now()

	span := TraceSpan{
		TraceID:   "trace-001",
		SpanID:    "span-001",
		Name:      "compile_rust_target",
		StartTime: now,
		EndTime:   now.Add(2 * time.Second),
		Tags:      map[string]string{"compiler": "rustc"},
	}

	exporter.RecordSpan(span)
	spans := exporter.ExportSpans()
	if len(spans) != 1 {
		t.Fatalf("expected 1 span exported, got %d", len(spans))
	}
	if spans[0].Name != "compile_rust_target" {
		t.Fatalf("unexpected span name: %s", spans[0].Name)
	}
}
