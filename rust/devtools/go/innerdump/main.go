// innerdump is a differential-testing aid for the Rust port: it lexes an
// inner-DSL snippet, runs ParseInner, and prints a canonical AST rendering.
// Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/olivierdevelops/capy/domain"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

func expr(x domain.Expr) string {
	switch n := x.(type) {
	case domain.NumberLit:
		if n.IsInt {
			return "int(" + strconv.FormatInt(n.I, 10) + ")"
		}
		return "flt(" + strconv.FormatFloat(n.F, 'g', -1, 64) + ")"
	case domain.StringLit:
		return "str(" + strconv.Quote(n.Value) + ")"
	case domain.BoolLit:
		return "bool(" + strconv.FormatBool(n.Value) + ")"
	case domain.NullLit:
		return "null"
	case domain.VarRef:
		return "var(" + steps(n.Steps) + ")"
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
			parts = append(parts, strconv.Quote(k)+"="+expr(n.Vals[i]))
		}
		return "obj{" + strings.Join(parts, ",") + "}"
	}
	return "UNKNOWN"
}

func steps(ss []domain.PathStep) string {
	parts := []string{}
	for _, s := range ss {
		if s.IsIndex {
			parts = append(parts, "["+expr(s.Index)+"]")
		} else {
			parts = append(parts, "."+s.Field)
		}
	}
	return strings.Join(parts, "")
}

func path(p domain.Path) string { return p.Root + steps(p.Steps) }

func block(b domain.InnerBlock, ind string) string {
	var sb strings.Builder
	for _, s := range b.Stmts {
		sb.WriteString(stmt(s, ind))
	}
	return sb.String()
}

func stmt(s domain.InnerStmt, ind string) string {
	switch n := s.(type) {
	case domain.SetStmt:
		return ind + "set " + path(n.Target) + " = " + expr(n.Value) + "\n"
	case domain.AppendStmt:
		return ind + "append " + path(n.Target) + " = " + expr(n.Value) + "\n"
	case domain.PrependStmt:
		return ind + "prepend " + path(n.Target) + " = " + expr(n.Value) + "\n"
	case domain.MergeStmt:
		return ind + "merge " + path(n.Target) + " = " + expr(n.Value) + "\n"
	case domain.DeleteStmt:
		return ind + "delete " + path(n.Target) + "\n"
	case domain.WriteStmt:
		return ind + "write " + expr(n.Value) + "\n"
	case domain.CallStmt:
		return ind + "call " + expr(n.Call) + "\n"
	case domain.IfStmt:
		out := ind + "if " + expr(n.Cond) + "\n" + block(n.Body, ind+"  ")
		if n.Else != nil {
			out += ind + "else\n" + block(*n.Else, ind+"  ")
		}
		return out + ind + "endif\n"
	case domain.LoopStmt:
		return ind + "loop key=" + n.KeyVar + " var=" + n.Var + " in " + expr(n.Iter) + "\n" +
			block(n.Body, ind+"  ") + ind + "endloop\n"
	}
	return ind + "UNKNOWNSTMT\n"
}

func main() {
	lex := orchfeatures.MakeLexer()
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	for _, p := range os.Args[1:] {
		b, err := os.ReadFile(p)
		if err != nil {
			fmt.Fprintf(w, "FILE\t%s\nREADERR\n", p)
			continue
		}
		fmt.Fprintf(w, "FILE\t%s\n", p)
		toks, lerr := lex.Tokenize(string(b))
		if lerr != nil {
			fmt.Fprintf(w, "LEXERR\t%s\n", lerr.Error())
			continue
		}
		blk, perr := orchfeatures.ParseInner(toks)
		if perr != nil {
			fmt.Fprintf(w, "PARSEERR\t%s\n", perr.Error())
			continue
		}
		fmt.Fprint(w, block(blk, ""))
	}
}
