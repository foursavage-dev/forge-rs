package migrator

import (
	"strings"
	"testing"
)

func TestSchemaMigrator(t *testing.T) {
	mig := NewSchemaMigrator()
	migrations := mig.GetAllMigrations()
	if len(migrations) != 4 {
		t.Fatalf("expected 4 migrations, got %d", len(migrations))
	}

	if mig.GetLatestVersion() != 4 {
		t.Fatalf("expected latest version 4, got %d", mig.GetLatestVersion())
	}

	script := mig.GenerateUpScript(2)
	if !strings.Contains(script, "create_build_runs_table") {
		t.Error("expected version 1 in script")
	}
	if !strings.Contains(script, "create_task_metrics_table") {
		t.Error("expected version 2 in script")
	}
	if strings.Contains(script, "create_cas_artifacts_table") {
		t.Error("did not expect version 3 in script")
	}

	downScript := mig.GenerateDownScript(4, 2)
	if !strings.Contains(downScript, "DROP TABLE IF EXISTS worker_telemetry;") {
		t.Error("expected rollback of version 4")
	}
	if !strings.Contains(downScript, "DROP TABLE IF EXISTS cas_artifacts;") {
		t.Error("expected rollback of version 3")
	}
	if strings.Contains(downScript, "DROP TABLE IF EXISTS build_runs;") {
		t.Error("did not expect rollback of version 1")
	}
}
