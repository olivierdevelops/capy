//! PLAN-2026-0001 — T-01, T-02, T-03. Source spans (R1–R4).
//!
//! Before this change, `FuncCall` carried a start `line`/`col` and nothing else,
//! `CaptureValue` carried no position at all, and every node produced by a
//! function-as-type capture was stamped `line: 0, col: 0`. A consumer could name
//! the statement an error was in, but never the operand it was about.

use capy_core::capy::Library;
use capy_core::domain::ast::{Block, FuncCall, Span};
use capy_core::orchestrator::features::{make_lexer, make_parser};

/// Parse a script through a library and hand back the tree.
fn parse(lib_src: &str, script: &str) -> Block {
    let lib = Library::new(lib_src).expect("library compiles");
    let dl = lib.domain();
    // R27 — retain comments so attachment can be asserted.
    let toks = make_lexer::tokenize_with_trivia(script, &dl.comments).expect("lex");
    make_parser::parse(toks, script, dl).expect("parse")
}

/// Walk every `FuncCall` reachable in the tree, including captures' sub-matches,
/// block bodies, closers and sections.
fn walk(b: &Block, f: &mut impl FnMut(&FuncCall)) {
    for st in &b.stmts {
        visit(st, f);
    }
}

fn visit(fc: &FuncCall, f: &mut impl FnMut(&FuncCall)) {
    f(fc);
    for cap in fc.captures.values() {
        for sub in &cap.sub {
            visit(sub, f);
        }
    }
    if let Some(b) = &fc.body {
        walk(b, f);
    }
    if let Some(c) = &fc.closer {
        visit(c, f);
    }
    for sec in fc.sections.values() {
        walk(sec, f);
    }
}

const FLAT: &str = r#"
extension txt

function greet
    arg literal "greet"
    arg capture who any
    arg capture when any
    write `hi
`
end
"#;

/// T-01 — a statement's span starts at its first token and ends one past the
/// last byte of its last token.
#[test]
fn statement_span_covers_the_statement() {
    let tree = parse(FLAT, "greet world now\n");
    let st = &tree.stmts[0];
    assert_eq!(st.span.start_line, 1);
    assert_eq!(st.span.start_col, 1, "starts at `greet`");
    assert_eq!(st.span.end_line, 1);
    // `greet world now` is 15 bytes from column 1, so the exclusive end is 16.
    assert_eq!(st.span.end_col, 16, "one past the final `now`");
    // R5 — the pre-existing render locals keep their meaning.
    assert_eq!((st.line, st.col), (st.span.start_line, st.span.start_col));
}

/// T-02 — two captures on one statement have distinct, non-overlapping spans
/// covering only their own tokens.
#[test]
fn capture_spans_are_distinct_and_tight() {
    let tree = parse(FLAT, "greet world now\n");
    let st = &tree.stmts[0];
    let who = &st.captures["who"];
    let when = &st.captures["when"];

    assert_eq!((who.span.start_col, who.span.end_col), (7, 12), "`world`");
    assert_eq!((when.span.start_col, when.span.end_col), (13, 16), "`now`");
    assert!(
        who.span.end_col <= when.span.start_col,
        "capture spans must not overlap: {:?} vs {:?}",
        who.span,
        when.span
    );
    // Each capture is inside its statement.
    for c in [who, when] {
        assert!(c.span.start_col >= st.span.start_col);
        assert!(c.span.end_col <= st.span.end_col);
    }
}

/// T-03 — **the defect this plan exists for**. Every node produced by a
/// function-as-type capture used to be stamped `line: 0, col: 0`. Nothing
/// reachable may have an unset span, and every child must sit inside its parent.
#[test]
fn no_reachable_node_has_an_unset_span() {
    // Three levels: `call` captures an `args`, which captures a `pair`.
    let lib = r#"
extension txt

function call
    arg literal "call"
    arg capture a args
    write `x`
end

function args
    arg literal "args"
    arg capture p pair
end

function pair
    arg literal "pair"
    arg capture l any
    arg capture r any
end
"#;
    let tree = parse(lib, "call args pair 1 2\n");

    let mut seen = 0usize;
    walk(&tree, &mut |fc| {
        seen += 1;
        assert!(
            !fc.span.is_unset(),
            "node {:?} has an unset span — this is the `line: 0, col: 0` regression",
            fc.func
        );
        assert_ne!(fc.line, 0, "node {:?} reports line 0", fc.func);
        for (name, cap) in &fc.captures {
            if cap.sub.is_empty() {
                continue;
            }
            for sub in &cap.sub {
                assert!(
                    !sub.span.is_unset(),
                    "nested node {:?} under capture {name:?} has an unset span",
                    sub.func
                );
                // Containment: a child's range lies inside its parent's.
                assert!(
                    sub.span.start_col >= fc.span.start_col
                        && sub.span.end_col <= fc.span.end_col,
                    "child {:?} {:?} escapes parent {:?} {:?}",
                    sub.func,
                    sub.span,
                    fc.func,
                    fc.span
                );
            }
        }
    });
    assert_eq!(seen, 3, "expected call -> args -> pair");
}

