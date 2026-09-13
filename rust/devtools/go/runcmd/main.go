// runcmd is a differential-testing aid: invokes orchestrator.RunCommand.
package main

import (
	"fmt"
	"os"

	"github.com/olivierdevelops/capy/orchestrator"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "usage: runcmd <lib> <cmd> [args...]")
		os.Exit(2)
	}
	if err := orchestrator.RunCommand(os.Args[1], os.Args[2], os.Args[3:]); err != nil {
		fmt.Printf("RUNCMDERR\t%s\n", err.Error())
		os.Exit(1)
	}
}
