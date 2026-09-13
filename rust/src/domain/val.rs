//! The live runtime value type — models Go's `any` as used by the evaluator.
//!
//! The Go engine passes values around as bare `any` and type-switches over
//! exactly this set (see `orchestrator/features/inner_evaluator.go`):
//!
//!   `nil`, `string`, `int`, `int64`, `float64`, `bool`, `[]any`,
//!   `map[string]any`, `[]string`
//!
//! `int` and `int64` are never distinguished behaviourally (every type switch
//! handles them identically), so both collapse to [`Val::Int`].
//!
//! `[]string` does NOT collapse into [`Val::List`]. Loop iteration treats the
//! two identically, but `ApplyHelper` in `infra/helpers.go` dispatches through
//! Go type *assertions* — `nonEmpty` does `args[0].([]string)` and `join` does
//! `args[1].([]any)`. A failed assertion yields nil, not a conversion, so the
//! distinction is observable and [`Val::StrList`] preserves it.
//!
//! Objects use `BTreeMap` because both map-iteration sites in the Go evaluator
//! explicitly sort keys ascending before iterating ("sort keys for
//! determinism"), and Go's `fmt` prints maps with sorted keys. `BTreeMap`
//! reproduces that ordering for free.

use crate::gofmt;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Null,
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Val>),
    /// Go's `[]string`, as produced by the `split` helper and the host's
    /// `args`. Distinct from [`Val::List`] only where Go type-asserts on it.
    StrList(Vec<String>),
    Obj(BTreeMap<String, Val>),
}

impl Val {
    pub fn obj() -> Val {
        Val::Obj(BTreeMap::new())
    }

    pub fn str(s: impl Into<String>) -> Val {
        Val::Str(s.into())
    }

    /// Go's `%v` verb. Distinct from [`Val::to_go_string`]: `%v` of nil is
    /// `<nil>`, and it is what list/map elements are rendered with.
    pub fn format_v(&self) -> String {
        match self {
            Val::Null => "<nil>".to_string(),
            Val::Str(s) => s.clone(),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => gofmt::format_float_g(*f),
            Val::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            // Go renders slices as `[a b c]`, space-separated.
            Val::List(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.format_v()).collect();
                format!("[{}]", parts.join(" "))
            }
            // Go renders []string identically to []any.
            Val::StrList(items) => format!("[{}]", items.join(" ")),
            // Go renders maps as `map[k1:v1 k2:v2]` with keys sorted.
            Val::Obj(m) => {
                let parts: Vec<String> =
                    m.iter().map(|(k, v)| format!("{}:{}", k, v.format_v())).collect();
                format!("map[{}]", parts.join(" "))
            }
        }
    }

    /// Port of `toString` in `orchestrator/features/inner_evaluator.go`.
    ///
    /// Differs from `%v` only for nil, which yields the empty string.
    pub fn to_go_string(&self) -> String {
        match self {
            Val::Null => String::new(),
            Val::Str(s) => s.clone(),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => gofmt::format_float_g(*f),
            Val::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            other => other.format_v(),
        }
    }

    /// Go truthiness as the evaluator applies it.
    pub fn is_truthy(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Str(s) => !s.is_empty(),
            Val::Int(i) => *i != 0,
            Val::Float(f) => *f != 0.0,
            Val::List(l) => !l.is_empty(),
            // FIDELITY: Go's `truthy` has arms for nil/bool/string/int64/int/
            // float64/[]any/map[string]any and then `return true`. `[]string`
            // hits that fallthrough, so an EMPTY []string is truthy in Go.
            Val::StrList(_) => true,
            Val::Obj(m) => !m.is_empty(),
        }
    }

    /// Go's `%T`, used in a handful of error messages.
    pub fn go_type_name(&self) -> &'static str {
        match self {
            Val::Null => "<nil>",
            Val::Str(_) => "string",
            Val::Int(_) => "int",
            Val::Float(_) => "float64",
            Val::Bool(_) => "bool",
            Val::List(_) => "[]interface {}",
            Val::StrList(_) => "[]string",
            Val::Obj(_) => "map[string]interface {}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_v_matches_go_composites() {
        // fmt.Sprintf("%v", []any{1, "a", nil}) == "[1 a <nil>]"
        let l = Val::List(vec![Val::Int(1), Val::str("a"), Val::Null]);
        assert_eq!(l.format_v(), "[1 a <nil>]");

        // fmt.Sprintf("%v", map[string]any{"b":2,"a":1}) == "map[a:1 b:2]"
        let mut m = BTreeMap::new();
        m.insert("b".to_string(), Val::Int(2));
        m.insert("a".to_string(), Val::Int(1));
        assert_eq!(Val::Obj(m).format_v(), "map[a:1 b:2]");
    }

    #[test]
    fn to_go_string_nil_is_empty() {
        assert_eq!(Val::Null.to_go_string(), "");
        assert_eq!(Val::Null.format_v(), "<nil>");
    }

    #[test]
    fn float_goes_through_go_g_format() {
        assert_eq!(Val::Float(1000000.0).to_go_string(), "1e+06");
        assert_eq!(Val::Float(1.5).to_go_string(), "1.5");
    }
}
