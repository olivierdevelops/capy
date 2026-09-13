// nativebench measures the native Go engine's per-run cost through the same
// public embedding API the wazero binding mirrors.
package main

import (
	"fmt"
	"os"
	"time"

	capy "github.com/olivierdevelops/capy"
)

func main() {
	libSrc, _ := os.ReadFile(os.Args[1])
	scriptSrc, _ := os.ReadFile(os.Args[2])
	n := 200
	lib, err := capy.NewLibrary(string(libSrc))
	if err != nil {
		panic(err)
	}
	for i := 0; i < 5; i++ {
		if _, err := lib.Run(string(scriptSrc)); err != nil {
			panic(err)
		}
	}
	t := time.Now()
	for i := 0; i < n; i++ {
		if _, err := lib.Run(string(scriptSrc)); err != nil {
			panic(err)
		}
	}
	fmt.Printf("native per Run: %v\n", (time.Since(t) / time.Duration(n)).Round(time.Microsecond))
}
