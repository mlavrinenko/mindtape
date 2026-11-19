//! The editor session `mt add` opens: one buffer in, one buffer out.
//!
//! [`run`] opens the editor on the scaffold [`super::scaffold`] renders, reads
//! the saved buffer back, and parses it into fill values. A buffer that does not
//! parse comes back as [`Session::Unparsed`] with the error, so the caller can
//! re-open it instead of dropping the answers. [`fill_until_it_fills`] runs that
//! loop until the buffer parses and fills. The `mindtape` library stays
//! format-blind. This glue lives in the CLI.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::Command;

use mindtape::{AddConfig, FieldOp, InteractiveMode, RawValue, SlotSpec};

use super::scaffold::{is_unfilled_required, scaffold};
use super::values::reject_unknown_keys;
use super::{markup_safe, yaml_values::parse_mapping};
use crate::diag::{Diag, Hint, Note, line};
use crate::pre_write::Problem;

type Result<T> = std::result::Result<T, Diag>;

/// The `#` prefixes a refusal block leaves above the values: the rendered
/// `error:`, `hint:` and `note:` lines [`rejected_buffer`] writes.
const ERROR_PREFIXES: [&str; 3] = ["# error: ", "# note: ", "# hint: "];

/// Decide whether `add` should open the interactive editor.
///
/// `force` is the `--edit` flag: it overrides a `never` setting and the `auto`
/// missing-slot trigger, but still requires a terminal. The non-interactive path
/// returns `false`.
///
/// `tty` should be `std::io::stdin().is_terminal() && std::io::stdout().is_terminal()`.
///
/// # Errors
/// Returns an error when the caller demands an editor (`--edit`, or `interactive`
/// set to `always`) but `tty` is false.
pub fn should_open(
    force: bool,
    config: &AddConfig,
    specs: &[SlotSpec],
    seed: &BTreeMap<String, RawValue>,
    tty: bool,
) -> Result<bool> {
    if config.interactive == InteractiveMode::Never && !force {
        return Ok(false);
    }
    if force || config.interactive == InteractiveMode::Always {
        if !tty {
            return Err(Diag::usage("interactive add needs a terminal")
                .hint(Hint::new("pass the values with -f or --set instead")));
        }
        return Ok(true);
    }
    Ok(tty && has_unfilled_required(specs, seed))
}

fn has_unfilled_required(specs: &[SlotSpec], seed: &BTreeMap<String, RawValue>) -> bool {
    specs
        .iter()
        .any(|spec| is_unfilled_required(spec) && !seed.contains_key(&spec.name))
}

/// The values one editor session saved, and the keys whose value differs from
/// the scaffold's own baseline. The scaffold pre-fills every slot default, so a
/// default the user left untouched is not their input and must not stop a hook
/// from filling the field.
pub struct Edited {
    /// The saved fill values.
    pub values: BTreeMap<String, RawValue>,
    /// The keys the editor changed or added.
    pub changed: BTreeSet<String>,
    /// The text the editor saved, kept so a refusal can re-open it.
    pub buffer: String,
}

/// A buffer a refusal sent back: the text the editor saved and the lines to
/// show above it when the editor re-opens.
pub struct Rejected {
    /// The text the editor saved.
    pub buffer: String,
    /// The refusal, one comment line each, without its `# ` prefix. A rule
    /// breach contributes one `error:` line per breach, and a `note:` line
    /// under it when a configured message replaced the check's own words.
    pub lines: Vec<String>,
}

impl Rejected {
    /// A refusal from one error: its message line.
    pub(super) fn from_error(buffer: String, error: &Diag) -> Self {
        Self {
            buffer,
            lines: vec![line(error)],
        }
    }

    /// A refusal from the problems that refused the filled task. Every breach
    /// gets its own `error:` line. A task that breaks more than one rule shows
    /// every breach at once, not one per round.
    pub(super) fn from_problems(buffer: String, problems: &[Problem]) -> Self {
        let mut lines = Vec::new();
        for problem in problems {
            push_lines(&mut lines, &format!("error: {}", problem.line));
            if let Some(detail) = &problem.detail {
                push_lines(&mut lines, &format!("note: {detail}"));
            }
        }
        Self { buffer, lines }
    }
}

/// Push `text` as one comment line per line it holds. A multi-line message
/// stays a comment block.
fn push_lines(lines: &mut Vec<String>, text: &str) {
    lines.extend(text.lines().map(ToOwned::to_owned));
}

/// What one editor session leaves: the values the buffer parsed into, or the
/// buffer itself when it did not parse.
pub enum Session {
    /// The buffer parsed into fill values.
    Filled(Edited),
    /// The buffer did not parse, so it comes back with the error.
    Unparsed(Rejected),
}

