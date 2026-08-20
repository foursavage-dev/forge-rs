package migrator

import (
	"errors"
	"fmt"
	"sync"
)

type MigrationState struct {
	AppliedVersion int
	AppliedHistory []int
}

type MigrationRunner struct {
	mu       sync.Mutex
	migrator *SchemaMigrator
	state    MigrationState
}

func NewMigrationRunner(mig *SchemaMigrator) *MigrationRunner {
	return &MigrationRunner{
		migrator: mig,
		state: MigrationState{
			AppliedVersion: 0,
			AppliedHistory: make([]int, 0),
		},
	}
}

func (r *MigrationRunner) ApplyNext() (int, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	all := r.migrator.GetAllMigrations()
	for _, m := range all {
		if m.Version > r.state.AppliedVersion {
			r.state.AppliedVersion = m.Version
			r.state.AppliedHistory = append(r.state.AppliedHistory, m.Version)
			return m.Version, nil
		}
	}
	return r.state.AppliedVersion, errors.New("no pending migrations")
}

func (r *MigrationRunner) ApplyAll() (int, error) {
	count := 0
	for {
		_, err := r.ApplyNext()
		if err != nil {
			break
		}
		count++
	}
	return count, nil
}

func (r *MigrationRunner) Rollback(steps int) (int, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	if steps <= 0 {
		return r.state.AppliedVersion, nil
	}

	for i := 0; i < steps; i++ {
		if len(r.state.AppliedHistory) == 0 {
			r.state.AppliedVersion = 0
			break
		}
		r.state.AppliedHistory = r.state.AppliedHistory[:len(r.state.AppliedHistory)-1]
		if len(r.state.AppliedHistory) > 0 {
			r.state.AppliedVersion = r.state.AppliedHistory[len(r.state.AppliedHistory)-1]
		} else {
			r.state.AppliedVersion = 0
		}
	}

	return r.state.AppliedVersion, nil
}

func (r *MigrationRunner) CurrentVersion() int {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.state.AppliedVersion
}

func (r *MigrationRunner) Status() string {
	r.mu.Lock()
	defer r.mu.Unlock()
	return fmt.Sprintf("Current Version: %d | Applied Count: %d", r.state.AppliedVersion, len(r.state.AppliedHistory))
}
