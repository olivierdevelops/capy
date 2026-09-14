// bench separates the three costs in the wazero path: one-time module
// compilation, per-call instantiation, and actual engine execution.
package main

import (
	"context"
	"fmt"
	"os"
	"time"

	rustbind "github.com/olivierdevelops/capy/rust/gobind"
)

func main() {
	ctx := context.Background()
	libSrc, _ := os.ReadFile(os.Args[1])
	scriptSrc, _ := os.ReadFile(os.Args[2])
	n := 200

	t0 := time.Now()
	rt, err := rustbind.NewRuntime(ctx)
	if err != nil {
		panic(err)
	}
	compile := time.Since(t0)
	defer rt.Close(ctx)

	lib, err := rt.NewLibrary(ctx, string(libSrc))
	if err != nil {
		panic(err)
	}

	// Warm up.
	for i := 0; i < 5; i++ {
		if _, err := lib.Run(ctx, string(scriptSrc)); err != nil {
			panic(err)
		}
	}

	t1 := time.Now()
	for i := 0; i < n; i++ {
		if _, err := lib.Run(ctx, string(scriptSrc)); err != nil {
			panic(err)
		}
	}
	per := time.Since(t1) / time.Duration(n)

	fmt.Printf("module compile (once):      %v\n", compile.Round(time.Millisecond))
	fmt.Printf("per Run (instantiate+exec): %v\n", per.Round(time.Microsecond))
}
