use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use crate::cli::format::format_task_typst;
use crate::cli::util::{home_dir, open_query_db, resolve_query_db_path, shorten_home};
use crate::config::{self, AgendaConfig, AgendaSection};
use crate::store::{SortDir, SortField, SortSpec, Store, TaskFilter, TaskView};

const DEFAULT_AGENDA_TOML: &str = include_str!("../../../lib/default-agenda.toml");

/// Generate a Typst agenda from configured sections.
#[derive(Parser, Debug)]
pub struct AgendaArgs {
    /// Path to `SQLite` database
    #[arg(long)]
    pub db: Option<PathBuf>,

    /// Path(s) to config file(s) (repeatable)
    #[arg(long)]
    pub config: Vec<PathBuf>,
}

/// Identity key for cross-section deduplication.
///
/// Prefers `task_id` (`UUIDv7`) when present — this correctly deduplicates the same
/// logical task imported across multiple files.  Falls back to `(file_path, position)`
/// for tasks without an explicit ID.
#[derive(Clone, PartialEq, Eq, Hash)]
enum TaskKey {
    Id(String),
    Position(PathBuf, i32),
}

fn task_key(task: &TaskView) -> TaskKey {
    if let Some(ref id) = task.task_id {
        TaskKey::Id(id.clone())
    } else {
        TaskKey::Position(task.file_path.clone(), task.position)
    }
}

/// Accumulates output and deduplication state across agenda sections.
struct AgendaState {
    out: String,
    seen: HashSet<TaskKey>,
}

impl AgendaArgs {
    /// Run the agenda command.
    ///
    /// # Errors
    /// Returns error if config loading, database open, or query fails.
    pub fn run(&self) -> Result<()> {
        let cfg = self.load_config()?;
        let agenda = match cfg.agenda {
            Some(ag) if !ag.sections.is_empty() => ag,
            _ => default_agenda()?,
        };

        let db_path = if let Some(ref p) = self.db {
            p.clone()
        } else {
            resolve_query_db_path(None)
        };
        let store = open_query_db(&db_path)?;
        let home = home_dir();
        let today = today_str();

        let global_filter = expand_filter(agenda.filter.as_deref(), &today);
        let global_on_empty = agenda.on_empty.as_deref().unwrap_or("show");

        let mut state = AgendaState {
            out: String::new(),
            seen: HashSet::new(),
        };
        state.out.push_str(&format!(
            "#import \"@local/mindtape:{}\": *\n",
            crate::TYPST_PACKAGE_VERSION
        ));

        for section in &agenda.sections {
            let on_empty = section
                .on_empty
                .as_deref()
                .unwrap_or(global_on_empty);
            let empty_msg = section
                .empty_message
                .as_deref()
                .or(agenda.empty_message.as_deref());

            let mut section_out = String::new();
            let is_empty = match section.kind.as_str() {
                "errors" => render_errors(&store, &home, &mut section_out)?,
                "tasks" => render_tasks(
                    &store,
                    section,
                    &today,
                    global_filter.as_deref(),
                    &home,
                    &mut state,
                    &mut section_out,
                )?,
                other => bail!("unknown agenda section kind: {other:?}"),
            };

            if is_empty && on_empty == "hide" {
                continue;
            }

            state.out.push('\n');
            state.out.push_str(&format!("== {}\n", section.name));
            state.out.push('\n');

            if is_empty {
                if let Some(msg) = empty_msg {
                    state.out.push_str(msg);
                    state.out.push('\n');
                } else {
                    state.out.push_str(&section_out);
                }
            } else {
                state.out.push_str(&section_out);
            }
        }

        print!("{}", state.out);
        Ok(())
    }

    fn load_config(&self) -> Result<config::Config> {
        if !self.config.is_empty() {
            return config::load_and_merge(&self.config).context("failed to load config files");
        }
        if let Some(path) = config::find_config() {
            return config::load_config(&path)
                .with_context(|| format!("failed to load config from {}", path.display()));
        }
        Ok(config::Config {
            database: None,
            watch: vec![],
            agenda: None,
        })
    }
}

fn default_agenda() -> Result<AgendaConfig> {
    let cfg: config::Config =
        toml::from_str(DEFAULT_AGENDA_TOML).context("failed to parse default agenda")?;
    cfg.agenda
        .ok_or_else(|| anyhow::anyhow!("default agenda has no [agenda] section"))
}

fn today_str() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    )
}

/// Replace `$today` with the current date in a filter expression.
fn expand_filter(filter: Option<&str>, today: &str) -> Option<String> {
    filter.map(|f| f.replace("$today", today))
}

/// Render errors section. Returns `true` if the section is empty.
fn render_errors(store: &impl Store, home: &str, out: &mut String) -> Result<bool> {
    let errors = store
        .list_file_errors()
        .context("failed to list file errors")?;
    if errors.is_empty() {
        out.push_str("No errors.\n");
        Ok(true)
    } else {
        for e in &errors {
            let path = shorten_home(&e.file_path, home);
            out.push_str(&format!("- `{path}`\n  ```\n  {}\n  ```\n", e.error));
        }
        Ok(false)
    }
}

/// Combine global and section filters with `&&`.
fn combine_filters(global: Option<&str>, section: Option<String>) -> Option<String> {
    match (global, section) {
        (Some(gf), Some(sf)) => Some(format!("({gf}) && ({sf})")),
        (Some(gf), None) => Some(gf.to_string()),
        (None, sf) => sf,
    }
}

