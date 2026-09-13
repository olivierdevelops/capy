//! Port of the parts of Go's `path/filepath` and `os` error formatting the
//! engine's observable behaviour depends on.
//!
//! Rust's `Path` deliberately does NOT normalise `..`, while Go's
//! `filepath.Clean` does so lexically — and cleaned paths appear verbatim in
//! error messages (`import cycle: <abs path>`), so the difference is visible.

use std::io;

/// Port of `filepath.Clean` (lexical normalisation, no filesystem access).
pub fn clean(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let rooted = path.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            // Eliminate empty and `.` elements.
            "" | "." => continue,
            ".." => {
                // Eliminate an inner `..` along with the element before it.
                if let Some(&last) = out.last() {
                    if last != ".." {
                        out.pop();
                        continue;
                    }
                }
                // A `..` that begins a rooted path is dropped entirely.
                if rooted {
                    continue;
                }
                out.push("..");
            }
            s => out.push(s),
        }
    }
    let joined = out.join("/");
    if rooted {
        format!("/{}", joined)
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// Port of `filepath.Join` — joins non-empty elements with `/` and cleans.
pub fn join(parts: &[&str]) -> String {
    let non_empty: Vec<&str> = parts.iter().copied().filter(|p| !p.is_empty()).collect();
    if non_empty.is_empty() {
        return String::new();
    }
    clean(&non_empty.join("/"))
}

/// Port of `filepath.IsAbs`.
pub fn is_abs(path: &str) -> bool {
    path.starts_with('/')
}

/// Port of `filepath.Dir`.
pub fn dir(path: &str) -> String {
    match path.rfind('/') {
        None => ".".to_string(),
        Some(i) => clean(&path[..i + 1]),
    }
}

/// Port of `filepath.Base`.
pub fn base(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    match trimmed.rfind('/') {
        None => trimmed.to_string(),
        Some(i) => trimmed[i + 1..].to_string(),
    }
}

/// Port of `filepath.Ext`.
///
/// Go scans the ORIGINAL path backwards until a separator, so `Ext("")` is `""`
/// and `Ext(".bashrc")` is `".bashrc"` — going via `base` gets both wrong.
pub fn ext(path: &str) -> String {
    let b = path.as_bytes();
    let mut i = b.len() as isize - 1;
    while i >= 0 && b[i as usize] != b'/' {
        if b[i as usize] == b'.' {
            return path[i as usize..].to_string();
        }
        i -= 1;
    }
    String::new()
}

/// Renders an I/O failure the way Go's `*os.PathError` does —
/// `open /x/y: no such file or directory` — rather than Rust's
/// `No such file or directory (os error 2)`. Error text reaches golden files.
pub fn io_error(op: &str, path: &str, e: &io::Error) -> String {
    let msg = match e.kind() {
        io::ErrorKind::NotFound => "no such file or directory".to_string(),
        io::ErrorKind::PermissionDenied => "permission denied".to_string(),
        io::ErrorKind::AlreadyExists => "file exists".to_string(),
        io::ErrorKind::InvalidInput => "invalid argument".to_string(),
        _ => {
            // `IsADirectory` / `NotADirectory` are unstable as named kinds; match
            // the raw errno so the common cases still read like Go.
            match e.raw_os_error() {
                Some(21) => "is a directory".to_string(),
                Some(20) => "not a directory".to_string(),
                _ => e.to_string(),
            }
        }
    };
    format!("{} {}: {}", op, path, msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values from Go's filepath.Clean.
    #[test]
    fn clean_matches_go() {
        let cases: &[(&str, &str)] = &[
            ("", "."),
            ("a/b", "a/b"),
            ("a//b", "a/b"),
            ("a/./b", "a/b"),
            ("a/b/../c", "a/c"),
            ("a/../../b", "../b"),
            ("/a/../../b", "/b"),
            ("/..", "/"),
            ("./a", "a"),
            ("a/b/", "a/b"),
            ("/", "/"),
            ("..", ".."),
            ("../..", "../.."),
        ];
        for (input, want) in cases {
            assert_eq!(&clean(input), want, "clean({input:?})");
        }
    }

    #[test]
    fn join_matches_go() {
        assert_eq!(join(&["a", "b"]), "a/b");
        assert_eq!(join(&["a/", "/b"]), "a/b");
        assert_eq!(join(&["", "b"]), "b");
        assert_eq!(join(&["a", "../b"]), "b");
        assert_eq!(join(&[]), "");
    }

    #[test]
    fn dir_base_ext_match_go() {
        assert_eq!(dir("/a/b/c.txt"), "/a/b");
        assert_eq!(dir("c.txt"), ".");
        assert_eq!(base("/a/b/c.txt"), "c.txt");
        assert_eq!(base("/a/b/"), "b");
        assert_eq!(ext("/a/b/c.txt"), ".txt");
        assert_eq!(ext("/a/b/c"), "");
        assert_eq!(ext(""), "");
        assert_eq!(ext(".bashrc"), ".bashrc");
        assert_eq!(ext("/a.b/c"), "");
    }
}
