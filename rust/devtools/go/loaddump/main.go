// loaddump is a differential-testing aid for the Rust port: it loads a .capy
// library through the full loader and prints the compiled domain.Library.
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
			parts = append(parts, q(k)+"="+expr(n.Vals[i]))
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

func ctxVal(v any) string {
	switch x := v.(type) {
	case nil:
		return "nil"
	case string:
		return "s:" + q(x)
	case int64:
		return "i:" + strconv.FormatInt(x, 10)
	case float64:
		return "f:" + strconv.FormatFloat(x, 'g', -1, 64)
	case bool:
		return "b:" + strconv.FormatBool(x)
	case []any:
		parts := []string{}
		for _, it := range x {
			parts = append(parts, ctxVal(it))
		}
		return "as:[" + strings.Join(parts, ",") + "]"
	case map[string]any:
		ks := []string{}
		for k := range x {
			ks = append(ks, k)
		}
		sort.Strings(ks)
		parts := []string{}
		for _, k := range ks {
			parts = append(parts, q(k)+":"+ctxVal(x[k]))
		}
		return "m:{" + strings.Join(parts, ",") + "}"
	}
	return "?"
}

func main() {
	lex := orchfeatures.MakeLexer()
	loader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	for _, p := range os.Args[1:] {
		fmt.Fprintf(w, "FILE\t%s\n", p)
		lib, err := loader.Load(p)
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			if ce, ok := err.(*domain.CapyError); ok && ce.Hint != "" {
				fmt.Fprintf(w, "HINT\t%s\n", ce.Hint)
			}
			continue
		}
		fmt.Fprintf(w, "ext=%s out=%s desc=%s name=%s ver=%s defimpl=%s\n",
			q(lib.Extension), q(lib.OutputFile), q(lib.Description), q(lib.LibName),
			q(lib.LibVersion), q(lib.DefaultImpl))
		fmt.Fprintf(w, "preprocess=%v comments=%v\n", lib.Preprocess, lib.Comments)
		if lib.FileTemplateAST != nil {
			fmt.Fprintf(w, "FILETEMPLATE\n%s", block(*lib.FileTemplateAST, "  "))
		}
		fks := []string{}
		for k := range lib.FilesAST {
			fks = append(fks, k)
		}
		sort.Strings(fks)
		for _, k := range fks {
			fmt.Fprintf(w, "FILEAST\t%s\n%s", q(k), block(*lib.FilesAST[k], "  "))
		}
		cks := []string{}
		for k := range lib.Context {
			cks = append(cks, k)
		}
		sort.Strings(cks)
		for _, k := range cks {
			fmt.Fprintf(w, "ctx\t%s\t%s\n", q(k), ctxVal(lib.Context[k]))
		}
		tks := []string{}
		for k := range lib.Types {
			tks = append(tks, k)
		}
		sort.Strings(tks)
		for _, k := range tks {
			t := lib.Types[k]
			fmt.Fprintf(w, "type\t%s\tbase=%s\tpat=%s\topts=%v\tgo=%s\tgc=%s\n",
				q(t.Name), q(t.Base), q(t.Pattern), t.Options, q(t.GroupOpen), q(t.GroupClose))
		}
		nks := []string{}
		for k := range lib.Functions {
			nks = append(nks, k)
		}
		sort.Strings(nks)
		for _, k := range nks {
			f := lib.Functions[k]
			fmt.Fprintf(w, "fn\t%s\tprio=%d\tdesc=%s\n", q(f.Name), f.Priority, q(f.Description))
			for _, a := range f.Args {
				fmt.Fprintf(w, "  arg\tkind=%s\tval=%s\tname=%s\ttype=%s\topt=%v\tdef=%s\trep=%s\tsep=%s\tjoin=%s\n",
					q(a.Kind), q(a.Value), q(a.Name), q(a.Type), a.Optional, q(a.Default),
					q(a.Repeat), q(a.Sep), q(a.Join))
			}
			for _, e := range f.Elements {
				fmt.Fprintf(w, "  el\tcap=%v\tlit=%s\tname=%s\ttype=%s\topt=%v\tdef=%s\trep=%s\tsep=%s\tjoin=%s\tisfn=%v\n",
					e.IsCapture, q(e.Literal), q(e.Name), q(e.CapType), e.Optional, q(e.Default),
					q(e.Repeat), q(e.Sep), q(e.Join), e.IsFunc)
			}
			if f.Block != nil {
				b := f.Block
				fmt.Fprintf(w, "  block\tcloser=%s\topen=%s\tclose=%s\tded=%v\tverb=%v\tsec=%v\n",
					q(b.Closer), q(b.Open), q(b.Close), b.IsDedent, b.IsVerbatim, b.Sections)
				for _, sg := range b.CloseSeq {
					fmt.Fprintf(w, "    seg\ttoks=%v\tref=%s\n", sg.Tokens, q(sg.Ref))
				}
			}
			if f.Lookahead != nil {
				fmt.Fprintf(w, "  lookahead\treq=%v\tforbid=%v\n", f.Lookahead.RequireIndent, f.Lookahead.ForbidIndent)
			}
			if f.TemplateAST != nil {
				fmt.Fprintf(w, "  TEMPLATE\n%s", block(*f.TemplateAST, "    "))
			}
			if f.RunAST != nil {
				fmt.Fprintf(w, "  RUN\n%s", block(*f.RunAST, "    "))
			}
		}
		mks := []string{}
		for k := range lib.Commands {
			mks = append(mks, k)
		}
		sort.Strings(mks)
		for _, k := range mks {
			c := lib.Commands[k]
			fmt.Fprintf(w, "cmd\t%s\tdesc=%s\n%s", q(c.Name), q(c.Description), block(c.Body, "  "))
		}
	}
}
