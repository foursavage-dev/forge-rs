package coordinator

import (
	"errors"
	"sort"
	"sync"
	"time"
)

type QueuedTask struct {
	TaskID     string    `json:"task_id"`
	Toolchain  string    `json:"toolchain"`
	Priority   int       `json:"priority"`
	Weight     float64   `json:"weight"`
	RetryCount int       `json:"retry_count"`
	MaxRetries int       `json:"max_retries"`
	Deadline   time.Time `json:"deadline"`
}

type TaskQueue struct {
	mu    sync.Mutex
	tasks []QueuedTask
}

func NewTaskQueue() *TaskQueue {
	return &TaskQueue{
		tasks: make([]QueuedTask, 0),
	}
}

func (q *TaskQueue) Push(task QueuedTask) {
	q.mu.Lock()
	defer q.mu.Unlock()
	q.tasks = append(q.tasks, task)
	q.sortInternal()
}

func (q *TaskQueue) sortInternal() {
	sort.Slice(q.tasks, func(i, j int) bool {
		if q.tasks[i].Priority != q.tasks[j].Priority {
			return q.tasks[i].Priority > q.tasks[j].Priority
		}
		return q.tasks[i].Weight > q.tasks[j].Weight
	})
}

func (q *TaskQueue) PopForToolchain(toolchain string) (QueuedTask, error) {
	q.mu.Lock()
	defer q.mu.Unlock()
	now := time.Now()
	for i, t := range q.tasks {
		if !t.Deadline.IsZero() && t.Deadline.Before(now) {
			continue
		}
		if t.Toolchain == toolchain || toolchain == "" || t.Toolchain == "*" {
			task := t
			q.tasks = append(q.tasks[:i], q.tasks[i+1:]...)
			return task, nil
		}
	}
	return QueuedTask{}, errors.New("no matching task in queue")
}

func (q *TaskQueue) Peek() (QueuedTask, error) {
	q.mu.Lock()
	defer q.mu.Unlock()
	if len(q.tasks) == 0 {
		return QueuedTask{}, errors.New("queue is empty")
	}
	return q.tasks[0], nil
}

func (q *TaskQueue) Cancel(taskID string) bool {
	q.mu.Lock()
	defer q.mu.Unlock()
	for i, t := range q.tasks {
		if t.TaskID == taskID {
			q.tasks = append(q.tasks[:i], q.tasks[i+1:]...)
			return true
		}
	}
	return false
}

func (q *TaskQueue) RequeueWithBackoff(task QueuedTask) bool {
	if task.RetryCount >= task.MaxRetries && task.MaxRetries > 0 {
		return false
	}
	task.RetryCount++
	q.Push(task)
	return true
}

func (q *TaskQueue) Len() int {
	q.mu.Lock()
	defer q.mu.Unlock()
	return len(q.tasks)
}

func (q *TaskQueue) Clear() {
	q.mu.Lock()
	defer q.mu.Unlock()
	q.tasks = q.tasks[:0]
}
