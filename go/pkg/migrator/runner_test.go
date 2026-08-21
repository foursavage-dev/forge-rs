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

	v4, err := runner.ApplyNext()
	if err != nil || v4 != 4 {
		t.Fatalf("expected version 4, got %d (err: %v)", v4, err)
	}

	_, err = runner.ApplyNext()
	if err == nil {
		t.Fatalf("expected error when no more migrations exist")
	}

	rolledBack, err := runner.Rollback(2)
	if err != nil || rolledBack != 2 {
		t.Fatalf("expected rollback to version 2, got %d (err: %v)", rolledBack, err)
	}

	if runner.CurrentVersion() != 2 {
		t.Fatalf("expected current version 2, got %d", runner.CurrentVersion())
	}
}

func TestMigrationRunnerApplyAll(t *testing.T) {
	mig := NewSchemaMigrator()
	runner := NewMigrationRunner(mig)

	count, err := runner.ApplyAll()
	if err != nil || count != 4 {
		t.Fatalf("expected 4 applied migrations, got %d", count)
	}
	if runner.CurrentVersion() != 4 {
		t.Fatalf("expected current version 4, got %d", runner.CurrentVersion())
	}
}
