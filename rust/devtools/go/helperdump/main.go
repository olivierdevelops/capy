// helperdump is a differential-testing aid for the Rust port: it calls every
// built-in template helper over a fixed corpus and prints canonical results.
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

// helperNames must mirror the `funcs` map in infra/helpers.go.
var helperNames = []string{
	"indent", "lower", "upper", "pascalCase", "camelCase", "snakeCase",
	"dasherize", "unquote", "unescape", "trimSuffix", "trimPrefix", "join",
	"split", "nonEmpty", "toQuoted", "escapeHtml", "decoded", "asString",
	"toPyLit", "toJSON", "toJSONIndent", "add", "sub", "mul", "div", "mod",
	"align", "percent", "stars",
}

var corpus = []any{
	nil,
	"",
	"hello world",
	`"quoted"`,
	`'single'`,
	"`backtick`",
	"Habit Tracker",
	"habit-tracker_name.thing",
	"camelCaseAlready",
	`a<b>&c"d'e`,
	"line1\nline2",
	`He said \"hi\"`,
	"tab\there",
	`"esc\nseq"`,
	"héllo wörld",
	"日本語",
	"ß İ ﬁ",
	"42", "-17", "3.5", " 42 ", "not_a_number",
	int64(7), int64(-3), int64(0),
	float64(1.5), float64(1000000),
	true, false,
	[]any{"a", "b", "c"},
	[]string{"a", "", "  ", "b"},
	[]any{int64(1), "two", nil, true},
	map[string]any{"b": int64(2), "a": "x"},
	string([]byte{92}) + "u00e9 and " + string([]byte{92}) + "x41",
	"0",
	"9223372036854775808",
}

func repr(v any) string {
	switch x := v.(type) {
	case nil:
		return "nil"
	case string:
		return "s:" + strconv.Quote(x)
	case int:
		return "i:" + strconv.Itoa(x)
	case int64:
		return "i:" + strconv.FormatInt(x, 10)
	case float64:
		return "f:" + strconv.FormatFloat(x, 'g', -1, 64)
	case bool:
		return "b:" + strconv.FormatBool(x)
	case []string:
		parts := make([]string, 0, len(x))
		for _, s := range x {
			parts = append(parts, strconv.Quote(s))
		}
		return "ss:[" + strings.Join(parts, ",") + "]"
	case []any:
		parts := make([]string, 0, len(x))
		for _, it := range x {
			parts = append(parts, repr(it))
		}
		return "as:[" + strings.Join(parts, ",") + "]"
	case map[string]any:
		keys := make([]string, 0, len(x))
		for k := range x {
			keys = append(keys, k)
		}
		sort.Strings(keys)
		parts := make([]string, 0, len(keys))
		for _, k := range keys {
			parts = append(parts, strconv.Quote(k)+":"+repr(x[k]))
		}
		return "m:{" + strings.Join(parts, ",") + "}"
	}
	return "?:" + fmt.Sprintf("%v", v)
}

// pairIdx is the curated index set used for 2-arity calls.
var pairLeft = []int{0, 1, 2, 9, 10, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 29, 30, 32}
var pairRight = []int{1, 2, 17, 22, 24, 29, 30}

func main() {
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	emit := func(name string, args []any, res any, ok bool, err error) {
		reprs := make([]string, 0, len(args))
		for _, a := range args {
			reprs = append(reprs, repr(a))
		}
		es := ""
		if err != nil {
			es = err.Error()
		}
		rs := "-"
		if res != nil {
			rs = repr(res)
		}
		fmt.Fprintf(w, "H\t%s\t[%s]\tok=%v\terr=%s\tres=%s\n",
			name, strings.Join(reprs, " | "), ok, strconv.Quote(es), rs)
	}
	for _, name := range helperNames {
		// 1-arity over the whole corpus.
		for _, v := range corpus {
			args := []any{v}
			res, ok, err := infra.ApplyHelper(name, args)
			emit(name, args, res, ok, err)
		}
		// 2-arity over the curated pair grid.
		for _, i := range pairLeft {
			for _, j := range pairRight {
				args := []any{corpus[i], corpus[j]}
				res, ok, err := infra.ApplyHelper(name, args)
				emit(name, args, res, ok, err)
			}
		}
		// 0-arity and 3-arity, to exercise the arity errors.
		for _, args := range [][]any{{}, {corpus[2], corpus[2], corpus[2]}} {
			res, ok, err := infra.ApplyHelper(name, args)
			emit(name, args, res, ok, err)
		}
	}
	// An unknown name must report ok=false.
	res, ok, err := infra.ApplyHelper("definitelyNotAHelper", []any{"x"})
	emit("definitelyNotAHelper", []any{"x"}, res, ok, err)
}
