> **HISTORICAL.** Describes an earlier design, written when the engine was
> implemented in Go. The engine is now Rust under `rust/`. Kept for reference.

# Capy — Value-Driven Markup Language

Capy is an embeddable scripting language designed for configuration-driven code generation and DSL authoring. It is intentionally simple at its core, and extensible through YAML library configuration.

---

## Philosophy

**One way to do one thing.**

Capy has two modes:

| Mode           | What it is |
|----------------|-----------|
| Core Capy      | Default syntax + built-in functions. Simple, minimal, predictable. |
| Extended Capy  | A YAML library defines custom functions, modules, operators, keywords, and type validation. The script language can look however the library author wants. |

The core language stays deliberately minimal. All domain-specific power comes from the library configuration — not from cramming features into the base syntax.

---

# Core Capy: The Default Language

Core Capy ships with no special operators, no separators, and a handful of built-in functions. You can extend it from a YAML library file.

## Syntax rules

- One statement per line  
- Indentation = block (4 spaces or 1 tab)  
- `#` starts a comment  
- Strings: `"..."` or `'...'`  
- Backtick `` ` `` strings are raw templates (Go text/template syntax)  
- Numbers and booleans are unquoted: `42`, `3.14`, `true`, `false`  
- Functions are called by name:  
```

greet "Alice"

```
- Sub-calls can be wrapped in parentheses:
```

log (str "Hello, " name)

````

---

## Variable interpolation

Only `${}` is allowed:

```capy
name = "Alice"
log `Welcome, ${name}!`
````

❌ Not allowed:

```capy
log `Hello, $name!`
```

---

## Variable assignment

```capy
name    = "Alice"
count   = 42
active  = true
ratio   = 3.14
```

---

## Accessing nested fields

```capy
user = get_user 1
welcome `Hello ${user.name} from ${user.address.city}`
```

---

## JSON objects: bare variable values

Inside JSON literals, identifiers resolve automatically:

```capy
name = "Alice"
age  = 30

send {"name": name, "age": age}
send {"name": name, "active": true, "role": null}
```

Keys must still be quoted.

---

## Control flow

### If

```capy
if count > 0
  print "non-empty"
end
```

### Loop

```capy
loop item in items
  process item
end
```

### Break / Continue

```capy
loop item in items
  if item == "skip"
    continue
  end
  process item
end
```

---

# Extended Capy: Library Configuration

Libraries are YAML files defining what Capy scripts can do.

---

## Minimal library

```yaml
# greet.yaml
extension: txt
output_file: output.txt

functions:
  greet:
    args:
      - key: name
        type: string
        default: "World"
    template: |
      Hello, {{ .name }}!
```

```capy
# script.capy
greet "Alice"
greet "Bob"
```

Output:

```
Hello, Alice!
Hello, Bob!
```

---

## Defining syntax

```yaml
syntax:
  extends: capy
  separators: [","]
  operators:
    "->":
      function: pipe
      precedence: 5
  keywords:
    describe: "describe_fn"
    route: "begin_route"
```

### Built-in presets

| Preset  | Style                     |    |        |
| ------- | ------------------------- | -- | ------ |
| default | minimal                   |    |        |
| capy    | arithmetic + control flow |    |        |
| python  | `and`, `or`, `not`        |    |        |
| js      | `&&`, `                   |    | `, `!` |
| c       | `++`, `--`                |    |        |
| shell   | pipes and redirects       |    |        |
| pipe    | `->`, `                   | >` |        |

---

## Module scoping

```yaml
on_start:
  run: |
    expose_module "db"

functions:
  begin_route:
    run: |
      expose_module "request"
      expose_module "response"

  end_route:
    run: |
      hide_module "request"
      hide_module "response"
```

---

## Private functions

```yaml
functions:
  set_var:
    private: true
    args:
      - key: name
        type: raw
      - key: value
    run: save_context name value
    template: ""
