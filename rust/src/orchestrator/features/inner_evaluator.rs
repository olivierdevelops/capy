//! Port of `orchestrator/features/inner_evaluator.go`.
//!
//! `InnerEvaluator` runs a `run:` snippet against:
//!   * captures: the bindings the outer match produced (read-only)
//!   * context:  the live accumulator (mutated by set/append/…)
//!   * locals:   the inner scope (loop variables; library-private bindings)
//!   * host:     the embedder-provided host capability surface
//!
//! It does NOT execute user-script code. It only updates `context`.
//!
//! STRUCTURAL NOTE: Go relies on maps and slices being reference types, so
//! `descend` can hand `applyOp` a plain `any` that still aliases the context.
//! Rust needs an explicit `&mut` walk. Index expressions are therefore resolved
//! into concrete keys BEFORE the mutable walk begins — same left-to-right
//! evaluation order, and index expressions are pure reads, so this is
//! behaviour-preserving.

use super::expr_to_text::{expr_to_text, var_ref_to_text};
use crate::domain::ast::{CallExpr, CaptureValue, Expr, InnerBlock, InnerStmt, Path, PathStep};
use crate::domain::errors::CapyError;
use crate::domain::host::{Host, NoOpHost};
use crate::domain::val::Val;
use crate::gofmt;
use crate::infra::helpers;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Scope maps used by the evaluator.
pub type Locals = BTreeMap<String, Val>;
pub type Captures = BTreeMap<String, CaptureValue>;

/// The `OnUnknownCall` hook. Returning `(Some(v), true)` / `(None, true)` means
/// "handled"; `(None, false)` means "still unknown, raise the normal error".
pub type UnknownCallHook =
    Arc<dyn Fn(&str, &[Val]) -> Result<(Option<Val>, bool), CapyError> + Send + Sync>;

pub struct InnerEvaluator {
    /// Always a [`Val::Obj`]. Held as a `Val` so the mutable path walk is
    /// uniform with nested containers.
    pub context: Val,
    pub host: Arc<dyn Host + Send + Sync>,
    /// Invoked when `run_primitive` doesn't recognise a call's name. Used by the
    /// command runner to add command-only primitives without growing the global
    /// primitive set.
    pub on_unknown_call: Option<UnknownCallHook>,
}

impl InnerEvaluator {
    pub fn new(context: BTreeMap<String, Val>, host: Arc<dyn Host + Send + Sync>) -> InnerEvaluator {
        InnerEvaluator { context: Val::Obj(context), host, on_unknown_call: None }
    }

    pub fn with_noop_host(context: BTreeMap<String, Val>) -> InnerEvaluator {
        InnerEvaluator::new(context, Arc::new(NoOpHost))
    }

    /// The context as a plain map, for callers that need the final accumulator.
    pub fn ctx_map(&self) -> &BTreeMap<String, Val> {
        match &self.context {
            Val::Obj(m) => m,
            _ => unreachable!("context is always an object"),
        }
    }

    /// Port of `Exec`.
    pub fn exec(&mut self, prog: &InnerBlock, captures: &Captures) -> Result<(), CapyError> {
        let mut locals: Locals = BTreeMap::new();
        self.exec_block(prog, captures, &mut locals)
    }

    /// Port of `ExecWithLocals`.
    ///
    /// Runs a state-mutation block with pre-seeded locals. The outer evaluator
    /// uses this to expose the rendered inner-block output as `body` to the run
    /// pass, so state mutations can capture or transform the rendered text.
    pub fn exec_with_locals(
        &mut self,
        prog: &InnerBlock,
        captures: &Captures,
        locals: &mut Locals,
    ) -> Result<(), CapyError> {
        self.exec_block(prog, captures, locals)
    }

    fn exec_block(
        &mut self,
        b: &InnerBlock,
        caps: &Captures,
        locals: &mut Locals,
    ) -> Result<(), CapyError> {
        for s in &b.stmts {
            self.exec_stmt(s, caps, locals)?;
        }
        Ok(())
    }

    /// Port of `execStmt`.
    fn exec_stmt(
        &mut self,
        s: &InnerStmt,
        caps: &Captures,
        locals: &mut Locals,
    ) -> Result<(), CapyError> {
        match s {
            InnerStmt::Set { target, value } => {
                let v = self.eval(value, caps, locals)?;
                self.write_path(target, v, caps, locals, "set")
            }
            InnerStmt::Append { target, value } => {
                let v = self.eval(value, caps, locals)?;
                self.write_path(target, v, caps, locals, "append")
            }
            InnerStmt::Prepend { target, value } => {
                let v = self.eval(value, caps, locals)?;
                self.write_path(target, v, caps, locals, "prepend")
            }
            InnerStmt::Merge { target, value } => {
                let v = self.eval(value, caps, locals)?;
                if !matches!(v, Val::Obj(_)) {
                    return Err(CapyError::msg("merge: value must be a map"));
                }
                self.write_path(target, v, caps, locals, "merge")
            }
            InnerStmt::Delete { target } => {
                self.write_path(target, Val::Null, caps, locals, "delete")
            }
            InnerStmt::If { cond, body, else_ } => {
                let v = self.eval(cond, caps, locals)?;
                if v.is_truthy() {
                    return self.exec_block(body, caps, locals);
                }
                if let Some(e) = else_ {
                    return self.exec_block(e, caps, locals);
                }
                Ok(())
            }
            InnerStmt::Loop(l) => {
                let v = self.eval(&l.iter, caps, locals)?;
                match v {
                    Val::List(coll) => {
                        for (i, item) in coll.iter().enumerate() {
                            let mut child = locals.clone();
                            child.insert(l.var.clone(), item.clone());
                            if !l.key_var.is_empty() {
                                child.insert(l.key_var.clone(), Val::Int(i as i64));
                            }
                            self.exec_block(&l.body, caps, &mut child)?;
                        }
                        Ok(())
                    }
                    Val::Obj(coll) => {
                        // Map iteration: sort keys for determinism. `BTreeMap`
                        // already yields sorted order, matching Go's explicit
                        // insertion sort here.
                        for (k, item) in coll.iter() {
                            let mut child = locals.clone();
                            child.insert(l.var.clone(), item.clone());
                            if !l.key_var.is_empty() {
                                child.insert(l.key_var.clone(), Val::Str(k.clone()));
                            }
                            self.exec_block(&l.body, caps, &mut child)?;
                        }
                        Ok(())
                    }
                    // FIDELITY: Go's execStmt loop has NO []string arm (unlike
                    // renderStmt, which does), so iterating a `split` result in
                    // RUN scope errors while the same loop in RENDER scope works.
                    _ => Err(CapyError::msg("loop iterable must be a list or map")),
                }
            }
            InnerStmt::Call(c) => self.run_primitive(c, caps, locals),
            InnerStmt::Write(_) => {
                // Should be translated away at library-load time. Reaching here
                // means the translator missed a case.
                Err(CapyError::msg("internal: unexpanded WriteStmt — please file a bug"))
            }
        }
    }

