//! Port of `domain/value.go`.
//!
//! NOTE: this mirrors the Go file for structural fidelity, but `domain.Value`
//! is dead code in the Go reference implementation — nothing outside
//! `domain/value.go` references it. The live runtime value type is
//! [`crate::domain::val::Val`], which models Go's `any`.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Null,
    Str,
    Int,
    Float,
    Bool,
    List,
    Object,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub kind: ValueKind,
    pub str: String,
    pub int: i64,
    pub flt: f64,
    pub bool: bool,
    pub list: Vec<Value>,
    pub obj: BTreeMap<String, Value>,
}

impl Default for Value {
    fn default() -> Self {
        Value {
            kind: ValueKind::Null,
            str: String::new(),
            int: 0,
            flt: 0.0,
            bool: false,
            list: Vec::new(),
            obj: BTreeMap::new(),
        }
    }
}

impl Value {
    pub fn null() -> Value {
        Value::default()
    }
    pub fn str(s: impl Into<String>) -> Value {
        Value { kind: ValueKind::Str, str: s.into(), ..Value::default() }
    }
    pub fn int_v(i: i64) -> Value {
        Value { kind: ValueKind::Int, int: i, ..Value::default() }
    }
    pub fn float_v(f: f64) -> Value {
        Value { kind: ValueKind::Float, flt: f, ..Value::default() }
    }
    pub fn bool_v(b: bool) -> Value {
        Value { kind: ValueKind::Bool, bool: b, ..Value::default() }
    }
    pub fn list_v(vs: Vec<Value>) -> Value {
        Value { kind: ValueKind::List, list: vs, ..Value::default() }
    }
    pub fn obj_v(m: BTreeMap<String, Value>) -> Value {
        Value { kind: ValueKind::Object, obj: m, ..Value::default() }
    }
}
