package coordinator

import (
	"errors"
	"sort"
	"sync"
)

type QueuedTask struct {
	TaskID    string
	Toolchain string
	Priority  int
	Weight    float64
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
	for i, t := range q.tasks {
		if t.Toolchain == toolchain || toolchain == "" {
			task := t
			q.tasks = append(q.tasks[:i], q.tasks[i+1:]...)
			return task, nil
		}
	}
	return QueuedTask{}, errors.New("no matching task in queue")
}

func (q *TaskQueue) Len() int {
	q.mu.Lock()
	defer q.mu.Unlock()
	return len(q.tasks)
}
