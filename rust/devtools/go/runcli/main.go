// runcli exercises the otherwise-unreachable AppOrchestrator.RunCLI path.
package main

import (
	"fmt"
	"os"

	"github.com/olivierdevelops/capy/orchestrator"
)

func main() {
	code := orchestrator.AppOrchestrator{}.RunCLI(os.Args[1], os.Args[2])
	fmt.Printf("[exit %d]\n", code)
}
