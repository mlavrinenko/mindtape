//! `mt set <task> <field>=<value>` — write one core field of an existing task.
//!
//! `mt` stamped `status:` (`flip`), `links:` (`link`), `reviewed:` (`review`)
//! and the stem (`mv`), and nothing else. Every other field — a priority worth
//! correcting, a tag worth adding, a due date worth moving — was a hand edit,
//! which is the one path `skills/mindtape/SKILL.md` opens by warning against:
//! it skips what `mt` stamps, resolves and validates, so a wrong value lands
//! and is only reported later, by `mt check`, on a file already saved.
//!
//! So this command is the generic writer for the fields that never earned one.
//! Three things keep it from becoming a second CLI of its own:
//!
//! - the token grammar is `mt add --set`'s, parsed by [`crate::cmd::add::tokens`]
//!   itself rather than re-spelled, so `tags+=auth` means here what it means
//!   there and a third operator can never appear on only one of them;
//! - the field table and the literal each value renders to live in
//!   [`mindtape::model::writable`], the same ones `mt add --set` uses, so the
//!   two commands cannot drift on a field name, a shape or an escaping rule;
//! - the write goes through [`crate::pre_write`], so a value the project
//!   refuses never reaches the file and there is nothing to repair.
//!
//! What it deliberately does not write is [`mindtape::model::writable::ELSEWHERE`]:
//! a name with its own command or add flag, whose shape carries more than a
//! value (a dated status, a typed edge, a renamed stem, a destination), and
//! `estimate`, whose unit dict has no unambiguous `<field>=<value>` spelling to
//! give it.

use clap::Args;
use mindtape::FieldOp;
use mindtape::RawValue;
use mindtape::model::writable::{self, Shape, WritableField};

use crate::cmd::add::tokens::{TokenOp, parse_token};
use crate::diag::{Command, Diag, Hint, Result, next_line};
use crate::pre_write;
use crate::task_edit::rewrite_field;
use crate::util;

/// Arguments for `mt set`.
#[derive(Args)]
pub struct SetArgs {
    #[command(flatten)]
    pub task: crate::cli::task_ref::TaskRef,
    /// The change: `<field>=<value>`, `<field>+=<value>` or `<field>@<path>`.
    ///
    /// `<field>+=<value>` adds to a list, and `<field>@<path>` reads the value
    /// from a file. `mt set --help` lists the fields you can set.
    pub assignment: String,
}

/// Run the `set` command.
///
/// # Errors
/// An unresolvable task, a token that is not an assignment, a field this
/// command does not write, a value of the wrong shape for the field, a
/// wrong-shaped file (no `task.with(…)` call), a value the project refuses, or
/// a failed write.
pub fn run(args: &SetArgs) -> Result<()> {
    let project = util::open_warned()?;

    let (task, abs) = util::resolve_task(&project, &args.task.task)?;
    let (name, op) = parse_assignment(&args.assignment)?;
    let field = writable(&name)?;
    let Some(value_text) = writable::render_op(field, &op, &task).map_err(Diag::from)? else {
        println!(
            "unchanged: {} already carries {:?} in {}",
            task.id,
            op_text(&op),
            field.name
        );
        return Ok(());
    };

    let updated = rewrite_field(&abs, field.name, &value_text)?;
    pre_write::write_checked(&project, &abs, updated)?;
    util::report_change("set", &task, &format!("{}: {value_text}", field.name));
    println!("{}", next_line(Command::mt().word("eval").arg(&task.id)));
    Ok(())
}

/// Parse one assignment through `mt add --set`'s own token parser, so the two
/// carriers cannot drift into two grammars.
///
/// # Errors
/// A token carrying no operator, an unreadable `@` file, or the `+@` operator
/// no command line produces.
fn parse_assignment(token: &str) -> Result<(String, FieldOp), Diag> {
    let (name, op) = parse_token(token)?;
    let op = match op {
        TokenOp::Set(text) | TokenOp::File(text) => FieldOp::Set(RawValue::Str(text)),
        TokenOp::Append(item) => FieldOp::Append(item),
    };
    Ok((name, op))
}

/// The value as the caller typed it — what a refusal or a no-op quotes back,
/// before any Typst literal was rendered from it.
fn op_text(op: &FieldOp) -> &str {
    match op {
        FieldOp::Set(RawValue::Str(text)) | FieldOp::Append(text) => text,
        FieldOp::Set(_) => "",
    }
}

/// Every field this command writes, in the emitter's field order.
///
/// Exposed for [`crate::cli::completion`]: the completer for `<field>=<value>` and
/// the refusal below read the same [`writable::WRITABLE`] table, so a tab press
/// can never offer a field this command would then name as unknown.
pub(crate) fn writable_fields() -> impl Iterator<Item = &'static str> {
    writable::names()
}

/// The long help of `mt set`, its field list read from [`writable::WRITABLE`]
/// so the help names exactly the fields this command writes.
pub(crate) fn long_about() -> String {
    let names: Vec<String> = writable::WRITABLE
        .iter()
        .map(|field| match field.shape {
            Shape::Num => format!("`{}` (as a number)", field.name),
            _ => format!("`{}`", field.name),
        })
        .collect();
    let list = match names.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    };
    format!(
        "Change one field of a task: `mt set <task> <field>=<value>`.\n\n\
         You can set {list}. Other fields have their own commands, such as \
         `mt flip` for `status`. mt checks the new value before it writes, so a \
         refused one leaves the file as it was."
    )
}

/// The writable field `name` names.
///
/// # Errors
/// A name another command or add flag owns (named, with a hint to it), or an
/// unknown one (named, with a nearest-match hint and a pointer to the writable
/// fields).
fn writable(name: &str) -> Result<&'static WritableField, Diag> {
    if let Some(field) = writable::get(name) {
        return Ok(field);
    }
    if let Some(owner) = writable::elsewhere(name) {
        return Err(Diag::usage("mt set does not write this field")
            .value(name)
            .hint(Hint::new(owner)));
    }
    let known: Vec<&str> = writable::names().collect();
    let diag = Diag::usage("unknown field").value(name);
    let diag = diag.did_you_mean(name, &known);
    Err(diag.hint(Hint::run(
        Command::mt().word("set").word("--help"),
        "to see the fields you can set",
    )))
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
