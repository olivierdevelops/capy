//! Port of `orchestrator/features/make_evaluator.go`.
//!
//! The transpiler-driver outer evaluator. It walks the parsed program top-down.
//! For each `FuncCall`:
//!
//! 1. Validate captured args against their declared types.
//! 2. If the function opens a block, recursively render the body block first so
//!    the rendered string is available as `body` in the function's write-style
//!    template.
//! 3. Render the function's `template_ast` and append to the parent block's body.
//! 4. Run the function's run snippet (mutates context only).
//! 5. After the body completes, render+run the closer `FuncCall` the same way.
//!
//! Once the program block is fully rendered, the orchestrator renders
//! `file_template_ast` with `body`=(top-level body) and `context`=(final).

use super::inner_evaluator::{InnerEvaluator, Locals};
use crate::domain::ast::{Block, Expr, FuncCall};
use crate::domain::errors::{suggest_closest, CapyError};
use crate::domain::host::{Host, NoOpHost};
use crate::domain::library::{Library, TypeDef};
use crate::domain::val::Val;
use crate::gofmt;
use regex::Regex;
use std::collections::BTreeMap;
use std::rc::Rc;

/// Port of the `runMulti` closure returned by `MakeEvaluatorWithHost`.
pub struct OuterEval<'a> {
    lib: &'a Library,
    pub inner: InnerEvaluator,
    /// The AST depth at which the next `render_func_call` will run.
    depth: usize,
    /// Function table as shared handles, so resolving a call's definition is a
    /// refcount bump rather than a deep clone of the whole `FuncDef`.
    fns: BTreeMap<String, Rc<crate::domain::library::FuncDef>>,
}

/// Port of `MakeEvaluator` — uses [`NoOpHost`].
pub fn run(program: &Block, lib: &Library) -> Result<String, CapyError> {
    let (out, _) = run_multi(program, lib, Rc::new(NoOpHost))?;
    Ok(out)
}

/// `MakeEvaluator().RunMulti` — the multi-file entry point with the sandboxed
/// host, matching [`crate::features::EvaluateMultiFn`].
pub fn run_noop_host_multi(
    program: &Block,
    lib: &Library,
) -> Result<(String, BTreeMap<String, String>), CapyError> {
    run_multi(program, lib, Rc::new(NoOpHost))
}

/// Port of `MakeEvaluatorWithHost(...).RunMulti`.
pub fn run_multi(
    program: &Block,
    lib: &Library,
    host: Rc<dyn Host>,
) -> Result<(String, BTreeMap<String, String>), CapyError> {
    let ctx = deep_copy_map(&lib.context);
    let fns: BTreeMap<String, Rc<crate::domain::library::FuncDef>> =
        lib.functions.iter().map(|(k, v)| (k.clone(), Rc::new(v.clone()))).collect();
    let mut ev =
        OuterEval { lib, inner: InnerEvaluator::new(ctx, host), depth: 0, fns };
    let body = ev.render_block(program)?;

    // Render the top-level file_template via the AST walker. Libraries with no
    // file_template emit `body` verbatim.
    let out = match &lib.file_template_ast {
        Some(ft) => {
            let mut locals: Locals = BTreeMap::new();
            locals.insert("body".to_string(), Val::Str(body.clone()));
            ev.inner
                .render_ast(ft, &locals)
                .map_err(|e| CapyError::msg(format!("file_template: {}", e)))?
        }
        None => body.clone(),
    };

    let mut files: BTreeMap<String, String> = BTreeMap::new();
    for (path, ast) in &lib.files_ast {
        let mut locals: Locals = BTreeMap::new();
        locals.insert("body".to_string(), Val::Str(body.clone()));
        let rendered_path = ev
            .inner
            .render_path(path, &locals)
            .map_err(|e| CapyError::msg(format!("file path {}: {}", gofmt::quote(path), e)))?;
        let rendered = ev.inner.render_ast(ast, &locals).map_err(|e| {
            CapyError::msg(format!("file {}: {}", gofmt::quote(&rendered_path), e))
        })?;
        files.insert(rendered_path, rendered);
    }
    Ok((out, files))
}

