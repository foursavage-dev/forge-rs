package migrator

import "testing"

func TestMigrationRunner(t *testing.T) {
	mig := NewSchemaMigrator()
	runner := NewMigrationRunner(mig)

	if runner.CurrentVersion() != 0 {
		t.Fatalf("expected version 0, got %d", runner.CurrentVersion())
	}

	v1, err := runner.ApplyNext()
	if err != nil || v1 != 1 {
		t.Fatalf("expected version 1, got %d (err: %v)", v1, err)
	}

	v2, err := runner.ApplyNext()
	if err != nil || v2 != 2 {
		t.Fatalf("expected version 2, got %d (err: %v)", v2, err)
	}

	v3, err := runner.ApplyNext()
	if err != nil || v3 != 3 {
		t.Fatalf("expected version 3, got %d (err: %v)", v3, err)
	}

	_, err = runner.ApplyNext()
	if err == nil {
		t.Fatalf("expected error when no more migrations exist")
	}
}
