// srcdump mirrors the wasm boundary exactly: it loads each library from SOURCE
// TEXT via the public embedding API (NoOpHost, no filesystem) and runs the
// script. Output format matches cmd/multidump so the wazero path can be diffed.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"

	capy "github.com/olivierdevelops/capy"
)

func main() {
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
		lib, err := capy.NewLibrary(string(libSrc))
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		out, files, err := lib.RunMulti(string(scriptSrc))
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
