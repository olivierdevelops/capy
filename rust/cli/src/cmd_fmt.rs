//! Port of `cmd/capy/cmd_fmt.go`.
//!
//! A minimal `.capy` formatter. Scope: enforce a few low-cost normalisations
//! that catch the most common style drift; not yet a full canonical-form
//! normaliser.
//!
//! What it does:
//!   * Strips trailing whitespace from every line.
//!   * Replaces hard-tab indentation with 4 spaces.
//!   * Collapses runs of more than one blank line to exactly one.
//!   * Ensures the file ends with exactly one trailing newline.
//!
//! What it does NOT do: re-order declarations, re-align arg lines, or re-flow
//! templates inside `write` backticks (significant whitespace).

use super::flags::{self, Spec};
use capy_core::gopath;

const FMT_SPEC: Spec = Spec { bools: &["check", "diff", "stdout"], strings: &[] };

/// Port of `cmdFmt`. Returns the process exit code so `--check` can exit 1.
pub fn cmd_fmt(args: &[String]) -> Result<i32, String> {
    let fs = flags::parse(&FMT_SPEC, args)?;
    let files = &fs.positionals;
    if files.is_empty() {
        return Err("usage: capy fmt [--check | --diff | --stdout] <file.capy>...".to_string());
    }
    let mut any_changed = false;
    for path in files {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {}", path, gopath::io_error("open", path, &e)))?;
        let formatted = format_capy(&raw);
        if formatted == raw {
            continue;
        }
        any_changed = true;
        if fs.bool("check") {
            eprintln!("would reformat {}", path);
        } else if fs.bool("diff") {
            println!("{}", simple_diff(&raw, &formatted));
        } else if fs.bool("stdout") {
            print!("{}", formatted);
        } else {
            std::fs::write(path, formatted.as_bytes())
                .map_err(|e| format!("write {}: {}", path, gopath::io_error("open", path, &e)))?;
            eprintln!("formatted {}", path);
        }
    }
    if fs.bool("check") && any_changed {
        return Ok(1);
    }
    Ok(0)
}

/// Port of `formatCapy`.
///
/// Avoids touching the inside of backtick literals, where whitespace is part of
/// the emitted output.
// The byte loops below advance by 2 over `\X` escape pairs, so an iterator-based
// loop cannot express them; indexing is required, not incidental.
#[allow(clippy::needless_range_loop)]
pub fn format_capy(src: &str) -> String {
    // Split into lines, tracking backtick state so a newline inside a literal
    // still ends a line for the purposes of the per-line pass below.
    let b = src.as_bytes();
    let mut lines: Vec<String> = Vec::new();
    let mut line: Vec<u8> = Vec::new();
    let mut in_backtick = false;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if in_backtick {
            line.push(c);
            if c == b'\\' && i + 1 < b.len() {
                line.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == b'`' {
                in_backtick = false;
            }
            if c == b'\n' {
                // Drop the newline from the stored line: the emit loop re-adds
                // exactly one. Keeping it here appended a second newline on every
                // pass, so `capy fmt` grew the file each run and never converged.
                if line.last() == Some(&b'\n') {
                    line.pop();
                }
                lines.push(String::from_utf8_lossy(&line).into_owned());
                line.clear();
            }
            i += 1;
            continue;
        }
        if c == b'`' {
            in_backtick = true;
            line.push(c);
            i += 1;
            continue;
        }
        if c == b'\n' {
            lines.push(String::from_utf8_lossy(&line).into_owned());
            line.clear();
            i += 1;
            continue;
        }
        line.push(c);
        i += 1;
    }
    if !line.is_empty() {
        lines.push(String::from_utf8_lossy(&line).into_owned());
    }

    // Apply per-line rules — but only where the text is NOT inside a multi-line
    // backtick, since whitespace there is part of the emitted output. Two states
    // matter per line:
    //
    //   start_in_backtick — the line's leading text is literal content
    //   end_in_backtick   — the line's TRAILING text is literal content
    //                       (true for the line that OPENS a backtick)
    //
    // Leading-indent normalisation is safe whenever the line starts outside a
    // literal; trailing-space stripping is only safe when the line both starts
    // and ends outside one. Getting that second condition wrong silently eats
    // significant whitespace on the `write \`…` opening line.
    let mut out = String::new();
    let mut in_backtick = false;
    let mut prev_blank = false;
    let n = lines.len();
    for (i, raw_line) in lines.iter().enumerate() {
        let mut ln = raw_line.clone();
        let start_in_backtick = in_backtick;
        // Advance the backtick state over this line's unescaped backticks.
        let lb = ln.as_bytes();
        let mut j = 0usize;
        while j < lb.len() {
            if lb[j] == b'\\' && j + 1 < lb.len() {
                j += 2;
                continue;
            }
            if lb[j] == b'`' {
                in_backtick = !in_backtick;
            }
            j += 1;
        }
        let end_in_backtick = in_backtick;
        if !start_in_backtick {
            ln = tabs_to_spaces(&ln, 4);
            if !end_in_backtick {
                ln = strip_trailing_spaces(&ln);
            }
        }
        // Skip extra consecutive blank lines (only outside backticks).
        if !start_in_backtick && !end_in_backtick && ln.trim().is_empty() {
            if prev_blank {
                continue;
            }
            prev_blank = true;
        } else {
            prev_blank = false;
        }
        out.push_str(&ln);
        if i < n - 1 || !ln.is_empty() {
            out.push('\n');
        }
    }
    // Ensure exactly one trailing newline.
    format!("{}\n", out.trim_end_matches('\n'))
}

