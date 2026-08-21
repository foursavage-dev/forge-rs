package telemetry

import (
	"sync"
	"time"
)

type TraceSpan struct {
	TraceID   string            `json:"trace_id"`
	SpanID    string            `json:"span_id"`
	ParentID  string            `json:"parent_id,omitempty"`
	Name      string            `json:"name"`
	StartTime time.Time         `json:"start_time"`
	EndTime   time.Time         `json:"end_time"`
	Tags      map[string]string `json:"tags"`
}

type OpenTelemetryExporter struct {
	mu    sync.RWMutex
	spans []TraceSpan
}

func NewOpenTelemetryExporter() *OpenTelemetryExporter {
	return &OpenTelemetryExporter{
		spans: make([]TraceSpan, 0),
	}
}

func (e *OpenTelemetryExporter) RecordSpan(span TraceSpan) {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.spans = append(e.spans, span)
}

func (e *OpenTelemetryExporter) ExportSpans() []TraceSpan {
	e.mu.RLock()
	defer e.mu.RUnlock()
	copied := make([]TraceSpan, len(e.spans))
	copy(copied, e.spans)
	return copied
}