impl<'a> OuterEval<'a> {
    fn render_block(&mut self, b: &Block) -> Result<String, CapyError> {
        let d = self.depth;
        self.render_block_at(b, d)
    }

    fn render_block_at(&mut self, b: &Block, depth: usize) -> Result<String, CapyError> {
        let mut out = String::new();
        for c in &b.stmts {
            out.push_str(&self.render_func_call_at(c, depth)?);
        }
        Ok(out)
    }

    /// Port of `renderFuncCallAt`.
    ///
    /// Renders one `FuncCall`, tracking the AST depth so templates can branch on
    /// whether they're rendered at the top level (depth == 0) or inside another
    /// block's body (depth > 0). The depth is exposed to the inner DSL as the
    /// integer local `depth` plus the boolean convenience `top_level`.
    fn render_func_call_at(&mut self, c: &FuncCall, depth: usize) -> Result<String, CapyError> {
        self.validate_args(c)?;
        let mut body_output = String::new();
        if let Some(body) = &c.body {
            if body.is_verbatim {
                // Verbatim blocks bypass nested rendering — their raw text IS
                // the body output.
                body_output = body.verbatim_text.clone();
            } else {
                body_output = self.render_block_at(body, depth + 1)?;
            }
        }
        // Multi-section blocks: render each parsed section sub-body independently
        // so the template can place them via a local named after the section
        // keyword (${rescue}, ${finally}).
        let mut section_outputs: BTreeMap<String, String> = BTreeMap::new();
        for (name, blk) in &c.sections {
            let s = self.render_block_at(blk, depth + 1)?;
            section_outputs.insert(name.clone(), s);
        }
        let mut out = self.render_template_at(c, &body_output, &section_outputs, depth)?;

        let fd = self.func_def(c)?;
        if let Some(run_ast) = fd.run_ast.clone() {
            // Expose the rendered inner-block output to the run pass as `body`
            // so state-mutation statements can stash the rendered text into
            // context. Do NOT shadow a user-defined capture also named `body` —
            // captures take precedence.
            let mut run_locals: Locals = BTreeMap::new();
            if !c.captures.contains_key("body") {
                run_locals.insert("body".to_string(), Val::Str(body_output.clone()));
            }
            self.inner
                .exec_with_locals(&run_ast, &c.captures, &mut run_locals)
                .map_err(|e| {
                    CapyError::msg(format!("function {} run: {}", gofmt::quote(&c.func), e))
                })?;
        }
        if let Some(closer) = &c.closer {
            out.push_str(&self.render_func_call_at(closer, depth)?);
        }
        Ok(out)
    }

    /// Resolves a `FuncCall`'s definition. Go stores a `*FuncDef` on the call;
    /// the port stores the name and looks it up here.
    fn func_def(
        &self,
        c: &FuncCall,
    ) -> Result<Rc<crate::domain::library::FuncDef>, CapyError> {
        match self.fns.get(&c.func) {
            Some(f) => Ok(f.clone()),
            None => Err(CapyError::msg(format!(
                "internal: call references unknown function {}",
                gofmt::quote(&c.func)
            ))),
        }
    }

