> **HISTORICAL DESIGN RECORD.** Written while the engine was implemented in Go.
> File paths and language references below describe that codebase; the engine is
> now Rust under `rust/`. Kept for the reasoning, not as current documentation.

# Examples — libraries under the unified `write` design

Companion to [`unified-write-block.md`](unified-write-block.md). Four
real libraries from `samples/` rewritten in the proposed shape, with
the current versions next to them.

The goal isn't to argue the design (that's the other doc); it's to
let you see what real authoring would feel like. Look for:

- How `run:` and `template:` collapse into one body.
- Whether the new shape is actually shorter / longer / clearer.
- What `text/template`-isms (`{{- range }}`, `{{ if eq … }}`,
  `{{ . | filter }}`) become in the new world.
- Where the rough edges are.

> **Status note.** Nothing here is implemented. Code that says
> `write \`…\`` is illustrative; the engine does not parse it
> today. The samples below use the current engine, so the
> "After" blocks are the proposed mental model, not runnable code.

---

## 1. `samples/transpile-py/lib.capy` — the canonical sample

A small imperative DSL → Python: `import`, `say`, `<ident> = …`,
`if`, `loop`. Hits every ingredient: stateful function (`import`),
output-only function (`say`), operator-shape (`assign`), block
openers (`if` / `loop`), and a file template that floats imports
to the top.

### Before (current — 70 lines)

```
extension py

context
    imports []
end

type Identifier
    base any
    pattern "^[A-Za-z_][A-Za-z0-9_]*$"
end

function import
    arg literal "import"
    arg capture name Identifier
    template_str ""
    run:
        append context.imports name
end

function say
    arg literal "say"
    arg capture msg any
    template_str "print({{ .msg }})\n"
end

function assign
    arg capture name Identifier
    arg literal "="
    arg capture value any
    template_str "{{ .name }} = {{ .value }}\n"
end

function if
    arg literal "if"
    arg capture cond any
    block_closer end
    template:
        if {{ .cond }}:
        {{ .body | indent 4 }}
end

function loop
    arg literal "loop"
    arg capture var ident
    arg literal "in"
    arg capture iter any
    block_closer end
    template:
        for {{ .var }} in {{ .iter }}:
        {{ .body | indent 4 }}
end

function end
end

file_template:
    {{- range .context.imports }}import {{ . }}
    {{ end }}
    {{- .body -}}
```

### After (proposed — 56 lines)

```
extension py

context
    imports []
end

type Identifier
    base any
    pattern "^[A-Za-z_][A-Za-z0-9_]*$"
end

function import
    arg literal "import"
    arg capture name Identifier
    append context.imports name
end

function say
    arg literal "say"
    arg capture msg any
    write `print(${msg})
`
end

function assign
    arg capture name Identifier
    arg literal "="
    arg capture value any
    write `${name} = ${value}
`
end

function if
    arg literal "if"
    arg capture cond any
    block_closer end
    write `if ${cond}:
${indent 4 body}`
end

function loop
    arg literal "loop"
    arg capture var ident
    arg literal "in"
    arg capture iter any
    block_closer end
    write `for ${var} in ${iter}:
${indent 4 body}`
end

function end
end

file_template
    for imp in context.imports
        write `import ${imp}
`
    end
    write body
end
```

**What changed.**

- `template_str ""` + `run: append context.imports name` →
  one `append` statement. The "empty template" boilerplate is gone.
- `template_str "print({{ .msg }})\n"` → `write \`print(${msg})\n\``.
  Same number of bytes, but the shell is a real function call to
  `write`, not a different sublanguage.
- `{{ .body | indent 4 }}` → `${indent 4 body}`. Same primitive,
  prefix-call syntax.
- `file_template` is a block, not a string. The `{{- range }} …
  {{ end }}` template incantation becomes a regular Capy `for … end`
  wrapping a `write` call. No `{{- -}}` whitespace dance — the
  newline is just inside the backticks where you put it.
- `write body` (no backticks needed — `body` is just a value)
  emits the concatenated top-level body.

14 lines saved across the file. The bigger win is that there is one
language: `write` is a statement, helpers are functions, control
flow is `for`/`if`/`end`.

---

## 2. `samples/source-imports/lib.capy` — block functions w/ Markdown

A menu DSL with three levels of nesting (`menu` > `section` >
`item`), multi-line templates that emit Markdown headings and blank
lines.

### Before (current — 47 lines, omitting `preprocess` block for brevity)

