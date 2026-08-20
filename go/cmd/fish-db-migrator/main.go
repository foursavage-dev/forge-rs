package main

import (
	"flag"
	"fmt"

	"github.com/requla11/fish/go/pkg/migrator"
)

func main() {
	dryRun := flag.Bool("dry-run", false, "Print SQL without executing")
	status := flag.Bool("status", false, "Show migration status")
	apply := flag.Bool("apply", false, "Apply pending migrations")
	flag.Parse()

	sm := migrator.NewSchemaMigrator()
	runner := migrator.NewMigrationRunner(sm)

	if *status {
		fmt.Println(runner.Status())
		return
	}

	if *dryRun {
		script := sm.GenerateUpScript(3)
		fmt.Println(script)
		return
	}

	if *apply {
		for {
			v, err := runner.ApplyNext()
			if err != nil {
				break
			}
			fmt.Printf("Applied migration version: %d\n", v)
		}
		fmt.Println("All pending migrations applied.")
		return
	}

	fmt.Printf("Fish DB Migrator: %d total migrations registered.\n", len(sm.GetAllMigrations()))
}