    /// Port of `runPrimitive`.
    fn run_primitive(
        &mut self,
        c: &CallExpr,
        caps: &Captures,
        locals: &mut Locals,
    ) -> Result<(), CapyError> {
        let name = c.name.join(".");
        // Evaluate all args once up-front.
        let mut arg_vals: Vec<Val> = Vec::with_capacity(c.args.len());
        for a in &c.args {
            arg_vals.push(self.eval(a, caps, locals)?);
        }
        match name.as_str() {
            "error" => {
                if arg_vals.is_empty() {
                    return Err(CapyError::msg("error"));
                }
                return Err(CapyError::msg(arg_vals[0].to_go_string()));
            }
            "print" => {
                let parts: Vec<String> = arg_vals.iter().map(|v| v.to_go_string()).collect();
                println!("{}", parts.join(" "));
                return Ok(());
            }
            "write_file" => {
                if arg_vals.len() != 2 {
                    return Err(CapyError::msg("write_file: expected 2 args (path, contents)"));
                }
                return self
                    .host
                    .write_file(&arg_vals[0].to_go_string(), &arg_vals[1].to_go_string());
            }
            "mkdir" => {
                if arg_vals.len() != 1 {
                    return Err(CapyError::msg("mkdir: expected 1 arg (path)"));
                }
                return self.host.mkdir(&arg_vals[0].to_go_string());
            }
            "exec" => {
                if arg_vals.is_empty() {
                    return Err(CapyError::msg("exec: expected at least 1 arg (cmd)"));
                }
                let cmd = arg_vals[0].to_go_string();
                let args: Vec<String> =
                    arg_vals[1..].iter().map(|a| a.to_go_string()).collect();
                return self.host.exec(&cmd, &args);
            }
            "cd" => {
                if arg_vals.len() != 1 {
                    return Err(CapyError::msg("cd: expected 1 arg (path)"));
                }
                let p = arg_vals[0].to_go_string();
                return std::env::set_current_dir(&p)
                    .map_err(|e| CapyError::msg(crate::gopath::io_error("chdir", &p, &e)));
            }
            _ => {}
        }
        // Last resort: hand off to the embedder's hook.
        if let Some(hook) = self.on_unknown_call.clone() {
            let (_, handled) = hook(&name, &arg_vals)?;
            if handled {
                return Ok(());
            }
        }
        Err(CapyError::msg(format!("unknown inner call {}", gofmt::quote(&name))))
    }

    // --- write side -------------------------------------------------------

    /// Port of `writePath`.
    ///
    /// Performs `op` on context (or locals if root is "locals"). The path's root
    /// must be either "context" or "locals".
    fn write_path(
        &mut self,
        p: &Path,
        value: Val,
        caps: &Captures,
        locals: &mut Locals,
        op: &str,
    ) -> Result<(), CapyError> {
        if p.root == "locals" {
            // `let X = …` desugars to `set locals.X …`.
            if p.steps.len() == 1 && !p.steps[0].is_index {
                locals.insert(p.steps[0].field.clone(), value);
                return Ok(());
            }
            return Err(CapyError::msg(format!(
                "{}: only single-name `locals.X` writes are supported (got {} steps)",
                op,
                p.steps.len()
            )));
        }
        if p.root != "context" {
            return Err(CapyError::msg(format!(
                "{}: only `context.*` and `locals.*` paths are writable, got root {}",
                op,
                gofmt::quote(&p.root)
            )));
        }
        if p.steps.is_empty() {
            return Err(CapyError::msg(format!(
                "{}: path must have at least one step under context",
                op
            )));
        }
        // Resolve index expressions before the mutable walk (see module note).
        let mut resolved: Vec<ResolvedStep> = Vec::with_capacity(p.steps.len());
        for step in &p.steps {
            if step.is_index {
                let idx = match &step.index {
                    Some(e) => self.eval(e, caps, locals)?,
                    None => Val::Null,
                };
                resolved.push(ResolvedStep::Index(idx));
            } else {
                resolved.push(ResolvedStep::Field(step.field.clone()));
            }
        }
        let last = resolved.len() - 1;
        let mut cur: &mut Val = &mut self.context;
        for rs in &resolved[..last] {
            cur = descend_mut(cur, rs)?;
        }
        apply_op(cur, &resolved[last], value, op)
    }

    // --- render side ------------------------------------------------------

    /// Port of `RenderAST`.
    ///
    /// Walks a write-style AST and emits the output text.
    ///
    /// Render-scope: `locals` carries the iteration variable for the current
    /// `for`, the special `body` value, and any captures the caller pre-stuffed
    /// in. `context` resolves to `self.context`. Captures from a function call
    /// appear in `locals` as their source-text form — what the user wrote appears
    /// verbatim in the output unless a helper transforms it.
    ///
    /// State-mutation statements are intentionally no-ops here — state changes
    /// happen on a separate pass via [`InnerEvaluator::exec`].
    pub fn render_ast(&self, b: &InnerBlock, locals: &Locals) -> Result<String, CapyError> {
        let mut out = String::new();
        self.render_block(b, locals, &mut out)?;
        Ok(out)
    }

    fn render_block(
        &self,
        b: &InnerBlock,
        locals: &Locals,
        out: &mut String,
    ) -> Result<(), CapyError> {
        for s in &b.stmts {
            self.render_stmt(s, locals, out)?;
        }
        Ok(())
    }

    /// Port of `renderStmt`.
    fn render_stmt(
        &self,
        s: &InnerStmt,
        locals: &Locals,
        out: &mut String,
    ) -> Result<(), CapyError> {
        match s {
            InnerStmt::Write(v) => {
                let val = self.eval_render(v, locals)?;
                out.push_str(&val.to_go_string());
                Ok(())
            }
            InnerStmt::If { cond, body, else_ } => {
                let v = self.eval_render(cond, locals)?;
                if v.is_truthy() {
                    return self.render_block(body, locals, out);
                }
                if let Some(e) = else_ {
                    return self.render_block(e, locals, out);
                }
                Ok(())
            }
            InnerStmt::Loop(l) => {
                let v = self.eval_render(&l.iter, locals)?;
                match v {
                    Val::List(coll) => {
                        for (i, item) in coll.iter().enumerate() {
                            let mut child = locals.clone();
                            child.insert(l.var.clone(), item.clone());
                            if !l.key_var.is_empty() {
                                child.insert(l.key_var.clone(), Val::Int(i as i64));
                            }
                            self.render_block(&l.body, &child, out)?;
                        }
                        Ok(())
                    }
                    // Host primitives like `args` return []string. Treat it like
                    // []any for iteration purposes.
                    Val::StrList(coll) => {
                        for (i, item) in coll.iter().enumerate() {
                            let mut child = locals.clone();
                            child.insert(l.var.clone(), Val::Str(item.clone()));
                            if !l.key_var.is_empty() {
                                child.insert(l.key_var.clone(), Val::Int(i as i64));
                            }
                            self.render_block(&l.body, &child, out)?;
                        }
                        Ok(())
                    }
                    Val::Obj(coll) => {
                        for (k, item) in coll.iter() {
                            let mut child = locals.clone();
                            child.insert(l.var.clone(), item.clone());
                            if !l.key_var.is_empty() {
                                child.insert(l.key_var.clone(), Val::Str(k.clone()));
                            }
                            self.render_block(&l.body, &child, out)?;
                        }
                        Ok(())
                    }
                    // no-op iteration
                    Val::Null => Ok(()),
                    other => Err(CapyError::msg(format!(
                        "render: loop iterable must be a list or map, got {}",
                        other.go_type_name()
                    ))),
                }
            }
            // Set/Append/Prepend/Merge/Delete/Call — render-side no-op.
            _ => Ok(()),
        }
    }

