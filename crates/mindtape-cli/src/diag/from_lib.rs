//! Library errors into a [`Diag`]. Each conversion matches every variant, so a
//! variant added to a library error doesn't compile until it picks its kind
//! and words here.
//!

use mindtape::config::{ConfigError, ConfigPlace};
use mindtape::recur::dates::RecurError;
use mindtape::{
    ConditionError, DateOverrideError, DictMiss, FillError, FilterError, Location, RefError,
    Severity, SlotKind, Unknown, Violation,
};

use super::{Command, Diag, Hint, Note};

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

/// An approval `mt allow` could not record.
impl From<mindtape::allow::AllowError> for Diag {
    fn from(err: mindtape::allow::AllowError) -> Self {
        use mindtape::allow::AllowError;
        match err {
            AllowError::NoDataHome => Self::failed("can't find the local data directory")
                .hint(Hint::new("set MINDTAPE_DATA_HOME to a directory")),
            AllowError::Write { path, source } => Self::cannot_write(&path, &source),
        }
    }
}

/// A pack source `mt packs` couldn't fetch, with the error's own next step.
/// The caller names the source with [`Diag::value`].
impl From<mindtape::config::pack_sources::FetchError> for Diag {
    fn from(err: mindtape::config::pack_sources::FetchError) -> Self {
        use mindtape::config::pack_sources::FetchError as E;
        let what = match &err {
            E::GitNotFound | E::Git { .. } => "can't run git for pack source",
            E::UnknownRef { .. } => "can't resolve pack source",
            E::DriveRelative { .. } => "can't locate pack source",
            E::BadCommit { .. }
            | E::MissingCommit { .. }
            | E::Unsupported { .. }
            | E::Mismatch { .. } => "can't fetch pack source",
            E::Occupied { .. } | E::Io(_) => "can't write pack source to the cache",
        };
        let diag = Self::failed(what).cause(&err);
        match err.hint() {
            Some(hint) => diag.hint(Hint::new(hint)),
            None => diag,
        }
    }
}

impl From<csv::IntoInnerError<csv::Writer<Vec<u8>>>> for Diag {
    fn from(err: csv::IntoInnerError<csv::Writer<Vec<u8>>>) -> Self {
        Self::failed("can't write CSV").cause(&err)
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
        FilterError::UnknownField(name) => quote_hint(
            in_filter(Diag::usage("unknown field").value(&name)),
            expr,
            &name,
        ),
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

/// `diag` with the filter rebuilt with `name` quoted as text as its hint, when
/// `expr` compares `name` with a field: `status == wip` means `status == "wip"`.
fn quote_hint(diag: Diag, expr: Option<&str>, name: &str) -> Diag {
    let quoted = expr.and_then(|expr| super::quoting::quoted_form(expr, name));
    match quoted {
        Some(quoted) => diag.hint(Hint::as_in("to match text, quote it", quoted)),
        None => diag,
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
            hint,
        } => invalid(message, place.as_ref(), unknown.as_ref(), *hint),
    }
}

/// A config that parsed but failed a check: `<file>:<line>: <key>: <message>`.
/// A second problem is a `note:` line. What the first problem's unknown name
/// could have been becomes hints, and so does its next step.
fn invalid(
    message: &str,
    place: Option<&ConfigPlace>,
    unknown: Option<&Unknown>,
    hint: Option<&'static str>,
) -> Diag {
    let mut diag = Diag::failed("");
    if let Some(place) = place {
        diag = diag.at(&place.file, place.line, None);
    }
    let diag = diag.reason(message).unknown(unknown);
    match hint {
        Some(hint) => diag.hint(Hint::new(hint)),
        None => diag,
    }
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
        (Some(origin), Some((line, col))) => diag
            .reason(message)
            .at_words(format!("{origin}:{line}:{col}")),
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
            .dict_miss(violation.dict_miss.as_ref())
            .detail(violation.detail.as_deref())
    }
}

impl Diag {
    /// The check's own words under a breach whose configured message replaced
    /// them, as a note.
    fn detail(self, detail: Option<&str>) -> Self {
        match detail {
            Some(detail) => self.note(Note::line(detail)),
            None => self,
        }
    }
}

impl Diag {
    /// The word a dictionary lacks with its nearest entries, and the command
    /// that lists them all, as far as the two-hint limit leaves room.
    pub(crate) fn dict_miss(self, miss: Option<&DictMiss>) -> Self {
        let Some(miss) = miss else {
            return self;
        };
        let list = Hint::run(
            Command::mt().word("dicts").arg(&miss.dict),
            "to see its entries",
        );
        [Hint::not_in_dict(miss.clone()), list]
            .into_iter()
            .fold(self, |diag, hint| {
                if diag.has_hint_room() {
                    diag.hint(hint)
                } else {
                    diag
                }
            })
    }
}

impl Diag {
    /// The place a breach or a load failure sits at: a file and line, or the
    /// words for one that has no file (`<new task from template 'task'>:7`).
    pub(crate) fn at_location(self, location: &Location) -> Self {
        match location {
            Location::File { file, line } => self.at(file, *line, None),
            Location::Builtin { .. } | Location::Pack { .. } | Location::Filled { .. } => {
                self.at_words(location.to_string())
            }
        }
    }
}

/// The one refusal for a task reference that named no task, wherever it was
/// typed: a `<task>` argument or a `links:` target.
fn reference(err: &RefError) -> Diag {
    let list = || Hint::run(Command::mt().word("ls"), "to see your tasks");
    match err {
        RefError::Path { path, source } => Diag::failed("can't resolve path")
            .value(mindtape::display_path(path))
            .io(source),
        RefError::Empty => Diag::usage("empty task reference").hint(list()),
        RefError::NoTaskAt(path) => Diag::failed("no task at").value(mindtape::display_path(path)),
        RefError::SharedStem { reference, paths } => paths.iter().fold(
            Diag::failed("stem names several tasks")
                .value(reference.clone())
                .hint(Hint::new("pass the path of the one you mean")),
            |diag, path| diag.note(Note::new("candidate").name(mindtape::display_path(path))),
        ),
        RefError::Ambiguous {
            reference,
            candidates,
        } => candidates.iter().fold(
            Diag::failed("stem hash names several tasks")
                .value(reference.clone())
                .hint(Hint::new("pass a longer hash or the stem")),
            |diag, (hash, stem)| diag.note(Note::new("candidate").name(format!("{hash} {stem}"))),
        ),
        RefError::NoMatch { reference, nearest } => {
            let diag = Diag::failed("no task").value(reference.clone());
            match nearest {
                Some(near) => diag.hint(Hint::did_you_mean(near)),
                None => diag.hint(list()),
            }
        }
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
            .values(names)
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
                diag.did_you_mean(&value, &known)
            }
            FillError::BadDate { slot, value } => Diag::usage("not a YYYY-MM-DD date")
                .value(value)
                .context("for slot", slot),
            FillError::UnknownKind { slot, kind } => Diag::failed("slot declares an unknown kind")
                .value(kind)
                .context("in slot", slot)
                .reason(format!(
                    "expected one of {}",
                    mindtape::SlotKind::spellings()
                )),
            FillError::BadMarkup { slot, message } => Diag::usage("slot is not valid Typst markup")
                .value(slot)
                .reason(message),
            FillError::Edit(message) => Diag::failed("can't rewrite the template").reason(message),
        }
    }
}
