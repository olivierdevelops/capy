//! Port of `infra/define_extractor.go`.

use super::capy_lib_parser::parse_capy_lib;
use crate::gofmt;
use regex::Regex;
use std::sync::OnceLock;

/// Port of `defineNameRE`.
fn define_name_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z_][A-Za-z_0-9]*$").unwrap())
}

/// Port of `ExtractDefines`.
///
/// Scans a Capy source file for `define NAME … end` blocks at the top level
/// (column-0 `define`, matched by column-0 `end`). Each block has the same body
/// shape as a function declaration in a `.capy` library file.
///
/// This is Capy's META-PROGRAMMING entry point: the source can extend the
/// library's grammar with new patterns that subsequent statements can then use,
/// without touching the library file at all.
///
/// Returns `(cleaned_source, lib_src)`:
///
/// * `cleaned_source` — the original source with all `define … end` blocks
///   REMOVED (so the parser only sees calls).
/// * `lib_src` — a synthetic `.capy` library text containing each block
///   rewritten as a `function NAME … end`. The caller loads this through the
///   normal library loader and merges the resulting functions into the working
///   library (source-defined functions OVERRIDE library functions of the same
///   name). Empty string when the source has no defines.
pub fn extract_defines(source: &str) -> Result<(String, String), String> {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut kept: Vec<&str> = Vec::new();
    // Collected define blocks, rewritten as `function … end`.
    let mut lib_blocks: Vec<String> = Vec::new();

    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        let stripped = line.trim_start_matches([' ', '\t']);
        // Only top-level `define NAME` (no leading indent) triggers a block.
        // This avoids accidentally swallowing the word "define" inside a
        // multi-line template body or a `run:` snippet.
        let indented = line != stripped;
        if indented || !stripped.starts_with("define ") {
            kept.push(line);
            i += 1;
            continue;
        }

        // Find the matching `end` at column 0.
        let start_line = i;
        let name_rest = stripped.strip_prefix("define ").unwrap_or(stripped);
        let mut name = name_rest.trim().to_string();
        // Allow a comment after the name, e.g. `define foo  # bar`.
        if let Some(hash) = name.find('#') {
            name = name[..hash].trim().to_string();
        }
        // The name must be a plain identifier — the lexer can't call a function
        // whose name contains punctuation or whitespace, so a `define "bad-name"`
        // would compile to dead code.
        if !define_name_re().is_match(&name) {
            return Err(format!(
                "line {}: `define` name must be a plain identifier (got {})",
                i + 1,
                gofmt::quote(&name)
            ));
        }

        let mut block_lines: Vec<String> = vec![format!("function {}", name)];
        let mut closed = false;
        // `enumerate` over the tail, then recover the absolute index — which the
        // body needs in order to advance the outer cursor past the block.
        for (off, l) in lines[i + 1..].iter().enumerate() {
            let j = i + 1 + off;
            let l = *l;
            let trim = l.trim();
            // `end` at column 0 closes the block.
            if trim == "end" && !l.starts_with(' ') && !l.starts_with('\t') {
                block_lines.push("end".to_string());
                closed = true;
                i = j + 1;
                break;
            }
            block_lines.push(l.to_string());
        }
        if !closed {
            return Err(format!(
                "line {}: `define {}` is missing matching `end`",
                start_line + 1,
                name
            ));
        }
        lib_blocks.push(block_lines.join("\n"));
    }

    if lib_blocks.is_empty() {
        // No defines: return the original source unchanged.
        return Ok((source.to_string(), String::new()));
    }

    // Quick sanity-parse so a malformed define fails here with a clear message
    // instead of in the lexer.
    let synthetic = format!("extension _\n\n{}\n", lib_blocks.join("\n\n"));
    if let Err(e) = parse_capy_lib(&synthetic) {
        return Err(format!("define block: {}", e));
    }
    Ok((kept.join("\n"), synthetic))
}