    /// Port of `evalRender`.
    ///
    /// Slightly different from `eval`: VarRef paths that resolve to nothing
    /// return an empty string (matching template-engine behaviour) instead of
    /// erroring; CallExpr first tries the helper table so `${unquote x}` works.
    fn eval_render(&self, x: &Expr, locals: &Locals) -> Result<Val, CapyError> {
        match x {
            Expr::Number(n) => {
                Ok(if n.is_int { Val::Int(n.i) } else { Val::Float(n.f) })
            }
            // Render-mode interpolation: `${expr}` may be a path, helper call, or
            // pipe chain — not just a path lookup.
            Expr::Str(v) => Ok(Val::Str(self.interpolate_render(v, locals)?)),
            Expr::Bool(b) => Ok(Val::Bool(*b)),
            Expr::Null => Ok(Val::Null),
            Expr::Var(steps) => match self.resolve_render(steps, locals) {
                // Templates tolerate missing paths — emit empty.
                Err(_) => Ok(Val::Str(String::new())),
                Ok(v) => Ok(v),
            },
            Expr::Not(inner) => {
                let v = self.eval_render(inner, locals)?;
                Ok(Val::Bool(!v.is_truthy()))
            }
            Expr::Compare(c) => {
                let l = self.eval_render(&c.left, locals)?;
                let r = self.eval_render(&c.right, locals)?;
                Ok(Val::Bool(cmp(&c.op, &l, &r)?))
            }
            Expr::Call(n) => {
                let name = n.name.join(".");
                let mut arg_vals: Vec<Val> = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    arg_vals.push(self.eval_render(a, locals)?);
                }
                // Try the template-helper table first (add/upper/unquote/…).
                let (res, ok, err) = helpers::apply_helper(&name, &arg_vals);
                if ok {
                    if let Some(e) = err {
                        return Err(CapyError::msg(e));
                    }
                    return Ok(res.unwrap_or(Val::Null));
                }
                // `len` is a builtin in templates but not in apply_helper.
                if name == "len" && arg_vals.len() == 1 {
                    return Ok(Val::Int(builtin_len(&arg_vals[0])));
                }
                // Fall through to the inner evaluator's CallExpr (regex_match,
                // env, …) with an empty capture scope, as Go does.
                let empty: Captures = BTreeMap::new();
                self.eval(x, &empty, locals)
            }
            other => Err(CapyError::msg(format!(
                "render: unsupported expression {}",
                expr_kind_name(other)
            ))),
        }
    }

    /// Port of `RenderPath`.
    ///
    /// Resolves a file-path string with write-style `${...}` interpolations.
    pub fn render_path(&self, path: &str, locals: &Locals) -> Result<String, CapyError> {
        self.interpolate_render(path, locals)
    }

    /// Port of `interpolateRender`.
    ///
    /// Walks a backtick string body and resolves `${expr}` markers in render
    /// scope. Unlike `interpolate_generic` (dotted paths only), this evaluates
    /// the full inner-expression grammar: helper calls, pipes, nested calls.
    fn interpolate_render(&self, s: &str, locals: &Locals) -> Result<String, CapyError> {
        let b = s.as_bytes();
        let mut out: Vec<u8> = Vec::with_capacity(b.len());
        let mut i = 0usize;
        while i < b.len() {
            if i + 1 < b.len() && b[i] == b'$' && b[i + 1] == b'{' {
                let mut j = i + 2;
                let mut depth = 1i64;
                while j < b.len() && depth > 0 {
                    if b[j] == b'{' {
                        depth += 1;
                    } else if b[j] == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if j >= b.len() {
                    return Err(CapyError::msg("unterminated ${...}"));
                }
                let expr = s[i + 2..j].trim();
                let v = self.eval_interp(expr, locals)?;
                out.extend_from_slice(v.to_go_string().as_bytes());
                i = j + 1;
            } else if b[i] == b'\\' && i + 1 < b.len() {
                // Go-style escape: \n / \t / \r / \` / \\, and \X (literal X)
                // for other characters.
                match b[i + 1] {
                    b'n' => out.push(b'\n'),
                    b't' => out.push(b'\t'),
                    b'r' => out.push(b'\r'),
                    b'`' => out.push(b'`'),
                    b'\\' => out.push(b'\\'),
                    other => out.push(other),
                }
                i += 2;
            } else {
                out.push(b[i]);
                i += 1;
            }
        }
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    /// Port of `evalInterp`.
    ///
    /// Parses + evaluates the body of a single `${...}`:
    ///
    /// ```text
    /// atom  := IDENT (. IDENT | [ EXPR ])*   -- path lookup
    ///        | NUMBER | STRING                -- literal
    ///        | ( EXPR )                       -- parens
    /// stage := atom+                          -- call: head=fn, rest=args
    /// expr  := stage ( | stage )*             -- pipe chain
    /// ```
    ///
    /// Pipes flow left to right with each stage receiving the running value as
    /// its FINAL argument.
    fn eval_interp(&self, expr: &str, locals: &Locals) -> Result<Val, CapyError> {
        let stages = split_interp_pipe_runtime(expr);
        let mut running = Val::Null;
        for (idx, stage) in stages.iter().enumerate() {
            let mut atoms = tokenise_interp_runtime(stage.trim());
            if atoms.is_empty() {
                continue;
            }
            if idx > 0 {
                // Pipe stage: previous value becomes the last argument.
                atoms.push(String::new());
            }
            let head = atoms[0].clone();
            let rest = &atoms[1..];
            let mut arg_vals: Vec<Val> = Vec::with_capacity(rest.len());
            for (ai, a) in rest.iter().enumerate() {
                if idx > 0 && ai == rest.len() - 1 {
                    // Last slot is the piped value.
                    arg_vals.push(running.clone());
                    continue;
                }
                arg_vals.push(self.eval_interp_atom(a, locals)?);
            }
            if atoms.len() == 1 {
                // Single atom: it's a value (path or literal), not a call.
                running = self.eval_interp_atom(&head, locals)?;
                continue;
            }
            // Multi-atom: call head with arg_vals as a helper / builtin.
            let (res, ok, err) = helpers::apply_helper(&head, &arg_vals);
            if ok {
                if let Some(e) = err {
                    return Err(CapyError::msg(e));
                }
                running = res.unwrap_or(Val::Null);
                continue;
            }
            // `len` builtin (templates have it but apply_helper doesn't).
            if head == "len" && arg_vals.len() == 1 {
                running = Val::Int(builtin_len(&arg_vals[0]));
                continue;
            }
            return Err(CapyError::msg(format!(
                "interp: unknown helper {}",
                gofmt::quote(&head)
            )));
        }
        Ok(running)
    }

    /// Port of `evalInterpAtom`.
    fn eval_interp_atom(&self, s: &str, locals: &Locals) -> Result<Val, CapyError> {
        if s.is_empty() {
            return Ok(Val::Str(String::new()));
        }
        let b = s.as_bytes();
        if b.len() >= 2 && b[0] == b'"' && b[b.len() - 1] == b'"' {
            // Quoted string literal. The lexer preserved every backslash
            // sequence verbatim from the backtick body, so TWO unescape passes
            // are needed to match the legacy Go-template path (backtick body
            // unescape + template string literal unescape). Concretely: source
            // `\\n` → pass 1: `\n` (backslash + n) → pass 2: a real newline.
            let inner = unescape_string_lit_inner(&unescape_string_lit_inner(
                &s[1..s.len() - 1],
            ));
            return Ok(Val::Str(inner));
        }
        if let Ok(v) = s.parse::<i64>() {
            return Ok(Val::Int(v));
        }
        if let Ok(v) = s.parse::<f64>() {
            return Ok(Val::Float(v));
        }
        if b.len() >= 2 && b[0] == b'(' && b[b.len() - 1] == b')' {
            return self.eval_interp(&s[1..s.len() - 1], locals);
        }
        // Path lookup: `a.b.c`, optionally with `[index]` steps.
        match self.eval_interp_path(s, locals) {
            // Missing paths render empty — matches Go template tolerance.
            Err(_) => Ok(Val::Str(String::new())),
            Ok(v) => Ok(v),
        }
    }

    /// Port of `evalInterpPath`.
    fn eval_interp_path(&self, s: &str, locals: &Locals) -> Result<Val, CapyError> {
        let b = s.as_bytes();
        let mut i = 0usize;
        // Consume a path name up to the next `.` or `[`.
        let read_segment = |i: &mut usize| -> String {
            let start = *i;
            while *i < b.len() && b[*i] != b'.' && b[*i] != b'[' {
                *i += 1;
            }
            s[start..*i].to_string()
        };
        let root = read_segment(&mut i);
        if root.is_empty() {
            return Err(CapyError::msg(format!("empty path atom {}", gofmt::quote(s))));
        }
        let mut cur: &Val = if let Some(v) = locals.get(&root) {
            v
        } else if root == "context" {
            &self.context
        } else {
            return Err(CapyError::msg(format!("undefined {}", gofmt::quote(&root))));
        };
        while i < b.len() {
            match b[i] {
                b'.' => {
                    i += 1;
                    let field = read_segment(&mut i);
                    match descend_read_ref(cur, &Val::Str(field.clone())) {
                        None => {
                            return Err(CapyError::msg(format!(
                                "cannot access {} on non-map ({})",
                                gofmt::quote(&field),
                                cur.go_type_name()
                            )))
                        }
                        Some(v) => cur = v,
                    }
                }
                b'[' => {
                    // Find the matching `]`, skipping nested brackets and quoted
                    // strings (so `m["a]b"]` and `grid[i][j]` parse).
                    let mut depth = 1i64;
                    let mut j = i + 1;
                    let mut in_str = false;
                    while j < b.len() {
                        let ch = b[j];
                        if in_str {
                            if ch == b'\\' {
                                j += 1;
                                j += 1;
                                continue;
                            }
                            if ch == b'"' {
                                in_str = false;
                            }
                            j += 1;
                            continue;
                        }
                        match ch {
                            b'"' => in_str = true,
                            b'[' => depth += 1,
                            b']' => depth -= 1,
                            _ => {}
                        }
                        if depth == 0 {
                            break;
                        }
                        j += 1;
                    }
                    if depth != 0 {
                        return Err(CapyError::msg(format!(
                            "unbalanced [ ] in path {}",
                            gofmt::quote(s)
                        )));
                    }
                    let inner = s[i + 1..j].trim();
                    let idx = self.eval_interp(inner, locals)?;
                    match descend_read_ref(cur, &idx) {
                        None => {
                            return Err(CapyError::msg(format!(
                                "cannot index into {}",
                                cur.go_type_name()
                            )))
                        }
                        Some(v) => cur = v,
                    }
                    i = j + 1;
                }
                other => {
                    return Err(CapyError::msg(format!(
                        "unexpected {} in path {}",
                        gofmt::quote_rune(other as char),
                        gofmt::quote(s)
                    )))
                }
            }
        }
        Ok(cur.clone())
    }

    // --- expression evaluator (independent of outer scope) ----------------

    /// Port of `eval`.
    pub fn eval(&self, x: &Expr, caps: &Captures, locals: &Locals) -> Result<Val, CapyError> {
        match x {
            Expr::Number(n) => Ok(if n.is_int { Val::Int(n.i) } else { Val::Float(n.f) }),
            Expr::Str(v) => {
                let s = interpolate_generic(v, |path| self.resolve_path(path, caps, locals))?;
                Ok(Val::Str(s))
            }
            Expr::Bool(b) => Ok(Val::Bool(*b)),
            Expr::Null => Ok(Val::Null),
            Expr::Var(steps) => self.resolve_path_steps(steps, caps, locals),
            Expr::Not(inner) => {
                let v = self.eval(inner, caps, locals)?;
                Ok(Val::Bool(!v.is_truthy()))
            }
            Expr::Compare(c) => {
                let l = self.eval(&c.left, caps, locals)?;
                let r = self.eval(&c.right, caps, locals)?;
                Ok(Val::Bool(cmp(&c.op, &l, &r)?))
            }
            Expr::List(items) => {
                let mut out: Vec<Val> = Vec::with_capacity(items.len());
                for it in items {
                    out.push(self.eval(it, caps, locals)?);
                }
                Ok(Val::List(out))
            }
            Expr::Obj(o) => {
                let mut out: BTreeMap<String, Val> = BTreeMap::new();
                for (i, k) in o.keys.iter().enumerate() {
                    let v = self.eval(&o.vals[i], caps, locals)?;
                    out.insert(k.clone(), v);
                }
                Ok(Val::Obj(out))
            }
            Expr::Call(n) => self.eval_call(x, n, caps, locals),
        }
    }

    /// The `CallExpr` arm of `eval` — inline built-ins plus the host surface.
    fn eval_call(
        &self,
        whole: &Expr,
        n: &CallExpr,
        caps: &Captures,
        locals: &Locals,
    ) -> Result<Val, CapyError> {
        let _ = whole;
        let name = n.name.join(".");
        match name.as_str() {
            "regex_match" => {
                if n.args.len() != 2 {
                    return Err(CapyError::msg("regex_match expects 2 args"));
                }
                let s = self.eval(&n.args[0], caps, locals)?;
                let p = self.eval(&n.args[1], caps, locals)?;
                let rx = Regex::new(&p.to_go_string())
                    .map_err(|e| CapyError::msg(go_regex_error(&p.to_go_string(), &e)))?;
                return Ok(Val::Bool(rx.is_match(&s.to_go_string())));
            }
            "env" => {
                // env "NAME" → string. Returns the OS env var, or "" if unset.
                if n.args.len() != 1 {
                    return Err(CapyError::msg("env expects 1 arg: env \"NAME\""));
                }
                let name_v = self.eval(&n.args[0], caps, locals)?;
                return Ok(Val::Str(self.host.env(&name_v.to_go_string())));
            }
            "arg" => {
                // arg N → string. The N-th positional CLI arg (zero-indexed).
                if n.args.len() != 1 {
                    return Err(CapyError::msg("arg expects 1 arg: arg INDEX"));
                }
                let idx_v = self.eval(&n.args[0], caps, locals)?;
                let idx = match to_int_go(&idx_v) {
                    None => {
                        return Err(CapyError::msg(format!(
                            "arg: index must be a number, got {}",
                            idx_v.go_type_name()
                        )))
                    }
                    Some(i) => i,
                };
                // Go passes a plain int; a negative index can't address a slot.
                let i = if idx < 0 { usize::MAX } else { idx as usize };
                return Ok(Val::Str(self.host.arg(i)));
            }
            "arg_count" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("arg_count takes no args"));
                }
                return Ok(Val::Int(self.host.arg_count() as i64));
            }
            "args" => {
                // args → []any (Go converts the host's []string element-wise).
                if !n.args.is_empty() {
                    return Err(CapyError::msg("args takes no args"));
                }
                let raw = self.host.args();
                return Ok(Val::List(raw.into_iter().map(Val::Str).collect()));
            }
            "os" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("os takes no args"));
                }
                return Ok(Val::Str(self.host.os()));
            }
            "arch" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("arch takes no args"));
                }
                return Ok(Val::Str(self.host.arch()));
            }
            "cwd" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("cwd takes no args"));
                }
                return Ok(Val::Str(self.host.cwd()?));
            }
            "home_dir" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("home_dir takes no args"));
                }
                return Ok(Val::Str(self.host.home_dir()?));
            }
            "read_file" => {
                // read_file "path" → string. Errors abort the transpilation.
                if n.args.len() != 1 {
                    return Err(CapyError::msg(
                        "read_file expects 1 arg: read_file \"PATH\"",
                    ));
                }
                let p = self.eval(&n.args[0], caps, locals)?;
                return Ok(Val::Str(self.host.read_file(&p.to_go_string())?));
            }
            "mktemp" => {
                // mktemp ".ext" → path to a fresh temp file.
                let mut suffix = String::new();
                if n.args.len() == 1 {
                    let v = self.eval(&n.args[0], caps, locals)?;
                    suffix = v.to_go_string();
                }
                return Ok(Val::Str(self.host.mk_temp(&suffix)?));
            }
            "mktemp_dir" => {
                if !n.args.is_empty() {
                    return Err(CapyError::msg("mktemp_dir takes no args"));
                }
                return Ok(Val::Str(self.host.mk_temp_dir()?));
            }
            "exec_capture" => {
                // exec_capture "cmd" "arg1" … → combined stdout.
                if n.args.is_empty() {
                    return Err(CapyError::msg(
                        "exec_capture: expected at least 1 arg (cmd)",
                    ));
                }
                let mut vs: Vec<String> = Vec::with_capacity(n.args.len());
                for a in &n.args {
                    vs.push(self.eval(a, caps, locals)?.to_go_string());
                }
                return Ok(Val::Str(self.host.exec_capture(&vs[0], &vs[1..])?));
            }
            _ => {}
        }
        // Evaluate arg values once for the remaining lookups.
        let mut arg_vals: Vec<Val> = Vec::with_capacity(n.args.len());
        for a in &n.args {
            arg_vals.push(self.eval(a, caps, locals)?);
        }
        // Template-helper bridge: the same helpers the renderer uses, so
        // libraries can pre-compute values in expression position.
        let (res, ok, err) = helpers::apply_helper(&name, &arg_vals);
        if ok {
            if let Some(e) = err {
                return Err(CapyError::msg(e));
            }
            return Ok(res.unwrap_or(Val::Null));
        }
        // Hook fallback (e.g. the command runner adds `compile script`).
        if let Some(hook) = &self.on_unknown_call {
            let (v, handled) = hook(&name, &arg_vals)?;
            if handled {
                return Ok(v.unwrap_or(Val::Null));
            }
        }
        Err(CapyError::msg(format!(
            "inner call {} not allowed in expression",
            gofmt::quote(&name)
        )))
    }

    /// Port of `evalExprFallback`.
    ///
    /// `eval` with one extra rule: an unresolved single-path VarRef returns the
    /// identifier as a string. That matches transpile semantics — the source name
    /// flows through to the output unchanged.
    fn eval_expr_fallback(
        &self,
        x: &Expr,
        caps: &Captures,
        locals: &Locals,
    ) -> Result<Val, CapyError> {
        if let Expr::Var(steps) = x {
            if !steps.is_empty() {
                let root = &steps[0].field;
                if !locals.contains_key(root)
                    && !caps.contains_key(root)
                    && root != "context"
                {
                    return Ok(Val::Str(var_ref_to_text(steps, &expr_to_text)));
                }
            }
        }
        self.eval(x, caps, locals)
    }

    /// Port of `resolvePath` — adapts a dotted `[]string` path to the
    /// step-based resolver.
    fn resolve_path(
        &self,
        path: &[String],
        caps: &Captures,
        locals: &Locals,
    ) -> Result<Val, CapyError> {
        let steps: Vec<PathStep> =
            path.iter().map(|name| PathStep::field(name.clone())).collect();
        self.resolve_path_steps(&steps, caps, locals)
    }

    /// Port of `resolvePathSteps` — walks locals → captures → context.
    fn resolve_path_steps(
        &self,
        steps: &[PathStep],
        caps: &Captures,
        locals: &Locals,
    ) -> Result<Val, CapyError> {
        if steps.is_empty() {
            return Err(CapyError::msg("empty path"));
        }
        let root = &steps[0].field;
        // A capture may need evaluating, which produces an owned value, so this
        // walk holds a Cow-style owned/borrowed split rather than a plain &Val.
        // Holds the evaluated value when the root resolves to a capture, so the
        // borrow below has something to point at.
        #[allow(unused_assignments)]
        let mut owned_from_capture: Option<Val> = None;
        let mut cur: &Val = if let Some(v) = locals.get(root) {
            v
        } else if let Some(v) = caps.get(root) {
            // Captures resolve to evaluated values when the inner DSL needs
            // them. String literals become strings (without source quotes),
            // numbers become ints/floats. Unresolved bare identifiers become
            // their literal name as a string (the transpile-mode convention).
            // Templates, in contrast, see the raw source text.
            owned_from_capture = Some(if v.is_expr {
                match &v.expr {
                    Some(e) => self.eval_expr_fallback(e, caps, locals)?,
                    None => Val::Null,
                }
            } else {
                Val::Str(v.text.clone())
            });
            owned_from_capture.as_ref().unwrap()
        } else if root == "context" {
            &self.context
        } else {
            return Err(CapyError::msg(format!("undefined {}", gofmt::quote(root))));
        };
        for step in &steps[1..] {
            let key = if step.is_index {
                match &step.index {
                    Some(e) => self.eval(e, caps, locals)?,
                    None => Val::Null,
                }
            } else {
                Val::Str(step.field.clone())
            };
            match descend_read_ref(cur, &key) {
                None => {
                    return Err(CapyError::msg(format!(
                        "cannot access {} on non-map",
                        key.format_v()
                    )))
                }
                Some(v) => cur = v,
            }
        }
        Ok(cur.clone())
    }

    /// Port of `resolveRender`.
    ///
    /// Looks up a step-based path in render scope: locals → context.
    fn resolve_render(&self, steps: &[PathStep], locals: &Locals) -> Result<Val, CapyError> {
        if steps.is_empty() {
            return Err(CapyError::msg("empty path"));
        }
        let root = &steps[0].field;
        let mut cur: &Val = if let Some(v) = locals.get(root) {
            v
        } else if root == "context" {
            &self.context
        } else {
            return Err(CapyError::msg(format!("undefined {}", gofmt::quote(root))));
        };
        for step in &steps[1..] {
            let key = if step.is_index {
                match &step.index {
                    Some(e) => self.eval_render(e, locals)?,
                    None => Val::Null,
                }
            } else {
                Val::Str(step.field.clone())
            };
            match descend_read_ref(cur, &key) {
                None => {
                    return Err(CapyError::msg(format!(
                        "cannot access {} on non-map ({})",
                        key.format_v(),
                        cur.go_type_name()
                    )))
                }
                Some(v) => cur = v,
            }
        }
        Ok(cur.clone())
    }
}