/// R2 — a block function's span must reach past its body to its closer. The
/// opener alone cannot know this: the body is not parsed until later.
#[test]
fn block_span_covers_body_and_closer() {
    let lib = r#"
extension txt

function wrap
    arg literal "wrap"
    block_closer end
    write `[${body}]`
end

function item
    arg literal "item"
    write `i`
end

function end
end
"#;
    let tree = parse(lib, "wrap\n    item\nend\n");
    let st = &tree.stmts[0];
    assert_eq!(st.span.start_line, 1, "starts at `wrap`");
    assert_eq!(st.span.end_line, 3, "ends on the `end` line, not line 1");
}

/// `Span::join` must ignore an unset operand rather than dragging the result
/// back to line 0 — the exact shape of the bug it exists to avoid.
#[test]
fn join_ignores_unset() {
    let real = Span::new(2, 5, 2, 9);
    assert_eq!(Span::join(real, Span::default()), real);
    assert_eq!(Span::join(Span::default(), real), real);
    let a = Span::new(1, 1, 1, 4);
    let b = Span::new(3, 2, 3, 8);
    assert_eq!(Span::join(a, b), Span::new(1, 1, 3, 8));
}

/// An optional capture that bound its default consumed no tokens, so it has no
/// source range. It must stay unset rather than inventing one.
#[test]
fn defaulted_capture_has_an_unset_span() {
    let lib = r##"
extension txt

function btn
    arg literal "btn"
    arg capture label any
    arg capture color any default "#000"
    write `b`
end
"##;
    let tree = parse(lib, "btn hello\n");
    let st = &tree.stmts[0];
    assert!(!st.captures["label"].span.is_unset(), "`hello` was consumed");
    assert!(
        st.captures["color"].span.is_unset(),
        "a defaulted capture consumed nothing, so it has no source range"
    );
}

// ── R27 — comment retention ─────────────────────────────────────────────────

const COMMENTED: &str = r##"
extension txt

comments
    line "#"
end

function greet
    arg literal "greet"
    arg capture who any
    write `hi ${who}
`
end
"##;

/// T-29 — a comment above a statement is retained and attached to it.
#[test]
fn leading_comment_is_attached() {
    let tree = parse(COMMENTED, "# who to greet
greet world
");
    let st = &tree.stmts[0];
    assert_eq!(st.leading_comments.len(), 1, "one comment leads the statement");
    let c = st.leading_comments[0];
    assert_eq!(c.start_line, 1, "the comment is on line 1");
    assert_eq!(c.start_col, 1);
    assert!(c.end_col > c.start_col, "the comment has a width: {c:?}");
}

/// T-31 — **review residual 1**. The node's own span excludes its comments.
/// A formatter needs both the node's range without them and their own ranges;
/// folding the two loses one irrecoverably.
#[test]
fn node_span_excludes_its_comments() {
    let tree = parse(COMMENTED, "# who to greet
greet world
");
    let st = &tree.stmts[0];
    assert_eq!(st.span.start_line, 2, "the statement starts at `greet`, not at the comment");
    assert_eq!(st.span.start_col, 1);
    assert_eq!(st.leading_comments[0].start_line, 1, "the comment kept its own position");
    assert!(
        st.leading_comments[0].start_line < st.span.start_line,
        "comment sits before the node it leads"
    );
}

/// Several stacked comments all attach, in source order.
#[test]
fn stacked_comments_all_attach_in_order() {
    let tree = parse(COMMENTED, "# one
# two
# three
greet world
");
    let st = &tree.stmts[0];
    assert_eq!(st.leading_comments.len(), 3);
    let lines: Vec<usize> = st.leading_comments.iter().map(|c| c.start_line).collect();
    assert_eq!(lines, vec![1, 2, 3], "source order preserved");
}

/// A trailing comment is retained as trivia but leads nothing, so it must not
/// be mis-attached to the statement it sits on.
#[test]
fn trailing_comment_does_not_attach_backwards() {
    let tree = parse(COMMENTED, "greet world # trailing
");
    let st = &tree.stmts[0];
    assert!(
        st.leading_comments.is_empty(),
        "a trailing comment does not lead this statement: {:?}",
        st.leading_comments
    );
}

/// T-32 — a comment at EOF leads no statement at all. It must be dropped
/// harmlessly rather than panicking or attaching to the last statement.
#[test]
fn comment_at_eof_attaches_to_nothing() {
    let tree = parse(COMMENTED, "greet world
# nothing follows
");
    assert_eq!(tree.stmts.len(), 1);
    assert!(tree.stmts[0].leading_comments.is_empty());
}

/// T-29 — retention must not change what is rendered. This is the R12 gate in
/// miniature: the same script, with and without comments, must produce the same
/// output.
#[test]
fn comments_do_not_change_rendered_output() {
    let lib = Library::new(COMMENTED).expect("library compiles");
    let plain = lib.run("greet world
").expect("run");
    let commented = lib
        .run("# a comment
greet world # and a trailing one
")
        .expect("run");
    assert_eq!(plain, commented, "comments must not affect output");
}

/// A library that declares NO comment markers has no comment syntax at all —
/// `#` is then ordinary source. Retention must not invent one.
#[test]
fn no_markers_means_no_comments() {
    let lib_src = r#"
extension txt

function greet
    arg literal "greet"
    arg capture who any
    write `hi ${who}
`
end
"#;
    let tree = parse(lib_src, "greet world
");
    assert!(tree.stmts[0].leading_comments.is_empty());
}
