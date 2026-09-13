// parsedump is a differential-testing aid for the Rust port: it loads a library,
// parses a script against it, and prints the resulting parse tree.
// Usage: parsedump <lib.capy> <script> [<lib.capy> <script> ...]
// Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/olivierdevelops/capy/domain"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

func q(s string) string { return strconv.Quote(s) }

func expr(x domain.Expr) string {
	switch n := x.(type) {
	case domain.NumberLit:
		if n.IsInt {
			return "int(" + strconv.FormatInt(n.I, 10) + ")"
		}
		return "flt(" + strconv.FormatFloat(n.F, 'g', -1, 64) + ")"
	case domain.StringLit:
		return "str(" + q(n.Value) + ")"
	case domain.BoolLit:
		return "bool(" + strconv.FormatBool(n.Value) + ")"
	case domain.NullLit:
		return "null"
	case domain.VarRef:
		parts := []string{}
		for _, s := range n.Steps {
			if s.IsIndex {
				parts = append(parts, "["+expr(s.Index)+"]")
			} else {
				parts = append(parts, "."+s.Field)
			}
		}
		return "var(" + strings.Join(parts, "") + ")"
	case domain.CallExpr:
		args := []string{}
		for _, a := range n.Args {
			args = append(args, expr(a))
		}
		return "call(" + strings.Join(n.Name, ".") + ";" + strings.Join(args, ",") + ")"
	case domain.CompareExpr:
		return "cmp(" + n.Op + ";" + expr(n.Left) + ";" + expr(n.Right) + ")"
	case domain.NotExpr:
		return "not(" + expr(n.X) + ")"
	case domain.ListLit:
		items := []string{}
		for _, it := range n.Items {
			items = append(items, expr(it))
		}
		return "list[" + strings.Join(items, ",") + "]"
	case domain.ObjLit:
		parts := []string{}
		for i, k := range n.Keys {
			parts = append(parts, q(k)+"="+expr(n.Vals[i]))
		}
		return "obj{" + strings.Join(parts, ",") + "}"
	}
	return "UNKNOWN"
}

func dumpBlock(w *bufio.Writer, b *domain.Block, ind string) {
	if b == nil {
		return
	}
	if b.IsVerbatim {
		fmt.Fprintf(w, "%sVERBATIM %s\n", ind, q(b.VerbatimText))
		return
	}
	for i := range b.Stmts {
		dumpCall(w, &b.Stmts[i], ind)
	}
}

func dumpCall(w *bufio.Writer, c *domain.FuncCall, ind string) {
	name := "<nil>"
	if c.Func != nil {
		name = c.Func.Name
	}
	fmt.Fprintf(w, "%scall %s @%d:%d\n", ind, name, c.Line, c.Col)
	ks := make([]string, 0, len(c.Captures))
	for k := range c.Captures {
		ks = append(ks, k)
	}
	sort.Strings(ks)
	for _, k := range ks {
		cv := c.Captures[k]
		e := "-"
		if cv.IsExpr && cv.Expr != nil {
			e = expr(cv.Expr)
		}
		fmt.Fprintf(w, "%s  cap %s text=%s isexpr=%v expr=%s nsub=%d\n",
			ind, q(k), q(cv.Text), cv.IsExpr, e, len(cv.Sub))
		for i := range cv.Sub {
			dumpCall(w, &cv.Sub[i], ind+"    ")
		}
	}
	if c.Body != nil {
		fmt.Fprintf(w, "%s  BODY\n", ind)
		dumpBlock(w, c.Body, ind+"    ")
	}
	secs := make([]string, 0, len(c.Sections))
	for k := range c.Sections {
		secs = append(secs, k)
	}
	sort.Strings(secs)
	for _, s := range secs {
		fmt.Fprintf(w, "%s  SECTION %s\n", ind, q(s))
		dumpBlock(w, c.Sections[s], ind+"    ")
	}
	if c.Closer != nil {
		fmt.Fprintf(w, "%s  CLOSER\n", ind)
		dumpCall(w, c.Closer, ind+"    ")
	}
}

func main() {
	lex := orchfeatures.MakeLexer()
	loader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	parser := orchfeatures.MakeParser()
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	args := os.Args[1:]
	for i := 0; i+1 < len(args); i += 2 {
		libPath, scriptPath := args[i], args[i+1]
		fmt.Fprintf(w, "PAIR\t%s\t%s\n", libPath, scriptPath)
		lib, err := loader.Load(libPath)
		if err != nil {
			fmt.Fprintf(w, "LOADERR\t%s\n", err.Error())
			continue
		}
		sb, err := os.ReadFile(scriptPath)
		if err != nil {
			fmt.Fprintf(w, "READERR\n")
			continue
		}
		src := string(sb)
		toks, err := lex.TokenizeWith(src, lib.Comments)
		if err != nil {
			fmt.Fprintf(w, "LEXERR\t%s\n", err.Error())
			continue
		}
		blk, err := parser.Parse(toks, src, lib)
		if err != nil {
			fmt.Fprintf(w, "PARSEERR\t%s\n", err.Error())
			if ce, ok := err.(*domain.CapyError); ok && ce.Hint != "" {
				fmt.Fprintf(w, "HINT\t%s\n", ce.Hint)
			}
			continue
		}
		dumpBlock(w, &blk, "")
	}
}
