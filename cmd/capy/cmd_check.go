package main

import (
	"fmt"
	"os"
	"sort"

	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

// `capy check <library.yaml>` parses + validates a library file without
// running any source. Useful for libraries-as-data CI.
func cmdCheck(args []string) error {
	if len(args) != 1 {
		return fmt.Errorf("usage: capy check <library.yaml>")
	}
	path := args[0]
	if _, err := os.Stat(path); err != nil {
		return err
	}
	loader := orchfeatures.MakeLibraryLoader(orchfeatures.MakeLexer().Tokenize)
	lib, err := loader.Load(path)
	if err != nil {
		return err
	}
	fmt.Printf("ok — %d function(s), %d type(s)\n", len(lib.Functions), len(lib.Types))
	// Sort both listings: Go map iteration is randomised, so the same library
	// printed a different order on every invocation — which makes `capy check`
	// output useless to diff in CI.
	fnNames := make([]string, 0, len(lib.Functions))
	for name := range lib.Functions {
		fnNames = append(fnNames, name)
	}
	sort.Strings(fnNames)
	for _, name := range fnNames {
		fmt.Printf("  function %s\n", name)
	}
	typeNames := make([]string, 0, len(lib.Types))
	for name := range lib.Types {
		typeNames = append(typeNames, name)
	}
	sort.Strings(typeNames)
	for _, name := range typeNames {
		fmt.Printf("  type     %s\n", name)
	}
	return nil
}