    /// Port of `renderTemplateAt`.
    fn render_template_at(
        &mut self,
        c: &FuncCall,
        body: &str,
        sections: &BTreeMap<String, String>,
        depth: usize,
    ) -> Result<String, CapyError> {
        let fd = self.func_def(c)?;
        let template_ast = match &fd.template_ast {
            None => return Ok(String::new()),
            Some(t) => t.clone(),
        };
        let mut locals: Locals = BTreeMap::new();
        locals.insert("body".to_string(), Val::Str(body.to_string()));
        locals.insert("depth".to_string(), Val::Int(depth as i64));
        locals.insert("top_level".to_string(), Val::Bool(depth == 0));
        locals.insert("line".to_string(), Val::Int(c.line as i64));
        locals.insert("col".to_string(), Val::Int(c.col as i64));
        // Seed every declared section local to "" so a template referencing
        // `${rescue}` renders empty (not undefined) when that section is omitted
        // at the call site, then overlay the rendered sub-bodies.
        if let Some(block) = &fd.block {
            for name in &block.sections {
                locals.insert(name.clone(), Val::Str(String::new()));
            }
        }
        for (k, v) in sections {
            locals.insert(k.clone(), Val::Str(v.clone()));
        }
        for (k, v) in &c.captures {
            // Function-typed captures (named nonterminals): render each matched
            // sub-FuncCall and concatenate. The result is the target text the
            // sub-construct produces.
            if !v.sub.is_empty() {
                // An optional `join "X"` on the capture inserts X between the
                // rendered sub-results (default: no separator).
                let mut join = String::new();
                for a in &fd.args {
                    if a.kind == "capture" && &a.name == k {
                        join = a.join.clone();
                        break;
                    }
                }
                let mut sb = String::new();
                for (i, sub) in v.sub.iter().enumerate() {
                    let s = self.render_func_call_at(sub, depth)?;
                    if i > 0 {
                        sb.push_str(&join);
                    }
                    sb.push_str(&s);
                }
                locals.insert(k.clone(), Val::Str(sb));
                continue;
            }
            // Templates always see the source-text form of a capture. This is
            // the transpiler model: what the user wrote appears in the target
            // unless a helper transforms it.
            locals.insert(k.clone(), Val::Str(v.text.clone()));
        }
        self.inner.render_ast(&template_ast, &locals)
    }

    /// Port of `validateArgs`.
    ///
    /// Walks the function's args and validates each capture against its declared
    /// type. Type checks are transpile-aware: a bare identifier (a VarRef) is
    /// accepted by every primitive type because at the target language's runtime
    /// it could refer to a value of that type. Library-defined types apply
    /// pattern/options to the source text.
    fn validate_args(&self, c: &FuncCall) -> Result<(), CapyError> {
        let fd = self.func_def(c)?;
        for a in &fd.args {
            if a.kind != "capture" {
                continue;
            }
            let cap = match c.captures.get(&a.name) {
                None => {
                    return Err(CapyError::msg(format!(
                        "function {}: missing capture {}",
                        gofmt::quote(&fd.name),
                        gofmt::quote(&a.name)
                    )))
                }
                Some(cv) => cv,
            };
            // Function-typed captures are structural matches, not flat tokens.
            if !cap.sub.is_empty() {
                continue;
            }
            if self.lib.functions.contains_key(&a.type_) {
                continue;
            }
            // A bare identifier reference is accepted by any primitive type.
            if cap.is_expr {
                if let Some(Expr::Var(_)) = &cap.expr {
                    // Library types still enforce their pattern/options.
                    if !self.lib.types.contains_key(&a.type_) {
                        continue;
                    }
                }
            }
            if let Err(e) = self.check_type(&a.type_, &cap.text) {
                // Go wraps into a *CapyError (preserving the hint) ONLY when the
                // inner error already is one; a plain inner error yields a plain
                // wrapper with the same text. The distinction is invisible in the
                // message but changes how `capy run` formats it.
                let msg = format!(
                    "function {} arg {}: {}",
                    gofmt::quote(&fd.name),
                    gofmt::quote(&a.name),
                    e.msg
                );
                if e.plain {
                    return Err(CapyError::msg(msg));
                }
                return Err(CapyError { msg, hint: e.hint, plain: false, ..Default::default() });
            }
        }
        Ok(())
    }