/// A path step with its index expression already evaluated.
enum ResolvedStep {
    Field(String),
    Index(Val),
}

/// Port of `descend` (the write-side walker), as a mutable borrow.
fn descend_mut<'a>(parent: &'a mut Val, step: &ResolvedStep) -> Result<&'a mut Val, CapyError> {
    match step {
        ResolvedStep::Index(idx) => {
            // Go's write-side `descend` requires a strict int64 for list
            // indices (unlike the read side's `asListIndex` coercions).
            match parent {
                Val::Obj(m) => {
                    let key = idx.to_go_string();
                    Ok(m.entry(key).or_insert(Val::Null))
                }
                Val::List(list) => {
                    let i = match idx {
                        Val::Int(i) => *i,
                        _ => return Err(CapyError::msg("list index must be int")),
                    };
                    let n = list.len() as i64;
                    // Negative indices count from the end: -1 → last element.
                    let i = if i < 0 { i + n } else { i };
                    if i < 0 || i >= n {
                        return Err(CapyError::msg(format!(
                            "list index {} out of range (len={})",
                            i, n
                        )));
                    }
                    Ok(&mut list[i as usize])
                }
                other => Err(CapyError::msg(format!(
                    "cannot index into {}",
                    other.go_type_name()
                ))),
            }
        }
        ResolvedStep::Field(name) => match parent {
            Val::Obj(m) => Ok(m.entry(name.clone()).or_insert(Val::Null)),
            other => Err(CapyError::msg(format!(
                "cannot descend {} on {}",
                gofmt::quote(name),
                other.go_type_name()
            ))),
        },
    }
}

