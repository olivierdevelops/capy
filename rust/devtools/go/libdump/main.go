// libdump is a differential-testing aid for the Rust port: it parses a .capy
// library file and prints the resulting RawLibrary canonically.
// Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/olivierdevelops/capy/infra"
)

func q(s string) string { return strconv.Quote(s) }

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
		keys := []string{}
		for k := range x {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		parts := []string{}
		for _, k := range keys {
			parts = append(parts, q(k)+":"+ctxVal(x[k]))
		}
		return "m:{" + strings.Join(parts, ",") + "}"
	}
	return "?"
}

func keys[V any](m map[string]V) []string {
	ks := make([]string, 0, len(m))
	for k := range m {
		ks = append(ks, k)
	}
	sort.Strings(ks)
	return ks
}

func main() {
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	var p infra.CapyLibParser
	for _, path := range os.Args[1:] {
		fmt.Fprintf(w, "FILE\t%s\n", path)
		lib, err := p.ParseFile(path)
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		fmt.Fprintf(w, "ext\t%s\noutfile\t%s\ndesc\t%s\nlibname\t%s\nlibver\t%s\ndefimpl\t%s\n",
			q(lib.Extension), q(lib.OutputFile), q(lib.Description), q(lib.LibName),
			q(lib.LibVersion), q(lib.DefaultImpl))
		fmt.Fprintf(w, "imports\t%v\npreprocess\t%v\ncomments\t%v\n",
			lib.Imports, lib.Preprocess, lib.Comments)
		fmt.Fprintf(w, "filetemplate\t%s\n", q(lib.FileTemplate))
		for _, k := range keys(lib.Files) {
			fmt.Fprintf(w, "file\t%s\t%s\n", q(k), q(lib.Files[k]))
		}
		for _, k := range keys(lib.Context) {
			fmt.Fprintf(w, "ctx\t%s\t%s\n", q(k), ctxVal(lib.Context[k]))
		}
		for _, k := range keys(lib.Types) {
			t := lib.Types[k]
			fmt.Fprintf(w, "type\t%s\tdesc=%s\tbase=%s\tpat=%s\topts=%v\tgo=%s\tgc=%s\n",
				q(k), q(t.Description), q(t.Base), q(t.Pattern), t.Options,
				q(t.GroupOpen), q(t.GroupClose))
		}
		for _, k := range keys(lib.Impls) {
			im := lib.Impls[k]
			fmt.Fprintf(w, "impl\t%s\tname=%s\tfile=%s\tdesc=%s\tver=%s\tdef=%v\n",
				q(k), q(im.Name), q(im.File), q(im.Description), q(im.Version), im.IsDefault)
		}
		for _, k := range keys(lib.Commands) {
			c := lib.Commands[k]
			fmt.Fprintf(w, "cmd\t%s\tdesc=%s\tbody=%s\n", q(k), q(c.Description), q(c.Body))
			for _, a := range c.Args {
				fmt.Fprintf(w, "  cmdarg\tname=%s\treq=%v\tdesc=%s\n", q(a.Name), a.Required, q(a.Description))
			}
			for _, fl := range c.Flags {
				fmt.Fprintf(w, "  cmdflag\tname=%s\tdesc=%s\tdef=%s\tbool=%v\n",
					q(fl.Name), q(fl.Description), q(fl.Default), fl.IsBool)
			}
		}
		for _, k := range keys(lib.Functions) {
			f := lib.Functions[k]
			fmt.Fprintf(w, "fn\t%s\tdesc=%s\tprio=%d\tbare=%v\tfbi=%v\tnfbi=%v\n",
				q(k), q(f.Description), f.Priority, f.Bare, f.FollowedByIndent, f.NotFollowedByIndent)
			fmt.Fprintf(w, "  body\t%s\n", q(f.Body))
			for _, a := range f.Args {
				fmt.Fprintf(w, "  arg\tkind=%s\tval=%s\tname=%s\ttype=%s\tdesc=%s\topt=%v\tdef=%s\trep=%s\tsep=%s\tjoin=%s\n",
					q(a.Kind), q(a.Value), q(a.Name), q(a.Type), q(a.Description),
					a.Optional, q(a.Default), q(a.Repeat), q(a.Sep), q(a.Join))
			}
			if f.Block != nil {
				b := f.Block
				fmt.Fprintf(w, "  block\tcloser=%s\topen=%s\tclose=%s\tded=%v\tverb=%v\tsections=%v\n",
					q(b.Closer), q(b.Open), q(b.Close), b.IsDedent, b.IsVerbatim, b.Sections)
				for _, sg := range b.CloseSeq {
					fmt.Fprintf(w, "    seg\ttext=%s\tref=%v\n", q(sg.Text), sg.IsRef)
				}
			}
		}
	}
}