```
extension md

context
    name ""
    sections []
end

function menu
    arg literal "menu"
    arg capture name string
    block_closer end
    template:
        # {{ .name | unquote }}
        {{ .body }}
    run:
        set context.name name
end

function section
    arg literal "section"
    arg capture name string
    block_closer end
    template:

        ## {{ .name | unquote }}

        {{ .body }}
end

function item
    arg literal "item"
    arg capture name string
    arg capture price string
    template_str "- **{{ .name | unquote }}** — {{ .price | unquote }}\n"
end

function note
    arg literal "note"
    arg capture text string
    template:
        > {{ .text | unquote }}

end

function end
end
```

### After (proposed)

```
extension md

context
    name ""
    sections []
end

function menu
    arg literal "menu"
    arg capture name string
    block_closer end
    set context.name name
    write `# ${unquote name}
${body}`
end

function section
    arg literal "section"
    arg capture name string
    block_closer end
    write `
## ${unquote name}

${body}`
end

function item
    arg literal "item"
    arg capture name string
    arg capture price string
    write `- **${unquote name}** — ${unquote price}
`
end

function note
    arg literal "note"
    arg capture text string
    write `> ${unquote text}

`
end

function end
end
```

**What changed.**

- `template:` blocks become `write \`…\`` calls with literal
  newlines where you want them (the leading blank line in `section`
  is now an actual newline at the start of the backtick body).
- `{{ .name | unquote }}` → `${unquote name}`. Reads the same way.
- `menu` had both `template:` and `run:`; the `set` statement now
  lives in the function body before the `write` call. The author
  can also flip the order: state-first or output-first, whichever
  reads better.
- No engineer ever asks "wait, is `template:` rendered before or
  after `run:`?" — there's just one body, top-down.

The "blank line is significant" cost is unchanged: you still have
to put exactly the bytes you want in the output. Backticks make
that easier to see than Go's template trimming rules.

---

## 3. `samples/multi-target-ws-server/lib-go.capy` — nested control flow

The hard test. The file template iterates handlers, and for each
handler dispatches on `action` with a 3-way if/else-if/else. This
is where Go `text/template` syntax gets dense (`{{- range }}`,
`{{- if eq … }}`, `{{- else if eq … }}`, `{{- end }}`).

### Before (current — file_template excerpt only)

```
file_template:
    // Generated by Capy.
    package main

    import (
        "log"
        "net/http"
        "strings"

        "github.com/gorilla/websocket"
    )

    var upgrader = websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}

    func wsHandler(w http.ResponseWriter, r *http.Request) {
        conn, err := upgrader.Upgrade(w, r, nil)
        if err != nil { log.Println(err); return }
        defer conn.Close()
        for {
            _, msg, err := conn.ReadMessage()
            if err != nil { return }
            text := string(msg)
            switch {
            {{- range .context.handlers }}
            case strings.HasPrefix(text, "{{ .name }} "):
                payload := strings.TrimPrefix(text, "{{ .name }} ")
                {{- if eq .action "reply" }}
                _ = conn.WriteMessage(websocket.TextMessage, []byte("pong "+payload))
                {{- else if eq .action "uppercase" }}
                _ = conn.WriteMessage(websocket.TextMessage, []byte(strings.ToUpper(payload)))
                {{- else if eq .action "close" }}
                return
                {{- end }}
            {{- end }}
            default:
                _ = conn.WriteMessage(websocket.TextMessage, []byte("unknown command"))
            }
        }
    }
```

### After (proposed)

```
file_template
    write `// Generated by Capy.
package main

