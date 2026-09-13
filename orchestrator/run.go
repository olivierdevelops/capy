package orchestrator

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/olivierdevelops/capy/domain"
	"github.com/olivierdevelops/capy/infra"
	orchfeatures "github.com/olivierdevelops/capy/orchestrator/features"
)

// Run loads a library from disk, reads a script, and produces the transpiled
// output as a string. Intended for embedding Capy programmatically (and for
// tests).
func Run(libraryPath, scriptPath string) (string, error) {
	out, _, err := RunMulti(libraryPath, scriptPath)
	return out, err
}

// RunMulti is like Run but also returns the rendered multi-file map for
// libraries that declared `file "path":` blocks. The map is empty for
// libraries that don't use multi-file output.
func RunMulti(libraryPath, scriptPath string) (string, map[string]string, error) {
	return RunMultiWithArgs(libraryPath, scriptPath, nil)
}

// stripShebang removes a leading `#!` line if present. Lets scripts
// be made executable via `#!/usr/bin/env capy --lib X` without
// confusing the lexer with the shebang line.
func stripShebang(src []byte) []byte {
	if len(src) < 2 || src[0] != '#' || src[1] != '!' {
		return src
	}
	for i := 0; i < len(src); i++ {
		if src[i] == '\n' {
			return src[i+1:]
		}
	}
	return nil
}

// RunMultiWithArgs is like RunMulti but also passes positional CLI args
// through to the inner `arg`/`args`/`arg_count` host primitives. The CLI
// uses this so `capy run lib.capy script.capy a b c` makes "a", "b",
// "c" visible to `arg 0`, `arg 1`, `arg 2`.
func RunMultiWithArgs(libraryPath, scriptPath string, userArgs []string) (string, map[string]string, error) {
	rawSrc, err := os.ReadFile(scriptPath)
	if err != nil {
		return "", nil, err
	}
	src := stripShebang(rawSrc)
	host := infra.OSHost{UserArgs: userArgs, BaseDir: filepath.Dir(scriptPath)}
	_ = domain.Host(host) // compile-time interface check

	lex := orchfeatures.MakeLexer()
	parser := orchfeatures.MakeParser()
	eval := orchfeatures.MakeEvaluatorWithHost(host)

	// Library FIRST — we need its `preprocess` declarations before we
	// can know which (if any) source-level inclusion directives are
	// allowed. Capy has no built-in preprocessor; everything is opt-in
	// per library.
	libLoader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	lib, err := libLoader.Load(libraryPath)
	if err != nil {
		return "", nil, err
	}

	// Now expand any inclusion directives the library declared. With
	// no `preprocess` block in the library, Preprocess returns the
	// source unchanged.
	expanded, err := infra.Preprocess(string(src), filepath.Dir(scriptPath), lib.Preprocess, host)
	if err != nil {
		return "", nil, err
	}

	// Extract any `define NAME ... end` blocks (metaprogramming): the
	// source can introduce new functions for the rest of itself to use.
	// `cleaned` is the source with the defines stripped; `defineLibSrc`
	// is a synthetic `.capy` library text the loader can compile.
	cleaned, defineLibSrc, err := infra.ExtractDefines(expanded)
	if err != nil {
		return "", nil, err
	}
	expanded = cleaned
	// Merge source-defined functions into the library. Source defines
	// WIN on conflict — `define foo ... end` in the script overrides
	// `function foo` from the library.
	if defineLibSrc != "" {
		defineLib, err := orchfeatures.LoadLibraryFromBytes("capy", []byte(defineLibSrc), lex.Tokenize)
		if err != nil {
			return "", nil, fmt.Errorf("define block: %v", err)
		}
		for name, fn := range defineLib.Functions {
			lib.Functions[name] = fn
		}
	}
	toks, err := lex.TokenizeWith(expanded, lib.Comments)
	if err != nil {
		return "", nil, err
	}
	prog, err := parser.Parse(toks, expanded, lib)
	if err != nil {
		return "", nil, err
	}
	return eval.RunMulti(prog, lib)
}

// RunStrings is like Run but takes the library and script contents directly.
// `libraryPath` is used only to resolve relative paths inside `import`
// directives — pass an empty string if your library has none.
func RunStrings(librarySrc, libraryPath, scriptSrc string) (string, error) {
	lex := orchfeatures.MakeLexer()
	parser := orchfeatures.MakeParser()
	eval := orchfeatures.MakeEvaluator()

	// Spill the library source to a temp file so the loader's file-
	// based API (which resolves `import` paths relative to the file)
	// has a stable basis. Most production callers should pass a real
	// libraryPath via Run() above instead.
	if libraryPath == "" {
		tmp, err := os.CreateTemp("", "capy-lib-*.capy")
		if err != nil {
			return "", err
		}
		defer os.Remove(tmp.Name())
		if _, err := tmp.WriteString(librarySrc); err != nil {
			return "", err
		}
		tmp.Close()
		libraryPath = tmp.Name()
	}
	_ = filepath.Base // reserved for future use

	libLoader := orchfeatures.MakeLibraryLoader(lex.Tokenize)
	lib, err := libLoader.Load(libraryPath)
	if err != nil {
		return "", err
	}
	toks, err := lex.TokenizeWith(scriptSrc, lib.Comments)
	if err != nil {
		return "", err
	}
	prog, err := parser.Parse(toks, scriptSrc, lib)
	if err != nil {
		return "", err
	}
	return eval.Run(prog, lib)
}
