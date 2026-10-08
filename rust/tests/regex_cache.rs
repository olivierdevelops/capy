//! The compiled-regex cache is bounded and does not change what callers observe
//! (PROP-2026-0007 E7, T-06, R9).

use capy_core::capy::Library;
use capy_core::infra::regex_cache::{cached_len, compiled};

#[test]
fn the_cache_never_grows_past_its_bound() {
    for i in 0..10_000 {
        let rx = compiled(&format!("^p{}$", i)).expect("valid pattern");
        assert!(rx.is_match(&format!("p{}", i)));
    }
    assert!(cached_len() <= 64, "cache grew to {}", cached_len());
}

#[test]
fn a_compile_error_is_reported_every_time_and_not_cached() {
    let before = cached_len();
    assert!(compiled("(unclosed").is_err());
    assert!(compiled("(unclosed").is_err());
    assert_eq!(cached_len(), before);
}

#[test]
fn a_repeated_pattern_matches_like_a_fresh_one() {
    let a = compiled("AB[0-9]{3}").unwrap();
    let b = compiled("AB[0-9]{3}").unwrap();
    for s in ["xx AB123", "AB12", "ab123", ""] {
        assert_eq!(a.is_match(s), b.is_match(s));
    }
}

#[test]
fn regex_match_in_a_library_is_unchanged() {
    let lib = Library::new(
        "extension txt\nfunction other\n    bare\n    arg capture line tail\n    if (regex_match line \"AB[0-9]{3}\")\n        write `HIT ${line}\n`\n    end\nend\n",
    )
    .unwrap();
    let out = lib.run("foo AB123 bar\nnothing here\nAB999\n").unwrap();
    assert_eq!(out, "HIT foo AB123 bar\nHIT AB999\n");
}
