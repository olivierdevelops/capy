//! A small, bounded cache of compiled regexes (PROP-2026-0007 E7).
//!
//! `regex_match` and the `pattern` check on custom types used to call
//! `Regex::new` on every evaluation. Compiling is orders of magnitude dearer than
//! matching, so a script that tests every line paid for it once per line.
//!
//! Bounded on purpose (ARCH-003): a pattern built from captured input could
//! otherwise grow the cache without limit. When full it is cleared; no LRU, no
//! new type. A compile error is never cached, so it keeps its exact message and
//! timing. `Regex` is reference-counted internally, so a clone is cheap.

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;

const CAPACITY: usize = 64;

thread_local! {
    static CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

/// The compiled form of `pattern`, compiling (and caching) it on first use.
pub fn compiled(pattern: &str) -> Result<Regex, regex::Error> {
    CACHE.with(|c| {
        if let Some(rx) = c.borrow().get(pattern) {
            return Ok(rx.clone());
        }
        let rx = Regex::new(pattern)?;
        let mut map = c.borrow_mut();
        if map.len() >= CAPACITY {
            map.clear();
        }
        map.insert(pattern.to_string(), rx.clone());
        Ok(rx)
    })
}

/// Number of cached patterns (tests).
pub fn cached_len() -> usize {
    CACHE.with(|c| c.borrow().len())
}
