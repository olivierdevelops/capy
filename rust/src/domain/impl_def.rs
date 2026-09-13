//! Port of `domain/impl.go`. Named `impl_def` because `impl` is a Rust keyword.

/// One implementation of a library's interface. The library's manifest
/// catalogues every available impl; the CLI selects one per invocation. The
/// selected impl's `file` is what the loader actually reads to get the
/// `FuncDef`s / `file_template` / etc.
///
/// Multiple impls let the same source language target multiple outputs — a
/// `chart` DSL can have impls that emit Mermaid, D3, or ASCII; the same source
/// survives the swap.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImplDef {
    pub name: String,
    /// Path relative to the manifest file's directory.
    pub file: String,
    pub description: String,
    pub version: String,
    pub is_default: bool,
}