/// Open the editor on `previous`, or on a scaffold for `specs` pre-filled from
/// `seed` when there is none, and parse the buffer it saves. `editor` is the
/// `[add] editor` to run, `None` when it is unset or not yet allowed
/// (`mt allow`), which falls back to `$VISUAL`.
///
/// # Errors
/// Returns an error when the editor fails to spawn or exits without saving.
/// The buffer it was editing stays on disk, and the error carries its path.
pub fn run(
    config: &AddConfig,
    editor: Option<&str>,
    specs: &[SlotSpec],
    seed: &BTreeMap<String, RawValue>,
    previous: Option<&Rejected>,
) -> Result<Session> {
    let fresh = scaffold(config, specs, seed);
    let baseline = parse_buffer(&fresh)?;
    let buffer = previous.map_or(fresh, rejected_buffer);
    let command = editor_command(editor);
    let edited = open_in_editor(&command, &buffer)?;
    Ok(parse_session(edited, &baseline))
}

/// What one fill loop fills: the template's slots and source, and the values
/// the editor starts from.
pub(super) struct Fill<'a> {
    pub(super) specs: &'a [SlotSpec],
    pub(super) source: &'a str,
    pub(super) seed: &'a BTreeMap<String, RawValue>,
}

/// Open the editor until the buffer it saves parses and fills. A refusal
/// re-opens the same buffer with the error above the values. The answers
/// survive a typo instead of starting over from the scaffold.
///
/// The fill here is the one [`super::pipeline::render`] runs again, so a buffer
/// that passes it cannot fail the fill later.
///
/// # Errors
/// Returns an error when the editor fails to spawn or exits without saving.
pub(super) fn fill_until_it_fills(
    config: &AddConfig,
    editor: Option<&str>,
    fill: &Fill<'_>,
    previous: Option<Rejected>,
) -> Result<Edited> {
    let mut retry = previous;
    loop {
        let edited = match run(config, editor, fill.specs, fill.seed, retry.as_ref())? {
            Session::Filled(edited) => edited,
            Session::Unparsed(rejected) => {
                retry = Some(rejected);
                continue;
            }
        };
        match refuse_buffer(&edited.values, fill.specs, fill.source) {
            Ok(()) => return Ok(edited),
            Err(error) => retry = Some(Rejected::from_error(edited.buffer, &error)),
        }
    }
}

/// Refuse a buffer whose keys the template does not declare, or whose values
/// the fill rejects. The fill runs here, before the hooks, so a bad value
/// re-opens the editor instead of failing the add.
fn refuse_buffer(
    values: &BTreeMap<String, RawValue>,
    specs: &[SlotSpec],
    source: &str,
) -> Result<()> {
    // The unknown-key refusal takes the `--set` shape, so a buffer typo gets
    // the same message, did-you-mean and hints as a `--set` typo.
    let ops: Vec<(String, FieldOp)> = values
        .iter()
        .map(|(name, value)| (name.clone(), FieldOp::Set(value.clone())))
        .collect();
    reject_unknown_keys(&ops, specs)?;
    mindtape::fill(source, &markup_safe(values, specs)).map_err(Diag::from)?;
    Ok(())
}

/// The saved buffer's values and changed keys, or the buffer itself when it
/// does not parse.
fn parse_session(edited: String, baseline: &BTreeMap<String, RawValue>) -> Session {
    let values = match parse_buffer(&edited) {
        Ok(values) => values,
        Err(error) => return Session::Unparsed(Rejected::from_error(edited, &error)),
    };
    let changed = changed_keys(baseline, &values);
    Session::Filled(Edited {
        values,
        changed,
        buffer: edited,
    })
}

