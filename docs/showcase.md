---
title: Live demos
---

# Live demos

Every demo on this page is a **real Capy library + script + generated
output**. Where the output is something a browser can render (HTML,
CSS, Markdown, Mermaid), the rendered version is **embedded inline
below the code** so you can see exactly what comes out.

For non-renderable targets (Python, SQL, JSON, …) you see the
source-and-generated pair.

All demos live in the [`samples/`](https://github.com/olivierdevelops/capy/tree/main/samples)
directory if you want to clone and run them yourself.

---

## ✨ New-feature showcase — 22 examples in the playground

The round-1, round-2 and round-6 language features each ship with a
focused, runnable example. They're grouped under the **✨ Features**
category at the top of the [playground](playground.md) dropdown — pick
one, read the source, edit it live, and watch the output change. Every
example below is a real `samples/<id>/` directory with a golden-tested
output.

| Example | Feature it shows off |
|---|---|
| [`html-xml-parser`](https://github.com/olivierdevelops/capy/tree/main/samples/html-xml-parser) | **parse HTML / XML** — one generic `element` function matches any `<tag>…</tag>` via a capture-bound `block_close_seq` + `attribute*` nonterminal; mismatched nesting is a hard error |
| [`template-sugar`](https://github.com/olivierdevelops/capy/tree/main/samples/template-sugar) | `template … end` instead of multi-line backtick `write` literals |
| [`optional-args`](https://github.com/olivierdevelops/capy/tree/main/samples/optional-args) | trailing capture with a `default` — one function, many call shapes |
| [`line-mapping`](https://github.com/olivierdevelops/capy/tree/main/samples/line-mapping) | `${line}` / `${col}` render locals for source↔output mapping |
| [`string-decoded`](https://github.com/olivierdevelops/capy/tree/main/samples/string-decoded) | `${decoded}` round-trips `\n` / `\t` and embedded `\"` quotes |
| [`verbatim-pre`](https://github.com/olivierdevelops/capy/tree/main/samples/verbatim-pre) | `block_verbatim` — raw byte-fidelity code blocks |
| [`backtick-codespan`](https://github.com/olivierdevelops/capy/tree/main/samples/backtick-codespan) | escapable backticks so `` \`code\` `` survives inside a capture |
| [`utf8-prose`](https://github.com/olivierdevelops/capy/tree/main/samples/utf8-prose) | bare UTF-8 prose (accents, CJK, emoji) — no quoting |
| [`multiline-strings`](https://github.com/olivierdevelops/capy/tree/main/samples/multiline-strings) | multi-line backtick captures in user scripts |
| [`inline-markdown`](https://github.com/olivierdevelops/capy/tree/main/samples/inline-markdown) | group types — `link [text](url)`, `bold **x**`, `strike ~~x~~` |
| [`feature-faq`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-faq) | FAQ: optional `open` flag + `template` + decode/escape |
| [`feature-pricing`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-pricing) | pricing tiers with optional period / CTA defaults |
| [`feature-callouts`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-callouts) | note / tip / warning callouts via one optional arg |
| [`feature-svg-badge`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-svg-badge) | inline SVG captured verbatim, byte-for-byte |
| [`feature-menu-i18n`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-menu-i18n) | multilingual menu — UTF-8 prose + `tail` capture |
| [`feature-changelog`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-changelog) | release notes: `${line}` + optional `kind` + verbatim block |
| [`feature-social-card`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-social-card) | social preview cards with an optional `theme` |
| [`feature-stepper`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-stepper) | onboarding stepper stamping `${line}` for scroll-sync |
| [`feature-glossary`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-glossary) | definition list: multi-line backticks + code spans |
| [`feature-quiz`](https://github.com/olivierdevelops/capy/tree/main/samples/feature-quiz) | multiple-choice quiz with an optional `correct` flag |
| [`mcp-widgets`](https://github.com/olivierdevelops/capy/tree/main/samples/mcp-widgets) | nested blocks + templates + decode/escape + context, together |
| [`math-plots`](https://github.com/olivierdevelops/capy/tree/main/samples/math-plots) | `template … end` + canvas plots (rendered live below) |

[Open the playground → ](playground.md) and choose the **✨ Features**
category. New to the language features? Start with [What's new](whats-new.md).

---

## 🔢 Indexed reads — lists & maps read back by index or key

A `[<expr>]` index now works in **read** position, mirroring the
write side: `${context.buf[i]}`, `${context.palette[name]}`,
`${context.memo[(sub n 1)]}`, `${context.stack[-1]}` and nested
`${context.grid[r][c]}`. Map parents key on the index's string form,
list parents key on an integer (negative counts from the end);
missing keys / out-of-range indices render empty. These ten samples —
grouped under the **🔢 Indexed reads** category in the
[playground](playground.md) — turn the idiom into real programs:

| Example | What it indexes |
|---|---|
| [`word-frequency`](https://github.com/olivierdevelops/capy/tree/main/samples/word-frequency) | map read-modify-write: `set context.counts[w] (add context.counts[w] 1)` |
| [`memo-fib`](https://github.com/olivierdevelops/capy/tree/main/samples/memo-fib) | computed indices `${context.memo[(sub n 1)]}` over a memo table |
| [`stack-top`](https://github.com/olivierdevelops/capy/tree/main/samples/stack-top) | negative indices `${context.stack[-1]}` / `[-2]` to peek a stack |
| [`enum-lookup`](https://github.com/olivierdevelops/capy/tree/main/samples/enum-lookup) | list-by-index lookup tables (`days[d]`, `months[m]`) |
| [`leaderboard`](https://github.com/olivierdevelops/capy/tree/main/samples/leaderboard) | fixed positions `[0]`/`[1]`/`[2]` + `[-1]` for last place |
| [`color-palette`](https://github.com/olivierdevelops/capy/tree/main/samples/color-palette) | map-by-name `${context.palette[name]}` to emit colored chips |
| [`symbol-types`](https://github.com/olivierdevelops/capy/tree/main/samples/symbol-types) | a symbol table keyed by captured name |
| [`register-allocator`](https://github.com/olivierdevelops/capy/tree/main/samples/register-allocator) | a name→register map read during emission |
| [`grid-game`](https://github.com/olivierdevelops/capy/tree/main/samples/grid-game) | a flat board indexed by a computed `r*cols + c` |
| [`emoji-react`](https://github.com/olivierdevelops/capy/tree/main/samples/emoji-react) | a key→glyph map read back by name in a template |

See [value-position indexing](design/value-position-indexing.md) for
the design notes and read/write parity rules.

---

## 📈 Math plots — plotting equations from a one-line DSL

A tiny DSL (`plot "sin(x)" … end`) turns into a self-contained HTML
page with canvas plots, axes, and an inline plotter. Showcases every
primitive shipped in this release: `template … end` sugar, the
`${escapeHtml}` / `${decoded}` helpers, UTF-8 prose in titles, and
`context.plots` accumulation. The library also ships a `command
"run"` block so `capy math-plots run page.plots` writes the HTML
next to the source and opens it in the browser.

```
title "A few familiar shapes"

plot "sin(x)"
    domain -6.28 6.28
    color "#4ef"
end

plot "sin(x) + 0.5*sin(3*x) + 0.25*sin(5*x)"
    domain -6.28 6.28
    color "#f4b"
end

plot "exp(-x*x)"
    domain -3 3
    color "#bf4"
    samples 400
end
```

Rendered live below:

<iframe src="../assets/demos/math-plots.html" sandbox="allow-scripts allow-same-origin" style="width: 100%; height: 540px; border: 0; border-radius: 12px; box-shadow: 0 12px 40px rgba(0,0,0,0.18); display: block; margin: 18px 0 24px;" title="Math plots rendered live from a Capy source"></iframe>

Full library + script: [`samples/math-plots/`](https://github.com/olivierdevelops/capy/tree/main/samples/math-plots).
Walk-through of every primitive it uses: [What's new](whats-new.md).

---

## 🏡 Capy for everyday things — no coding needed

These four demos use vocabularies designed for ordinary tasks: a
recipe, an invitation, a meal plan, a child's reading log. The
"language" you write is plain English with a handful of keywords.
Open the iframes to see the polished output.

=== "Recipe card"

    Source you write:

    ```
    recipe "Lemon olive oil cake"
        serves 8
        time "45 minutes"

        ingredient "all-purpose flour"   "1 1/2 cups"
        ingredient "olive oil"           "3/4 cup"
        ingredient "sugar"               "1 cup"
        ingredient "lemon zest"          "2 tablespoons"

        step "Preheat oven to 350F."
        step "Whisk flour and sugar."
        step "Add oil and lemon zest."
        step "Bake for 35-40 minutes."

        tip "Glaze with powdered sugar and lemon juice."
    end
    ```

    Generated HTML card (open it, print it, email it):

    <iframe src="../assets/demos/recipe-card.html" width="100%" height="540" style="border: 1px solid #e8d9b0; background: #fdf6e3; border-radius: 8px;"></iframe>

    [Full sample → `samples/recipe-card/`](https://github.com/olivierdevelops/capy/tree/main/samples/recipe-card)

=== "Event invitation"

    ```
    invite "Maya turns 6!"
        host "The Patel family"

        when "Saturday, June 14"
        time "2:00 - 5:00 pm"
        where "Lincoln Park, Pavilion 3"
        address "200 Park Avenue, Springfield"

        rsvp_by "June 7"
        rsvp_to "maya@example.com"

        note "There will be a unicorn cake and a butterfly hunt."
        note "Wear something you can run in. Sunscreen recommended."

        bring "A book to add to Maya's library (any age)"
    end
    ```

    <iframe src="../assets/demos/event-invite.html" width="100%" height="640" style="border: 1px solid #d4b8e8; background: linear-gradient(135deg, #ffd5e8, #cfe9ff); border-radius: 8px;"></iframe>

    [Full sample → `samples/event-invite/`](https://github.com/olivierdevelops/capy/tree/main/samples/event-invite)

=== "Weekly meal plan"

    ```
    week "March 10 - 16"
        serves 4

        monday    "Sheet-pan salmon with broccoli and lemon"
        tuesday   "Pasta with brown butter and sage"
        wednesday "Black bean tacos with avocado and lime"
        thursday  "Leftover salmon salads with greens"
        friday    "Homemade pizza night (kids choose toppings)"
        saturday  "Slow-cooker chicken stew"
        sunday    "Roast vegetables and quinoa bowls"

        note "Buy fresh fish on Sunday or Monday for best quality."
        note "Make extra rice on Wednesday for Thursday lunches."
    end
    ```

    <iframe src="../assets/demos/weekly-meal-plan.html" width="100%" height="640" style="border: 1px solid #c5e0c5; background: #f0f7f0; border-radius: 8px;"></iframe>

    [Full sample → `samples/weekly-meal-plan/`](https://github.com/olivierdevelops/capy/tree/main/samples/weekly-meal-plan)

=== "Reading log (for kids)"

    ```
    log "Emma's reading log" age 7
        goal 500

        book "Charlotte's Web"           pages 184  rating 5
        book "The Wild Robot"            pages 277  rating 5
        book "Mr. Popper's Penguins"     pages 138  rating 4
        book "Frog and Toad Together"    pages 64   rating 5
        book "Junie B. Jones #1"         pages 72   rating 3
    end
    ```

    Progress bar fills toward the yearly goal. Stars come from the
    rating number. Update through the year by adding more `book` lines.

    <iframe src="../assets/demos/reading-log.html" width="100%" height="540" style="border: 1px solid #f4d8a8; background: #fff4d6; border-radius: 8px;"></iframe>

    [Full sample → `samples/reading-log/`](https://github.com/olivierdevelops/capy/tree/main/samples/reading-log)

**Why this matters.** You don't need a degree in computer science
to use Capy. The vocabularies above (`recipe`, `serves`,
`ingredient`, `step`; `invite`, `host`, `when`, `where`; `monday`,
`tuesday`, …) are designed for ordinary tasks — and someone (you,
a teammate, or an AI) can design a new vocabulary for any task you
do more than twice.

[Read the non-programmer guide → `docs/for-everyone.md`](for-everyone.md)

---

## 🎚️ Progressive abstraction — pick your level of control

Same library, three abstraction levels. Start with a 4-line one-shot;
take more control as your needs grow; drop to raw HTML/CSS via
escape hatches when the abstraction isn't enough.

=== "Level 1 — minimal (4 lines)"

    ```
    landing "Capy"
            tagline "Describe what you want. Capy produces what you need."
            cta_text "Open the playground"
            cta_link "https://olivierdevelops.github.io/capy/playground/"
    ```

    You declare WHAT; the library decides EVERYTHING else.

    <iframe src="../assets/demos/abstraction-level-1.html" width="100%" height="320" style="border: 1px solid #30363d; border-radius: 6px; background: #0f172a;" title="Level 1"></iframe>

=== "Level 2 — block style (~12 lines)"

    ```
    landing "Capy"
        hero "Capy" "Describe what you want. Capy produces what you need."

        feature "Zero default grammar"  "Every keyword is defined by the library."
        feature "55+ samples"           "Recipes, invoices, Android apps."
        feature "Browser playground"    "Compiler runs as WebAssembly."
        feature "MCP server included"   "Plug into Claude / Cursor / Zed."

        cta "Open the playground" "https://..."
    end
    ```

    You take control of WHICH sections appear and in what order;
    visual identity stays library-owned.

    <iframe src="../assets/demos/abstraction-level-2.html" width="100%" height="420" style="border: 1px solid #30363d; border-radius: 6px; background: #0f172a;" title="Level 2"></iframe>

=== "Level 3 — escape hatches (~30 lines)"

    ```
    landing "Capy — Pro"
        raw_head "<meta name='theme-color' content='#4f46e5'>"
        style_override "body { background: linear-gradient(...); } .hero h1 { ... }"

        hero "Capy — Pro" "Same engine. Same grammar. Take exactly the control you need."

        feature "..." "..."
        feature "Metaprogramming"  "Source declares its own DSL primitives."

        raw_section "<section style='...'>...custom HTML...</section>"
        cta "Open the playground" "..."
        raw_footer "<a href='...'>github.com/olivierdevelops/capy</a>"
    end
    ```

    Now you have **escape hatches**: literal HTML in `<head>`,
    stylesheet overrides, custom sections, replaced footer. The
    library never gets in your way.

    <iframe src="../assets/demos/abstraction-level-3.html" width="100%" height="520" style="border: 1px solid #30363d; border-radius: 6px; background: #0f172a;" title="Level 3"></iframe>

=== "Why it matters"

    | Concern | Capy's answer |
    |---|---|
    | "Tools are great until you need something they didn't anticipate" | Same library exposes Level 1 → 2 → 3. Drop a level, never switch tools. |
    | "Different teammates want different control" | Marketing uses Level 1, product uses Level 2, brand uses Level 3 — same library. |
    | "Escape hatches are second-class citizens" | `raw_head` / `style_override` / `raw_section` are first-class primitives; libraries are designed around them. |
    | "What if I need to go off-script for one launch?" | Drop to Level 3 for that page; revert to Level 1 for the next. |

[Full sample → `samples/progressive-abstraction/`](https://github.com/olivierdevelops/capy/tree/main/samples/progressive-abstraction) ·
[Pattern docs → `progressive-abstraction.md`](progressive-abstraction.md)

---

## 🧬 Metaprogramming — source extends its own grammar

A `define NAME ... end` block in a Capy source file introduces a new
function. The rest of the source can then call it — no library
change required. Use it to DRY up repetitive boilerplate or
prototype DSL extensions before promoting them to the library.

=== "Source with three inline defines"

    ```
    define heading
        arg literal "heading"
        arg capture text string
        write `# ${unquote text}

`
    end

    define quote
        arg literal "quote"
        arg capture text string
        arg capture who string
        write `> ${unquote text}
>
> — *${unquote who}*

`
    end

    define checklist_item
        arg literal "todo"
        arg capture done ident
        arg capture text string
        if eq done "yes"
            write `- [x] ${unquote text}
`
        else
            write `- [ ] ${unquote text}
`
        end
    end

    # Use them — library has no `heading`, `quote`, or `todo`.
    heading "Today's todos"
    todo yes "Ship metaprogramming"
    todo no  "Update the docs"
    quote "Description over implementation." "Capy"
    ```

=== "Generated output"

    ```markdown
    # Today's todos
    - [x] Ship metaprogramming
    - [ ] Update the docs
    > Description over implementation.
    >
    > — *Capy*
    ```

=== "When to use it"

    | Pattern | Right tool |
    |---|---|
    | Repetitive boilerplate in one source file | `define` |
    | Reused across many sources in one project | shared `.capy` file + `@import` |
    | Reused across many projects | library-level `function` |
    | Truly project-specific UI / behavior | `define` (don't bloat the shared library) |

    Source defines OVERRIDE library functions of the same name —
    use it to specialize without forking.

[Full sample → `samples/metaprogramming/`](https://github.com/olivierdevelops/capy/tree/main/samples/metaprogramming) ·
[Pattern docs →](metaprogramming.md)

---

## 📦 Cross-platform installer — one source, three shells

Declare install steps once; emit `install.sh`, `install.ps1`, AND
`install.bat` from the same 9-line source via multi-file output.
Every platform stays in lock-step because they're regenerated from
one declaration.

=== "Source (9 lines)"

    ```
    app "capy-agent"
    install "jq"
    install "curl"
    install "ripgrep"
    mkdir "/var/log/capy-agent"
    setenv "CAPY_AGENT_PORT" "8080"
    service "capy-agent"
    end
    ```

=== "→ install.sh (POSIX)"

    ```bash
    #!/usr/bin/env bash
    set -euo pipefail
    if command -v apt-get >/dev/null 2>&1; then
      PM="sudo apt-get install -y"
    elif command -v brew >/dev/null 2>&1; then
      PM="brew install"
    fi
    $PM jq
    $PM curl
    $PM ripgrep
    mkdir -p /var/log/capy-agent
    export CAPY_AGENT_PORT=8080
    # systemd unit emitted here
    sudo systemctl enable --now capy-agent
    ```

=== "→ install.ps1 (PowerShell)"

    ```powershell
    $ErrorActionPreference = "Stop"
    choco install -y jq
    choco install -y curl
    choco install -y ripgrep
    New-Item -ItemType Directory -Force -Path "/var/log/capy-agent"
    [Environment]::SetEnvironmentVariable("CAPY_AGENT_PORT", "8080", "Machine")
    sc.exe create capy-agent binPath= "..." start= auto
    sc.exe start capy-agent
    ```

=== "→ install.bat (cmd.exe)"

    ```batch
    @echo off
    choco install -y jq
    choco install -y curl
    choco install -y ripgrep
    if not exist "/var/log/capy-agent" mkdir "/var/log/capy-agent"
    setx CAPY_AGENT_PORT "8080" /M
    sc create capy-agent binPath= "..." start= auto
    sc start capy-agent
    ```

=== "Why it matters"

    - **No drift.** Add `install "tmux"` once → all three scripts get it.
    - **No bash-in-PowerShell horror.** Each target speaks its native
      syntax; the library handles translation.
    - **CI-friendly.** Generate all three in one step, ship them all,
      let the installer pick the right one at runtime.

---

## 🔌 WebSocket server — 10 lines → ~80 lines of Go

A chat-server DSL declares routes, typed fields, and **abstract
handler verbs** (`broadcast`, `announce`). The source contains zero
target-language code — no Go fragments, no JSON literals, no embedded
expressions — so the same source survives unchanged when you swap the
library to a Node / Python / Rust ws library.

=== "Source (10 lines)"

    ```
    server "8080"
    route "/chat"

    field chat user string
    field chat text string

    field join user string

    on chat broadcast
    on join announce
    end
    ```

=== "→ Go (excerpt)"

    ```go
    type ChatMsg struct {
        User string `json:"user"`
        Text string `json:"text"`
    }
    type JoinMsg struct {
        User string `json:"user"`
    }

    func wsHandler(hub *Hub) http.HandlerFunc {
        return func(w http.ResponseWriter, r *http.Request) {
            conn, _ := upgrader.Upgrade(w, r, nil)
            hub.Add(conn); defer hub.Remove(conn); defer conn.Close()
            for {
                _, raw, err := conn.ReadMessage()
                if err != nil { return }
                var env Envelope
                json.Unmarshal(raw, &env)
                switch env.Type {
                case "chat":
                    var m ChatMsg; json.Unmarshal(env.Data, &m)
                    // verb=broadcast → echo to all clients.
                    hub.Broadcast("chat", m)
                case "join":
                    var m JoinMsg; json.Unmarshal(env.Data, &m)
                    // verb=announce → log + broadcast.
                    log.Printf("join: %+v", m)
                    hub.Broadcast("join", m)
                }
            }
        }
    }
    ```

=== "Why it matters"

    - **Zero target-language code in the source.** No Go fragments
      embedded as strings, no `hub.Broadcast(...)` literals to escape.
      Every handler is an abstract verb the library translates.
    - **Adding a verb = adding a library case.** Want `archive` or
      `rate_limit`? Add it once to the dispatch switch in `lib.capy`;
      every source that mentions the verb gets the new behaviour.
    - **Network plumbing is library territory.** Upgrade headers,
      JSON envelope, dispatch — written once in the library.
    - **Source = contract.** The 10-line source IS the protocol spec.
      Generate the server, a TypeScript client, and OpenAPI docs
      from the same declaration.

---

## 🌐 Same WebSocket DSL → Go AND Node AND Python

The exact same 7-line source produces a runnable echo server in three
target languages by swapping just the library file. Spec lives in
the source; implementation choice lives in the library.

=== "Source (identical across all three)"

    ```
    server "8080"
    route "/echo"

    handler ping   reply
    handler shout  uppercase
    handler bye    close
    end
    ```

=== "→ Go (gorilla/websocket)"

    ```go
    case strings.HasPrefix(text, "ping "):
        payload := strings.TrimPrefix(text, "ping ")
        _ = conn.WriteMessage(websocket.TextMessage,
            []byte("pong "+payload))
    case strings.HasPrefix(text, "shout "):
        payload := strings.TrimPrefix(text, "shout ")
        _ = conn.WriteMessage(websocket.TextMessage,
            []byte(strings.ToUpper(payload)))
    case strings.HasPrefix(text, "bye "):
        return
    ```

=== "→ Node (`ws` package)"

    ```js
    if (text.startsWith("ping ")) {
      ws.send("pong " + text.slice(5));
      return;
    }
    if (text.startsWith("shout ")) {
      ws.send(text.slice(6).toUpperCase());
      return;
    }
    if (text.startsWith("bye ")) {
      ws.close();
      return;
    }
    ```

=== "→ Python (asyncio + websockets)"

    ```python
    if text.startswith("ping "):
        await websocket.send("pong " + text[5:])
        continue
    if text.startswith("shout "):
        await websocket.send(text[6:].upper())
        continue
    if text.startswith("bye "):
        await websocket.close(); return
    ```

=== "Why it matters"

    - **Source survives target changes.** Same source still parses
      if you rewrite the library to emit Rust or Elixir tomorrow.
    - **No translation drift.** The dispatcher logic is encoded once
      per library and applies uniformly to every handler.
    - **Test once, ship N times.** The behaviour contract is one source;
      pick the runtime per deployment.

---

## ⚙️ Custom assembly — one source, many ISAs

A 5-op assembly DSL (`data`, `func`, `write`, `exit`, `end`) +
three libraries = runnable hello-world on x86_64, ARM64, AND RISC-V64.
New architecture? Add a library; the source survives.

=== "Source (5 lines)"

    ```
    data hello "Hello, world\\n"

    func _start
        write 1 hello 13
        exit 0
    end
    ```

=== "→ x86_64 Linux (System V)"

    ```asm
    .data
    hello:
        .ascii "Hello, world\n"
    .text
    .global _start
    _start:
        mov $1, %rax              # syscall: write
        mov $1, %rdi              # arg0: fd
        lea hello(%rip), %rsi     # arg1: buffer
        mov $13, %rdx             # arg2: count
        syscall
        mov $60, %rax             # syscall: exit
        mov $0, %rdi
        syscall
    ```

=== "→ AArch64 Linux"

    ```asm
    .data
    hello:
        .ascii "Hello, world\n"
    .text
    .global _start
    _start:
        mov x8, #64               // syscall: write
        mov x0, #1                // arg0: fd
        adr x1, hello             // arg1: buffer
        mov x2, #13               // arg2: count
        svc #0
        mov x8, #93               // syscall: exit
        mov x0, #0
        svc #0
    ```

=== "→ RV64I Linux"

    ```asm
    .data
    hello:
        .ascii "Hello, world\n"
    .text
    .global _start
    _start:
        li a7, 64                 # syscall: write
        li a0, 1                  # arg0: fd
        la a1, hello              # arg1: buffer
        li a2, 13                 # arg2: count
        ecall
        li a7, 93                 # syscall: exit
        li a0, 0
        ecall
    ```

=== "Why it matters"

    - **Source survives ISA changes.** The 5-line DSL doesn't mention
      registers, syscall numbers, or trap instructions. The library
      owns everything platform-specific.
    - **Adding a target = adding a library.** Want POWER, MIPS,
      s390x, or FreeBSD-on-x86_64 (different syscall numbers)?
      Write a new `lib-<arch>-<os>.capy`. The source doesn't change.
    - **`(os)` and `(arch)` host primitives** let one smart library
      detect the host at transpile time and emit code for IT — useful
      for cross-platform install scripts that vary their own asm.
    - **Pedagogical.** Students write one program, see it materialise
      in three ISAs side by side. Differences become visible at the
      instruction level instead of buried in compiler internals.

---

## 🔌 Host capabilities — env vars, CLI args, file reads

Libraries can pull values from outside the source at transpile time via
four inner-DSL primitives: `(env "NAME")`, `(arg N)`, `(args)`,
`(read_file "PATH")`. The CLI's `OSHost` exposes real `os.Getenv` /
`os.Args` / `os.ReadFile`; the playground's sandboxed `NoOpHost` returns
empty values so libraries can't smuggle your filesystem into a browser.

=== "Source (5 lines)"

    ```
    service "checkout-api"
    replicas 6
    load_keys_from "api-keys.txt"
    end
    ```

=== "Library (excerpt)"

    ```
    function service
        arg literal "service"
        arg capture name string
        set context.service name
        # Pull deploy metadata from the host.
        set context.environment  (env "ENV")
        set context.database_url (env "DATABASE_URL")
        set context.version (arg 0)
        set context.region  (arg 1)
    end

    function load_keys_from
        arg literal "load_keys_from"
        arg capture path string
        # read_file resolves relative to the script dir.
        # Errors abort the transpilation cleanly.
        set context.api_keys (read_file path)
    end
    ```

=== "Run"

    ```sh
    ENV=production DATABASE_URL=postgres://db.internal/prod \
      capy run lib.capy script.capy v2.3.1 us-west-2
    ```

=== "Output (deployment.yaml)"

    ```yaml
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: checkout-api
      labels:
        environment: production
        region: us-west-2
        version: v2.3.1
    spec:
      replicas: 6
      template:
        spec:
          containers:
            - name: checkout-api
              image: registry/checkout-api:v2.3.1
              env:
                - name: DATABASE_URL
                  value: postgres://db.internal/prod
                - name: REGION
                  value: us-west-2
              args:
                - "--key=sk_live_alpha_e91f2c0a"
                - "--key=sk_live_beta_77d4ab19"
                - "--key=sk_live_telemetry_2bc88f01"
    ```

=== "Why it matters"

    - **Same source, every environment.** The 5-line script is
      identical in staging, production, and prod-eu. The host supplies
      what changes; the library encodes what doesn't.
    - **Secrets stay where they belong.** `read_file` keeps API keys
      in a sibling file (gitignored, KMS-encrypted, mounted from a
      CSI driver) rather than baked into source.
    - **CI-friendly.** `capy run lib.capy script.capy $VERSION $REGION`
      flows naturally from a build pipeline. No template-language
      gymnastics.
    - **Sandboxed by default.** The wasm playground and embedded Rust
      API both use `NoOpHost` — no library author can exfiltrate
      `$AWS_SECRET_ACCESS_KEY` from a hosted playground. Opt in to
      real host access with `lib.SetHost(infra.OSHost{...})`.

---

## 📖 Auto-generated library reference docs

Annotate functions, args, and types with `description "..."` and
`capy docs lib.capy` produces a Markdown reference. The same
renderer powers the **DOCS** tab in the playground — every library
ships with browsable, regenerable documentation.

=== "Annotated library (excerpt)"

    ```
    extension html
    description "Recipe DSL for home cooks. Six keywords produce a
                 polished printable HTML recipe card."

    function recipe
        description "Open a new recipe with a title. Wraps the rest
                     of the file; closed by `end`."
        arg literal "recipe"
        arg capture title string  "Display name of the dish, shown as the H1."
        block_closer end
        set context.title title
    end

    function ingredient
        description "Add one ingredient. Listed in a two-column grid."
        arg literal "ingredient"
        arg capture name string  "Ingredient name, e.g. `\"olive oil\"`."
        arg capture qty string   "Quantity with unit, e.g. `\"3/4 cup\"`."
        append context.ingredients {name: name, qty: qty}
    end
    ```

=== "`capy docs samples/recipe-card/lib.capy`"

    ````markdown
    # Library reference (→ `.html`)

    Recipe DSL for home cooks. Six keywords produce a polished
    printable HTML recipe card.

    | | |
    |---|---|
    | **Output extension** | `.html` |
    | **Functions** | 7 |
    | **Types** | 0 |

    ## Functions

    ### `recipe`

    Open a new recipe with a title. Wraps the rest of the file;
    closed by `end`.

    ```
    recipe <title>
    ```

    | Argument | Type | Description |
    |---|---|---|
    | `title` | `string` | Display name of the dish, shown as the H1. |

    **Opens an indented block** — body runs until `end`.

    ### `ingredient`

    Add one ingredient. Listed in a two-column grid.

    ```
    ingredient <name> <qty>
    ```

    | Argument | Type | Description |
    |---|---|---|
    | `name` | `string` | Ingredient name, e.g. `"olive oil"`. |
    | `qty` | `string` | Quantity with unit, e.g. `"3/4 cup"`. |
    ````

=== "Why this matters"

    - **Onboarding** — new contributors read the auto-generated docs
      and know every function the team uses without grep-ing the
      library file.
    - **CI gate** — fail the build if `capy docs lib.capy` doesn't
      match the committed `LIB_REFERENCE.md`. Docs stay in lock-step
      with implementation.
    - **AI agents** — handing the agent a typed library + a
      human-readable reference improves call-site accuracy.
    - **Marketing** — "50-keyword schema DSL" lands harder when each
      keyword has a one-sentence explanation a non-expert can read.

    See [Auto-generated library docs](library-documentation.md) for
    the full pattern + CI workflow.

---

## 🩺 Errors that tell you how to fix them

Every Capy error names what went wrong, hints at how to fix it,
and points at the exact column with a caret. Did-you-mean
suggestions cover typos in DSL keywords, type names, and enum
values.

=== "Typo in a DSL keyword"

    Source:

    ```
    endpiont GET "/users"
    ```

    Capy output:

    ```
    error: no library function matches token "endpiont"
      hint: did you mean "endpoint"?
      1 │ endpiont GET "/users"
        │ ^
    ```

    Levenshtein-distance lookup against every library function
    name. When no close match exists, the hint lists what *is*
    available.

=== "Value violates an enum"

    Library declares:

    ```
    type LogLevel
        options "trace" "debug" "info" "warn" "error" "fatal"
    end
    ```

    Source: `log_level verbose`

    Capy output:

    ```
    error: function "log_level" arg "lvl": value "verbose" is not in options for type "LogLevel"
      hint: valid options: trace, debug, info, warn, error, fatal
    ```

    Or, when the value is close to a real option:

    ```
    error: function "env" arg "stage": value "prudo" is not in options for type "Env"
      hint: did you mean "prod"? valid options: dev, staging, prod
    ```

=== "Value violates a regex"

    ```
    owner "not-an-email"
    ```

    ```
    error: function "owner" arg "who": value "not-an-email" does not match pattern for type "Email"
      hint: type "Email" requires the value to match regex /^[^@]+@[^@]+\.[^@]+$/
    ```

    The hint includes the regex so authors can see what's wrong
    without opening the library.

[Full guide → `errors-and-debugging.md`](errors-and-debugging.md)

---

## 🔗 Source-file imports — `@import` for content reuse

Inside a `script.capy`, the `@import "path"` directive splices the
contents of another `.capy` file into place. Indentation auto-tracks
so imported content nests inside the surrounding block.

=== "Source with @import"

    ```
    menu "Capy Cafe — Spring 2026"

        section "Mains"
            item "House pasta"               "$16"
            item "Sheet-pan salmon"          "$22"
            item "Black bean tacos (3)"      "$14"
        end

        @import "shared/drinks.capy"
        @import "shared/desserts.capy"

        note "All dishes are made fresh."
    end
    ```

=== "shared/drinks.capy"

    ```
    section "Drinks"
        item "Espresso"        "$3"
        item "Cappuccino"      "$4.50"
        item "Cold brew"       "$5"
        item "Mint lemonade"   "$5"
    end
    ```

=== "Generated Markdown"

    ```markdown
    # Capy Cafe — Spring 2026

    ## Mains
    - **House pasta** — $16
    - **Sheet-pan salmon** — $22
    - **Black bean tacos (3)** — $14

    ## Drinks
    - **Espresso** — $3
    - **Cappuccino** — $4.50
    - **Cold brew** — $5
    - **Mint lemonade** — $5

    ## Desserts
    - **Olive oil cake** — $8
    - **Chocolate mousse** — $9
    - **Affogato** — $7

    > All dishes are made fresh.
    ```

**Use it for:** shared sections of authored content (price lists,
menu sections, footers, author bios), splitting long sources for
readability, multi-environment configs. Cycles are detected by
absolute path. `@include` is a synonym of `@import`.

[Full sample → `samples/source-imports/`](https://github.com/olivierdevelops/capy/tree/main/samples/source-imports)

---

## 🎨 Design systems — one component, three frameworks

Encode your house style in a library; generate React, Vue, and
Svelte with **identical** visual semantics. Same composition,
three frameworks, zero drift.

=== "Source (8 lines)"

    ```
    page "Settings"
        button "Save changes"      variant primary  size lg
        button "Discard"            variant ghost    size lg
        card title "Profile"
            field email     "alice@example.com"
            field display   "Alice Chen"
            field timezone  "America/Los_Angeles"
        end
        card title "Danger zone"
            button "Delete account" variant danger   size md
        end
    end
    ```

=== "→ React TSX"

    ```tsx
    const BUTTON_VARIANT = {
      primary: "bg-indigo-600 hover:bg-indigo-700 text-white",
      ghost:   "bg-transparent hover:bg-slate-100 text-slate-700 border border-slate-200",
      danger:  "bg-red-600 hover:bg-red-700 text-white",
    } as const;
    // Button / Card / Field defined identically across frameworks

    export default function SettingsPage() {
      return (
        <main className="max-w-2xl mx-auto p-8 space-y-3">
          <h1 className="text-3xl font-bold text-slate-900">Settings</h1>
          <Button variant="primary" size="lg">Save changes</Button>
          <Button variant="ghost" size="lg">Discard</Button>
          <Card title="Profile">
            <Field label="email" value="alice@example.com" />
            <Field label="display" value="Alice Chen" />
          </Card>
          <Card title="Danger zone">
            <Button variant="danger" size="md">Delete account</Button>
          </Card>
        </main>
      );
    }
    ```

=== "→ Vue 3 SFC"

    ```html
    <template>
      <main class="max-w-2xl mx-auto p-8 space-y-3">
        <h1 class="text-3xl font-bold text-slate-900">Settings</h1>
        <Button variant="primary" size="lg">Save changes</Button>
        <Card title="Profile">
          <Field label="email" value="alice@example.com" />
          ...
        </Card>
      </main>
    </template>
    ```

=== "→ Svelte"

    ```html
    <script lang="ts">
      import Button from "./Button.svelte";
      import Card from "./Card.svelte";
      import Field from "./Field.svelte";
    </script>

    <main class="max-w-2xl mx-auto p-8 space-y-3">
      <h1 class="text-3xl font-bold text-slate-900">Settings</h1>
      <Button variant="primary" size="lg">Save changes</Button>
      <Card title="Profile">
        <Field label="email" value="alice@example.com" />
        ...
      </Card>
    </main>
    ```

**Identical Tailwind classes across all three.** Change a token in
the library; every framework regenerates. Add a 4th target (Lit,
Solid, Astro, SwiftUI) by writing a 40-line library.

[Full sample → `samples/design-system-components/`](https://github.com/olivierdevelops/capy/tree/main/samples/design-system-components) ·
[Pattern → `docs/design-systems.md`](design-systems.md)

---

## 🧪 Backend code with auto-wired tests

Every `handler` declaration produces a Go stub AND a smoke test.
Directory layout and the "every handler has a test" rule live in
the library — new contributors can't deviate.

=== "Source (4 lines)"

    ```
    handler ListUsers   method GET     path "/users"        returns "[]User"
    handler GetUser     method GET     path "/users/{id}"   returns "User"
    handler CreateUser  method POST    path "/users"        accepts "UserCreateRequest"  returns "User"
    handler DeleteUser  method DELETE  path "/users/{id}"   returns "void"
    ```

=== "→ handlers.go"

    ```go
    package handlers

    import "net/http"

    func Mount(mux *http.ServeMux) {
        mux.HandleFunc("GET /users", ListUsers)
        mux.HandleFunc("GET /users/{id}", GetUser)
        mux.HandleFunc("POST /users", CreateUser)
        mux.HandleFunc("DELETE /users/{id}", DeleteUser)
    }

    // GET /users — returns []User
    func ListUsers(w http.ResponseWriter, r *http.Request) {
        http.Error(w, "ListUsers not implemented", http.StatusNotImplemented)
    }
    // ... three more stubs ...
    ```

=== "→ handlers_test.go (auto-generated!)"

    ```go
    package handlers

    import (
        "net/http"
        "net/http/httptest"
        "testing"
    )

    func Test_ListUsers_RouteRegistered(t *testing.T) {
        mux := http.NewServeMux()
        Mount(mux)
        req := httptest.NewRequest("GET", "/users", nil)
        rr := httptest.NewRecorder()
        mux.ServeHTTP(rr, req)
        if rr.Code != http.StatusNotImplemented {
            t.Errorf("expected 501, got %d", rr.Code)
        }
    }
    // ... one test per declared handler ...
    ```

    `cargo test --workspace` on the generated output **passes** — verified.
    As the developer implements each handler, the stub stops
    returning 501 → the test fails → they replace it with real
    assertions.

**There's no way to declare a handler without its matching test.**

[Full sample → `samples/backend-with-tests/`](https://github.com/olivierdevelops/capy/tree/main/samples/backend-with-tests) ·
[Pattern → `docs/backend-codegen.md`](backend-codegen.md)

---

## 💡 Capy as an idea language

The most ambitious framing: Capy lets you describe **ideas**;
libraries are **implementers**. When the Go server isn't fast
enough and you want Rust, you swap libraries — not rewrite the
code.

The [multi-language-demo](https://github.com/olivierdevelops/capy/tree/main/samples/multi-language-demo)
ships this concretely: the same 10-line source compiles to
**Python, JavaScript, Go, Rust, AND C**. Identical semantics,
five idiomatic implementations.

```
script.capy (the idea)
       │
   ┌───┼───┬───┬───┐
   ▼   ▼   ▼   ▼   ▼
  .py .js .go .rs .c
   └───┴───┴───┴───┘
   every output verified to print 12 when run
```

Mobile apps follow the same pattern: `samples/android-app/` and
`samples/ios-app/` accept the **same source shape** and emit
Kotlin or SwiftUI. The "habit tracker" idea is platform-agnostic;
which native stack to ship is a library decision.

**The implication:** outgrowing an implementation no longer means
rewriting the system. Add a new target library; run benchmarks;
switch over a service at a time. The contract stays stable.

[Pattern → `docs/idea-language.md`](idea-language.md)

---

## 🛠️ Generate a whole project — web app, Android, iOS, libtorch

The same multi-file mechanism scaffolds **real projects** across
very different stacks. Below are four canonical demos. Each takes
12–17 lines of plain DSL and emits a full, compilable file tree
for its target.

=== "Habit-tracker web app — 3 files"

    Source:

    ```
    app "Habit Tracker"
        description "A tiny daily-habit tracker."
        color_primary "#4f46e5"
        color_bg "#0f172a"

        habit drink_water  "Drink 8 glasses of water"
        habit read         "Read 20 pages"
        habit walk         "30 min walk"
        habit code         "Practice coding"
        habit meditate     "Meditate 10 min"
    end
    ```

    Output:

    ```
    out/
    ├── index.html       (references styles.css + app.js)
    ├── app.js           (localStorage persistence + streaks)
    └── styles.css       (themed by source variables)
    ```

    [Full sample → `samples/webapp-trio/`](https://github.com/olivierdevelops/capy/tree/main/samples/webapp-trio)

=== "Android app skeleton — 7 files"

    ```
    app "Habit Tracker"
        package "com.example.habits"
        min_sdk 24
        target_sdk 34
        version_name "0.1.0"

        screen Home    "Today's habits"
        screen History "Past 30 days"

        feature Home    "Daily checklist"
        feature History "Calendar view"
    end
    ```

    →

    ```
    out/
    ├── settings.gradle.kts
    ├── README.md
    └── app/
        ├── build.gradle.kts
        └── src/main/
            ├── AndroidManifest.xml
            ├── java/MainActivity.kt
            └── res/{layout,values}/...
    ```

    [Full sample → `samples/android-app/`](https://github.com/olivierdevelops/capy/tree/main/samples/android-app)

=== "iOS SwiftUI skeleton — 6 files"

    **Same source shape as the Android demo** — only the library
    swaps. One declaration, two platforms.

    ```
    app "Habit Tracker"
        bundle_id "com.example.habits"
        version "0.1.0"
        deployment_target "16.0"

        screen Home    "Today's habits"
        feature Home   "Daily checklist"
    end
    ```

    →

    ```
    out/
    ├── Info.plist
    ├── Package.swift
    └── Sources/
        ├── App.swift          (@main App entry)
        ├── RootView.swift
        └── Screens.swift
    ```

    `HabitTrackerApp` comes from `"Habit Tracker"` via the
    `pascalCase` template helper.

    [Full sample → `samples/ios-app/`](https://github.com/olivierdevelops/capy/tree/main/samples/ios-app)

=== "libtorch C++ ML trainer — 5 files"

    ```
    model "MNIST classifier"
        dataset MNIST
        batch_size 64
        epochs 10
        optimizer adam

        layer conv2d   in 1   out 32   kernel 3
        layer relu
        layer maxpool  kernel 2
        layer conv2d   in 32  out 64   kernel 3
        layer relu
        layer maxpool  kernel 2
        layer flatten
        layer linear   in 1600 out 128
        layer relu
        layer linear   in 128  out 10
    end
    ```

    →

    ```
    out/
    ├── CMakeLists.txt
    ├── run.sh
    └── src/
        ├── model.h        (torch::nn::Module + register_module + forward)
        └── main.cpp       (training loop with optimizer + checkpointing)
    ```

    The same architecture could target PyTorch Python,
    TensorFlow/Keras, ONNX, TFLite — write a new library, keep
    the source.

    [Full sample → `samples/libtorch-train/`](https://github.com/olivierdevelops/capy/tree/main/samples/libtorch-train)

=== "How to use it"

    Two flags:

    ```sh
    # Write a directory tree
    capy run --out-dir build  lib.capy script.capy

    # OR bundle to a zip
    capy run --zip project.zip  lib.capy script.capy
    ```

    [Full pattern → `one-source-many-files.md`](one-source-many-files.md)

---

## 🌳 Multi-file projects (basic) — one source, a whole project tree

A single Capy library can declare any number of output files. Run
with `--out-dir generated` and Capy writes the entire tree
(subdirectories included).

=== "Source (9 lines)"

    ```
    project "todo-api"
        description "A tiny TODO REST service"
        author "you@example.com"

        route GET    "/health"     health_check
        route GET    "/todos"      list_todos
        route POST   "/todos"      create_todo
        route GET    "/todos/{id}" get_todo
        route DELETE "/todos/{id}" delete_todo
    end
    ```

=== "Generated tree"

    ```
    generated/
    ├── README.md
    ├── pyproject.toml
    ├── .gitignore
    ├── src/
    │   ├── main.py            ← FastAPI app with 5 routes mounted
    │   └── handlers.py        ← Handler stubs
    └── tests/
        └── test_smoke.py      ← Smoke tests
    ```

    Add a `route` line to the source, re-run, and every file that
    mentions routes regenerates. The other files (.gitignore,
    pyproject.toml) stay identical.

=== "Library snippet"

    The library has six `file "..."` blocks at the top level:

    ```
    file "README.md"
        write `# ${context.name | unquote}
${context.description | unquote}
`
    end

    file "src/main.py"
        write `"""Generated FastAPI app for ${context.name | unquote}."""
from fastapi import FastAPI
from . import handlers

app = FastAPI(title=${context.name | toQuoted})

`
        for r in context.routes
            write `@app.${r.method | lower}(${r.path | toQuoted})
async def ${r.handler}_endpoint(*args, **kwargs):
    return await handlers.${r.handler}(*args, **kwargs)

`
        end
    end

    file "tests/test_smoke.py"
        ...
    end
    ```

    Each block has a path (subdirectories OK) and a write-style
    body with access to `context` and (for block functions) `body`.

[Full sample → `samples/multi-file-project/`](https://github.com/olivierdevelops/capy/tree/main/samples/multi-file-project) ·
[Pattern docs → multi-file & imports](multi-file-and-imports.md)

---

## 🧩 Library composition — split shared types & syntax

Libraries can `import` other libraries. Use this to keep validators
(Email, URL, Semver) and shared syntax helpers (`tag`, `note`,
`meta`) in one canonical place that every project imports.

=== "Layout"

    ```
    lib-composition/
    ├── lib.capy                ← main: imports + project-specific functions
    ├── common/
    │   ├── types.capy          ← shared types
    │   └── syntax.capy         ← shared functions
    └── script.capy
    ```

=== "Main library"

    ```
    import "common/types.capy"
    import "common/syntax.capy"

    extension md

    function post
        arg literal "post"
        arg capture title string
        block_closer end
        write `# ${unquote title}

*By ${unquote context.meta.author}*

Tags: `
        for t in context.tags
            write `#${t} `
        end
        write `

---

${body}
`
    end
    ```

=== "Imported types (common/types.capy)"

    ```
    type Email
        pattern "^[^@]+@[^@]+\\.[^@]+$"
    end

    type URL
        pattern "^https?://[^ ]+$"
    end

    type Semver
        pattern "^[0-9]+\\.[0-9]+\\.[0-9]+(-[a-zA-Z0-9.]+)?$"
    end
    ```

=== "Imported functions (common/syntax.capy)"

    ```
    function tag
        arg literal "tag"
        arg capture name ident
        append context.tags name
    end

    function note
        arg literal "note"
        arg capture text string
        write `> **Note:** ${unquote text}
`
    end
    ```

After `capy check lib.capy`: **6 functions, 4 types** — three
functions and all four types come from imports, three functions
are local. Conflicts resolve importer-wins; cycles are detected.

[Full sample → `samples/lib-composition/`](https://github.com/olivierdevelops/capy/tree/main/samples/lib-composition)

---

## 📜 Grammar as contract — one source, many consumers

A Capy grammar isn't just a parser definition — it's a **machine-
verified contract**. Once it parses, downstream consumers can start
building against it *before* the libraries that target it are
implemented. Add new targets later without touching the source.

=== "Source (the contract)"

    ```
    api "PetStore" version "1.0.0"

    endpoint GET "/pets"
        summary "List all pets"
        returns "Pet[]"
    end

    endpoint POST "/pets"
        summary "Create a pet"
        param body "Pet"
        returns "Pet"
    end

    endpoint GET "/pets/{id}"
        summary "Get a pet by ID"
        param path id int
        returns "Pet"
    end

    endpoint DELETE "/pets/{id}"
        summary "Delete a pet"
        param path id int
        returns "void"
    end
    ```

    Frontend devs can mock against this *today*. Library
    implementations can land next week, next month, never — the
    contract is stable.

=== "→ OpenAPI YAML"

    ```yaml
    openapi: 3.0.3
    info:
      title: PetStore
      version: 1.0.0
    paths:
      - path: "/pets"
        method: GET
        summary: "List all pets"
        returns: "Pet[]"
      - path: "/pets"
        method: POST
        summary: "Create a pet"
        param: { in: body, schema: "Pet" }
        returns: "Pet"
      - path: "/pets/{id}"
        method: GET
        param: { in: path, name: id, type: int }
        returns: "Pet"
    ```

=== "→ TypeScript client stubs"

    ```typescript
    // PetStore — generated TypeScript client.

    // GET "/pets"
    // List all pets
    export async function GET_handler(path: string): Promise<unknown> {
      const res = await fetch(path, { method: "GET" });
      return res.json();
    }

    // POST "/pets"
    // Create a pet
    // @param body "Pet"
    export async function POST_handler(path: string): Promise<unknown> {
      const res = await fetch(path, { method: "POST" });
      return res.json();
    }
    ```

=== "→ Markdown API docs"

    ```markdown
    # PetStore API — v1.0.0

    *Generated from the canonical Capy contract. Edit script.capy, not this file.*

    ## `GET "/pets"`
    List all pets

    - **Returns**: `Pet[]`

    ## `POST "/pets"`
    Create a pet

    - **Request body**: `Pet`
    - **Returns**: `Pet`
    ```

All three outputs come from the same `script.capy`. Every commit
runs a golden test that proves they still match. Add a 4th target
(Postman? FastAPI server? Rust client?) by writing one 30-line
library — the contract guarantees compatibility.

[Full sample → `samples/contract-first-api/`](https://github.com/olivierdevelops/capy/tree/main/samples/contract-first-api) ·
[Pattern docs → `grammar-as-contract.md`](grammar-as-contract.md)

---

## ⚡ Supercharge existing syntax — Capy as a preprocessor

Capy doesn't have to invent a new language. The most practical
pattern is to put Capy macros *on top of* an existing target — SQL,
Markdown, HTML, Dockerfile, Kubernetes — so authors get rich
declarations while the runtime still consumes plain target syntax.

=== "SQL DDL with macros"

    Source (`script.capy`):

    ```
    table users
        pk id
        col email "varchar(255) UNIQUE NOT NULL"
        col name  "varchar(100)"
        timestamps
    end

    table posts
        pk id
        fk author_id -> users
        col title "varchar(255) NOT NULL"
        col body  "text"
        timestamps
        soft_delete
    end

    index posts author_id
    ```

    Expanded Postgres DDL (`capy run lib.capy script.capy`):

    ```sql
    CREATE TABLE users (
      id bigserial PRIMARY KEY,
      email varchar(255) UNIQUE NOT NULL,
      name varchar(100),
      created_at timestamptz NOT NULL DEFAULT now(),
      updated_at timestamptz NOT NULL DEFAULT now()
    );
    CREATE TABLE posts (
      id bigserial PRIMARY KEY,
      author_id bigint NOT NULL REFERENCES users(id),
      title varchar(255) NOT NULL,
      body text,
      created_at timestamptz NOT NULL DEFAULT now(),
      updated_at timestamptz NOT NULL DEFAULT now(),
      deleted_at timestamptz
    );
    CREATE INDEX ix_posts_author_id ON posts(author_id);
    ```

    `psql -f schema.sql` — your database doesn't know Capy exists.
    Capy is just a preprocessor that ran before `psql`.

    [Full sample → `samples/supercharge-sql/`](https://github.com/olivierdevelops/capy/tree/main/samples/supercharge-sql)

=== "Markdown blog with components"

    Source:

    ```
    post "Adopting Capy at Acme" date "2026-05-24" author "Alice"
        tag rust
        tag devtools

        para "We replaced 3 generators with one Capy library."
        h2 "Why Capy"
        bullet "Single source, multiple targets."
        bullet "Library doubles as the spec."

        callout note "This post is itself generated by Capy."

        card "Generators retired" "3" "all replaced by 1 library"
    end
    ```

    Output is **real Markdown** — YAML frontmatter, blockquote
    callouts, inline HTML cards. Drop it into Hugo / Jekyll / MkDocs
    / Next.js / Astro; they all render it natively.

    ```markdown
    ---
    title: "Adopting Capy at Acme"
    date: "2026-05-24"
    author: "Alice"
    tags: ["rust", "devtools"]
    ---

    # Adopting Capy at Acme

    *By Alice · 2026-05-24*

    We replaced 3 generators with one Capy library.

    ## Why Capy

    - Single source, multiple targets.
    - Library doubles as the spec.

    > **NOTE:** This post is itself generated by Capy.

    <div class="metric-card">
      <h3>Generators retired</h3>
      <p class="metric">3</p>
      <p class="caption">all replaced by 1 library</p>
    </div>
    ```

    [Full sample → `samples/supercharge-markdown/`](https://github.com/olivierdevelops/capy/tree/main/samples/supercharge-markdown)

=== "The pattern"

    Any textual host format can be supercharged this way:

    | Host format          | What Capy adds                                         |
    |----------------------|--------------------------------------------------------|
    | **SQL DDL**          | `pk` / `fk` / `timestamps` / `soft_delete` macros      |
    | **Markdown**         | Frontmatter, callouts, cards, code blocks              |
    | **HTML**             | Component primitives → plain HTML+CSS+JS               |
    | **Dockerfile**       | `base` / `apt` / `pip` / multi-stage shortcuts         |
    | **GitHub Actions**   | `job` / `steps` shorthand → full workflow YAML         |
    | **Terraform HCL**    | Module shortcuts, env-aware defaults                   |
    | **Kubernetes**       | One-liner deployments → full manifests                 |
    | **OpenAPI**          | Endpoint shorthand → full operation + schema           |
    | **Mermaid**          | High-level diagram syntax → node + edge DSL            |

    The recipe is always identical: define a Capy library whose
    `file_template` outputs the host format. Authors compose at
    the high level; the existing runtime consumes the low-level
    output unchanged.

    [Full pattern docs → `extending-existing-syntax.md`](extending-existing-syntax.md)

---

## 🔒 Named variables + type checking

Capy captures are **named** and **typed**. Built-in kinds (`int`,
`string`, `bool`, `ident`, …) get checked at the token level;
library-defined types add `pattern:` (regex), `options:` (enum), and
`base:` (inheritance). Bad input becomes a precise transpile-time
error pointing at the offending value — not a silent mis-render or
a runtime surprise.

=== "Library — typed schema"

    ```
    # Custom types
    type Email
        pattern "^[^@]+@[^@]+\\.[^@]+$"
    end

    type Semver
        pattern "^[0-9]+\\.[0-9]+\\.[0-9]+(-[a-zA-Z0-9.]+)?$"
    end

    type LogLevel
        options "trace" "debug" "info" "warn" "error" "fatal"
    end

    type Env
        options "dev" "staging" "prod"
    end

    type Port
        base int                  # validation inheritance
    end

    type ServiceName
        pattern "^[a-z][a-z0-9-]{1,30}$"
    end

    # Named typed captures
    function service
        arg literal "service"
        arg capture name ServiceName   # ← named "name", typed ServiceName
        arg literal "version"
        arg capture ver Semver
        block_closer end
        write `service ${name} {
  version = ${ver}
${indent 2 body}
}
`
    end

    function port
        arg literal "port"
        arg capture n Port             # ← named "n", typed Port (= int)
        write `port = ${n}
`
    end

    function owner
        arg literal "owner"
        arg capture who Email
        write `owner = ${who}
`
    end
    # ... env / log_level / brand_color / tls ...
    ```

=== "Valid source → clean output"

    Source:

    ```
    service "api-gateway" version "2.4.1"
        env prod
        port 8443
        owner "platform@example.com"
        log_level info
        brand_color "#4dd9c0"
        tls true
    end
    ```

    Generated HCL:

    ```hcl
    service api-gateway {
      version = 2.4.1
      env = prod
      port = 8443
      owner = platform@example.com
      log_level = info
      brand_color = #4dd9c0
      tls = true
    }
    ```

=== "Invalid source → precise errors"

    Each line below violates its declared type:

    ```
    service "Bad Name!" version "v2"
        env production
        port 99999
        owner "not-an-email"
        log_level verbose
        brand_color "blue"
        tls maybe
    end
    ```

    Running it:

    ```
    $ capy run lib.capy script-invalid.capy
    error: function "service" arg "name": value "Bad Name!"
           does not match pattern for type "ServiceName"
    ```

    Fix that line, re-run, hit the next error. The transpilation
    refuses to emit until every value satisfies its type.

**Why this matters.** The library *is* your config schema. New
contributors learn what fields exist and what values are valid by
reading the `type:` blocks — no separate spec to maintain. Typos
like `log_level verbose` are caught at the boundary instead of
becoming a silent no-op in production.

[Full sample → `samples/typed-config-dsl/`](https://github.com/olivierdevelops/capy/tree/main/samples/typed-config-dsl)

---

## 🕹️ Event-driven game DSL — bindings & handlers, not just config

The Capy source for these games isn't a list of constants — it
declares **entities, key bindings, AND event handlers**. The library
compiles those declarations into a JS dispatch table + action
implementations. Rebind a key, change a scoring rule, or delete a
game-over condition by editing one line. The library never changes.

=== "Breakout — entities + keys + events"

    **Source** (`script.capy`):

    ```
    game "Breakout" 480 320

    paddle width 80 height 10 color "#3df" speed 7
    ball   radius 6 color "#fff" speed 4

    bricks rows 5 cols 8 width 56 height 14 gap 4
    brick_color 0 "#f55"
    brick_color 1 "#fa4"
    brick_color 2 "#ff4"
    brick_color 3 "#4f6"
    brick_color 4 "#4af"

    on_key "ArrowLeft"  paddle_left
    on_key "ArrowRight" paddle_right
    on_key " "          launch_ball
    on_key "r"          reset

    on_event brick_hit   destroy_brick add_score 10
    on_event paddle_hit  bounce_with_spin
    on_event ball_lost   lose_life
    on_event all_cleared win

    lives 3
    ```

    The four `on_event` lines are **the entire game-logic glue**.
    Want bricks worth 50 points instead of 10? Change one number.
    Want the ball to NOT bounce off the paddle? Delete one line.

    <iframe src="../assets/demos/breakout.html" width="100%" height="400" style="border: 1px solid #444; background: #0a0a14;"></iframe>

    ← / → to move · space to launch · R to restart

    [Library + source → `samples/interactive-breakout/`](https://github.com/olivierdevelops/capy/tree/main/samples/interactive-breakout)

=== "Snake — bindings, events, dual-mapped keys"

    **Source**:

    ```
    game "Snake" 400 400
    grid cols 20 rows 20
    tick every 110

    on_key "ArrowUp"    turn_up
    on_key "ArrowDown"  turn_down
    on_key "ArrowLeft"  turn_left
    on_key "ArrowRight" turn_right
    on_key "w"          turn_up
    on_key "s"          turn_down
    on_key "a"          turn_left
    on_key "d"          turn_right
    on_key " "          pause_toggle
    on_key "r"          reset

    on_event eat_food   grow add_score 10
    on_event hit_wall   game_over
    on_event hit_self   game_over

    snake_color "#9fa"
    food_color  "#f44"
    save_best   "snake_best"
    ```

    Both arrow keys AND WASD map to the same actions — two
    `on_key` lines per direction. Delete `on_event hit_self` and
    the snake passes through itself. Change `add_score 10` to
    `add_score 25` for double points.

    <iframe src="../assets/demos/snake.html" width="100%" height="460" style="border: 1px solid #2a3; background: #0a140a;"></iframe>

=== "What the library generates"

    The DSL above compiles to a JS dispatch table that looks like:

    ```javascript
    const KEY_BINDINGS = {
      "ArrowLeft":  "paddle_left",
      "ArrowRight": "paddle_right",
      " ":          "launch_ball",
      "r":          "reset",
    };

    const EVENT_HANDLERS = {
      brick_hit: (arg) => {
        ACTIONS["destroy_brick"](arg);
        ACTIONS["add_score"](arg, 10);   // ← the number from the DSL
      },
      paddle_hit:  (arg) => { ACTIONS["bounce_with_spin"](arg); },
      ball_lost:   (arg) => { ACTIONS["lose_life"](arg); },
      all_cleared: (arg) => { ACTIONS["win"](arg); },
    };

    document.addEventListener("keydown", (e) => {
      const a = KEY_BINDINGS[e.key];
      if (a) ACTIONS[a]();
    });
    ```

    `ACTIONS` is a table of named JS functions baked into the
    library's `file_template` — `paddle_left`, `launch_ball`,
    `bounce_with_spin`, `lose_life`, etc. The user's DSL never
    contains JS; it just names which action runs for which input
    or which event.

This is the pattern: Capy DSLs can carry **behavior**, not just
configuration. The library provides primitives; the source composes
them.

---

## 🎮 Interactive HTML — see the rendered output

These are full HTML documents. The Capy source is short; the
generated HTML+CSS+JS is dropped into an `<iframe>` so you can
actually use it.

=== "Canvas game"

    **Source** (`script.capy`):

    ```
    game "Block Hopper" 480 320

    sprite player "#4dd" 220 280 40 20
    sprite enemy  "#f64" 100 100 30 30
    sprite goal   "#fd0" 420 20  20 20

    on_key "ArrowLeft"  player -4 0
    on_key "ArrowRight" player  4 0
    on_key "ArrowUp"    player  0 -4
    on_key "ArrowDown"  player  0 4

    tick enemy_bounce "sprites.enemy.x += 1; if (sprites.enemy.x > 450) sprites.enemy.x = 0;"
    ```

    **Generated HTML** (full file, ~67 lines):

    ```html
    <!doctype html>
    <html lang="en">
    <head>
      <title>Block Hopper</title>
      <style>
        body { background: #111; display: grid; place-items: center; }
        canvas { background: #222; border: 1px solid #444; }
      </style>
    </head>
    <body>
      <canvas id="c" width="480" height="320"></canvas>
      <script>
        const canvas = document.getElementById("c");
        const ctx = canvas.getContext("2d");
        const keys = {};
        window.addEventListener("keydown", e => keys[e.key] = true);
        window.addEventListener("keyup",   e => keys[e.key] = false);
        const sprites = {
          player: { x: 220, y: 280, w: 40, h: 20, color: "#4dd" },
          enemy:  { x: 100, y: 100, w: 30, h: 30, color: "#f64" },
          goal:   { x: 420, y: 20,  w: 20, h: 20, color: "#fd0" },
        };
        function update() {
          if (keys["ArrowLeft"])  sprites.player.x += -4;
          if (keys["ArrowRight"]) sprites.player.x +=  4;
          if (keys["ArrowUp"])    sprites.player.y += -4;
          if (keys["ArrowDown"])  sprites.player.y +=  4;
          sprites.enemy.x += 1; if (sprites.enemy.x > 450) sprites.enemy.x = 0;
        }
        function draw() {
          ctx.clearRect(0, 0, canvas.width, canvas.height);
          for (const s of Object.values(sprites)) {
            ctx.fillStyle = s.color;
            ctx.fillRect(s.x, s.y, s.w, s.h);
          }
        }
        function loop() { update(); draw(); requestAnimationFrame(loop); }
        loop();
      </script>
    </body>
    </html>
    ```

    **Rendered** — click the canvas, then use arrow keys:

    <iframe src="../assets/demos/canvas-game.html" width="100%" height="380" style="border:1px solid #ccc;border-radius:6px;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-canvas-game)

=== "Landing page"

    **Source** (`script.capy`):

    ```
    title "Capy — DSLs in YAML"
    hero  "Define a language. Get a transpiler." "Capy is a tiny engine that turns a YAML file into a working code generator."

    feature "🌱" "Zero default grammar" "Your library is the language."
    feature "⚡" "Fast"                  "Single-binary Rust. Boots in milliseconds."
    feature "🧩" "50 sample DSLs"        "From Python to Mermaid to a real x86-64 transpiler."

    cta "Get started" "/docs/getting-started"
    cta "GitHub"      "https://github.com/olivierdevelops/capy"
    ```

    **Generated** — a complete responsive HTML page with embedded CSS,
    hero section, features grid, and CTAs.

    **Rendered**:

    <iframe src="../assets/demos/landing-page.html" width="100%" height="500" style="border:1px solid #ccc;border-radius:6px;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-landing-page)

=== "HTML email"

    **Source** (`script.capy`):

    ```
    subject "Welcome to Capy"
    preview "Your account is ready."

    heading "Welcome to Capy!"
    para    "Thanks for signing up. You're all set to start building DSLs."
    para    "Click below to read the getting-started guide."
    button  "Get started" "https://capy.dev/getting-started"
    divider
    footer  "Sent by Capy. Unsubscribe at capy.dev/unsubscribe."
    ```

    **Generated** — an HTML email with all styles inlined (the format
    that survives Gmail, Outlook, etc.).

    **Rendered**:

    <iframe src="../assets/demos/email.html" width="100%" height="500" style="border:1px solid #ccc;border-radius:6px;background:#f4f4f4;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-email-html)

=== "HTML form"

    **Source** (`script.capy`):

    ```
    form "/contact"
        field name "text" "Your name"
        field email "email" "Email address"
        textarea message "Message"
    end
    ```

    **Generated**:

    ```html
    <form action="/contact" method="post">
      <label for="name">Your name</label>
      <input id="name" name="name" type="text" />
      <label for="email">Email address</label>
      <input id="email" name="email" type="email" />
      <label for="message">Message</label>
      <textarea id="message" name="message"></textarea>
      <button type="submit">Submit</button>
    </form>
    ```

    **Rendered** — try typing in the fields:

    <iframe src="../assets/demos/form.html" width="100%" height="420" style="border:1px solid #ccc;border-radius:6px;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-form)

=== "Component card"

    **Source** (`script.capy`):

    ```
    component card "Welcome" {
        text "Capy makes transpilers easy."
        text "Try editing this component."
    }
    ```

    **Generated**:

    ```html
    <div id="card" class="card">
      <h3>"Welcome"</h3>
      <p>"Capy makes transpilers easy."</p>
      <p>"Try editing this component."</p>
    </div>
    ```

    **Rendered**:

    <iframe src="../assets/demos/component-card.html" width="100%" height="280" style="border:1px solid #ccc;border-radius:6px;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/html-component)

=== "CSS animations"

    **Source** (`script.capy`):

    ```
    keyframe pulse
        at 0   transform = "scale(1)"
        at 50  transform = "scale(1.1)"
        at 100 transform = "scale(1)"
    end

    class ".card"
        background = "#fff"
        border_radius = "8px"
        animate slide_in "0.4s" "ease-out"
    end

    class ".badge"
        animate pulse "1.2s" "ease-in-out"
    end
    ```

    **Generated CSS** — `@keyframes` rules and animated classes.

    **Rendered** — the card slides in; the badge pulses:

    <iframe src="../assets/demos/css-animations.html" width="100%" height="280" style="border:1px solid #ccc;border-radius:6px;"></iframe>

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-css-animations)

---

## 📊 Diagrams — generated Mermaid

Capy emits Mermaid; the docs site renders it inline.

=== "Flowchart"

    **Source** (`script.capy`):

    ```
    flowchart LR
        node a "Source"
        node b "Lexer"
        node c "Parser"
        node d "Evaluator"
        node e "Output"
        a -> b
        b -> c
        c -> d : "match + render"
        d -> e
    end
    ```

    **Generated Mermaid** — rendered live:

    ```mermaid
    flowchart LR
      a[Source]
      b[Lexer]
      c[Parser]
      d[Evaluator]
      e[Output]
      a --> b
      b --> c
      c -->|match + render| d
      d --> e
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-mermaid)

=== "State diagram"

    **Source** (`script.capy`):

    ```
    machine Order
        state Pending
        state Paid
        state Shipped
        state Delivered

        Pending -> Paid on "payment"
        Paid -> Shipped on "fulfill"
        Shipped -> Delivered on "arrival"

        final Delivered
    end
    ```

    **Generated state diagram** — rendered live:

    ```mermaid
    stateDiagram-v2
      [*] --> Order
      state Pending
      state Paid
      state Shipped
      state Delivered
      Pending --> Paid : payment
      Paid --> Shipped : fulfill
      Shipped --> Delivered : arrival
      Delivered --> [*]
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-statemachine)

---

## 📝 Rendered Markdown — generated and shown inline

The Capy output IS Markdown, so MkDocs renders it directly on this
page. What you see below is the actual generated text, formatted.

=== "Todo list"

    **Source** (`script.capy`):

    ```
    section "Today"
    todo "Write the launch blog post"
    done  "Tag v0.1.0"
    todo  "Test install script on Linux"

    section "This week"
    todo "Publish VS Code extension"
    done "Move .codestyle to docs"
    ```

    **Generated and rendered**:

    ## Today

    - [ ] Write the launch blog post
    - [x] Tag v0.1.0
    - [ ] Test install script on Linux

    ## This week

    - [ ] Publish VS Code extension
    - [x] Move .codestyle to docs

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-markdown-todo)

=== "Invoice"

    **Source** (`script.capy`):

    ```
    number "INV-2026-001"
    date   "2026-05-23"
    bill_to "Acme Corp"

    item "Consulting hours"     8 "$120.00"
    item "Capy enterprise plan" 1 "$2000.00"
    item "Onboarding workshop"  2 "$500.00"
    ```

    **Generated and rendered**:

    ## Invoice INV-2026-001

    **To:** Acme Corp
    **Date:** 2026-05-23

    | Item                  | Qty | Unit price |
    |-----------------------|----:|-----------:|
    | Consulting hours      |   8 | $120.00    |
    | Capy enterprise plan  |   1 | $2000.00   |
    | Onboarding workshop   |   2 | $500.00    |

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-invoice)

=== "Changelog"

    **Source** (`script.capy`):

    ```
    version "0.2.0" "2026-06-15"
        added   "Configurable surface syntax"
        added   "else arm on inner if"
        fixed   "Indentation tokenisation edge case"
    end

    version "0.1.0" "2026-05-23"
        added   "Initial public release"
        added   "Type system with pattern + options"
        added   "Two block modes"
    end
    ```

    **Generated and rendered**:

    ## [0.2.0] — 2026-06-15

    - Added: Configurable surface syntax
    - Added: else arm on inner if
    - Fixed: Indentation tokenisation edge case

    ## [0.1.0] — 2026-05-23

    - Added: Initial public release
    - Added: Type system with pattern + options
    - Added: Two block modes

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-changelog)

---

## 💻 Code generation — source + generated, side by side

These targets are runnable code rather than rendered output. Save
the generated file, run it.

=== "Python"

    **Source**:

    ```
    import json
    import os
    say "hello, world"

    if x
        say "x is set"
    end

    loop n in [1, 2, 3]
        say n
    end
    ```

    **Generated `out.py`**:

    ```python
    import json
    import os
    print("hello, world")
    if x:
        print("x is set")

    for n in [1, 2, 3]:
        print(n)
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-py)

=== "PostgreSQL"

    **Source**:

    ```
    table users
        pk     id
        unique email "varchar(255)"
        col    name  "varchar(255) NOT NULL"
    end

    table posts
        pk     id
        fk     author_id -> users
        col    title "varchar(255) NOT NULL"
    end

    index users email
    index posts author_id
    ```

    **Generated `schema.sql`**:

    ```sql
    CREATE TABLE users (
      id bigserial PRIMARY KEY,
      email varchar(255) UNIQUE NOT NULL,
      name varchar(255) NOT NULL
    );
    CREATE TABLE posts (
      id bigserial PRIMARY KEY,
      author_id bigint NOT NULL REFERENCES users(id),
      title varchar(255) NOT NULL
    );

    CREATE INDEX ix_users_email ON users(email);
    CREATE INDEX ix_posts_author_id ON posts(author_id);
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-postgres-schema)

=== "Express server"

    **Source**:

    ```
    port 8080

    use "morgan('combined')"
    get  "/health" "res.json({ok: true})"
    post "/users"  "const u = req.body; res.status(201).json({id: 42, ...u})"
    ```

    **Generated `server.js`**:

    ```javascript
    const express = require("express");
    const app = express();

    app.use(express.json());
    app.use(morgan('combined'));

    app.get("/health", (req, res) => {
      res.json({ok: true})
    });

    app.post("/users", (req, res) => {
      const u = req.body; res.status(201).json({id: 42, ...u})
    });

    app.listen(8080, () => { console.log("listening on", 8080); });
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-express-server)

=== "Terraform"

    **Source**:

    ```
    provider "aws" "us-east-1"

    resource "aws_instance" web
        ami = "ami-0c55b159cbfafe1f0"
        instance_type = "t3.micro"
        tag "Name" "capy-web"
    end
    ```

    **Generated `main.tf`**:

    ```hcl
    provider "aws" {
      region = "us-east-1"
    }

    resource "aws_instance" "web" {
      ami = "ami-0c55b159cbfafe1f0"
      instance_type = "t3.micro"
      tags = { "Name" = "capy-web" }
    }
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-terraform)

=== "Kubernetes"

    **Source**:

    ```
    deployment capy_api
    image    "ghcr.io/olivierdevelops/capy:0.1.0"
    replicas 3
    port     8080
    ```

    **Generated `deployment.yaml`**:

    ```yaml
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: capy_api
    spec:
      replicas: 3
      template:
        spec:
          containers:
            - name: capy_api
              image: ghcr.io/olivierdevelops/capy:0.1.0
              ports:
                - containerPort: 8080
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-kubernetes)

=== "Slack Block Kit"

    **Source**:

    ```
    header  "📦 Build complete"
    section "Branch *main* built in *4m 12s* and is ready to deploy."
    divider
    section "Tests: 124/124 passing"
    button  "View build" "https://ci.example.com/build/1234"
    ```

    **Generated JSON** (POST to a Slack webhook):

    ```json
    {
      "blocks": [
        { "type": "header", "text": { "type": "plain_text", "text": "📦 Build complete" } },
        { "type": "section", "text": { "type": "mrkdwn", "text": "Branch *main* built in *4m 12s* and is ready to deploy." } },
        { "type": "divider" },
        { "type": "section", "text": { "type": "mrkdwn", "text": "Tests: 124/124 passing" } },
        { "type": "actions", "elements": [{ "type": "button", "text": { "type": "plain_text", "text": "View build" }, "url": "https://ci.example.com/build/1234" }] }
      ]
    }
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/transpile-slack-blocks)

=== "Assembly (x86-64)"

    **Source**:

    ```
    program "sum-demo"
        var x = 5
        var y = 7
        add x y
        store result
        exit 0
    end
    ```

    **Generated `demo.asm`** (assembles with `nasm -felf64 demo.asm -o demo.o && ld demo.o -o demo`):

    ```asm
    section .data
        x: dq 0
        y: dq 0

    section .text
        global _start

    _start:
        mov rax, 5
        mov [x], rax
        mov rax, 7
        mov [y], rax
        mov rax, [x]
        add rax, [y]
        mov [result], rax
        mov rdi, 0
        mov rax, 60
        syscall
    ```

    [Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/assembly)

---

## 🏗️ One scene → five 3D / game tools

The same 8-line scene description compiled to **runnable scripts for
Blender, SketchUp, Rhino, Unity, and Unreal**. No host-API
vocabulary in the source — just primitives.

=== "Source (`script.capy`)"

    ```
    scene "Studio"

    cube   red    0 0 0   2
    sphere blue   4 0 0   1
    plane  gray   0 0 0   10

    light   5 5 5
    camera  7 7 3
    ```

=== "→ Blender (Python `bpy`)"

    ```python
    import bpy

    _COLORS = {
        "red":   (1.0, 0.1, 0.1, 1.0),
        "blue":  (0.1, 0.3, 1.0, 1.0),
        "green": (0.1, 0.8, 0.2, 1.0),
        "gray":  (0.6, 0.6, 0.6, 1.0),
    }

    def _paint(obj, color):
        mat = bpy.data.materials.new(name=color)
        mat.diffuse_color = _COLORS.get(color, (1, 1, 1, 1))
        obj.data.materials.append(mat)

    bpy.context.scene.name = "Studio"
    # cube (red)
    bpy.ops.mesh.primitive_cube_add(location=(0, 0, 0), size=2)
    _paint(bpy.context.active_object, "red")
    # sphere (blue)
    bpy.ops.mesh.primitive_uv_sphere_add(location=(4, 0, 0), radius=1)
    _paint(bpy.context.active_object, "blue")
    # plane (gray)
    bpy.ops.mesh.primitive_plane_add(location=(0, 0, 0), size=10)
    _paint(bpy.context.active_object, "gray")
    bpy.ops.object.light_add(type='POINT', location=(5, 5, 5))
    bpy.ops.object.camera_add(location=(7, 7, 3))
    ```

    Paste into Blender's Scripting tab. Done.

=== "→ SketchUp (Ruby)"

    ```ruby
    # Generated by Capy — paste into the SketchUp Ruby Console.
    model = Sketchup.active_model
    ent = model.active_entities

    # … helpers _add_box / _add_sphere / _add_plane / _paint elided …

    model.name = "Studio"
    # cube (red)
    _add_box(ent, 0, 0, 0, 2, "red")
    # sphere (blue)
    _add_sphere(ent, 4, 0, 0, 1, "blue")
    # plane (gray)
    _add_plane(ent, 0, 0, 0, 10, "gray")
    model.active_view.camera = Sketchup::Camera.new([7, 7, 3], [0, 0, 0], [0, 0, 1])
    ```

=== "→ Rhino (C# / RhinoCommon)"

    ```csharp
    using System.Drawing;
    using Rhino;
    using Rhino.DocObjects;
    using Rhino.Geometry;

    public static class CapyScene
    {
        public static void Build(RhinoDoc doc)
        {
            // cube (red)
            {
                var c = new Point3d(0, 0, 0);
                double s = 2 / 2.0;
                var box = new Box(new Plane(c, Vector3d.ZAxis),
                                  new Interval(-s, s), new Interval(-s, s), new Interval(-s, s));
                doc.Objects.AddBox(box, Attr("red"));
            }
            // sphere (blue)
            {
                var sph = new Sphere(new Point3d(4, 0, 0), 1);
                doc.Objects.AddSphere(sph, Attr("blue"));
            }
            // … plane / light / camera …
        }
    }
    ```

    Drop into a Grasshopper C# scripting component, or compile as a
    Rhino plugin command.

=== "→ Unity (C# MonoBehaviour)"

    ```csharp
    using UnityEngine;
    using System.Collections.Generic;

    public class CapyScene : MonoBehaviour
    {
        // … COLORS table + Spawn helper elided …

        void Start()
        {
            gameObject.name = "Studio";
            // cube (red)
            Spawn(PrimitiveType.Cube, new Vector3(0, 0, 0), Vector3.one * 2, "red");
            // sphere (blue)
            Spawn(PrimitiveType.Sphere, new Vector3(4, 0, 0), Vector3.one * 1 * 2f, "blue");
            // plane (gray)
            Spawn(PrimitiveType.Plane, new Vector3(0, 0, 0), Vector3.one * 10 * 0.1f, "gray");
            // point light
            {
                var go = new GameObject("PointLight");
                go.transform.position = new Vector3(5, 5, 5);
                go.AddComponent<Light>().type = LightType.Point;
            }
            // camera
            {
                var go = new GameObject("Camera");
                go.transform.position = new Vector3(7, 7, 3);
                go.transform.LookAt(Vector3.zero);
                go.AddComponent<Camera>();
            }
        }
    }
    ```

    Drop into `Assets/Scripts/CapyScene.cs` and attach to an empty
    GameObject.

=== "→ Unreal (Python editor scripting)"

    ```python
    import unreal

    # … _PRIMITIVES table + _spawn_primitive helper elided …

    unreal.log("Studio")
    # cube (red)
    _spawn_primitive("Cube", unreal.Vector(0*100, 0*100, 0*100), 2, "red")
    # sphere (blue)
    _spawn_primitive("Sphere", unreal.Vector(4*100, 0*100, 0*100), 1*2, "blue")
    # plane (gray)
    _spawn_primitive("Plane", unreal.Vector(0*100, 0*100, 0*100), 10, "gray")
    unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.PointLight, unreal.Vector(5*100, 5*100, 5*100))
    unreal.EditorLevelLibrary.spawn_actor_from_class(
        unreal.CameraActor, unreal.Vector(7*100, 7*100, 3*100))
    ```

    Note the `*100` — Capy's library handles the meters-to-Unreal-cm
    conversion so the source stays in human units.

**Why this matters for 3D / game pipelines.** The same procedural
building gets rewritten in five different host APIs — and they
*drift*. An algorithm change means hand-editing five scripts. LLMs
hallucinate API names because each tool's vocabulary is slightly
different. With Capy you write the scene **once**; each library
encodes one tool's quirks. Add Maya MEL, Houdini Python, Three.js,
glTF — write a 50-line library, never touch the source.

[Full sample → `samples/3d-tools-demo/`](https://github.com/olivierdevelops/capy/tree/main/samples/3d-tools-demo)

---

## 🌍 One source → five programming languages

The same 10-line `script.capy` compiled to **five different
programming languages** by five different libraries. Each output is a
real, runnable program that prints `12`.

=== "Source (`script.capy`)"

    ```
    fn add(a, b)
        return a + b
    end

    main
        let x = 5
        let y = 7
        let z = add(x, y)
        print z
    end
    ```

    Ten lines. Defines a function, calls it, prints the result. The
    grammar (`fn`, `return`, `main`, `let`, `print`) is defined by the
    libraries — not by Capy itself.

=== "→ Python"

    ```python
    def add(a, b):
        return a + b


    if __name__ == "__main__":
        x = 5
        y = 7
        z = add(x, y)
        print(z)
    ```

=== "→ JavaScript"

    ```javascript
    function add(a, b) {
      return a + b;
    }


    (function main() {
      const x = 5;
      const y = 7;
      const z = add(x, y);
      console.log(z);
    })();
    ```

=== "→ Go"

    ```go
    package main

    import "fmt"


    func add(a int, b int) int {
        return a + b
    }


    func main() {
        x := 5
        y := 7
        z := add(x, y)
        fmt.Println(z)
    }
    ```

=== "→ Rust"

    ```rust
    fn add(a: i32, b: i32) -> i32 {
        return a + b;
    }


    fn main() {
        let x: i32 = 5;
        let y: i32 = 7;
        let z: i32 = add(x, y);
        println!("{}", z);
    }
    ```

=== "→ C"

    ```c
    #include <stdio.h>

    int add(int a, int b) {
        return a + b;
    }


    int main(void) {
        int x = 5;
        int y = 7;
        int z = add(x, y);
        printf("%d\n", z);
        return 0;
    }
    ```

**Why this matters.** Maintaining the "same logic in N languages"
problem is real: client SDKs that drift, an algorithm needed in
Python *and* C++, a validator that runs in the browser *and* on the
server. With Capy you write the logic **once**; adding a sixth target
is a ~50-line library file. The next time you change the algorithm,
all five (or six, or ten) outputs regenerate.

[Full sample → `samples/multi-language-demo/`](https://github.com/olivierdevelops/capy/tree/main/samples/multi-language-demo)

### Bonus: the library itself

Libraries are `.capy` files. Same grammar as the user scripts above —
one parser, one mental model.

```
extension c

function fn
    arg literal "fn"
    arg capture name ident
    arg literal "("
    arg capture a ident
    arg literal ","
    arg capture b ident
    arg literal ")"
    block_closer end
    write `int ${name}(int ${a}, int ${b}) {
${indent 4 body}
}
`
end
```

See [`.capy` libraries](capy-libraries.md) for the full grammar.

---

## 🔀 Same source, three targets

The clearest demonstration of "the library is the grammar". One
input file, three libraries, three completely different artifacts.

=== "Source"

    The same `script.capy` for all three:

    ```
    user alice 30 active
    user bob   25 inactive
    user carol 42 active
    ```

=== "→ SQL"

    Running `capy run lib_sql.yaml script.capy` produces SQL inserts:

    ```sql
    INSERT INTO users (name, age, status) VALUES ('alice', 30, 'active');
    INSERT INTO users (name, age, status) VALUES ('bob', 25, 'inactive');
    INSERT INTO users (name, age, status) VALUES ('carol', 42, 'active');
    ```

=== "→ JSON"

    Running `capy run lib_json.yaml script.capy` produces JSON:

    ```json
    {
      "users": [
        { "name": "alice", "age": 30, "status": "active" },
        { "name": "bob",   "age": 25, "status": "inactive" },
        { "name": "carol", "age": 42, "status": "active" }
      ]
    }
    ```

=== "→ Markdown"

    Running `capy run lib_md.yaml script.capy` produces Markdown:

    | Name  | Age | Status   |
    |-------|----:|----------|
    | alice | 30  | active   |
    | bob   | 25  | inactive |
    | carol | 42  | active   |

The libraries are 8–15 lines each. Add a fourth target (CSV, YAML,
HTML table, …) by writing a fourth library — never touch the source.

[Full sample →](https://github.com/olivierdevelops/capy/tree/main/samples/multi-target-demo)

---

## What's not shown here

The 50 sample demos in the repo include more code-gen targets that
don't fit on a single doc page:

- **Backend frameworks**: Flask, FastAPI, Cobra CLI
- **Schemas**: Prisma, Zod, XState v5, Protobuf, GraphQL
- **Config**: nginx, systemd, GitHub Actions, cron, .env, Dockerfile,
  Makefile, OpenAPI, Prometheus alerts, Chrome MV3 manifest
- **Other**: CSV, Markdown CV, Markdown blog with YAML front matter,
  Markdown API reference

[Browse all 50 demos →](https://github.com/olivierdevelops/capy/tree/main/samples)

---

## How to run any of these locally

```sh
cargo install --git https://github.com/olivierdevelops/capy capy-cli
git clone https://github.com/olivierdevelops/capy
cd capy
capy run samples/transpile-canvas-game/lib.capy samples/transpile-canvas-game/script.capy > game.html
open game.html
```

Or just look at the `lib.capy` — it's the entire grammar in 30–60 lines.
