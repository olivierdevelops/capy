package infra

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// helper: write a temp file with content; return absolute path.
func writeTmp(t *testing.T, dir, name, content string) string {
	t.Helper()
	p := filepath.Join(dir, name)
	if err := os.MkdirAll(filepath.Dir(p), 0755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(p, []byte(content), 0644); err != nil {
		t.Fatal(err)
	}
	return p
}

func TestPreprocess_NoImports(t *testing.T) {
	got, err := Preprocess("hello\nworld\n", "/tmp", []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if got != "hello\nworld\n" {
		t.Errorf("expected passthrough, got %q", got)
	}
}

func TestPreprocess_BasicImport(t *testing.T) {
	dir := t.TempDir()
	writeTmp(t, dir, "shared.capy", "shared line one\nshared line two\n")
	main := "before\n@import \"shared.capy\"\nafter\n"
	got, err := Preprocess(main, dir, []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	want := "before\nshared line one\nshared line two\nafter\n"
	if got != want {
		t.Errorf("got:\n%s\nwant:\n%s", got, want)
	}
}

func TestPreprocess_IndentationPreserved(t *testing.T) {
	dir := t.TempDir()
	writeTmp(t, dir, "inner.capy", "first\nsecond\n")
	main := "outer\n    @import \"inner.capy\"\n"
	got, err := Preprocess(main, dir, []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	// Each non-blank line of the import should have 4 leading spaces.
	for _, line := range strings.Split(got, "\n") {
		if line == "" || line == "outer" {
			continue
		}
		if !strings.HasPrefix(line, "    ") {
			t.Errorf("expected 4-space indent, got %q", line)
		}
	}
}

func TestPreprocess_NestedImport(t *testing.T) {
	dir := t.TempDir()
	writeTmp(t, dir, "a.capy", "from-a\n@import \"b.capy\"\nback-from-a\n")
	writeTmp(t, dir, "b.capy", "from-b\n")
	got, err := Preprocess("@import \"a.capy\"\n", dir, []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(got, "from-a") || !strings.Contains(got, "from-b") || !strings.Contains(got, "back-from-a") {
		t.Errorf("nested import didn't expand both files:\n%s", got)
	}
}

func TestPreprocess_DetectsCycle(t *testing.T) {
	dir := t.TempDir()
	writeTmp(t, dir, "a.capy", "@import \"b.capy\"\n")
	writeTmp(t, dir, "b.capy", "@import \"a.capy\"\n")
	_, err := Preprocess("@import \"a.capy\"\n", dir, []string{"@import", "@include"}, OSHost{})
	if err == nil || !strings.Contains(err.Error(), "cycle") {
		t.Errorf("expected cycle error, got %v", err)
	}
}

func TestPreprocess_MissingFile(t *testing.T) {
	_, err := Preprocess("@import \"nope.capy\"\n", t.TempDir(), []string{"@import", "@include"}, OSHost{})
	if err == nil {
		t.Error("expected error for missing file")
	}
}

func TestPreprocess_IncludeIsSynonym(t *testing.T) {
	dir := t.TempDir()
	writeTmp(t, dir, "x.capy", "included\n")
	got, err := Preprocess("@include \"x.capy\"\n", dir, []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(got, "included") {
		t.Errorf("@include didn't expand: %q", got)
	}
}

func TestPreprocess_LinesWithoutImportAreUntouched(t *testing.T) {
	// "@import 'no quotes around path'" should NOT be treated as an import.
	src := "@import broken\nother line\n"
	got, err := Preprocess(src, t.TempDir(), []string{"@import", "@include"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if got != src {
		t.Errorf("malformed import should be passthrough, got %q", got)
	}
}

func TestPreprocess_NoDirectivesIsNoOp(t *testing.T) {
	// With no library-declared directives, every `@import` line
	// passes through untouched — even if the file would exist.
	dir := t.TempDir()
	writeTmp(t, dir, "x.capy", "should-not-appear\n")
	src := "@import \"x.capy\"\nkept\n"
	got, err := Preprocess(src, dir, nil, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if got != src {
		t.Errorf("expected source unchanged when no directives declared; got %q", got)
	}
}

func TestPreprocess_UnknownDirectiveIgnored(t *testing.T) {
	// Library declares only @use; @import lines stay as plain text.
	dir := t.TempDir()
	writeTmp(t, dir, "x.capy", "inlined\n")
	src := "@import \"x.capy\"\n@use \"x.capy\"\n"
	got, err := Preprocess(src, dir, []string{"@use"}, OSHost{})
	if err != nil {
		t.Fatal(err)
	}
	if !contains(got, "@import \"x.capy\"") {
		t.Errorf("@import should be passthrough when not in directives; got %q", got)
	}
	if !contains(got, "inlined") {
		t.Errorf("@use should expand; got %q", got)
	}
}

func contains(s, sub string) bool {
	for i := 0; i+len(sub) <= len(s); i++ {
		if s[i:i+len(sub)] == sub {
			return true
		}
	}
	return false
}