/// Render tasks section. Returns `true` if the section is empty.
#[allow(clippy::too_many_arguments)]
fn render_tasks(
    store: &impl Store,
    section: &AgendaSection,
    today: &str,
    global_filter: Option<&str>,
    home: &str,
    state: &mut AgendaState,
    out: &mut String,
) -> Result<bool> {
    let done = match section.status.as_deref() {
        Some("done") => Some(true),
        Some("all") => None,
        Some("pending") | None => Some(false),
        Some(other) => bail!("unknown status filter: {other:?}"),
    };

    let sort = section
        .sort
        .iter()
        .map(|s| parse_sort_spec(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("invalid sort spec in agenda config: {e}"))?;

    let section_expr = expand_filter(section.filter.as_deref(), today);
    let filter = TaskFilter {
        done,
        folder: None,
        watch_root: None,
        limit: section.limit,
        sort,
        expr: combine_filters(global_filter, section_expr),
    };

    let tasks = store
        .query_tasks(&filter)
        .context("failed to query tasks")?;

    let dedup = section.deduplicate.unwrap_or(true);
    let mut count = 0;
    for task in &tasks {
        let key = task_key(task);
        if dedup && state.seen.contains(&key) {
            continue;
        }
        out.push_str(&format_task_typst(task));
        let file_label = shorten_home(&task.file_path, home);
        let path_display = if file_label.contains(' ') {
            format!(" // \"{file_label}\"")
        } else {
            format!(" // {file_label}")
        };
        out.push_str(&path_display);
        out.push('\n');
        if dedup {
            state.seen.insert(key);
        }
        count += 1;
    }
    if count == 0 {
        out.push_str("No tasks.\n");
    }
    Ok(count == 0)
}

fn parse_sort_spec(input: &str) -> Result<SortSpec, String> {
    let (field_str, dir_str) = match input.split_once(':') {
        Some((field, dir)) => (field, Some(dir)),
        None => (input, None),
    };

    let field = match field_str {
        "due" => SortField::Due,
        "start" => SortField::Start,
        "rank" => SortField::Rank,
        "id" => SortField::Id,
        "file" => SortField::File,
        "position" | "pos" => SortField::Position,
        "title" => SortField::Title,
        "status" => SortField::Status,
        _ => return Err(format!("unknown sort field: {field_str}")),
    };

    let dir = match dir_str {
        Some("asc") | None => SortDir::Asc,
        Some("desc") => SortDir::Desc,
        Some(other) => return Err(format!("unknown sort direction: {other}")),
    };

    Ok(SortSpec { field, dir })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn expand_filter_replaces_today() {
        let result = expand_filter(Some("due < \"$today\""), "2026-03-16");
        assert_eq!(result.unwrap(), "due < \"2026-03-16\"");
    }

    #[test]
    fn expand_filter_none_passthrough() {
        assert!(expand_filter(None, "2026-03-16").is_none());
    }

    #[test]
    fn expand_filter_no_placeholder() {
        let result = expand_filter(Some("rank >= 100"), "2026-03-16");
        assert_eq!(result.unwrap(), "rank >= 100");
    }

    #[test]
    fn default_agenda_parses() {
        let agenda = default_agenda().unwrap();
        assert!(!agenda.sections.is_empty());
        assert!(agenda.filter.is_some());
        assert_eq!(agenda.sections[0].name, "Errors");
        assert_eq!(agenda.sections[0].kind, "errors");
        let last = agenda.sections.last().unwrap();
        assert_eq!(last.name, "Recent");
        assert_eq!(last.kind, "tasks");
    }

    #[test]
    fn combine_filters_both() {
        let result = combine_filters(Some("a"), Some("b".to_string()));
        assert_eq!(result.unwrap(), "(a) && (b)");
    }

    #[test]
    fn combine_filters_global_only() {
        let result = combine_filters(Some("a"), None);
        assert_eq!(result.unwrap(), "a");
    }

    #[test]
    fn combine_filters_section_only() {
        let result = combine_filters(None, Some("b".to_string()));
        assert_eq!(result.unwrap(), "b");
    }

    #[test]
    fn combine_filters_neither() {
        assert!(combine_filters(None, None).is_none());
    }

    #[test]
    fn dedup_excludes_seen_tasks() {
        let task = TaskView {
            title: "Buy milk".to_string(),
            is_done: false,
            position: 0,
            file_path: PathBuf::from("todo.typ"),
            file_title: None,
            due: None,
            start: None,
            rank: None,
            task_id: None,
            tags: vec![],
            milestone: None,
            watch_root: None,
        };
        let key = task_key(&task);
        let mut seen = HashSet::new();
        seen.insert(key.clone());
        assert!(seen.contains(&task_key(&task)));
    }

    #[test]
    fn parse_sort_spec_basic() {
        let spec = parse_sort_spec("due:asc").unwrap();
        assert_eq!(spec.field, SortField::Due);
        assert_eq!(spec.dir, SortDir::Asc);
    }

    #[test]
    fn parse_sort_spec_desc() {
        let spec = parse_sort_spec("rank:desc").unwrap();
        assert_eq!(spec.field, SortField::Rank);
        assert_eq!(spec.dir, SortDir::Desc);
    }

    #[test]
    fn parse_sort_spec_default_asc() {
        let spec = parse_sort_spec("title").unwrap();
        assert_eq!(spec.field, SortField::Title);
        assert_eq!(spec.dir, SortDir::Asc);
    }

    #[test]
    fn parse_sort_spec_invalid_field() {
        assert!(parse_sort_spec("invalid").is_err());
    }
}