/// Port of `applyOp`.
fn apply_op(
    parent: &mut Val,
    step: &ResolvedStep,
    value: Val,
    op: &str,
) -> Result<(), CapyError> {
    // List element target: `set context.buf[i] value` overwrites element i in
    // place. `append`/`prepend` target a *nested* list at element i. Negative
    // indices count from the end, matching the read side.
    if let ResolvedStep::Index(idx_val) = step {
        if let Val::List(list) = parent {
            let i = match idx_val {
                Val::Int(i) => *i,
                other => {
                    return Err(CapyError::msg(format!(
                        "{}: list index must be int, got {}",
                        op,
                        other.go_type_name()
                    )))
                }
            };
            let n = list.len() as i64;
            let i = if i < 0 { i + n } else { i };
            if i < 0 || i >= n {
                return Err(CapyError::msg(format!(
                    "{}: list index {} out of range (len={})",
                    op, i, n
                )));
            }
            let slot = &mut list[i as usize];
            match op {
                "set" => *slot = value,
                "append" => {
                    // Go: `inner, _ := list[i].([]any)` — a non-list slot
                    // yields nil, so the result is a fresh one-element list.
                    let mut inner = match slot {
                        Val::List(v) => v.clone(),
                        _ => Vec::new(),
                    };
                    inner.push(value);
                    *slot = Val::List(inner);
                }
                "prepend" => {
                    let inner = match slot {
                        Val::List(v) => v.clone(),
                        _ => Vec::new(),
                    };
                    let mut out = vec![value];
                    out.extend(inner);
                    *slot = Val::List(out);
                }
                "delete" => {
                    return Err(CapyError::msg(format!(
                        "{}: cannot delete a list element by index (set it instead)",
                        op
                    )))
                }
                _ => {
                    return Err(CapyError::msg(format!(
                        "{}: unsupported op on list element",
                        op
                    )))
                }
            }
            return Ok(());
        }
    }
    let key = match step {
        ResolvedStep::Index(idx) => idx.to_go_string(),
        ResolvedStep::Field(f) => f.clone(),
    };
    let m = match parent {
        Val::Obj(m) => m,
        _ => return Err(CapyError::msg(format!("{}: target parent is not a map", op))),
    };
    match op {
        "set" => {
            m.insert(key, value);
        }
        "delete" => {
            m.remove(&key);
        }
        "append" => {
            let mut list = match m.get(&key) {
                Some(Val::List(v)) => v.clone(),
                _ => Vec::new(),
            };
            list.push(value);
            m.insert(key, Val::List(list));
        }
        "prepend" => {
            let list = match m.get(&key) {
                Some(Val::List(v)) => v.clone(),
                _ => Vec::new(),
            };
            let mut out = vec![value];
            out.extend(list);
            m.insert(key, Val::List(out));
        }
        "merge" => {
            let mut existing = match m.get(&key) {
                Some(Val::Obj(v)) => v.clone(),
                _ => BTreeMap::new(),
            };
            if let Val::Obj(src) = &value {
                for (k, v) in src {
                    existing.insert(k.clone(), v.clone());
                }
            }
            m.insert(key, Val::Obj(existing));
        }
        _ => return Err(CapyError::msg(format!("unknown op {}", gofmt::quote(op)))),
    }
    Ok(())
}

