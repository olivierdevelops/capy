package main

import (
	"flag"
	"fmt"
	"os"
	"strings"
)

// cmdFmt is a minimal `.capy` formatter. v0.20 scope: enforce a
// few low-cost normalisations that catch the most common style
// drift; not yet a full canonical-form normaliser (that needs a
// proper round-trip through the AST and is deferred per design
// § 8.2).
//
// What this version does:
//   - Strips trailing whitespace from every line.
//   - Replaces hard-tab indentation with 4 spaces.
//   - Collapses runs of more than one blank line to exactly one.
//   - Ensures the file ends with exactly one trailing newline.
//
// What it does NOT do (yet):
//   - Re-order top-level declarations (extension → context → types
//     → functions → file_template → commands).
//   - Re-align arg lines within functions.
//   - Re-flow templates inside `write` backticks (significant
//     whitespace; would change output).
//
// Usage:
//
//	capy fmt <file.capy>           # rewrite in place
//	capy fmt --check <file.capy>   # exit 1 if not formatted
//	capy fmt --diff <file.capy>    # print diff (vs. formatted)
//	capy fmt --stdout <file.capy>  # print formatted to stdout
func cmdFmt(args []string) error {
	fs := flag.NewFlagSet("fmt", flag.ContinueOnError)
	check := fs.Bool("check", false, "exit 1 if file isn't formatted; don't modify")
	diff := fs.Bool("diff", false, "print diff between current and formatted output")
	toStdout := fs.Bool("stdout", false, "print formatted output to stdout instead of writing")
	if err := fs.Parse(args); err != nil {
		return err
	}
	files := fs.Args()
	if len(files) == 0 {
		return fmt.Errorf("usage: capy fmt [--check | --diff | --stdout] <file.capy>...")
	}
	anyChanged := false
	for _, path := range files {
		raw, err := os.ReadFile(path)
		if err != nil {
			return fmt.Errorf("read %s: %v", path, err)
		}
		formatted := formatCapy(string(raw))
		if formatted == string(raw) {
			continue
		}
		anyChanged = true
		switch {
		case *check:
			fmt.Fprintf(os.Stderr, "would reformat %s\n", path)
		case *diff:
			fmt.Println(simpleDiff(string(raw), formatted))
		case *toStdout:
			fmt.Print(formatted)
		default:
			if err := os.WriteFile(path, []byte(formatted), 0644); err != nil {
				return fmt.Errorf("write %s: %v", path, err)
			}
			fmt.Fprintf(os.Stderr, "formatted %s\n", path)
		}
	}
	if *check && anyChanged {
		os.Exit(1)
	}
	return nil
}

// formatCapy applies the conservative normalisation rules described
// in cmdFmt's doc. We avoid touching the inside of backtick literals
// (where whitespace is part of the emitted output).
func formatCapy(src string) string {
	// Walk character-by-character, tracking when we're inside a
	// backtick literal so we can skip our normalisations there.
	var out strings.Builder
	inBacktick := false
	var line strings.Builder
	var lines []string
	flushLine := func() {
		lines = append(lines, line.String())
		line.Reset()
	}
	for i := 0; i < len(src); i++ {
		c := src[i]
		if inBacktick {
			line.WriteByte(c)
			if c == '\\' && i+1 < len(src) {
				line.WriteByte(src[i+1])
				i++
				continue
			}
			if c == '`' {
				inBacktick = false
			}
			if c == '\n' {
				// Drop the newline from the stored line: flushLine() records the
				// line break itself, and the emit loop re-adds exactly one. Keeping
				// it here appended a second newline on every pass, so `capy fmt`
				// grew the file each run and never converged.
				cur := line.String()
				line.Reset()
				line.WriteString(strings.TrimSuffix(cur, "\n"))
				flushLine()
			}
			continue
		}
		if c == '`' {
			inBacktick = true
			line.WriteByte(c)
			continue
		}
		if c == '\n' {
			flushLine()
			continue
		}
		line.WriteByte(c)
	}
	if line.Len() > 0 {
		flushLine()
	}
	// Apply per-line rules — but only where the text is NOT inside a
	// multi-line backtick, since whitespace there is part of the emitted
	// output. Two states matter per line:
	//
	//	startInBacktick — the line's leading text is literal content
	//	endInBacktick   — the line's TRAILING text is literal content
	//	                  (true for the line that OPENS a backtick)
	//
	// Leading-indent normalisation is safe whenever the line starts outside a
	// literal; trailing-space stripping is only safe when the line both starts
	// and ends outside one. Getting that second condition wrong silently eats
	// significant whitespace on the `write \`…` opening line.
	inBacktick = false
	prevBlank := false
	for i, ln := range lines {
		startInBacktick := inBacktick
		// Advance the backtick state over this line's unescaped backticks.
		for j := 0; j < len(ln); j++ {
			if ln[j] == '\\' && j+1 < len(ln) {
				j++
				continue
			}
			if ln[j] == '`' {
				inBacktick = !inBacktick
			}
		}
		endInBacktick := inBacktick
		if !startInBacktick {
			ln = tabsToSpaces(ln, 4)
			if !endInBacktick {
				ln = stripTrailingSpaces(ln)
			}
		}
		// Skip extra consecutive blank lines (only outside backticks).
		if !startInBacktick && !endInBacktick && strings.TrimSpace(ln) == "" {
			if prevBlank {
				continue
			}
			prevBlank = true
		} else {
			prevBlank = false
		}
		out.WriteString(ln)
		if i < len(lines)-1 || ln != "" {
			out.WriteByte('\n')
		}
	}
	// Ensure exactly one trailing newline.
	s := out.String()
	s = strings.TrimRight(s, "\n") + "\n"
	return s
}

func stripTrailingSpaces(line string) string {
	return strings.TrimRight(line, " \t")
}

func tabsToSpaces(line string, width int) string {
	if !strings.ContainsRune(line, '\t') {
		return line
	}
	// Only convert leading tabs (don't touch tabs inside strings).
	i := 0
	for i < len(line) && (line[i] == ' ' || line[i] == '\t') {
		i++
	}
	prefix := line[:i]
	rest := line[i:]
	prefix = strings.ReplaceAll(prefix, "\t", strings.Repeat(" ", width))
	return prefix + rest
}

// simpleDiff returns a unified-ish diff between a and b, line by
// line, with `-` / `+` markers. Not a real diff algorithm; just
// shows pre/post for short edits.
func simpleDiff(a, b string) string {
	la := strings.Split(a, "\n")
	lb := strings.Split(b, "\n")
	var out strings.Builder
	n := len(la)
	if len(lb) > n {
		n = len(lb)
	}
	for i := 0; i < n; i++ {
		var ax, bx string
		if i < len(la) {
			ax = la[i]
		}
		if i < len(lb) {
			bx = lb[i]
		}
		if ax == bx {
			out.WriteString("  " + ax + "\n")
			continue
		}
		if ax != "" {
			out.WriteString("- " + ax + "\n")
		}
		if bx != "" {
			out.WriteString("+ " + bx + "\n")
		}
	}
	return out.String()
}
