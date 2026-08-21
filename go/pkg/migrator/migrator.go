package migrator

import (
	"fmt"
	"sort"
)

type Migration struct {
	Version     int    `json:"version"`
	Description string `json:"description"`
	UpSQL       string `json:"up_sql"`
	DownSQL     string `json:"down_sql"`
}

type SchemaMigrator struct {
	migrations []Migration
}

func NewSchemaMigrator() *SchemaMigrator {
	return &SchemaMigrator{
		migrations: []Migration{
			{
				Version:     1,
				Description: "create_build_runs_table",
				UpSQL:       "CREATE TABLE IF NOT EXISTS build_runs (id TEXT PRIMARY KEY, status TEXT, duration_ms INTEGER, created_at TIMESTAMP);",
				DownSQL:     "DROP TABLE IF EXISTS build_runs;",
			},
			{
				Version:     2,
				Description: "create_task_metrics_table",
				UpSQL:       "CREATE TABLE IF NOT EXISTS task_metrics (task_id TEXT PRIMARY KEY, run_id TEXT, toolchain TEXT, cached INTEGER, duration_ms INTEGER);",
				DownSQL:     "DROP TABLE IF EXISTS task_metrics;",
			},
			{
				Version:     3,
				Description: "create_cas_artifacts_table",
				UpSQL:       "CREATE TABLE IF NOT EXISTS cas_artifacts (digest TEXT PRIMARY KEY, size_bytes INTEGER, compressed_bytes INTEGER, hit_count INTEGER);",
				DownSQL:     "DROP TABLE IF EXISTS cas_artifacts;",
			},
			{
				Version:     4,
				Description: "create_worker_telemetry_table",
				UpSQL:       "CREATE TABLE IF NOT EXISTS worker_telemetry (worker_id TEXT, timestamp TIMESTAMP, cpu_pct REAL, memory_bytes INTEGER, active_jobs INTEGER, PRIMARY KEY(worker_id, timestamp));",
				DownSQL:     "DROP TABLE IF EXISTS worker_telemetry;",
			},
		},
	}
}

func (m *SchemaMigrator) AddMigration(mig Migration) {
	for i, existing := range m.migrations {
		if existing.Version == mig.Version {
			m.migrations[i] = mig
			return
		}
	}
	m.migrations = append(m.migrations, mig)
}

func (m *SchemaMigrator) GetAllMigrations() []Migration {
	sorted := make([]Migration, len(m.migrations))
	copy(sorted, m.migrations)
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].Version < sorted[j].Version
	})
	return sorted
}

func (m *SchemaMigrator) GetLatestVersion() int {
	all := m.GetAllMigrations()
	if len(all) == 0 {
		return 0
	}
	return all[len(all)-1].Version
}

func (m *SchemaMigrator) GenerateUpScript(targetVersion int) string {
	var script string
	for _, mig := range m.GetAllMigrations() {
		if mig.Version <= targetVersion {
			script += fmt.Sprintf("Migration %d: %s\n%s\n", mig.Version, mig.Description, mig.UpSQL)
		}
	}
	return script
}

func (m *SchemaMigrator) GenerateDownScript(fromVersion int, toVersion int) string {
	var script string
	all := m.GetAllMigrations()
	for i := len(all) - 1; i >= 0; i-- {
		mig := all[i]
		if mig.Version <= fromVersion && mig.Version > toVersion {
			script += fmt.Sprintf("Rollback %d: %s\n%s\n", mig.Version, mig.Description, mig.DownSQL)
		}
	}
	return script
}
