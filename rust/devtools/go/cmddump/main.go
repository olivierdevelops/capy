// cmddump is a differential-testing aid for the command runner: it prints each
// command's generated help plus the result of ParseCommandArgs over a grid of
// argv shapes. Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"
	"strconv"

	"github.com/olivierdevelops/capy/domain"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
	"github.com/olivierdevelops/capy/orchestrator"
)

func q(s string) string { return strconv.Quote(s) }

func reprAny(v any) string {
	switch x := v.(type) {
	case nil:
		return "nil"
	case string:
		return "s:" + q(x)
	case bool:
		return "b:" + strconv.FormatBool(x)
	}
	return fmt.Sprintf("?:%v", v)
}

var argvGrid = [][]string{
	{},
	{"one"},
	{"one", "two"},
	{"one", "two", "three"},
	{"--verbose"},
	{"--out", "x.txt"},
	{"--out=x.txt"},
	{"--out"},
	{"--nope"},
	{"--verbose", "one"},
	{"one", "--verbose", "two"},
	{"-h"},
}

func main() {
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	lex := orchfeatures.MakeLexer()
	loader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	for _, libPath := range os.Args[1:] {
		fmt.Fprintf(w, "LIB\t%s\n", libPath)
		lib, err := loader.Load(libPath)
		if err != nil {
			fmt.Fprintf(w, "LOADERR\t%s\n", err.Error())
			continue
		}
		names := make([]string, 0, len(lib.Commands))
		for n := range lib.Commands {
			names = append(names, n)
		}
		sort.Strings(names)
		for _, n := range names {
			cmd := lib.Commands[n]
			fmt.Fprintf(w, "CMD\t%s\tdesc=%s\tnargs=%d\tnflags=%d\n",
				q(cmd.Name), q(cmd.Description), len(cmd.Args), len(cmd.Flags))
			// Capture the generated help by redirecting stdout.
			r, wr, _ := os.Pipe()
			old := os.Stdout
			os.Stdout = wr
			orchestrator.PrintCommandHelp(lib, cmd)
			wr.Close()
			os.Stdout = old
			buf := make([]byte, 1<<16)
			nn, _ := r.Read(buf)
			fmt.Fprintf(w, "HELP\n%s\nENDHELP\n", string(buf[:nn]))
			for _, argv := range argvGrid {
				pos, flags, extra, perr := orchestrator.ParseCommandArgs(cmd, argv)
				if perr != nil {
					fmt.Fprintf(w, "  ARGS %v -> ERR %s\n", argv, perr.Error())
					continue
				}
				pk := make([]string, 0, len(pos))
				for k := range pos {
					pk = append(pk, k)
				}
				sort.Strings(pk)
				fk := make([]string, 0, len(flags))
				for k := range flags {
					fk = append(fk, k)
				}
				sort.Strings(fk)
				fmt.Fprintf(w, "  ARGS %v -> pos{", argv)
				for i, k := range pk {
					if i > 0 {
						fmt.Fprintf(w, ",")
					}
					fmt.Fprintf(w, "%s=%s", q(k), reprAny(pos[k]))
				}
				fmt.Fprintf(w, "} flags{")
				for i, k := range fk {
					if i > 0 {
						fmt.Fprintf(w, ",")
					}
					fmt.Fprintf(w, "%s=%s", q(k), reprAny(flags[k]))
				}
				fmt.Fprintf(w, "} extra%v\n", extra)
			}
			_ = domain.CommandDef{}
		}
	}
}
