//! Library errors into a [`Diag`]. Each conversion matches every variant, so a
//! variant added to a library error doesn't compile until it picks its kind
//! and words here.
//!
//! `From<anyhow::Error>` is the bridge for code that still returns `anyhow`:
//! a `Diag` or a library error anywhere in the chain becomes the message, and
//! the contexts above it stay in front, word for word. The fan-out tasks
//! convert those call sites, and the bridge goes when the last one does.

use mindtape::config::{ConfigError, ConfigPlace};
use mindtape::recur::dates::RecurError;
use mindtape::{
    ConditionError, DateOverrideError, FilterError, Location, RefError, Severity, Violation,
};

use super::{Command, Diag, Hint, Note};

impl From<anyhow::Error> for Diag {
    fn from(err: anyhow::Error) -> Self {
        let mut legacy = Vec::new();
        for cause in err.chain() {
            if let Some(mut diag) = known(cause) {
                legacy.append(&mut diag.0.legacy);
                diag.0.legacy = legacy;
                return diag;
            }
            legacy.push(cause.to_string());
        }
        let mut diag = Self::legacy("");
        diag.0.legacy = legacy;
        diag
    }
}

/// `cause` as a `Diag` when it is one or a library error converted here. A
/// [`FilterError`] isn't, because its kind depends on where the filter came
/// from: the command line's is a usage error, a config's isn't.
fn known(cause: &(dyn std::error::Error + 'static)) -> Option<Diag> {
    if let Some(diag) = cause.downcast_ref::<Diag>() {
        return Some(diag.clone());
    }
    if let Some(refused) = cause.downcast_ref::<crate::task_edit::Refused>() {
        return Some(refused.diag.clone());
    }
    if let Some(err) = cause.downcast_ref::<ConfigError>() {
        return Some(config(err));
    }
    if let Some(err) = cause.downcast_ref::<RecurError>() {
        return Some(err.clone().into());
    }
    if let Some(err) = cause.downcast_ref::<RefError>() {
        return Some(reference(err));
    }
    if let Some(err) = cause.downcast_ref::<ConditionError>() {
        return Some(err.clone().into());
    }
    cause
        .downcast_ref::<DateOverrideError>()
        .map(|err| err.clone().into())
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
        FilterError::UnknownStatus { value, nearest } => {
            let diag = in_filter(Diag::usage("unknown status").value(value));
            match nearest {
                Some(nearest) => diag.hint(Hint::did_you_mean(nearest)),
                None => diag.hint(Hint::run(
                    Command::mt().word("flip").word("--list"),
                    "to see the statuses",
                )),
            }
        }
        FilterError::UnknownView(name) => in_filter(Diag::usage("unknown view").value(name)),
        FilterError::UnknownDict { name, hint } => {
            in_filter(Diag::usage("unknown dict").value(name))
                .reason(hint.trim_start_matches("; ").to_owned())
        }
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

fn config(err: &ConfigError) -> Diag {
    match err {
        ConfigError::Io(source) => Diag::failed("can't read config").cause(source),
        ConfigError::Parse(error) => toml_error(error, None),
        ConfigError::ParseIn { origin, error } => toml_error(error, Some(origin)),
        ConfigError::Invalid { message, place } => invalid(message, place.as_ref()),
    }
}

/// A config that parsed but failed a check: `<file>:<line>: <key>: <message>`.
/// A second problem is a `note:` line. A `; did you mean: x?` clause on the
/// first becomes a hint.
fn invalid(message: &str, place: Option<&ConfigPlace>) -> Diag {
    let (first, rest) = message.split_once('\n').unwrap_or((message, ""));
    let (first, near) = split_near_match(first);
    let mut diag = Diag::failed("");
    if let Some(place) = place {
        diag = diag.at(&place.file, place.line, None);
    }
    let text = if rest.is_empty() {
        first
    } else {
        format!("{first}\n{rest}")
    };
    diag = diag.reason(text);
    match near {
        Some(name) => diag.hint(Hint::did_you_mean(name)),
        None => diag,
    }
}

/// `line` without its `; did you mean: <name>?` clause, and the name.
fn split_near_match(line: &str) -> (String, Option<String>) {
    const CLAUSE: &str = "; did you mean: ";
    let Some(start) = line.find(CLAUSE) else {
        return (line.to_owned(), None);
    };
    let after = &line[start + CLAUSE.len()..];
    let Some((name, tail)) = after.split_once('?') else {
        return (line.to_owned(), None);
    };
    (format!("{}{tail}", &line[..start]), Some(name.to_owned()))
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
