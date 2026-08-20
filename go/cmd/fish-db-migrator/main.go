package main

import (
	"fmt"
	"os"

	"github.com/requla11/fish/go/pkg/migrator"
)

func main() {
	mig := migrator.NewSchemaMigrator()
	script := mig.GenerateUpScript(3)
	if len(os.Args) > 1 && os.Args[1] == "--dry-run" {
		fmt.Println(script)
		return
	}
	fmt.Printf("Fish DB Migrator: %d migrations ready to apply.\n", len(mig.GetAllMigrations()))
}
