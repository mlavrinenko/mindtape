//! Library errors into a [`Diag`]. Each conversion matches every variant, so a
//! variant added to a library error doesn't compile until it picks its kind
//! and words here.
//!

use mindtape::config::{ConfigError, ConfigPlace};
use mindtape::recur::dates::RecurError;
use mindtape::{
    ConditionError, DateOverrideError, FillError, FilterError, Location, RefError, Severity,
    SlotKind, Unknown, Violation,
};

use super::{Command, Diag, Hint, Kind, Note};

/// A retire or rewrite that [`typst_cleanup`] could not plan or apply. A
/// file that git can't pin a tombstone to is the project's rule refusing the
/// work; a failed read, write or splice is the work failing.
impl From<mindtape::config::hooks::HookError> for Diag {
    fn from(err: mindtape::config::hooks::HookError) -> Self {
        filter(err.source, Some(&err.when)).rule(format!("on.flip[{}].when", err.index))
    }
}

/// Serialising a result for output failed, in the format named.
macro_rules! output_error {
    ($($err:ty => $what:literal),+ $(,)?) => {$(
        impl From<$err> for Diag {
            fn from(err: $err) -> Self {
                Self::failed($what).cause(&err)
            }
        }
    )+};
}

output_error! {
    serde_json::Error => "can't write JSON",
    csv::Error => "can't write CSV",
    toml::ser::Error => "can't write TOML",
    typst_edit::EditError => "can't apply the edit",
    clap::Error => "can't read the arguments",
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for Diag {
    fn from(err: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        Self::failed("can't write CSV").cause(&err)
    }
}

impl From<typst_cleanup::Error> for Diag {
    fn from(err: typst_cleanup::Error) -> Self {
        cleanup(&err)
    }
}

fn cleanup(err: &typst_cleanup::Error) -> Diag {
    use typst_cleanup::Error;
    match err {
        Error::OutsideRepo(path) => Diag::refused("task file is outside the git repository")
            .value(path.display().to_string()),
        Error::Mint(path, source) => mint(path, source),
        Error::Read(path, source) => Diag::cannot_read(path, source),
        Error::Write(path, source) => Diag::cannot_write(path, source),
        Error::Rewrite(path, source) => Diag::failed("can't rewrite links")
            .value(path.display().to_string())
            .cause(source),
        Error::Git(message) => Diag::verbatim(Kind::Failed, format!("git: {message}")),
    }
}

/// A tombstone that could not be minted for the file at `path`.
fn mint(path: &std::path::Path, source: &swhid_mint::git::Error) -> Diag {
    use swhid_mint::git::Error;
    let shown = path.display().to_string();
    let commit_first = Hint::new("commit the file first: the tombstone pins its last-touch commit");
    match source {
        Error::NotTracked(_) => Diag::refused("task file is not tracked by git")
            .value(shown)
            .hint(commit_first),
        Error::Dirty(_) => Diag::refused("task file has uncommitted changes")
            .value(shown)
            .hint(commit_first),
        Error::Git(message) => {
            Diag::verbatim(Kind::Failed, format!("git: {message}")).at_file(path)
        }
    }
}

impl From<DateOverrideError> for Diag {
    fn from(err: DateOverrideError) -> Self {
        match err {
            DateOverrideError::InvalidNow(value) => Self::failed("MINDTAPE_NOW is not a date")
                .value(value)
                .hint(Hint::new(
                    "set it to YYYY-MM-DD or an RFC 3339 timestamp, or unset it",
                )),
        }
    }
}

/// A filter that failed to parse or evaluate. `expr` is the filter as the
/// user typed it, quoted in the message; pass it whenever the call site has
/// it.
pub(crate) fn filter(err: FilterError, expr: Option<&str>) -> Diag {
    let in_filter = |diag: Diag| match expr {
        Some(expr) => diag.context("in filter", expr),
        None => diag,
    };
    let of_filter = |diag: Diag| match expr {
        Some(expr) => diag.value(expr),
        None => diag,
    };
    match err {
        FilterError::Parse(reason) => of_filter(Diag::usage("can't parse filter")).reason(reason),
        FilterError::Unsupported(reason) => {
            of_filter(Diag::usage("unsupported in filter")).reason(reason)
        }
        FilterError::Invalid(reason) => of_filter(Diag::usage("invalid filter")).reason(reason),
        FilterError::Call { problem, example } => of_filter(Diag::usage("invalid filter"))
            .reason(problem)
            .hint(Hint::example(example)),
        FilterError::UnknownFunction { name, nearest } => {
            let diag = in_filter(Diag::usage("unknown function").value(name));
            match nearest {
                Some(nearest) => diag.hint(Hint::did_you_mean(format!("{nearest}()"))),
                None => diag,
            }
        }
        FilterError::UnknownField(name) => in_filter(Diag::usage("unknown field").value(name)),
        FilterError::UnknownStatus { value, unknown } => in_filter(
            Diag::usage("unknown status")
                .value(value)
                .unknown(Some(&unknown)),
        ),
        FilterError::UnknownView(name) => in_filter(Diag::usage("unknown view").value(name)),
        FilterError::UnknownDict { name, unknown } => in_filter(
            Diag::usage("unknown dict")
                .value(name)
                .unknown(Some(&unknown)),
        ),
        FilterError::UnknownPathSet { name, unknown } => in_filter(
            Diag::usage("unknown path set")
                .value(format!("@{name}"))
                .reason(format!("declare it as [dicts.{name}]"))
                .unknown(Some(&unknown)),
        ),
        FilterError::Git { call, reason } => {
            Diag::failed("can't read git").reason(format!("{call}(): {reason}"))
        }
    }
}

impl From<FilterError> for Diag {
    fn from(err: FilterError) -> Self {
        filter(err, None)
    }
}

impl From<ConditionError> for Diag {
    fn from(err: ConditionError) -> Self {
        match err {
            ConditionError::Parse(reason) => Diag::failed("can't parse condition").reason(reason),
            ConditionError::Unsupported(reason) => {
                Diag::failed("unsupported in condition").reason(reason)
            }
            ConditionError::TaskField(name) => Diag::failed("a condition can't read a task field")
                .value(name)
                .hint(Hint::new(
                    "use on_branch(), tty(), env(), weekday(), now() or format()",
                )),
            ConditionError::UnknownPredicate(name) => {
                Diag::failed("unknown predicate in condition")
                    .value(name)
                    .hint(Hint::new(
                        "use on_branch, tty, env, weekday, now, format, nonempty, contested, named or blocked",
                    ))
            }
            ConditionError::Invalid(reason) => Diag::failed("invalid condition").reason(reason),
        }
    }
}

pub(super) fn config(err: &ConfigError) -> Diag {
    match err {
        ConfigError::Io(source) => Diag::failed("can't read config").cause(source),
        ConfigError::Parse(error) => toml_error(error, None),
        ConfigError::ParseIn { origin, error } => toml_error(error, Some(origin)),
        ConfigError::Invalid {
            message,
            place,
            unknown,
        } => invalid(message, place.as_ref(), unknown.as_ref()),
    }
}

/// A config that parsed but failed a check: `<file>:<line>: <key>: <message>`.
/// A second problem is a `note:` line. What the first problem's unknown name
/// could have been becomes hints.
fn invalid(message: &str, place: Option<&ConfigPlace>, unknown: Option<&Unknown>) -> Diag {
    let mut diag = Diag::failed("");
    if let Some(place) = place {
        diag = diag.at(&place.file, place.line, None);
    }
    diag.reason(message).unknown(unknown)
}

impl Diag {
    /// The nearest known name as `hint: did you mean x?` and the names to
    /// choose from as `hint: use one of: a, b`.
    pub(crate) fn unknown(self, unknown: Option<&Unknown>) -> Self {
        let Some(unknown) = unknown else {
            return self;
        };
        let near = unknown.nearest.iter().cloned().map(Hint::did_you_mean);
        let names = (!unknown.names.is_empty()).then(|| Hint::one_of(unknown.names.clone()));
        near.chain(names).fold(self, Self::hint)
    }
}

/// A TOML syntax error on one line: `<origin>:<line>:<col>: can't parse
/// config: <toml's message>`, with no source snippet. Without an origin, the
/// line and column lead the reason.
fn toml_error(error: &toml::de::Error, origin: Option<&str>) -> Diag {
    let diag = Diag::failed("can't parse config");
    let message = error.message().lines().collect::<Vec<_>>().join(", ");
    let shown = error.to_string();
    let at = shown
        .lines()
        .next()
        .and_then(|first| first.strip_prefix("TOML parse error at line "))
        .and_then(|rest| rest.split_once(", column "));
    match (origin, at) {
        (Some(origin), Some((line, col))) => {
            let mut diag = diag.reason(message);
            diag.0.place = Some(format!("{origin}:{line}:{col}"));
            diag
        }
        (Some(origin), None) => diag.value(origin).reason(message),
        (None, Some((line, col))) => diag.reason(format!("line {line}, column {col}: {message}")),
        (None, None) => diag.reason(message),
    }
}

impl From<ConfigError> for Diag {
    fn from(err: ConfigError) -> Self {
        config(&err)
    }
}

impl From<RecurError> for Diag {
    fn from(err: RecurError) -> Self {
        match err {
            RecurError::RruleNotImplemented => {
                Diag::failed("rrule(...) needs mt built with the rrule feature")
            }
            RecurError::DateOverflow => Diag::failed("date arithmetic overflowed"),
            RecurError::NoMatchingWeekday => {
                Diag::failed("no weekday in the on: set was reachable within a period")
            }
            RecurError::CatchUpExhausted(due) => {
                Diag::failed("recurrence did not reach today from").value(due)
            }
            RecurError::RruleExpansion(reason) => Diag::failed("can't expand rrule").reason(reason),
        }
    }
}

/// A rule breach: `tasks/x.typ:2: rules.one-wip: <message>`. The message is
/// the config's or the check's own words. An `error` severity refused the
/// work; a `warn` one refused nothing.
impl From<&Violation> for Diag {
    fn from(violation: &Violation) -> Self {
        let diag = match violation.severity {
            Severity::Error => Self::refused(""),
            Severity::Warn => Self::warning(""),
        };
        diag.at_location(&violation.location)
            .rule(violation.field.clone())
            .reason(violation.message.clone())
            .unknown(violation.unknown.as_ref())
    }
}

impl Diag {
    /// The place a breach or a load failure sits at: a file and line, or the
    /// words for one that has no file (`the task as filled:7`).
    pub(crate) fn at_location(mut self, location: &Location) -> Self {
        match location {
            Location::File { file, line } => self.at(file, *line, None),
            Location::Builtin { .. } | Location::Pack { .. } | Location::Filled { .. } => {
                self.0.place = Some(location.to_string());
                self
            }
        }
    }
}

fn reference(err: &RefError) -> Diag {
    match err {
        RefError::Path { path, source } => Diag::failed("can't resolve path")
            .value(path.display().to_string())
            .io(source),
        RefError::Empty => Diag::usage("empty task reference")
            .hint(Hint::run(Command::mt().word("ls"), "to see your tasks")),
        RefError::NoTaskAt(path) => Diag::failed("no task at").value(path.display().to_string()),
        RefError::Ambiguous { reference, stems } => stems.iter().fold(
            Diag::failed("stem hash names several tasks")
                .value(reference.clone())
                .hint(Hint::new("pass a longer hash or the stem")),
            |diag, stem| diag.note(Note::new("candidate").name(stem.clone())),
        ),
        RefError::NoMatch(reference) => Diag::failed("no task").value(reference.clone()),
    }
}

impl From<RefError> for Diag {
    fn from(err: RefError) -> Self {
        reference(&err)
    }
}

/// A slot fill that was rejected. A missing, mistyped or out-of-choice value is
/// the command line's; a bad kind in the template or a failed rewrite is not.
impl From<FillError> for Diag {
    fn from(err: FillError) -> Self {
        match err {
            FillError::Missing(names) => Diag::usage(if names.len() == 1 {
                "missing a required slot"
            } else {
                "missing required slots"
            })
            .value(names.join(", "))
            .hint(Hint::new("pass each with --set name=value, or with -f"))
            .hint(Hint::run(
                Command::mt().word("add").word("--describe"),
                "to see the slots",
            )),
            FillError::Type {
                slot,
                want: SlotKind::List | SlotKind::MultiChoice,
                got,
            } => Diag::usage("slot expects a list")
                .value(slot.clone())
                .reason(format!("got {got}"))
                .hint(Hint::command(
                    "add items one at a time",
                    Command::mt()
                        .word("add")
                        .word("--set")
                        .arg(format!("{slot}+=value")),
                ))
                .hint(Hint::new("or pass a JSON or YAML array with -f")),
            FillError::Type { slot, want, got } => Diag::usage("slot has the wrong type")
                .value(slot)
                .reason(format!("expects {}, got {got}", want.as_str())),
            FillError::BadChoice {
                slot,
                value,
                choices,
            } => {
                let known: Vec<&str> = choices.iter().map(String::as_str).collect();
                let diag = Diag::usage("not one of the slot's choices")
                    .value(value.clone())
                    .context("for slot", slot)
                    .reason(format!("expected one of {}", choices.join(", ")));
                match mindtape::suggest::nearest(&value, &known) {
                    Some(near) => diag.hint(Hint::did_you_mean(near)),
                    None => diag,
                }
            }
            FillError::BadDate { slot, value } => Diag::usage("not a YYYY-MM-DD date")
                .value(value)
                .context("for slot", slot),
            FillError::UnknownKind { slot, kind } => Diag::failed("slot declares an unknown kind")
                .value(kind)
                .context("in slot", slot)
                .reason(
                    "expected one of string, int, float, real, bool, date, list, text, raw, choice",
                ),
            FillError::BadMarkup { slot, message } => Diag::usage("slot is not valid Typst markup")
                .value(slot)
                .reason(message),
            FillError::Edit(message) => Diag::failed("can't rewrite the template").reason(message),
        }
    }
}
