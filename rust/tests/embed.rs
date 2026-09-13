//! Port of `capy_test.go` — end-to-end behaviour through the embedding API.

use capy_core::capy::Library;

fn must_lib(src: &str) -> Library {
    Library::new(src).unwrap_or_else(|e| panic!("NewLibrary: {e}"))
}

fn must_run(lib: &Library, script: &str) -> String {
    lib.run(script).unwrap_or_else(|e| panic!("Run: {e}"))
}

/// Port of `TestEmbed_InlineCapyLibrary`.
#[test]
fn inline_capy_library() {
    let lib = must_lib(
        r##"
extension html

function button
    arg literal "button"
    arg capture label string
    write `<button>${label}</button>
`
end

function link
    arg literal "link"
    arg capture text string
    arg capture href string
    write `<a href=${href}>${text}</a>
`
end
"##,
    );
    let out = must_run(&lib, "button \"Click me\"\nlink \"Home\" \"/index.html\"\n");
    assert_eq!(
        out,
        "<button>\"Click me\"</button>\n<a href=\"/index.html\">\"Home\"</a>\n"
    );
    assert_eq!(lib.extension(), "html");
}

/// Port of `TestEmbed_ReuseLibrary` — one Library, many scripts.
#[test]
fn reuse_library() {
    let lib = must_lib(
        r##"
extension txt

function say
    arg literal "say"
    arg capture msg any
    write `[${msg}]
`
end
"##,
    );
    for src in ["say one", "say two", "say three"] {
        let out = must_run(&lib, src);
        assert!(out.contains('['), "run {src:?} output: {out:?}");
    }
}

/// Port of `TestEmbed_ReportsErrors`.
#[test]
fn reports_errors() {
    Library::new("nonsense top-level directive")
        .expect_err("expected error for invalid library");
}

