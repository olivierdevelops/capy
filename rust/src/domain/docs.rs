//! Port of `domain/docs.go`.

use super::library::{FuncDef, Library};

/// Port of `RenderLibraryDocs`.
///
/// Produces Markdown reference documentation for a loaded library:
///
///   * Library description (top-level)
///   * Output metadata (extension, output_file)
///   * One section per declared TYPE with its constraints
///   * One section per declared FUNCTION with arg table + example usage
///   * Notes about block functions and file outputs
pub fn render_library_docs(lib: &Library) -> String {
    let mut b = String::new();

    // ─── Header ───────────────────────────────────────────────────
    let title = if !lib.extension.is_empty() {
        format!("Library reference (→ `.{}`)", lib.extension)
    } else {
        "Library reference".to_string()
    };
    b.push_str(&format!("# {}\n\n", title));

    if !lib.description.is_empty() {
        b.push_str(&format!("{}\n\n", lib.description));
    } else {
        b.push_str(
            "*This library has no top-level `description`. \
Add one to summarize what it generates and who should use it.*\n\n",
        );
    }

    // Metadata strip
    b.push_str("| | |\n|---|---|\n");
    if !lib.extension.is_empty() {
        b.push_str(&format!("| **Output extension** | `.{}` |\n", lib.extension));
    }
    if !lib.output_file.is_empty() {
        b.push_str(&format!("| **Default output file** | `{}` |\n", lib.output_file));
    }
    b.push_str(&format!("| **Functions** | {} |\n", lib.functions.len()));
    b.push_str(&format!("| **Types** | {} |\n", lib.types.len()));
    if !lib.files_ast.is_empty() {
        // `BTreeMap` keys already iterate sorted, matching Go's sortedKeys.
        let keys: Vec<&str> = lib.files_ast.keys().map(|s| s.as_str()).collect();
        b.push_str(&format!(
            "| **Multi-file outputs** | {} (`{}`) |\n",
            lib.files_ast.len(),
            keys.join("`, `")
        ));
    }
    b.push('\n');

    // ─── Types ────────────────────────────────────────────────────
    if !lib.types.is_empty() {
        b.push_str("## Types\n\n");
        for (name, t) in &lib.types {
            b.push_str(&format!("### `{}`\n\n", name));
            if !t.description.is_empty() {
                b.push_str(&format!("{}\n\n", t.description));
            }
            let mut rules: Vec<String> = Vec::new();
            if !t.base.is_empty() && t.base != "any" {
                rules.push(format!("inherits from `{}`", t.base));
            }
            if !t.pattern.is_empty() {
                rules.push(format!("must match regex `{}`", t.pattern));
            }
            if !t.options.is_empty() {
                let opts: Vec<String> =
                    t.options.iter().map(|o| format!("`{}`", o)).collect();
                rules.push(format!("must be one of: {}", opts.join(", ")));
            }
            if rules.is_empty() {
                rules.push("no constraints (accepts any value)".to_string());
            }
            for r in &rules {
                b.push_str(&format!("- {}\n", r));
            }
            b.push('\n');
        }
    }

    // ─── Functions ────────────────────────────────────────────────
    if !lib.functions.is_empty() {
        b.push_str("## Functions\n\n");
        // Sort functions: priority-tagged ones first, then alphabetical.
        let mut names: Vec<&String> = lib.functions.keys().collect();
        names.sort_by(|a, b2| {
            let pi = lib.functions[*a].priority;
            let pj = lib.functions[*b2].priority;
            if pi != pj {
                return pj.cmp(&pi);
            }
            a.cmp(b2)
        });
        for name in names {
            render_func(&mut b, &lib.functions[name]);
        }
    }

    b
}

/// Port of `renderFunc`.
fn render_func(b: &mut String, fnv: &FuncDef) {
    // Signature reconstruction: synthesize a source-side call shape from the
    // args. Literals appear verbatim; captures appear as `<name>`.
    let parts: Vec<String> = fnv
        .args
        .iter()
        .map(|a| {
            if a.kind == "literal" {
                a.value.clone()
            } else {
                format!("<{}>", a.name)
            }
        })
        .collect();

    b.push_str(&format!("### `{}`\n\n", fnv.name));
    if !fnv.description.is_empty() {
        b.push_str(&format!("{}\n\n", fnv.description));
    }

    b.push_str(&format!("```\n{}\n```\n\n", parts.join(" ")));

    // Arg table only if there are captures.
    let has_captures = fnv.args.iter().any(|a| a.kind == "capture");
    if has_captures {
        b.push_str("| Argument | Type | Description |\n|---|---|---|\n");
        for a in &fnv.args {
            if a.kind != "capture" {
                continue;
            }
            let desc = if a.description.is_empty() {
                "*(no description)*"
            } else {
                a.description.as_str()
            };
            b.push_str(&format!("| `{}` | `{}` | {} |\n", a.name, a.type_, desc));
        }
        b.push('\n');
    }

    if let Some(block) = &fnv.block {
        if !block.sections.is_empty() {
            b.push_str(&format!(
                "**Opens a multi-section block** — sections `{}`, closed by `{}`.\n\n",
                block.sections.join("`, `"),
                block.closer
            ));
        } else if !block.closer.is_empty() {
            b.push_str(&format!(
                "**Opens an indented block** — body runs until `{}`.\n\n",
                block.closer
            ));
        } else if !block.open.is_empty() {
            b.push_str(&format!(
                "**Opens a delimited block** — `{}` … `{}`.\n\n",
                block.open, block.close
            ));
        }
    }
    if let Some(la) = &fnv.lookahead {
        if la.require_indent {
            b.push_str("*Matches only when followed by an indented block.*\n\n");
        } else if la.forbid_indent {
            b.push_str("*Matches only when not followed by an indented block.*\n\n");
        }
    }
    if fnv.priority > 0 {
        b.push_str(&format!(
            "*Priority: {} — wins over lower-priority functions when patterns overlap.*\n\n",
            fnv.priority
        ));
    }
}
