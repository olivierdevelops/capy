# `.capy` libraries

A Capy library is a `.capy` file that declares the grammar and
output for one source-language → target-output transpilation.
Same syntax all the way through: the same indentation rules,
string literals, and comment conventions your user-facing scripts
have. One mental model, one parser.

```sh
capy run lib.capy script.capy
```

## A complete example

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

function return
    arg literal "return"
    arg capture l any
    arg literal "+"
    arg capture r any
    write `return ${l} + ${r};
`
end

function end
end

file_template
    write `#include <stdio.h>

${body}`
end
```

Run this against the right script and it emits a C source file.

## The full surface

```
extension <STR>                           # output file extension
output_file <STR>                         # optional: write here instead of stdout

comments                                  # opt-in user-script comments
    line "#"                              # zero defaults — declare what you want
end

context                                   # initial schema for the accumulating context
    imports []
    title ""
end

type <NAME>                               # optional: library-defined types
    pattern "^…regex…$"                   # OR options "a" "b" "c"
    base int                              # optional: validation inherits from base
end

function <NAME>
    description "…"                       # surfaced by `capy docs`
    priority <INT>                        # optional: higher wins ambiguous matches
    arg literal <STR>                     # match this token literally
    arg capture <NAME> <TYPE>             # capture a token: any | ident | int | string | ...
    block_closer <NAME>                   # block opener: body runs until <NAME> appears
    block_open <STR> close <STR>          # OR: explicit delimiters
    block_sections <S>... closer <NAME>   # OR: multi-section block (try/rescue/finally)
    when_followed_by indent               # only match when an indented body follows
    when_not_followed_by indent           # only match when one does NOT (context-sensitive)

    # Function body — inner-DSL statements interleaved freely:
    write `text ${capture} ${helper x}`   # emit + ${EXPR} interpolation
    set context.field value               # mutate state
    append context.list value             # …or push to a list
    if cond                               # conditional / control flow
        write `…`
    else if other
        write `…`
    end
    for x in context.items
        write `${x}
`
    end
end

file_template                             # whole-file assembler
    write `…${body}…`
end

file "path/to/output.ext"                 # multi-file output
    write `…body…`
end
```

- **Strings** use double quotes with C-style escapes (`\n`, `\t`,
  `\"`, `\\`) — or backticks (multi-line, with `${EXPR}` interpolation).
- **Bare words** are accepted for `extension`, type names, and
  capture names.
- **Indentation** delimits block bodies — the block ends at `end`
  (for `function` / `file_template` / `file "X"`).
- **Comments** start with `#` and run to end of line **inside the
  manifest**. For user-script comments, declare them in the
  `comments` block (zero defaults — see [grammar-as-contract](grammar-as-contract.md)).

## Templates and the inner DSL share one grammar

The body of a `function`, a `file_template`, or a `file "X"` is
all the same inner-DSL: statements (`set`, `append`, `if`, `for`,
…) and `write` calls with `${EXPR}` interpolations. The renderer
walks the parsed AST directly — there's no second template
language, no Go-template `{{ … }}` syntax, no separate template
runtime.

Inside `${…}` you can:

- Look up paths: `${name}`, `${context.title}`, `${item.x}`.
- Call helpers: `${unquote text}`, `${toQuoted name}`,
  `${pascalCase context.title}`, `${indent 4 body}`,
  `${add i 1}`, `${stars rating}`.
- Chain helpers via pipe: `${context.name | pascalCase | unquote}`.
- Nest with parens: `${toQuoted (upper x)}`.
- Reference loop variables: `for it in context.items` makes `it`
  available as `${it.field}` inside the loop body.

The same helpers also work in expression positions of `set` /
`let` / `append`: `set context.total (add context.total pages)`.

## Why one format, one grammar

A previous version of Capy let you write libraries in YAML with
Go `text/template` for output bodies. That meant the author was
juggling three languages at every edit: the source DSL, the
manifest format, and the template language. The renderer is now
the same AST walker the inner DSL uses, so there's exactly one
grammar to learn and one parser to debug.

See [docs/design/migration-write-style.md](design/migration-write-style.md)
for the history of the migration.
