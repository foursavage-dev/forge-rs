package coordinator

import "testing"

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
