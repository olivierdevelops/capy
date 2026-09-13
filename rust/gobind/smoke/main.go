// smoke drives the wasm-backed engine over every sample lib/script pair and
// prints results in the same format cmd/multidump uses, so the wazero path can
// be diffed against the native Go engine.
package main

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"sort"

	rustbind "github.com/olivierdevelops/capy/rustbind"
)

func main() {
	ctx := context.Background()
	rt, err := rustbind.NewRuntime(ctx)
	if err != nil {
		fmt.Fprintln(os.Stderr, "runtime:", err)
		os.Exit(1)
	}
	defer rt.Close(ctx)

	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	args := os.Args[1:]
	for i := 0; i+1 < len(args); i += 2 {
		libPath, scriptPath := args[i], args[i+1]
		fmt.Fprintf(w, "PAIR\t%s\t%s\n", libPath, scriptPath)
		libSrc, err := os.ReadFile(libPath)
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		scriptSrc, err := os.ReadFile(scriptPath)
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		lib, err := rt.NewLibrary(ctx, string(libSrc))
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		out, files, err := lib.RunMulti(ctx, string(scriptSrc))
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		fmt.Fprintf(w, "OUTPUT\t%d bytes\n%s\nENDOUTPUT\n", len(out), out)
		keys := make([]string, 0, len(files))
		for k := range files {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		for _, k := range keys {
			fmt.Fprintf(w, "FILE\t%s\t%d bytes\n%s\nENDFILE\n", k, len(files[k]), files[k])
		}
	}
}