    /// Port of `checkType`.
    ///
    /// Validates a capture's source text against its declared type. Built-in
    /// kinds inspect the textual form; library-defined types apply
    /// pattern/options.
    fn check_type(&self, t: &str, text: &str) -> Result<(), CapyError> {
        match t {
            // Free-form token captures — the parser already enforced their shape
            // at capture time, so any captured text is valid here.
            "" | "any" | "raw" | "ident" | "tail" | "word" | "dotted_ident" => return Ok(()),
            "string" => {
                // String captures parse as StringLit and the source-text form is
                // quoted. Accept any quoted string.
                let b = text.as_bytes();
                if b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'' || b[0] == b'`') {
                    return Ok(());
                }
                return Err(CapyError::msg(format!(
                    "expected string literal, got {}",
                    gofmt::quote(text)
                )));
            }
            "int" => {
                if text.parse::<i64>().is_ok() {
                    return Ok(());
                }
                return Err(CapyError::msg(format!(
                    "expected int literal, got {}",
                    gofmt::quote(text)
                )));
            }
            "float" => {
                if text.parse::<f64>().is_ok() {
                    return Ok(());
                }
                return Err(CapyError::msg(format!(
                    "expected float literal, got {}",
                    gofmt::quote(text)
                )));
            }
            "bool" => {
                if text == "true" || text == "false" {
                    return Ok(());
                }
                return Err(CapyError::msg(format!(
                    "expected bool literal, got {}",
                    gofmt::quote(text)
                )));
            }
            _ => {}
        }
        // Library-defined type.
        let td: &TypeDef = match self.lib.types.get(t) {
            None => {
                return Err(CapyError::msg(format!("unknown type {}", gofmt::quote(t))))
            }
            Some(td) => td,
        };
        // Group types: the parser already enforced the delimiters at capture
        // time, so the captured text is valid by construction.
        if !td.group_open.is_empty() {
            return Ok(());
        }
        if !td.base.is_empty() && td.base != "any" {
            self.check_type(&td.base, text)?;
        }
        // Apply pattern against the un-quoted form of strings, otherwise the text.
        let mut probe = text.to_string();
        let b = text.as_bytes();
        if b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'' || b[0] == b'`') {
            match gofmt::unquote(text) {
                Ok(u) => probe = u,
                Err(_) => {
                    if b[0] == b'\'' || b[0] == b'`' {
                        // strconv.Unquote doesn't accept '/` — strip manually.
                        probe = text[1..text.len() - 1].to_string();
                    }
                }
            }
        }
        if !td.pattern.is_empty() {
            let rx = Regex::new(&td.pattern).map_err(|e| {
                CapyError::msg(format!(
                    "type {} has bad regex: {}",
                    gofmt::quote(&td.name),
                    e
                ))
            })?;
            if !rx.is_match(&probe) {
                return Err(CapyError::structured(format!(
                    "value {} does not match pattern for type {}",
                    gofmt::quote(&probe),
                    gofmt::quote(&td.name)
                ))
                .with_hint(format!(
                    "type {} requires the value to match regex /{}/",
                    gofmt::quote(&td.name),
                    td.pattern
                )));
            }
        }
        if !td.options.is_empty() && !td.options.contains(&probe) {
            let mut ce = CapyError::structured(format!(
                "value {} is not in options for type {}",
                gofmt::quote(&probe),
                gofmt::quote(&td.name)
            ));
            match suggest_closest(&probe, &td.options, 2) {
                Some(best) => {
                    ce.hint = format!(
                        "did you mean {}? valid options: {}",
                        gofmt::quote(&best),
                        td.options.join(", ")
                    );
                }
                None => {
                    ce.hint = format!("valid options: {}", td.options.join(", "));
                }
            }
            return Err(ce);
        }
        Ok(())
    }
}

/// Port of `deepCopyMap`.
fn deep_copy_map(m: &BTreeMap<String, Val>) -> BTreeMap<String, Val> {
    // `Val` is a value type here, so a plain clone already deep-copies the
    // nested lists and maps Go had to walk explicitly.
    m.clone()
}