/// Port of `stripTrailingSpaces`.
fn strip_trailing_spaces(line: &str) -> String {
    line.trim_end_matches([' ', '\t']).to_string()
}

/// Port of `tabsToSpaces` — converts LEADING tabs only, so tabs inside strings
/// are untouched.
fn tabs_to_spaces(line: &str, width: usize) -> String {
    if !line.contains('\t') {
        return line.to_string();
    }
    let b = line.as_bytes();
    let mut i = 0usize;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    let prefix = &line[..i];
    let rest = &line[i..];
    format!("{}{}", prefix.replace('\t', &" ".repeat(width)), rest)
}

/// Port of `simpleDiff` — a unified-ish line diff with `-` / `+` markers. Not a
/// real diff algorithm; just shows pre/post for short edits.
fn simple_diff(a: &str, b: &str) -> String {
    let la: Vec<&str> = a.split('\n').collect();
    let lb: Vec<&str> = b.split('\n').collect();
    let mut out = String::new();
    let n = la.len().max(lb.len());
    for i in 0..n {
        let ax = la.get(i).copied().unwrap_or("");
        let bx = lb.get(i).copied().unwrap_or("");
        if ax == bx {
            out.push_str(&format!("  {}\n", ax));
            continue;
        }
        if !ax.is_empty() {
            out.push_str(&format!("- {}\n", ax));
        }
        if !bx.is_empty() {
            out.push_str(&format!("+ {}\n", bx));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_whitespace_and_tabs() {
        assert_eq!(format_capy("a   \n\tb\n"), "a\n    b\n");
    }

    #[test]
    fn collapses_blank_runs_and_fixes_trailing_newline() {
        assert_eq!(format_capy("a\n\n\n\nb\n\n\n"), "a\n\nb\n");
        assert_eq!(format_capy("a"), "a\n");
    }

    #[test]
    fn preserves_significant_whitespace_inside_backticks() {
        // Trailing spaces and tabs inside a backtick literal are significant and
        // must survive — they are part of the emitted output.
        let out = format_capy("write `line   \n\tkept\n`\n");
        assert!(out.contains("line   "), "trailing spaces inside a backtick kept");
        assert!(out.contains("\tkept"), "leading tab inside a backtick kept");
    }

    /// The formatter must CONVERGE: a second pass is a no-op. It previously
    /// inserted a blank line after every line inside a multi-line backtick
    /// (the splitter kept the newline on the line and the emit loop added
    /// another), so repeated runs grew the file without limit.
    #[test]
    fn is_idempotent() {
        let src = "write `line   \n\tkept\n`\n";
        let once = format_capy(src);
        let twice = format_capy(&once);
        assert_eq!(twice, once, "a second pass must change nothing");
        // And significant whitespace inside the literal survives.
        assert!(once.contains("line   "), "trailing spaces inside a backtick kept");
        assert!(once.contains("\tkept"), "leading tab inside a backtick kept");
    }
}