/// Port of `TestEmbed_Decoded` — `${decoded x}` must resolve `\"` and `\n` even
/// when the value contains unescaped quotes.
#[test]
fn decoded_resolves_embedded_escapes() {
    let lib = must_lib(
        r##"
extension txt

function p
    arg literal "p"
    arg capture text string
    write `${decoded text}
`
end
"##,
    );
    let cases: &[(&str, &str)] = &[
        (r##"p "He said \"hi\"""##, "He said \"hi\"\n"),
        (r##"p "line1\nline2""##, "line1\nline2\n"),
        (r##"p "tab\there""##, "tab\there\n"),
        (
            r##"p "<div class=\"card\">\n  body\n</div>""##,
            "<div class=\"card\">\n  body\n</div>\n",
        ),
    ];
    for (input, want) in cases {
        assert_eq!(&must_run(&lib, input), want, "decoded({input})");
    }
}

/// Port of `TestEmbed_EscapeHtml` — all five HTML-significant characters.
#[test]
fn escape_html_neutralises_all_five() {
    let lib = must_lib(
        r##"
extension html

function p
    arg literal "p"
    arg capture text string
    write `<p>${escapeHtml (decoded text)}</p>
`
end
"##,
    );
    let out = must_run(&lib, r##"p "a & b < c > d \" e ' f""##);
    assert_eq!(out, "<p>a &amp; b &lt; c &gt; d &quot; e &#39; f</p>\n");
}

/// Port of `TestEmbed_VerbatimRawBytes` — `block_verbatim` preserves blank lines
/// and comment-marker lines byte-for-byte, even though `#` is a declared marker
/// the lexer would normally strip.
#[test]
fn verbatim_preserves_raw_bytes() {
    let lib = must_lib(
        r##"
extension txt

comments
    line "#"
end

function pre
    arg capture lang ident
    block_verbatim end
    write `[${lang}]
${body}---
`
end

function end
end
"##,
    );
    let out = must_run(
        &lib,
        "pre md\n    intro\n    # Heading\n\n    ## Subheading\nend",
    );
    assert_eq!(out, "[md]\nintro\n# Heading\n\n## Subheading\n---\n");
}

/// Port of `TestEmbed_OptionalArgs` — trailing optional captures with defaults.
#[test]
fn optional_args_bind_defaults() {
    let lib = must_lib(
        r##"
extension html

function button
    arg literal "button"
    arg capture label string
    arg capture variant string default "primary"
    arg capture kind string default "button"
    write `<button type="${decoded kind}" class="btn-${decoded variant}">${decoded label}</button>
`
end
"##,
    );
    let cases: &[(&str, &str)] = &[
        (
            r##"button "Save""##,
            "<button type=\"button\" class=\"btn-primary\">Save</button>\n",
        ),
        (
            r##"button "Delete" "danger""##,
            "<button type=\"button\" class=\"btn-danger\">Delete</button>\n",
        ),
        (
            r##"button "Submit" "primary" "submit""##,
            "<button type=\"submit\" class=\"btn-primary\">Submit</button>\n",
        ),
    ];
    for (input, want) in cases {
        assert_eq!(&must_run(&lib, input), want, "optional-args({input})");
    }
}

/// Port of `TestEmbed_OptionalArgsMustBeTrailing` — load-time validation.
#[test]
fn optional_args_must_be_trailing() {
    let err = Library::new(
        r##"
extension txt

function bad
    arg literal "bad"
    arg capture a string default "x"
    arg capture b string
    write `${a}${b}
`
end
"##,
    )
    .expect_err("expected trailing-only validation error");
    assert!(
        err.to_string().contains("optional"),
        "error should mention the optional-trailing rule, got: {err}"
    );
}

/// Port of `TestEmbed_BareAndTail` — a `bare` function with a `tail` capture
/// matches a whole free-form line, including UTF-8 content.
#[test]
fn bare_and_tail_match_whole_line() {
    let lib = must_lib(
        r##"
extension html

function line
    bare
    arg capture content tail
    write `<p>${content}</p>
`
end
"##,
    );
    let out = must_run(
        &lib,
        "Each line — yes, em-dashes — becomes a <p>.\nCafé au lait 北京 🎉",
    );
    assert_eq!(
        out,
        "<p>Each line — yes, em-dashes — becomes a <p>.</p>\n<p>Café au lait 北京 🎉</p>\n"
    );
}

/// Port of `TestEmbed_MultilineBacktick` — backtick captures span newlines.
#[test]
fn multiline_backtick_capture() {
    let lib = must_lib(
        r##"
extension txt

function p
    arg literal "p"
    arg capture text string
    write `${decoded text}
`
end
"##,
    );
    let out = must_run(&lib, "p `one\ntwo\nthree`");
    assert_eq!(out.trim(), "one\ntwo\nthree");
}

/// Port of `TestEmbed_BacktickCodeSpan` — an escaped backtick inside a backtick
/// capture decodes back to a literal backtick.
#[test]
fn backtick_code_span() {
    let lib = must_lib(
        r##"
extension html

function md
    arg literal "md"
    arg capture src string
    write `<p>${decoded src}</p>
`
end
"##,
    );
    let out = must_run(&lib, "md `inline \\`code\\` here`");
    assert_eq!(out, "<p>inline `code` here</p>\n");
}

/// Introspection exposes arg shapes and sorts by function name.
#[test]
fn introspect_reports_shapes_sorted() {
    let lib = must_lib(
        r##"
extension html

comments
    line "#"
end

function zebra
    arg literal "zebra"
    arg capture n int "how many"
end

function alpha
    description "First one."
    arg literal "alpha"
    arg capture label string
    arg capture variant string default "primary"
    block_closer end
end

function end
end
"##,
    );
    let fns = lib.introspect();
    let names: Vec<&str> = fns.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "end", "zebra"], "introspect must be name-sorted");
    assert_eq!(lib.function_names(), vec!["alpha", "end", "zebra"]);
    assert_eq!(lib.comment_markers(), vec!["#"]);

    let alpha = &fns[0];
    assert_eq!(alpha.description, "First one.");
    assert_eq!(alpha.block, "closer:end");
    // literal + two captures, the last optional with a default.
    assert_eq!(alpha.args.len(), 3);
    assert_eq!(alpha.args[0].kind, "literal");
    assert_eq!(alpha.args[0].value, "alpha");
    assert_eq!(alpha.args[1].kind, "capture");
    assert_eq!(alpha.args[1].type_, "string");
    assert!(!alpha.args[1].optional);
    assert!(alpha.args[2].optional);
    assert_eq!(alpha.args[2].default, "primary");

    let zebra = &fns[2];
    assert_eq!(zebra.args[1].description, "how many");
    assert_eq!(zebra.args[1].type_, "int");
}

/// Metaprogramming: a `define … end` block in the SCRIPT adds a function the
/// rest of the script can call, and overrides a library function of the same
/// name.
#[test]
fn script_defines_override_library() {
    let lib = must_lib(
        r##"
extension txt

function greet
    arg literal "greet"
    arg capture who string
    write `library says hello ${decoded who}
`
end
"##,
    );
    // No defines: the library's version runs.
    assert_eq!(must_run(&lib, r##"greet "world""##), "library says hello world\n");

    // A script-level define of the same name wins.
    let out = must_run(
        &lib,
        "define greet\n    arg literal \"greet\"\n    arg capture who string\n    write `script says hi ${decoded who}\n`\nend\n\ngreet \"world\"\n",
    );
    assert_eq!(out, "script says hi world\n");
}

/// `Library` must be `Send + Sync` so downstream crates can share one compiled
/// library across threads — an axum/tokio handler, a `rayon` map, a `OnceLock`
/// global. This has no Go counterpart (a `*capy.Library` is usable from any
/// goroutine by construction); in Rust it is a property of the types we hold,
/// so assert it at compile time rather than trusting it to survive refactors.
/// The engine uses `Arc`, never `Rc`, precisely to keep this true.
#[test]
fn library_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}
    assert_send_sync::<capy_core::capy::Library>();
    assert_send_sync::<capy_core::domain::errors::CapyError>();
    assert_send_sync::<capy_core::domain::val::Val>();
}

/// The thread-safety guarantee above, exercised for real: one `Library` behind
/// an `Arc`, transpiling concurrently on several threads. `run` takes `&self`,
/// so no lock is needed — each call builds its own accumulating context.
#[test]
fn shared_library_runs_concurrently() {
    let lib = std::sync::Arc::new(must_lib(
        r##"
extension txt

function greet
    arg literal "greet"
    arg capture who string
    write `hello ${decoded who}
`
end
"##,
    ));
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let lib = std::sync::Arc::clone(&lib);
            std::thread::spawn(move || {
                let script = format!("greet \"thread-{i}\"\n");
                let out = lib.run(&script).expect("run on worker thread");
                assert_eq!(out, format!("hello thread-{i}\n"));
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker thread panicked");
    }
}