/// Re-open the previous buffer with its refusal above the values. The function
/// drops the previous refusal block first, so a buffer that comes back twice
/// carries one refusal, not a stack of them.
fn rejected_buffer(rejected: &Rejected) -> String {
    let mut out = String::new();
    for line in &rejected.lines {
        if line.is_empty() {
            out.push_str("#\n");
        } else {
            out.push_str("# ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str(strip_error_lines(&rejected.buffer));
    out
}

/// `buffer` without the refusal block a previous round put above the values.
/// The block starts with an `# error:` line and runs until a line that is not
/// one of ours. The scaffold's own comments below it stay.
fn strip_error_lines(buffer: &str) -> &str {
    let mut offset = 0;
    let mut in_block = false;
    for line in buffer.split_inclusive('\n') {
        let text = line.trim_end_matches('\n');
        let ours = ERROR_PREFIXES.iter().any(|prefix| text.starts_with(prefix))
            || (in_block && text == "#");
        if !ours {
            break;
        }
        in_block = true;
        offset += line.len();
    }
    &buffer[offset..]
}

/// The keys whose saved value differs from `baseline`, including a key only
/// the saved map holds.
fn changed_keys(
    baseline: &BTreeMap<String, RawValue>,
    saved: &BTreeMap<String, RawValue>,
) -> BTreeSet<String> {
    saved
        .iter()
        .filter(|(key, value)| baseline.get(*key).is_none_or(|seeded| seeded != *value))
        .map(|(key, _)| key.clone())
        .collect()
}

/// Last-resort editor when neither `[add] editor` nor `$VISUAL`/`$EDITOR` says
/// anything: the one editor the platform always has. `vi` is POSIX-mandated and
/// absent from Windows, where `notepad` is the equivalent floor. It also blocks
/// until closed, which is what this path needs.
#[cfg(not(windows))]
const FALLBACK_EDITOR: &str = "vi";
#[cfg(windows)]
const FALLBACK_EDITOR: &str = "notepad";

/// Resolve the editor command: config override, then `$VISUAL`, then `$EDITOR`,
/// then [`FALLBACK_EDITOR`].
fn editor_command(override_cmd: Option<&str>) -> String {
    if let Some(cmd) = override_cmd
        && !cmd.trim().is_empty()
    {
        return cmd.trim().to_owned();
    }
    for var in ["VISUAL", "EDITOR"] {
        if let Ok(value) = std::env::var(var)
            && !value.trim().is_empty()
        {
            return value;
        }
    }
    FALLBACK_EDITOR.to_owned()
}

/// Write `initial` to a scratch `.yaml` file, run `command` on it, and read it
/// back after the editor exits.
fn open_in_editor(command: &str, initial: &str) -> Result<String> {
    let mut file = tempfile::Builder::new()
        .prefix("mindtape-add-")
        .suffix(".yaml")
        .tempfile()
        .map_err(|err| Diag::failed("can't create the scratch buffer").io(&err))?;
    file.write_all(initial.as_bytes())
        .and_then(|()| file.flush())
        .map_err(|err| Diag::failed("can't write the scratch buffer").io(&err))?;
    let path = file.into_temp_path();

    let mut parts = command.split_whitespace();
    let Some(program) = parts.next() else {
        return Err(Diag::failed("the editor command is empty")
            .hint(Hint::new("set [add] editor, $VISUAL or $EDITOR")));
    };
    let status = Command::new(program)
        .args(parts)
        .arg(&*path)
        .status()
        .map_err(|err| {
            Diag::failed("can't start the editor")
                .value(command)
                .io(&err)
        })?;
    if !status.success() {
        let diag = Diag::failed("editor exited without saving").value(command);
        return Err(keep_buffer(diag, path));
    }
    std::fs::read_to_string(&path)
        .map_err(|err| Diag::failed("can't read the edited buffer").io(&err))
}

/// Keep the scratch buffer on disk and name it. An editor that exits non-zero
/// is the one way out of the re-open loop. The file is the only copy of the
/// answers, so it outlives the command.
fn keep_buffer(diag: Diag, mut path: tempfile::TempPath) -> Diag {
    let kept = path.to_path_buf();
    path.disable_cleanup(true);
    diag.note(Note::new("the buffer is kept").value(mindtape::display_path(&kept)))
}

/// Parse a saved buffer into fill values, dropping null and empty entries. The
/// YAML -> `RawValue` conversion itself lives in `yaml_values`, shared with the
/// `-f/--values` YAML path. Only the empty-string drop is specific to this
/// caller. Blank means "use the default" in the editor scaffold, but a `-f`
/// file's `""` is a value the other caller must keep.
///
/// # Errors
/// Returns an error when the buffer is not a YAML mapping, a key is non-string,
/// or a value has an unsupported shape (nested mapping, non-string list item).
pub(super) fn parse_buffer(text: &str) -> Result<BTreeMap<String, RawValue>> {
    let values = parse_mapping(text, None)?;
    Ok(drop_blank(values))
}

/// Drop the entries the editor treats as "not filled". One is a `null`, already
/// `None` from the shared converter. The other is an empty string: the shared
/// converter keeps it as a value for the `-f` caller, but blank here means
/// "fall through to the default".
fn drop_blank(values: BTreeMap<String, Option<RawValue>>) -> BTreeMap<String, RawValue> {
    values
        .into_iter()
        .filter_map(|(key, value)| {
            let raw = value?;
            if matches!(&raw, RawValue::Str(text) if text.is_empty()) {
                return None;
            }
            Some((key, raw))
        })
        .collect()
}

#[cfg(test)]
#[path = "interactive_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "interactive_tests_editor.rs"]
mod tests_editor;
