package coordinator

import (
	"testing"
	"time"
)

func TestTaskQueuePriority(t *testing.T) {
	q := NewTaskQueue()
	q.Push(QueuedTask{TaskID: "low", Toolchain: "rust", Priority: 1, Weight: 10.0})
	q.Push(QueuedTask{TaskID: "high", Toolchain: "rust", Priority: 10, Weight: 1.0})
	q.Push(QueuedTask{TaskID: "medium", Toolchain: "go", Priority: 5, Weight: 2.0})

	if q.Len() != 3 {
		t.Fatalf("expected length 3, got %d", q.Len())
	}

	top, err := q.PopForToolchain("rust")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if top.TaskID != "high" {
		t.Fatalf("expected 'high', got '%s'", top.TaskID)
	}

	goTask, err := q.PopForToolchain("go")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if goTask.TaskID != "medium" {
		t.Fatalf("expected 'medium', got '%s'", goTask.TaskID)
	}
}

func TestTaskQueueOperations(t *testing.T) {
	q := NewTaskQueue()
	q.Push(QueuedTask{TaskID: "task-1", Toolchain: "rust", Priority: 5, MaxRetries: 2})
	q.Push(QueuedTask{TaskID: "task-2", Toolchain: "rust", Priority: 1, Deadline: time.Now().Add(-1 * time.Minute)})

	peeked, err := q.Peek()
	if err != nil || peeked.TaskID != "task-1" {
		t.Fatalf("expected peek task-1, got %v, err=%v", peeked, err)
	}

	popped, err := q.PopForToolchain("rust")
	if err != nil || popped.TaskID != "task-1" {
		t.Fatalf("expected pop task-1, got %v", popped)
	}

	_, err = q.PopForToolchain("rust")
	if err == nil {
		t.Fatal("expected expired task-2 to be skipped")
	}

	requeued := q.RequeueWithBackoff(popped)
	if !requeued {
		t.Fatal("expected requeue to succeed")
	}
	if q.Len() != 2 {
		t.Fatalf("expected length 2, got %d", q.Len())
	}

	cancelled := q.Cancel("task-1")
	if !cancelled {
		t.Fatal("expected task-1 to be cancelled")
	}

	q.Clear()
	if q.Len() != 0 {
		t.Fatalf("expected empty queue after clear, got %d", q.Len())
	}
}