/// A borrowable NULL so [`descend_read_ref`] can hand back a reference for a
/// missing key without allocating.
static NULL_VAL: Val = Val::Null;

/// Borrowing twin of [`descend_read`]: identical semantics, but returns a
/// reference so a path walk doesn't deep-clone every container it passes
/// through. Go gets this for free because maps and slices are reference types.
fn descend_read_ref<'a>(parent: &'a Val, key: &Val) -> Option<&'a Val> {
    match parent {
        Val::Obj(m) => Some(m.get(&key.to_go_string()).unwrap_or(&NULL_VAL)),
        Val::List(p) => {
            let i = match as_list_index(key) {
                None => return Some(&NULL_VAL),
                Some(i) => i,
            };
            let n = p.len() as i64;
            let i = if i < 0 { i + n } else { i };
            if i < 0 || i >= n {
                return Some(&NULL_VAL);
            }
            Some(&p[i as usize])
        }
        Val::Null => Some(&NULL_VAL),
        _ => None,
    }
}

/// Port of `descendRead`, owning variant.
///
/// Superseded by [`descend_read_ref`] at every call site (borrowing avoids
/// deep-cloning containers mid-walk); kept because it is the literal shape of
/// the Go function this file ports.
#[allow(dead_code)]
fn descend_read(parent: &Val, key: &Val) -> Option<Val> {
    match parent {
        Val::Obj(m) => Some(m.get(&key.to_go_string()).cloned().unwrap_or(Val::Null)),
        Val::List(p) => {
            // Accept int / int64 / whole float64 / numeric string.
            let i = match as_list_index(key) {
                None => return Some(Val::Null),
                Some(i) => i,
            };
            let n = p.len() as i64;
            let i = if i < 0 { i + n } else { i };
            if i < 0 || i >= n {
                return Some(Val::Null);
            }
            Some(p[i as usize].clone())
        }
        Val::Null => Some(Val::Null),
        // FIDELITY: Go's descendRead has arms for map / []any / nil only, so
        // `[]string` is NOT indexable and falls to the (nil, false) default.
        _ => None,
    }
}

