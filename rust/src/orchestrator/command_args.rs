//! Port of `orchestrator/command_args.go`.

use crate::domain::errors::CapyError;
use crate::domain::val::Val;
use crate::domain::{CommandDef, CommandFlag, Library};
use crate::gofmt;
use std::collections::BTreeMap;

/// The parsed result of [`parse_command_args`].
pub struct ParsedArgs {
    /// Keyed by declared positional name → value.
    pub pos: BTreeMap<String, Val>,
    /// Keyed by trimmed flag name → value (string for normal flags, bool for
    /// `is_bool` flags).
    pub flags: BTreeMap<String, Val>,
    /// Positional args supplied beyond what the command declared (surfaced as
    /// `context.extra` so authors can opt in to variadic shapes).
    pub extra: Vec<String>,
}

/// Port of `ParseCommandArgs`.
///
/// Walks `args` once, consuming flags by name and then positional arguments by
/// declared order. Errors when a required positional is missing or an unknown
/// flag appears.
pub fn parse_command_args(
    cmd: &CommandDef,
    args: &[String],
) -> Result<ParsedArgs, CapyError> {
    let mut pos: BTreeMap<String, Val> = BTreeMap::new();
    let mut flags: BTreeMap<String, Val> = BTreeMap::new();
    let mut extra: Vec<String> = Vec::new();

    // Seed flag defaults.
    for f in &cmd.flags {
        let key = trim_flag_name(&f.name);
        if f.is_bool {
            flags.insert(key, Val::Bool(false));
        } else {
            flags.insert(key, Val::Str(f.default.clone()));
        }
    }

    let mut pos_idx = 0usize;
    let mut i = 0usize;
    while i < args.len() {
        let a = &args[i];
        if a.starts_with('-') {
            // Flag.
            let (name, value, has_eq) = split_flag(a);
            let fd = match find_flag(cmd, name) {
                None => {
                    return Err(CapyError::msg(format!(
                        "unknown flag {} (command {})",
                        gofmt::quote(a),
                        gofmt::quote(&cmd.name)
                    )))
                }
                Some(f) => f,
            };
            if fd.is_bool {
                flags.insert(trim_flag_name(&fd.name), Val::Bool(true));
                i += 1;
                continue;
            }
            if has_eq {
                flags.insert(trim_flag_name(&fd.name), Val::Str(value.to_string()));
                i += 1;
                continue;
            }
            // `--flag VALUE` form — consume the next arg.
            if i + 1 >= args.len() {
                return Err(CapyError::msg(format!(
                    "flag {} expects a value",
                    gofmt::quote(&fd.name)
                )));
            }
            flags.insert(trim_flag_name(&fd.name), Val::Str(args[i + 1].clone()));
            i += 2;
            continue;
        }
        // Positional.
        if pos_idx < cmd.args.len() {
            pos.insert(cmd.args[pos_idx].name.clone(), Val::Str(a.clone()));
            pos_idx += 1;
        } else {
            extra.push(a.clone());
        }
        i += 1;
    }

    // Missing required positional?
    for j in pos_idx..cmd.args.len() {
        if cmd.args[j].required {
            return Err(CapyError::msg(format!(
                "missing required argument {} for command {}",
                gofmt::quote(&cmd.args[j].name),
                gofmt::quote(&cmd.name)
            )));
        }
        // Optional with no value — bind empty so context lookups don't error.
        pos.insert(cmd.args[j].name.clone(), Val::Str(String::new()));
    }

    Ok(ParsedArgs { pos, flags, extra })
}

/// Port of `PrintCommandHelp`.
///
/// Renders a generated help screen for the command from its declared
/// args / flags / description. Go writes straight to stdout; this returns the
/// text so callers can print or test it.
pub fn command_help(lib: &Library, cmd: &CommandDef) -> String {
    let mut out = String::new();
    let lib_name =
        if lib.lib_name.is_empty() { "<lib>".to_string() } else { lib.lib_name.clone() };
    out.push_str(&format!(
        "{} — {}\n\n",
        cmd.name,
        fallback(&cmd.description, "(no description)")
    ));
    // Usage line.
    out.push_str(&format!("USAGE\n    capy {} {}", lib_name, cmd.name));
    for f in &cmd.flags {
        if f.is_bool {
            out.push_str(&format!(" [{}]", f.name));
        } else {
            out.push_str(&format!(" [{} VALUE]", f.name));
        }
    }
    for a in &cmd.args {
        if a.required {
            out.push_str(&format!(" <{}>", a.name));
        } else {
            out.push_str(&format!(" [{}]", a.name));
        }
    }
    out.push('\n');
    if !cmd.args.is_empty() {
        out.push_str("\nARGUMENTS\n");
        for a in &cmd.args {
            let req = if a.required { "(required)" } else { "(optional)" };
            // Go's `%-12s` left-pads the name to 12 columns.
            out.push_str(&format!("    {:<12}  {} {}\n", a.name, a.description, req));
        }
    }
    if !cmd.flags.is_empty() {
        out.push_str("\nFLAGS\n");
        // Sort for stable output.
        let mut flags_sorted: Vec<CommandFlag> = cmd.flags.clone();
        flags_sorted.sort_by(|a, b| a.name.cmp(&b.name));
        for f in &flags_sorted {
            let mut extra = f.description.clone();
            if !f.default.is_empty() {
                extra = format!("{} (default: {})", extra, f.default);
            }
            out.push_str(&format!("    {:<12}  {}\n", f.name, extra.trim()));
        }
    }
    out
}

/// Port of `findFlag`.
fn find_flag<'a>(cmd: &'a CommandDef, name: &str) -> Option<&'a CommandFlag> {
    cmd.flags.iter().find(|f| f.name == name)
}

/// Port of `trimFlagName`.
fn trim_flag_name(name: &str) -> String {
    name.trim_start_matches('-').to_string()
}

/// Port of `splitFlag` — `--foo=bar` → `("--foo", "bar", true)`;
/// `--foo` → `("--foo", "", false)`.
fn split_flag(s: &str) -> (&str, &str, bool) {
    match s.find('=') {
        Some(i) => (&s[..i], &s[i + 1..], true),
        None => (s, "", false),
    }
}

/// Port of `fallback`.
fn fallback(s: &str, def: &str) -> String {
    if s.is_empty() {
        def.to_string()
    } else {
        s.to_string()
    }
}
