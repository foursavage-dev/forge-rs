package telemetry

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"sync"
	"time"
)

func generateID() string {
	b := make([]byte, 16)
	_, _ = rand.Read(b)
	return hex.EncodeToString(b)
}

type TraceSpan struct {
	TraceID   string            `json:"trace_id"`
	SpanID    string            `json:"span_id"`
	ParentID  string            `json:"parent_id,omitempty"`
	Name      string            `json:"name"`
	StartTime time.Time         `json:"start_time"`
	EndTime   time.Time         `json:"end_time"`
	Tags      map[string]string `json:"tags"`
	Kind      string            `json:"kind"` // internal, server, client
	Status    string            `json:"status"` // ok, error
	Events    []SpanEvent       `json:"events,omitempty"`
}

type SpanEvent struct {
	Name      string            `json:"name"`
	Timestamp time.Time         `json:"timestamp"`
	Attributes map[string]string `json:"attributes"`
}

type SpanContext struct {
	TraceID string
	SpanID  string
	Baggage map[string]string
}

func (sc SpanContext) ToW3CHeader() string {
	return fmt.Sprintf("00-%s-%s-01", sc.TraceID, sc.SpanID)
}

func ParseW3CHeader(header string) (SpanContext, error) {
	// Format: 00-trace_id-span_id-flags
	var traceID, spanID string
	_, err := fmt.Sscanf(header, "00-%32s-%16s-01", &traceID, &spanID)
	if err != nil {
		return SpanContext{}, fmt.Errorf("invalid traceparent header: %w", err)
	}
	return SpanContext{
		TraceID: traceID,
		SpanID:  spanID,
	}, nil
}

type OpenTelemetryExporter struct {
	mu          sync.RWMutex
	spans       []TraceSpan
	serviceName string
	endpoint    string
	batchSize   int
	headers     map[string]string
}

func NewOpenTelemetryExporter() *OpenTelemetryExporter {
	return &OpenTelemetryExporter{
		spans:       make([]TraceSpan, 0),
		serviceName: "fish-go",
		endpoint:    "http://localhost:4317",
		batchSize:   100,
		headers:     make(map[string]string),
	}
}

func NewOpenTelemetryExporterWithConfig(serviceName, endpoint string) *OpenTelemetryExporter {
	return &OpenTelemetryExporter{
		spans:       make([]TraceSpan, 0),
		serviceName: serviceName,
		endpoint:    endpoint,
		batchSize:   100,
		headers:     make(map[string]string),
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

func (e *OpenTelemetryExporter) StartSpan(ctx context.Context, name string) (context.Context, TraceSpan) {
	// Check if parent exists in context
	parentID := ""
	traceID := generateID()
	
	if parentSpan, ok := ctx.Value("otel-span").(TraceSpan); ok {
		traceID = parentSpan.TraceID
		parentID = parentSpan.SpanID
	}

	span := TraceSpan{
		TraceID:   traceID,
		SpanID:    generateID()[:16],
		ParentID:  parentID,
		Name:      name,
		StartTime: time.Now(),
		Tags:      make(map[string]string),
		Kind:      "internal",
		Status:    "ok",
		Events:    make([]SpanEvent, 0),
	}

	newCtx := context.WithValue(ctx, "otel-span", span)
	return newCtx, span
}

func (e *OpenTelemetryExporter) FinishSpan(span *TraceSpan, err error) {
	span.EndTime = time.Now()
	if err != nil {
		span.Status = "error"
		span.Tags["error"] = err.Error()
	}
	e.RecordSpan(*span)
}

func (e *OpenTelemetryExporter) AddEvent(span *TraceSpan, name string, attrs map[string]string) {
	event := SpanEvent{
		Name:       name,
		Timestamp:  time.Now(),
		Attributes: attrs,
	}
	span.Events = append(span.Events, event)
}

func (e *OpenTelemetryExporter) ToOTLPJSON() map[string]interface{} {
	e.mu.RLock()
	defer e.mu.RUnlock()

	var otlpSpans []map[string]interface{}
	for _, span := range e.spans {
		duration := span.EndTime.Sub(span.StartTime)
		otlpSpan := map[string]interface{}{
			"traceId":           span.TraceID,
			"spanId":            span.SpanID,
			"parentSpanId":      span.ParentID,
			"name":              span.Name,
			"startTimeUnixNano": span.StartTime.UnixNano(),
			"endTimeUnixNano":   span.EndTime.UnixNano(),
			"durationMs":        duration.Milliseconds(),
			"kind":              span.Kind,
			"status":            span.Status,
			"attributes":        span.Tags,
		}
		otlpSpans = append(otlpSpans, otlpSpan)
	}

	return map[string]interface{}{
		"resourceSpans": []map[string]interface{}{
			{
				"resource": map[string]interface{}{
					"attributes": []map[string]interface{}{
						{"key": "service.name", "value": map[string]string{"stringValue": e.serviceName}},
					},
				},
				"scopeSpans": []map[string]interface{}{
					{
						"scope": map[string]interface{}{
							"name":    "fish-go",
							"version": "0.4.0",
						},
						"spans": otlpSpans,
					},
				},
			},
		},
	}
}

func (e *OpenTelemetryExporter) ExportToEndpoint() error {
	payload := e.ToOTLPJSON()
	data, err := json.Marshal(payload)
	if err != nil {
		return err
	}
	
	// In production, POST to e.endpoint/v1/traces
	// For now, just validate JSON
	_ = data
	return nil
}

func (e *OpenTelemetryExporter) GetTraceDuration(traceID string) time.Duration {
	e.mu.RLock()
	defer e.mu.RUnlock()
	
	var minStart, maxEnd time.Time
	first := true
	
	for _, span := range e.spans {
		if span.TraceID == traceID {
			if first {
				minStart = span.StartTime
				maxEnd = span.EndTime
				first = false
			} else {
				if span.StartTime.Before(minStart) {
					minStart = span.StartTime
				}
				if span.EndTime.After(maxEnd) {
					maxEnd = span.EndTime
				}
			}
		}
	}
	
	if first {
		return 0
	}
	return maxEnd.Sub(minStart)
}

func (e *OpenTelemetryExporter) GetCriticalPath(traceID string) []TraceSpan {
	e.mu.RLock()
	defer e.mu.RUnlock()
	
	var traceSpans []TraceSpan
	for _, span := range e.spans {
		if span.TraceID == traceID {
			traceSpans = append(traceSpans, span)
		}
	}
	
	// Sort by start time
	for i := 0; i < len(traceSpans); i++ {
		for j := i + 1; j < len(traceSpans); j++ {
			if traceSpans[j].StartTime.Before(traceSpans[i].StartTime) {
				traceSpans[i], traceSpans[j] = traceSpans[j], traceSpans[i]
			}
		}
	}
	
	return traceSpans
}

func (e *OpenTelemetryExporter) Clear() {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.spans = make([]TraceSpan, 0)
}

func (e *OpenTelemetryExporter) Count() int {
	e.mu.RLock()
	defer e.mu.RUnlock()
	return len(e.spans)
}
