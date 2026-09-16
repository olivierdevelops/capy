---
title: Errors & debugging
---

# Errors & debugging

Capy errors are designed to be **actionable**: each one tells you
where the problem is, what the engine expected instead, and (when
possible) a concrete fix to try.

## Anatomy of a Capy error

```
error: function "service" arg "name": value "Bad Name!" does not match pattern for type "ServiceName"
  hint: type "ServiceName" requires the value to match regex /^[a-z][a-z0-9-]{1,30}$/
  3 │     service "Bad Name!" version "2.4.1"
    │             ^
```

Four parts:

1. **`error: …`** — what went wrong, named in terms of the
   library's vocabulary (which function, which argument, which type).
2. **`hint: …`** — how to fix it. For pattern violations the regex
   is shown; for option mismatches the valid options are listed; for
   typos the closest known name is suggested.
3. **`<line> │ <source>`** — the offending line, verbatim.
4. **`    │      ^`** — a caret pointing at the column.

## Three common errors, walked through

### 1. Unknown function (typo in DSL keyword)

```
endpiont GET "/users"
```

```
error: no library function matches token "endpiont"
  hint: did you mean "endpoint"?
  1 │ endpiont GET "/users"
    │ ^
```

The engine compared `endpiont` to every library function name
using Levenshtein distance. `endpoint` was within edit-distance 2,
so it was suggested. Fix: change `endpiont` → `endpoint`.

When no close match exists, the hint lists what *is* available:

```
error: no library function matches token "zzqq"
  hint: library functions start with one of: endpoint, param, returns, api
```

### 2. Value violates a type's options enum

```
log_level verbose
```

```
error: function "log_level" arg "lvl": value "verbose" is not in options for type "LogLevel"
  hint: valid options: trace, debug, info, warn, error, fatal
```

The library declared:

```
type LogLevel
    options "trace" "debug" "info" "warn" "error" "fatal"
end
```

`verbose` isn't on the list. The hint shows every valid value so
the author can pick one. When the value is close to a valid option,
`did you mean ...?` appears too:

```
error: function "env" arg "stage": value "prudo" is not in options for type "Env"
  hint: did you mean "prod"? valid options: dev, staging, prod
```

### 3. Value violates a type's pattern regex

```
owner "not-an-email"
```

```
error: function "owner" arg "who": value "not-an-email" does not match pattern for type "Email"
  hint: type "Email" requires the value to match regex /^[^@]+@[^@]+\.[^@]+$/
```

The hint includes the regex so authors can see what's wrong without
opening the library.

### 4. Unknown type (typo in library)

```
arg capture port Posrt
```

```
error: function "service" arg "port" capture has unknown type "Posrt"
  hint: did you mean "Port"?
```

Same Levenshtein-suggestion mechanism applied to the type table.

## Source file location

Errors include the location in `file:line:col` format — same as
gcc, go, rustc, etc. Most editors recognize this and turn it into a
clickable link:

```
script.capy:3:5: function "service" arg "name": value "Bad Name!" does not match pattern...
```

(The bare `.Error()` form. The CLI's prettier `FormatWithSource`
rendering with caret pointing is shown above.)

## The `capy check` command

For library authoring, use `capy check` to validate without running
any source. It catches:

- Unknown types referenced by captures (with a "did you mean" hint).
- Block functions whose closer is missing.
- Conflicts where multiple functions try to match the same syntax.

```sh
$ capy check lib.capy
ok — 8 function(s), 7 type(s)
  function service
  function endpoint
  ...
  type     ServiceName
  ...
```

If something is wrong, the error is shown immediately:

```sh
$ capy check broken.capy
function "service" arg "name" capture has unknown type "ServiceNme"
  hint: did you mean "ServiceName"?
```

## When the error has no source

Some errors happen before the parser runs (library load failures,
file-not-found, malformed `.capy`). These don't have a line number
for your *script* — they're problems with the library or filesystem.
They're still wrapped in the same `error: ... / hint: ...` format
where helpful.

## Debugging write blocks

When a `write` block or `file_template` produces unexpected output,
the typical mistakes are:

1. **Capturing a string but rendering without `unquote`**. Strings
   captured with type `string` keep their quotes. Use
   `${name | unquote}` to strip them for human-facing output.
2. **Indent drift in multi-line backtick literals**. Backtick
   literals preserve newlines and indentation verbatim — every
   leading space appears in the output. Use the `indent N` helper
   on a sub-expression rather than visually indenting in source.
3. **Forgetting a trailing newline**. `write \`foo\`` emits `foo`
   with no terminator. End your literal with a literal newline if
   you want one.

Run with `--out-dir` and look at the generated files for multi-file
projects, or pipe to `less` for single-output.

## Embedding-API errors

When using Capy from Rust (`Library::new(...)`), the same
`*domain.CapyError` is returned. You can type-assert it to render
your own UI:

```rust
use capy_core::domain::errors::format_with_source;

match lib.run(source) {
    Ok(out) => { /* … */ }
    Err(e) => {
        // Every error is a CapyError, with the position and hint already on it.
        println!("at line {}: {}", e.line, e.msg);
        if !e.hint.is_empty() {
            println!("  → {}", e.hint);
        }
        // Or let the engine render it the way the CLI does — `error:` prefix,
        // hint line, and a caret under the offending column:
        eprintln!("{}", format_with_source(&e, source));
    }
}
```

`domain.FormatWithSource(err, sourceString)` is the same rendering
the CLI uses — drop it in any tool that wants the same look.

## Reporting a confusing error

If you hit an error that *should* have a hint but doesn't, or whose
location is wrong: file an issue with the minimal `lib.capy` +
`script.capy` that reproduces it. The hint engine has grown
incrementally — every new "this would have been better as a hint"
becomes a one-line addition.


## `left recursion — it can match itself without consuming a token`

A library rule whose first element is a capture of a function that leads back to
it. The matcher would descend forever, so the library is refused when it loads
rather than at run time.

```text
function "expr": left recursion — it can match itself without consuming a token
(cycle: expr -> expr)
```

The cycle in parentheses is the path the matcher would take. Fix it by consuming
something first — see
[Recursive captures](library-authoring.md#recursive-captures-right-recursive-not-left-recursive).

Before this check existed, such a library passed `capy check` and then aborted
the process with a stack overflow, which an embedding program could not catch.

## `no library function matches` on deeply nested input

A nonterminal descent stops at 64 levels. Source nested deeper than that is
refused; the message currently falls back to the generic "no library function
matches" rather than naming the limit. If you hit this with hand-written source,
the grammar is usually the problem rather than the depth.
