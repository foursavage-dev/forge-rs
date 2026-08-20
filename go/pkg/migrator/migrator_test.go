package migrator

import (
	"strings"
	"testing"
)

func TestSchemaMigrator(t *testing.T) {
	mig := NewSchemaMigrator()
	migrations := mig.GetAllMigrations()
	if len(migrations) != 3 {
		t.Fatalf("expected 3 migrations, got %d", len(migrations))
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
}
