package infra

import (
	"fmt"
	"path/filepath"
	"strings"

	"github.com/olivierdevelops/capy/domain"
)

// Preprocess walks a source string line by line and expands any
// inclusion directives the LIBRARY has opted into (passed as
// `directives`). Each directive name (e.g. `"@import"`,
// `"@include"`) when seen at the start of a source line is replaced
// by the contents of the referenced file (recursively).
//
// The engine ships with NO default directives — pass an empty
// `directives` slice and Preprocess is a no-op. This keeps Capy's
// "zero predefined grammar" promise intact: even universal-looking
// constructs like `@import` are opt-in per library.
//
// Reads go through the supplied domain.Host, NOT the filesystem directly. That
// matters for the sandbox: `capy.Library` defaults to domain.NoOpHost, so an
// embedded caller (or the wasm playground) that never granted filesystem access
// cannot be made to read arbitrary files by a library declaring `@import`.
// Previously this used os.ReadFile unconditionally, which bypassed the same host
// gate that the `read_file` inner-DSL primitive respects.
//
// Path resolution: relative paths are resolved against `dir`, then made absolute
// so the host passes them through verbatim rather than re-prefixing its BaseDir.
//
// Cycle detection: imports are tracked by absolute path. If file A
// imports B which imports A, the loader stops with a clear error.
//
// The directive itself must be at the START of a line (after
// optional leading whitespace, which is preserved on the inlined
// content for visual nesting).
func Preprocess(source, dir string, directives []string, h domain.Host) (string, error) {
	if len(directives) == 0 {
		return source, nil
	}
	if h == nil {
		h = domain.NoOpHost{}
	}
	return preprocess(source, dir, directives, map[string]bool{}, h)
}

func preprocess(source, dir string, directives []string, visited map[string]bool, h domain.Host) (string, error) {
	var out strings.Builder
	lines := strings.Split(source, "\n")
	for i, line := range lines {
		// Find leading indent so imported content can be re-indented to
		// match the @import line's column. This is what authors expect:
		// imports nest inside whatever block they appear in.
		indentLen := 0
		for indentLen < len(line) && (line[indentLen] == ' ' || line[indentLen] == '\t') {
			indentLen++
		}
		trimmed := line[indentLen:]
		if d, path, ok := matchImport(trimmed, directives); ok {
			absPath := resolvePath(path, dir)
			if visited[absPath] {
				return "", fmt.Errorf("line %d: import cycle: %s", i+1, absPath)
			}
			b, err := h.ReadFile(absPath)
			if err != nil {
				return "", fmt.Errorf("line %d: %s %q: %v", i+1, d, path, err)
			}
			visited[absPath] = true
			expanded, err := preprocess(b, filepath.Dir(absPath), directives, visited, h)
			if err != nil {
				return "", err
			}
			delete(visited, absPath)
			indent := line[:indentLen]
			for j, l := range strings.Split(strings.TrimRight(expanded, "\n"), "\n") {
				if j > 0 {
					out.WriteString("\n")
				}
				if strings.TrimSpace(l) == "" {
					continue // keep blank lines blank
				}
				out.WriteString(indent)
				out.WriteString(l)
			}
			if i < len(lines)-1 {
				out.WriteString("\n")
			}
			continue
		}
		out.WriteString(line)
		if i < len(lines)-1 {
			out.WriteString("\n")
		}
	}
	return out.String(), nil
}

// matchImport recognises any directive in `directives` (e.g.
// `@import`, `@include`, or library-chosen names like `@use`) at the
// start of a line, followed by a quoted path. Returns the directive
// name, the path, and ok=true on match.
func matchImport(line string, directives []string) (string, string, bool) {
	for _, d := range directives {
		if !strings.HasPrefix(line, d) {
			continue
		}
		rest := strings.TrimSpace(line[len(d):])
		if len(rest) < 2 || rest[0] != '"' {
			continue
		}
		// Find the closing quote.
		end := strings.IndexByte(rest[1:], '"')
		if end < 0 {
			continue
		}
		path := rest[1 : 1+end]
		// Allow trailing comment after the directive.
		after := strings.TrimSpace(rest[2+end:])
		if after != "" && !strings.HasPrefix(after, "#") {
			continue
		}
		return d, path, true
	}
	return "", "", false
}

func resolvePath(path, dir string) string {
	if filepath.IsAbs(path) {
		return filepath.Clean(path)
	}
	joined := filepath.Clean(filepath.Join(dir, path))
	// Absolute, so the host reads exactly this path instead of resolving it
	// against its own BaseDir a second time.
	if abs, err := filepath.Abs(joined); err == nil {
		return abs
	}
	return joined
}