/// Port of `asListIndex`.
///
/// Accepts int, int64, whole-number float64, and a numeric string — the last
/// because a captured int read in TEMPLATE scope arrives as its source text.
fn as_list_index(v: &Val) -> Option<i64> {
    match v {
        Val::Int(n) => Some(*n),
        Val::Float(n) => {
            if *n == (*n as i64) as f64 {
                Some(*n as i64)
            } else {
                None
            }
        }
        Val::Str(n) => n.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// Port of `unescapeStringLitInner`.
///
/// One pass of Go-style escape decoding over the unquoted inner of a string
/// literal: `\n`→newline, `\t`→tab, `\r`→CR, `\"`→`"`, `\\`→`\`, `\X`→X.
fn unescape_string_lit_inner(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            match b[i + 1] {
                b'n' => out.push(b'\n'),
                b't' => out.push(b'\t'),
                b'r' => out.push(b'\r'),
                b'"' => out.push(b'"'),
                b'\\' => out.push(b'\\'),
                other => out.push(other),
            }
            i += 2;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Port of `splitInterpPipeRuntime` — splits on top-level `|`.
fn split_interp_pipe_runtime(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut depth = 0i64;
    let mut brack = 0i64;
    let mut in_str = false;
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        if in_str {
            cur.push(c);
            if c == b'\\' && i + 1 < b.len() {
                cur.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => {
                in_str = true;
                cur.push(c);
            }
            b'(' => {
                depth += 1;
                cur.push(c);
            }
            b')' => {
                depth -= 1;
                cur.push(c);
            }
            b'[' => {
                brack += 1;
                cur.push(c);
            }
            b']' => {
                if brack > 0 {
                    brack -= 1;
                }
                cur.push(c);
            }
            b'|' => {
                if depth == 0 && brack == 0 {
                    out.push(String::from_utf8_lossy(&cur).into_owned());
                    cur.clear();
                    i += 1;
                    continue;
                }
                cur.push(c);
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(String::from_utf8_lossy(&cur).into_owned());
    }
    out
}

/// Port of `tokeniseInterpRuntime`.
///
/// Tokenises a single pipe stage. Parens stay together as one token; quoted
/// strings stay together; whitespace separates atoms. A postfix `[index]` stays
/// attached to its path atom.
fn tokenise_interp_runtime(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut cur: Vec<u8> = Vec::new();
    let mut in_str = false;
    let mut depth = 0i64;
    let mut brack = 0i64;
    let mut i = 0usize;
    let flush = |out: &mut Vec<String>, cur: &mut Vec<u8>| {
        out.push(String::from_utf8_lossy(cur).into_owned());
        cur.clear();
    };
    while i < b.len() {
        let c = b[i];
        if in_str {
            cur.push(c);
            if c == b'\\' && i + 1 < b.len() {
                cur.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
                if depth == 0 && brack == 0 {
                    flush(&mut out, &mut cur);
                }
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            in_str = true;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == b'(' {
            depth += 1;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == b')' {
            depth -= 1;
            cur.push(c);
            if depth == 0 && brack == 0 {
                flush(&mut out, &mut cur);
            }
            i += 1;
            continue;
        }
        if c == b'[' {
            brack += 1;
            cur.push(c);
            i += 1;
            continue;
        }
        if c == b']' {
            if brack > 0 {
                brack -= 1;
            }
            cur.push(c);
            i += 1;
            continue;
        }
        if depth > 0 || brack > 0 {
            cur.push(c);
            i += 1;
            continue;
        }
        if c == b' ' || c == b'\t' {
            if !cur.is_empty() {
                flush(&mut out, &mut cur);
            }
            i += 1;
            continue;
        }
        cur.push(c);
        i += 1;
    }
    if !cur.is_empty() {
        out.push(String::from_utf8_lossy(&cur).into_owned());
    }
    out
}

/// Port of `cmp`.
pub fn cmp(op: &str, l: &Val, r: &Val) -> Result<bool, CapyError> {
    let eq = || -> bool {
        if let (Val::Str(a), Val::Str(b)) = (l, r) {
            return a == b;
        }
        if let (Some(a), Some(b)) = (num_any(l), num_any(r)) {
            return a == b;
        }
        if let (Val::Bool(a), Val::Bool(b)) = (l, r) {
            return a == b;
        }
        if matches!(l, Val::Null) && matches!(r, Val::Null) {
            return true;
        }
        false
    };
    match op {
        "==" => return Ok(eq()),
        "!=" => return Ok(!eq()),
        _ => {}
    }
    let c: i64 = if let (Some(a), Some(b)) = (num_any(l), num_any(r)) {
        if a < b {
            -1
        } else if a > b {
            1
        } else {
            0
        }
    } else if let (Val::Str(a), Val::Str(b)) = (l, r) {
        if a < b {
            -1
        } else if a > b {
            1
        } else {
            0
        }
    } else {
        return Err(CapyError::msg("incomparable values"));
    };
    match op {
        "<" => Ok(c < 0),
        ">" => Ok(c > 0),
        "<=" => Ok(c <= 0),
        ">=" => Ok(c >= 0),
        _ => Err(CapyError::msg(format!("unknown comparator {}", gofmt::quote(op)))),
    }
}

/// Port of `numAny`.
fn num_any(v: &Val) -> Option<f64> {
    match v {
        Val::Int(x) => Some(*x as f64),
        Val::Float(x) => Some(*x),
        _ => None,
    }
}

/// The `len` builtin templates expose (not part of the helper table).
fn builtin_len(v: &Val) -> i64 {
    match v {
        Val::List(x) => x.len() as i64,
        Val::Obj(x) => x.len() as i64,
        // Go measures a string's BYTE length.
        Val::Str(x) => x.len() as i64,
        _ => 0,
    }
}

/// Go's `%T` for the expression node kinds that reach the render error path.
fn expr_kind_name(x: &Expr) -> &'static str {
    match x {
        Expr::Number(_) => "domain.NumberLit",
        Expr::Str(_) => "domain.StringLit",
        Expr::Bool(_) => "domain.BoolLit",
        Expr::Null => "domain.NullLit",
        Expr::Var(_) => "domain.VarRef",
        Expr::Call(_) => "domain.CallExpr",
        Expr::Compare(_) => "domain.CompareExpr",
        Expr::Not(_) => "domain.NotExpr",
        Expr::List(_) => "domain.ListLit",
        Expr::Obj(_) => "domain.ObjLit",
    }
}

/// Port of `interpolateGeneric` — `${path}` substitution with a supplied
/// resolver. Only dotted paths; used by the non-render `eval` path.
fn interpolate_generic<F>(s: &str, mut resolve: F) -> Result<String, CapyError>
where
    F: FnMut(&[String]) -> Result<Val, CapyError>,
{
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0usize;
    while i < b.len() {
        if i + 1 < b.len() && b[i] == b'$' && b[i + 1] == b'{' {
            let mut j = i + 2;
            let mut depth = 1i64;
            while j < b.len() && depth > 0 {
                if b[j] == b'{' {
                    depth += 1;
                } else if b[j] == b'}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            if j >= b.len() {
                return Err(CapyError::msg("unterminated ${...}"));
            }
            let expr = &s[i + 2..j];
            let path: Vec<String> =
                expr.trim().split('.').map(|x| x.to_string()).collect();
            let v = resolve(&path)?;
            out.extend_from_slice(v.to_go_string().as_bytes());
            i = j + 1;
        } else if b[i] == b'\\' && i + 1 < b.len() {
            // One layer of `\X` → X. Library authors use this to embed literal
            // quotes and dollar signs. To pass a target-language escape through
            // to the output, double the backslash in source: `\\n` → `\n`.
            out.push(b[i + 1]);
            i += 2;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// Port of `toInt` in `inner_evaluator.go` — coerces a runtime value to an int.
/// Accepts int / int64 / float64 (truncated) / numeric strings.
fn to_int_go(v: &Val) -> Option<i64> {
    match v {
        Val::Int(x) => Some(*x),
        Val::Float(x) => Some(*x as i64),
        Val::Str(x) => x.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// Renders a regex compile failure roughly the way Go's `regexp` does. Only
/// reachable via a library-supplied bad pattern; no golden pins the text.
fn go_regex_error(pattern: &str, e: &regex::Error) -> String {
    format!("error parsing regexp: `{}`: {}", pattern, e)
}


#[cfg(test)]
mod tests {
    //! Ports of `inner_evaluator_indexread_test.go` and
    //! `inner_evaluator_listwrite_test.go`. These live in-crate because they
    //! exercise private helpers, mirroring Go's in-package test placement.

    use super::*;
    use crate::domain::ast::NumberLit;

    fn ev(ctx: Vec<(&str, Val)>) -> InnerEvaluator {
        let m: BTreeMap<String, Val> =
            ctx.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        InnerEvaluator::with_noop_host(m)
    }

    fn list(items: &[&str]) -> Val {
        Val::List(items.iter().map(|s| Val::str(*s)).collect())
    }

    fn obj(kvs: &[(&str, &str)]) -> Val {
        Val::Obj(kvs.iter().map(|(k, v)| (k.to_string(), Val::str(*v))).collect())
    }

    /// `[<int>]` index step with a literal integer.
    fn idx_step(i: i64) -> PathStep {
        PathStep::index(Expr::Number(NumberLit { is_int: true, i, f: 0.0 }))
    }

    fn var_idx_step(name: &str) -> PathStep {
        PathStep::index(Expr::Var(vec![PathStep::field(name)]))
    }

    fn ctx_of(e: &InnerEvaluator, key: &str) -> Val {
        e.ctx_map().get(key).cloned().unwrap()
    }

    /// Port of `TestDescendRead` — map key, list index (positive / negative /
    /// out-of-range), wrong-type index, and a scalar parent.
    #[test]
    fn descend_read_covers_both_containers() {
        let m = obj(&[("a", "x"), ("1", "one")]);
        assert_eq!(descend_read(&m, &Val::str("a")), Some(Val::str("x")));
        // Map key is the stringified index — Int(1) hits the "1" key.
        assert_eq!(descend_read(&m, &Val::Int(1)), Some(Val::str("one")));
        // Missing map key: container, but null value.
        assert_eq!(descend_read(&m, &Val::str("nope")), Some(Val::Null));

        let l = list(&["a", "b", "c"]);
        assert_eq!(descend_read(&l, &Val::Int(1)), Some(Val::str("b")));
        // Negative index counts from the end.
        assert_eq!(descend_read(&l, &Val::Int(-1)), Some(Val::str("c")));
        assert_eq!(descend_read(&l, &Val::Int(0)), Some(Val::str("a")));
        // Out of range: container, null value (tolerant, no panic).
        assert_eq!(descend_read(&l, &Val::Int(9)), Some(Val::Null));
        // Wrong-type index into a list: miss, but still a container.
        assert_eq!(descend_read(&l, &Val::str("x")), Some(Val::Null));
        // Scalar parent: not a container.
        assert_eq!(descend_read(&Val::str("scalar"), &Val::Int(0)), None);
    }

    /// The borrowing twin must agree with the owning one everywhere.
    #[test]
    fn descend_read_ref_agrees_with_owning_variant() {
        let containers =
            [obj(&[("a", "x")]), list(&["a", "b"]), Val::Null, Val::str("scalar"), Val::Int(3)];
        let keys = [Val::str("a"), Val::str("nope"), Val::Int(0), Val::Int(-1), Val::Int(9)];
        for c in &containers {
            for k in &keys {
                let owned = descend_read(c, k);
                let borrowed = descend_read_ref(c, k).cloned();
                assert_eq!(owned, borrowed, "mismatch for container {c:?} key {k:?}");
            }
        }
    }

    /// Port of `TestWritePathListIndex` — in-place overwrite of a list element,
    /// negative indices, and out-of-range errors.
    #[test]
    fn write_path_list_index() {
        let mut e = ev(vec![("buf", list(&["a", "b", "c"]))]);
        let caps: Captures = BTreeMap::new();
        let mut locals: Locals = BTreeMap::new();

        let p = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("buf"), idx_step(1)],
        };
        e.write_path(&p, Val::str("B"), &caps, &mut locals, "set").expect("set buf[1]");
        assert_eq!(ctx_of(&e, "buf"), list(&["a", "B", "c"]), "neighbours must be untouched");

        // Negative index: -1 is the last element.
        let p_neg = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("buf"), idx_step(-1)],
        };
        e.write_path(&p_neg, Val::str("Z"), &caps, &mut locals, "set").expect("set buf[-1]");
        assert_eq!(ctx_of(&e, "buf"), list(&["a", "B", "Z"]));

        // Out of range errors rather than panicking or growing the list.
        let p_oor = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("buf"), idx_step(9)],
        };
        e.write_path(&p_oor, Val::str("x"), &caps, &mut locals, "set")
            .expect_err("set buf[9] should error (len=3)");
    }

    /// Port of `TestWritePathListIndexAppend` — append/prepend against a nested
    /// list stored at an element index.
    #[test]
    fn write_path_list_index_append() {
        let mut e = ev(vec![("rows", Val::List(vec![list(&["x"]), list(&["y"])]))]);
        let caps: Captures = BTreeMap::new();
        let mut locals: Locals = BTreeMap::new();

        let p_app = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("rows"), idx_step(0)],
        };
        e.write_path(&p_app, Val::str("x2"), &caps, &mut locals, "append")
            .expect("append rows[0]");
        let p_pre = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("rows"), idx_step(1)],
        };
        e.write_path(&p_pre, Val::str("y0"), &caps, &mut locals, "prepend")
            .expect("prepend rows[1]");

        assert_eq!(
            ctx_of(&e, "rows"),
            Val::List(vec![list(&["x", "x2"]), list(&["y0", "y"])])
        );
    }

    /// Port of `TestResolvePathStepsIndex` — value-position index reads through
    /// the inner-DSL resolver, including a computed index and a write round-trip.
    #[test]
    fn resolve_path_steps_index() {
        let mut e = ev(vec![
            ("buf", list(&["a", "b", "c"])),
            ("known", obj(&[("x", "1"), ("y", "2")])),
        ]);
        let caps: Captures = BTreeMap::new();
        let mut locals: Locals = BTreeMap::new();
        locals.insert("i".to_string(), Val::Int(2));
        locals.insert("name".to_string(), Val::str("y"));

        // context.buf[i] with local i=2 → "c".
        let steps =
            vec![PathStep::field("context"), PathStep::field("buf"), var_idx_step("i")];
        assert_eq!(e.resolve_path_steps(&steps, &caps, &locals).unwrap(), Val::str("c"));

        // context.known[name] with local name="y" → "2".
        let steps =
            vec![PathStep::field("context"), PathStep::field("known"), var_idx_step("name")];
        assert_eq!(e.resolve_path_steps(&steps, &caps, &locals).unwrap(), Val::str("2"));

        // Negative literal index: context.buf[-1] → "c".
        let steps =
            vec![PathStep::field("context"), PathStep::field("buf"), idx_step(-1)];
        assert_eq!(e.resolve_path_steps(&steps, &caps, &locals).unwrap(), Val::str("c"));

        // Round-trip: write buf[0] then read it back by index.
        let wp = Path {
            root: "context".to_string(),
            steps: vec![PathStep::field("buf"), idx_step(0)],
        };
        e.write_path(&wp, Val::str("A"), &caps, &mut locals, "set").expect("write buf[0]");
        let steps = vec![PathStep::field("context"), PathStep::field("buf"), idx_step(0)];
        assert_eq!(e.resolve_path_steps(&steps, &caps, &locals).unwrap(), Val::str("A"));
    }

    /// Port of `TestEvalInterpPathIndex` — template-position `${…[…]}` reads.
    #[test]
    fn eval_interp_path_index() {
        let e = ev(vec![
            ("buf", list(&["a", "b", "c"])),
            ("known", obj(&[("k", "kv")])),
            (
                "grid",
                Val::List(vec![list(&["r0c0", "r0c1"]), list(&["r1c0", "r1c1"])]),
            ),
        ]);
        let mut locals: Locals = BTreeMap::new();
        locals.insert("i".to_string(), Val::Int(1));
        locals.insert("j".to_string(), Val::Int(0));
        locals.insert("key".to_string(), Val::str("k"));

        let cases: &[(&str, &str)] = &[
            ("context.buf[i]", "b"),         // list by local index
            ("context.buf[-1]", "c"),        // negative literal
            ("context.known[key]", "kv"),    // map by captured key
            ("context.grid[i][j]", "r1c0"),  // nested index
            ("context.buf[(sub i 1)]", "a"), // computed index expression
            ("context.buf[99]", ""),         // out of range → tolerant empty
        ];
        for (atom, want) in cases {
            let got = e.eval_interp_atom(atom, &locals).expect(atom);
            assert_eq!(&got.to_go_string(), want, "{atom}");
        }
    }
}
