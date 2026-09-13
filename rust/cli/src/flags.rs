//! A tiny stand-in for Go's `flag` package, matching the behaviours the CLI
//! depends on: `--name value`, `--name=value`, boolean presence flags, and
//! "parsing stops at the first positional" (which is why several Go commands
//! call `reorderFlagsFirst` first).

use std::collections::BTreeMap;

pub struct Flags {
    pub bools: BTreeMap<String, bool>,
    pub strings: BTreeMap<String, String>,
    pub positionals: Vec<String>,
}

/// Declares the flag shape for one subcommand.
pub struct Spec {
    /// Flag names (without dashes) that take no value.
    pub bools: &'static [&'static str],
    /// Flag names (without dashes) that take a value.
    pub strings: &'static [&'static str],
}

impl Flags {
    pub fn bool(&self, name: &str) -> bool {
        self.bools.get(name).copied().unwrap_or(false)
    }
    pub fn str(&self, name: &str) -> &str {
        self.strings.get(name).map(|s| s.as_str()).unwrap_or("")
    }
}

/// Parses `args` against `spec`. Unlike Go's `flag`, this keeps scanning past
/// positionals — every Go caller that cares already normalises with
/// [`reorder_flags_first`], and the ones that don't only ever pass flags first.
pub fn parse(spec: &Spec, args: &[String]) -> Result<Flags, String> {
    let mut out = Flags {
        bools: spec.bools.iter().map(|b| (b.to_string(), false)).collect(),
        strings: spec.strings.iter().map(|s| (s.to_string(), String::new())).collect(),
        positionals: Vec::new(),
    };
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        if a == "--" {
            out.positionals.extend_from_slice(&args[i + 1..]);
            break;
        }
        if let Some(rest) = a.strip_prefix('-') {
            let rest = rest.strip_prefix('-').unwrap_or(rest);
            let (name, inline) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (rest, None),
            };
            if spec.bools.contains(&name) {
                out.bools.insert(name.to_string(), true);
                i += 1;
                continue;
            }
            if spec.strings.contains(&name) {
                match inline {
                    Some(v) => {
                        out.strings.insert(name.to_string(), v);
                        i += 1;
                    }
                    None => {
                        if i + 1 >= args.len() {
                            return Err(format!("flag needs an argument: -{}", name));
                        }
                        out.strings.insert(name.to_string(), args[i + 1].clone());
                        i += 2;
                    }
                }
                continue;
            }
            return Err(format!("flag provided but not defined: -{}", name));
        }
        out.positionals.push(a.clone());
        i += 1;
    }
    Ok(out)
}

/// Port of `reorderFlagsFirst` in `cmd_new.go`.
///
/// Pulls every `--flag VALUE` / `-f VALUE` / `--flag=VALUE` pair to the front so
/// Go's `flag.Parse` picks them all up before hitting any positional.
pub fn reorder_flags_first(args: &[String]) -> Vec<String> {
    let mut flags: Vec<String> = Vec::new();
    let mut pos: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        if a.len() > 1 && a.starts_with('-') {
            flags.push(a.clone());
            // If it isn't `--name=value` form and has a value arg, take it too.
            if !a.contains('=')
                && i + 1 < args.len()
                && !args[i + 1].starts_with('-')
            {
                flags.push(args[i + 1].clone());
                i += 1;
            }
            i += 1;
            continue;
        }
        pos.push(a.clone());
        i += 1;
    }
    flags.extend(pos);
    flags
}