import (
    "log"
    "net/http"
    "strings"

    "github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{CheckOrigin: func(r *http.Request) bool { return true }}

func wsHandler(w http.ResponseWriter, r *http.Request) {
    conn, err := upgrader.Upgrade(w, r, nil)
    if err != nil { log.Println(err); return }
    defer conn.Close()
    for {
        _, msg, err := conn.ReadMessage()
        if err != nil { return }
        text := string(msg)
        switch {
`
    for h in context.handlers
        write `        case strings.HasPrefix(text, "${h.name} "):
            payload := strings.TrimPrefix(text, "${h.name} ")
`
        if eq h.action "reply"
            write `            _ = conn.WriteMessage(websocket.TextMessage, []byte("pong "+payload))
`
        end
        if eq h.action "uppercase"
            write `            _ = conn.WriteMessage(websocket.TextMessage, []byte(strings.ToUpper(payload)))
`
        end
        if eq h.action "close"
            write `            return
`
        end
    end
    write `        default:
            _ = conn.WriteMessage(websocket.TextMessage, []byte("unknown command"))
        }
    }
}
`
end
```

**What changed.**

- The "static prologue" (everything before the `switch`) is one
  big `write` call with raw multi-line content. No interpolation,
  no escaping — the bytes are just there.
- The dynamic part is a regular `for` loop over `context.handlers`,
  with `if eq h.action "X"` branches inside it.
- No `{{- … }}` whitespace trimming. Where the current template
  uses `{{- if eq .action "reply" }}` to swallow whitespace, the
  new shape just writes the line with its own newline at the end.

**The honest trade-off.** Three separate `if` statements
(replacing the current `if/else-if/else if/end` chain) is more
typing than a switch. This is the strongest argument for the
[open question on `else`](unified-write-block.md#open-questions):
adding an `else` arm makes this collapse to one `if/else if/else if`
chain. Either way, the structure is the same as the host language
the author already knows — no "wait, what's the template
equivalent of else-if?" lookup.

---

## 4. `samples/cross-platform-installer/lib.capy` — multi-file output

Three `file "…"` blocks emitting `install.sh` / `install.ps1` /
`install.bat` from the same accumulated context. Iteration and
conditionals inside each file template.

### Before (current — `install.sh` block excerpted)

```
file "install.sh":
    #!/usr/bin/env bash
    # Generated by Capy. Edit lib.capy / script.capy, not this file.
    set -euo pipefail

    echo "Installing {{ .context.app_name | unquote }}…"
    {{ range .context.packages }}
    $PM {{ . | unquote }}
    {{- end }}
    {{ range .context.dirs }}
    mkdir -p {{ . | unquote }}
    {{- end }}
    {{ range .context.env }}
    export {{ .name | unquote }}={{ .value | unquote }}
    {{- end }}
    {{- if .context.service }}
    sudo systemctl enable --now {{ .context.service | unquote }}
    {{- end }}
    echo "✓ {{ .context.app_name | unquote }} installed"
```

### After (proposed)

```
file "install.sh"
    write `#!/usr/bin/env bash
# Generated by Capy. Edit lib.capy / script.capy, not this file.
set -euo pipefail

echo "Installing ${unquote context.app_name}…"
`
    for pkg in context.packages
        write `$PM ${unquote pkg}
`
    end
    for d in context.dirs
        write `mkdir -p ${unquote d}
`
    end
    for kv in context.env
        write `export ${unquote kv.name}=${unquote kv.value}
`
    end
    if context.service
        write `sudo systemctl enable --now ${unquote context.service}
`
    end
    write `echo "✓ ${unquote context.app_name} installed"
`
end
```

**What changed.**

- `file "X":` is now a block (with `end`), not a colon-suffixed
  string. The body is plain Capy code — same shape as a function
  body. (Equally valid: `file "X" \`…\`` for the simple case where
  the whole file is one write.)
- `{{ range .context.packages }} … {{- end }}` → `for pkg in
  context.packages … end`. The trailing `{{- end }}` whitespace
  hack disappears: the only newline emitted per iteration is the
  one you put inside the backtick.
- Same shape repeats across the three files (`install.sh`,
  `install.ps1`, `install.bat`). After the rewrite, each is just a
  longer top-down sequence of `write` calls and `for`/`if` blocks.

---

## What these rewrites reveal

1. **Line counts go down modestly** (~20% across these four
   samples). The bigger win isn't terseness; it's that there's only
   one language to read.

2. **`{{- -}}` whitespace control is the biggest carry-over cost
   to abandon.** Authors who currently lean on it have to be more
   explicit about where newlines go. That's not bad — it's just
   different. The bytes you write are the bytes you get.

3. **`else` is dearly missed.** Three of the four samples would
   collapse one or two extra `if` blocks into an `if / else if /
   else` chain. The open question becomes the open requirement.

4. **`file "X" … end` as a block is a happy side effect.** Today
   the multi-file syntax is its own special form
   (`file "X":` colon-suffixed). Under the new design, it's just
   a function-like block whose body emits to a named output file.
   One less special case.

5. **The hardest sample is the one with deeply nested template
   conditionals.** `multi-target-ws-server`'s `if eq .action "X"`
   chain takes more vertical space without `else`. Anyone whose
   library leans on Go template's `{{ if … }}{{ else if … }}{{ end }}`
   should weigh this before migrating.

6. **No sample got LONGER** in this rewrite. That's a useful sanity
   check that the design isn't accidentally verbose for the common
   case.

## Next steps

If the next move is implementation, these four samples are the
right test corpus:

- `transpile-py` (1) covers state accumulation, block openers, and
  the file template.
- `source-imports` (2) covers nested blocks + multi-line text +
  per-function `set` + `write`.
- `multi-target-ws-server` (3) covers loops with nested
  conditionals — the worst case for the proposed shape.
- `cross-platform-installer` (4) covers multi-file output + mixed
  loops + conditionals.

Get those four producing byte-identical output to today's
implementation and the design's load-bearing claims are validated.
The remaining ~55 samples will be mechanical.