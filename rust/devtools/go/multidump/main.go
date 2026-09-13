// multidump is a differential-testing aid: it runs RunMulti and prints the
// primary output plus every generated file, so the multi-file path (which the
// golden suite cannot express) can be diffed. Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"

	"github.com/olivierdevelops/capy/orchestrator"
)

func main() {
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	args := os.Args[1:]
	for i := 0; i+1 < len(args); i += 2 {
		lib, script := args[i], args[i+1]
		fmt.Fprintf(w, "PAIR\t%s\t%s\n", lib, script)
		out, files, err := orchestrator.RunMulti(lib, script)
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
