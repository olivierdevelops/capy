//! Port of `cmd/playground-bundle/main.go`.
//!
//! Reads a curated list of samples from `samples/` and writes a single JSON file
//! the browser playground can fetch.
//!
//! Usage: `playground-bundle > docs/assets/playground/samples.json`

mod curated;

use capy_core::gojson::marshal_string as jstr;
use curated::CURATED;
use std::io::Write;
use std::path::PathBuf;

/// Port of `sample`. Field order matters: Go's encoder emits struct fields in
/// declaration order, and this is written out by hand to preserve it.
struct Sample {
    id: String,
    category: String,
    title: String,
    description: String,
    hint: String,
    library: String,
    script: String,
}

/// Serialises the bundle exactly as Go's
/// `json.NewEncoder(os.Stdout)` + `SetIndent("", "  ")` does — including the
/// default HTML escaping of `<`, `>` and `&`, and the trailing newline `Encode`
/// appends.
fn encode_bundle(samples: &[Sample], categories: &[String]) -> String {
    let mut b = String::new();
    b.push_str("{\n  \"samples\": ");
    if samples.is_empty() {
        // Go emits `null` for a nil slice, `[]` for an empty non-nil one; the
        // bundler only ever appends, so a nil slice means "nothing bundled".
        b.push_str("null");
    } else {
        b.push_str("[\n");
        for (i, s) in samples.iter().enumerate() {
            b.push_str("    {\n");
            let fields: [(&str, &String); 7] = [
                ("id", &s.id),
                ("category", &s.category),
                ("title", &s.title),
                ("description", &s.description),
                ("hint", &s.hint),
                ("library", &s.library),
                ("script", &s.script),
            ];
            for (j, (k, v)) in fields.iter().enumerate() {
                b.push_str("      ");
                b.push_str(&jstr(k));
                b.push_str(": ");
                b.push_str(&jstr(v));
                if j + 1 < fields.len() {
                    b.push(',');
                }
                b.push('\n');
            }
            b.push_str("    }");
            if i + 1 < samples.len() {
                b.push(',');
            }
            b.push('\n');
        }
        b.push_str("  ]");
    }
    b.push_str(",\n  \"categories\": ");
    if categories.is_empty() {
        b.push_str("null");
    } else {
        b.push_str("[\n");
        for (i, c) in categories.iter().enumerate() {
            b.push_str("    ");
            b.push_str(&jstr(c));
            if i + 1 < categories.len() {
                b.push(',');
            }
            b.push('\n');
        }
        b.push_str("  ]");
    }
    b.push_str("\n}\n");
    b
}

fn main() {
    let root = match std::env::current_dir() {
        Err(e) => fail(&e.to_string()),
        Ok(r) => r,
    };
    let samples_dir: PathBuf = root.join("samples");

    let mut samples: Vec<Sample> = Vec::new();
    let mut categories: Vec<String> = Vec::new();
    let mut seen_cat: std::collections::HashSet<&str> = std::collections::HashSet::new();

    for c in CURATED {
        let lib_name = if c.library_file.is_empty() { "lib.capy" } else { c.library_file };
        let lib_path = samples_dir.join(c.id).join(lib_name);
        let lib_bytes = match std::fs::read_to_string(&lib_path) {
            Ok(b) => Some(b),
            Err(e) => {
                // Fall back to lib.yaml only when the curated entry didn't
                // explicitly name a library file.
                if c.library_file.is_empty() {
                    match std::fs::read_to_string(samples_dir.join(c.id).join("lib.yaml")) {
                        Ok(b) => Some(b),
                        Err(e2) => {
                            eprintln!(
                                "playground-bundle: SKIP {} ({})",
                                c.id,
                                capy_io_err(&samples_dir.join(c.id).join("lib.yaml"), &e2)
                            );
                            None
                        }
                    }
                } else {
                    eprintln!(
                        "playground-bundle: SKIP {} ({})",
                        c.id,
                        capy_io_err(&lib_path, &e)
                    );
                    None
                }
            }
        };
        let lib_bytes = match lib_bytes {
            None => continue,
            Some(b) => b,
        };

        let script_name = if c.script_file.is_empty() { "script.capy" } else { c.script_file };
        let script_path = samples_dir.join(c.id).join(script_name);
        let script_bytes = match std::fs::read_to_string(&script_path) {
            Err(e) => {
                eprintln!(
                    "playground-bundle: SKIP {}/{} ({})",
                    c.id,
                    script_name,
                    capy_io_err(&script_path, &e)
                );
                continue;
            }
            Ok(b) => b,
        };

        samples.push(Sample {
            id: c.id.to_string(),
            category: c.category.to_string(),
            title: c.title.to_string(),
            description: c.description.to_string(),
            hint: c.hint.to_string(),
            library: lib_bytes,
            script: script_bytes,
        });
        if seen_cat.insert(c.category) {
            categories.push(c.category.to_string());
        }
    }

    eprintln!(
        "playground-bundle: bundled {} samples across {} categories",
        samples.len(),
        categories.len()
    );

    let out = encode_bundle(&samples, &categories);
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    if let Err(e) = w.write_all(out.as_bytes()) {
        fail(&e.to_string());
    }
}

/// Renders a read failure the way Go's `*os.PathError` does, so SKIP lines match.
fn capy_io_err(path: &std::path::Path, e: &std::io::Error) -> String {
    let msg = match e.kind() {
        std::io::ErrorKind::NotFound => "no such file or directory".to_string(),
        std::io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        _ => e.to_string(),
    };
    format!("open {}: {}", path.display(), msg)
}

fn fail(msg: &str) -> ! {
    eprintln!("playground-bundle: {}", msg);
    std::process::exit(1);
}
