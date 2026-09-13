// lexdump is a differential-testing aid for the Rust port: it dumps the
// token stream the Go lexer produces, in a stable line format the Rust
// lexdump reproduces exactly. Not part of the shipped CLI.
package main

import (
	"bufio"
	"fmt"
	"os"

	"github.com/olivierdevelops/capy/domain"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

func kindName(k domain.TokenKind) string {
	switch k {
	case domain.TokIdent:
		return "IDENT"
	case domain.TokNumber:
		return "NUMBER"
	case domain.TokString:
		return "STRING"
	case domain.TokTemplate:
		return "TEMPLATE"
	case domain.TokPunct:
		return "PUNCT"
	case domain.TokLParen:
		return "LPAREN"
	case domain.TokRParen:
		return "RPAREN"
	case domain.TokLBrace:
		return "LBRACE"
	case domain.TokRBrace:
		return "RBRACE"
	case domain.TokLBrack:
		return "LBRACK"
	case domain.TokRBrack:
		return "RBRACK"
	case domain.TokNewline:
		return "NEWLINE"
	case domain.TokIndent:
		return "INDENT"
	case domain.TokDedent:
		return "DEDENT"
	case domain.TokEOF:
		return "EOF"
	}
	return "?"
}

func main() {
	lex := orchfeatures.MakeLexer()
	w := bufio.NewWriter(os.Stdout)
	defer w.Flush()
	for _, path := range os.Args[1:] {
		b, err := os.ReadFile(path)
		if err != nil {
			fmt.Fprintf(w, "FILE\t%s\nREADERR\t%v\n", path, err)
			continue
		}
		fmt.Fprintf(w, "FILE\t%s\n", path)
		toks, err := lex.Tokenize(string(b))
		if err != nil {
			fmt.Fprintf(w, "ERR\t%s\n", err.Error())
			continue
		}
		for _, t := range toks {
			fmt.Fprintf(w, "T\t%s\t%q\t%d\t%d\t%d\n", kindName(t.Kind), t.Text, t.Line, t.Col, t.Width)
		}
	}
}