```

Usage:

```capy
x = 20        # OK
# set_var x 20  # ERROR
```

---

## Configurable call syntax (`call_style`)

```yaml
syntax:
  call_style:
    delimiter: "()"
    separator: ","
    named_sep: ""
    required: false
```

Examples:

| Style   | Example            |
| ------- | ------------------ |
| default | `func arg1 arg2`   |
| comma   | `func arg1, arg2`  |
| parens  | `func(arg1, arg2)` |
| named   | `func{key: value}` |

---

## Statement terminator

```yaml
syntax:
  statement_end: ";"
```

```capy
x = 1; y = 2; greet x
```

---

# Argument Types and Validation

## Built-in types

| Type   | Description |
| ------ | ----------- |
| any    | no check    |
| string | string      |
| raw    | unquoted    |
| int    | integer     |
| float  | float       |
| bool   | boolean     |

---

## Example

```yaml
functions:
  set_limit:
    args:
      - key: limit
        type: int
        default: 100
```

```capy
set_limit 50        # OK
set_limit "fifty"   # ERROR
```

---

## Raw type

```yaml
functions:
  use_theme:
    args:
      - key: name
        type: raw
```

```capy
use_theme dark
use_theme "light"
```

---

# Custom Types

## Regex validation

```yaml
types:
  Email:
    validate: |
      if not (regex_match value "^[^@]+@[^@]+\\.[^@]+$")
        error "Expected a valid email address"
      end
```

---

## Enum validation

```yaml
types:
  Status:
    options: ["todo", "in-progress", "done", "blocked"]
```

---

## Combined validation

```yaml
types:
  Priority:
    options: ["low", "medium", "high", "critical"]
    validate: |
      if value == "critical"
        if not (regex_match deadline "^[0-9]{4}-")
          error "critical requires deadline"
        end
      end
```

---

# Sample Programs

## 1. Hello World

```capy
greet "World"
```

---

## 2. Variables and branches

```capy
name   = "Bob"
age    = 25
active = true

log `User: ${name}, age: ${age}`

if active
  allow name
end
```

---

## 3. Loop with condition

```capy
colors = ["red", "green", "blue"]

loop color in colors
  if color == "green"
    continue
  end
  use color
end
```

---

## 4. JSON with variables

```capy
host = "localhost"
port = 5432

connect {"host": host, "port": port, "ssl": true}
```

---

## 5. Simple HTTP route

```capy
begin_route "GET", "/api/hello"
  name = request.path_var "name"
  response.json '{"message": "Hello, ${name}!"}'
end_route
```

---

## 6. Form POST

```capy
begin_route "POST", "/api/messages"
  request.read_form
  body   = request.form_field "body"
  author = request.form_field "author"
  db.query 'INSERT INTO messages (body, author) VALUES (${body}, ${author})'
  response.json '{"id": {{.LastInsertID}}, "success": true}'
end_route
```

---

## 7. Bulk form

```capy
fields = request.form_fields {"title": "text", "body": "text", "draft": "text"}
```

---

## 8. Reusable types

```capy
define_type "Article"
  define_field "title",  "string"
  define_field "body",   "string"
end_type
```

---

## 9. Multi-step pipeline

```capy
post     = db.query "post", 'SELECT * FROM posts WHERE id=${id}'
comments = db.query "comments", 'SELECT * FROM comments WHERE post_id=${id}'
```

---

## 10. Custom validation

```capy
create_author "alice@example.com" "alice-in-tech"
```

---

# Responsibility Split

| Concern            | Library (YAML) | Script (Capy) |
| ------------------ | -------------- | ------------- |
| Functions          | ✅              |               |
| Types & validation | ✅              |               |
| Syntax             | ✅              |               |
| Modules            | ✅              |               |
| Data flow          |                | ✅             |
| Routes             |                | ✅             |
| Queries            |                | ✅             |

```