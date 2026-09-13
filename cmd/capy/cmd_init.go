package main

import (
	"fmt"
	"os"
	"path/filepath"
)

// `capy init [<dir>]` scaffolds a starter library + script in the target dir
// (default `.`). Refuses to overwrite existing files.
func cmdInit(args []string) error {
	dir := "."
	if len(args) > 0 {
		dir = args[0]
	}
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	// An ORDERED list, not a map: map iteration is randomised, so both the
	// "created …" order and — worse — which file the "refusing to overwrite"
	// error named varied per run, making the failure impossible to assert on.
	files := []struct{ name, content string }{
		{"lib.yaml", starterLib},
		{"script.capy", starterScript},
		{"README.md", starterReadme},
	}
	for _, f := range files {
		name, content := f.name, f.content
		p := filepath.Join(dir, name)
		if _, err := os.Stat(p); err == nil {
			return fmt.Errorf("refusing to overwrite existing file: %s", p)
		}
		if err := os.WriteFile(p, []byte(content), 0644); err != nil {
			return err
		}
		fmt.Println("created", p)
	}
	fmt.Println()
	fmt.Println("Next:")
	fmt.Println("  capy run", filepath.Join(dir, "lib.yaml"), filepath.Join(dir, "script.capy"))
	return nil
}

const starterLib = `# A starter Capy library. Edit to define your own source language.
extension: txt

context:
  lines: []

functions:
  say:
    args:
      - { kind: capture, name: msg, type: any }
    template: "say {{ .msg }}\n"
    run: |
      append context.lines msg

file_template: |
  {{- .body -}}
  --- captured {{ len .context.lines }} line(s) ---
`

const starterScript = `say "hello, world"
say "this is a starter script"
`

const starterReadme = `# My Capy project

Run:

    capy run lib.yaml script.capy

Edit ` + "`lib.yaml`" + ` to define your own functions, types, and file template.
See https://github.com/olivierdevelops/capy for documentation.
`
