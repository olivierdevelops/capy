// embeddump is a differential-testing aid: it exercises the public embedding
// API (capy.NewLibraryFromFile, Introspect, RenderLibraryDocs, …) on each
// library. Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"
	"strconv"

	capy "github.com/olivierdevelops/capy"
)

func q(s string) string { return strconv.Quote(s) }

func main() {
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	args := os.Args[1:]
	for i := 0; i+1 < len(args); i += 2 {
		libPath, scriptPath := args[i], args[i+1]
		fmt.Fprintf(w, "PAIR\t%s\t%s\n", libPath, scriptPath)
		lib, err := capy.NewLibraryFromFile(libPath)
		if err != nil {
			fmt.Fprintf(w, "NEWERR\t%s\n", err.Error())
			continue
		}
		fmt.Fprintf(w, "ext=%s outfile=%s\n", q(lib.Extension()), q(lib.OutputFile()))
		names := lib.FunctionNames()
		// NOTE: Go's FunctionNames() is documented as sorted but isn't; sort
		// here so the comparison isolates genuine differences.
		sort.Strings(names)
		fmt.Fprintf(w, "names=%v\n", names)
		fmt.Fprintf(w, "comments=%v\n", lib.CommentMarkers())
		for _, fi := range lib.Introspect() {
			fmt.Fprintf(w, "fn\t%s\tdesc=%s\tblock=%s\tprio=%d\n",
				q(fi.Name), q(fi.Description), q(fi.Block), fi.Priority)
			for _, a := range fi.Args {
				fmt.Fprintf(w, "  arg\tkind=%s\tval=%s\tname=%s\ttype=%s\tdesc=%s\topt=%v\tdef=%s\n",
					q(a.Kind), q(a.Value), q(a.Name), q(a.Type), q(a.Description), a.Optional, q(a.Default))
			}
		}
		fmt.Fprintf(w, "DOCS\n%s\nENDDOCS\n", capy.RenderLibraryDocs(lib))
		sb, rerr := os.ReadFile(scriptPath)
		if rerr != nil {
			fmt.Fprintf(w, "READERR\n")
			continue
		}
		out, files, err := lib.RunMulti(string(sb))
		if err != nil {
			fmt.Fprintf(w, "RUNERR\t%s\n", err.Error())
			continue
		}
		fmt.Fprintf(w, "OUT\t%d\n%s\nENDOUT\n", len(out), out)
		keys := make([]string, 0, len(files))
		for k := range files {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		for _, k := range keys {
			fmt.Fprintf(w, "FILE\t%s\t%d\n%s\nENDFILE\n", k, len(files[k]), files[k])
		}
	}
}
