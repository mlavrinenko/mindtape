use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Parser;

use mindtape::cli::{self, Cli, Command, OutputFormat, StatusArgs, FilesArgs};
use mindtape::config;
use mindtape::eval;
use mindtape::store::{SqliteStore, Store, TaskFilter};
use mindtape::watcher::Watcher;
use mindtape::world;

fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    let args = cli::preprocess_args(raw_args);
    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Command::Watch(args)) => run_watch(&args),
        Some(Command::List(args)) => run_list(&args),
        Some(Command::Search(args)) => run_search(&args),
        Some(Command::Agenda(args)) => run_agenda(&args),
        Some(Command::Status(args)) => run_status(&args),
        Some(Command::Files(args)) => run_files(&args),
        Some(Command::Deps(args)) => run_deps(&args),
        Some(Command::Check(args)) => run_check(&args),
        None => {
            // Eval mode (backwards compat: `mindtape file.typ`)
            let Some(file) = cli.file else {
                bail!("no file specified (use --help for usage)");
            };
            run_eval(&file, cli.due, cli.limit)
        }
    }
}

fn run_eval(file: &Path, due: bool, limit: Option<usize>) -> Result<()> {
    if !file.exists() {
        bail!("file not found: {}", file.display());
    }

    let world = world::MindTapeWorld::new(file)
        .with_context(|| format!("failed to create world for {}", file.display()))?;

    let tasks = eval::eval_file(&world)
        .with_context(|| format!("failed to evaluate {}", file.display()))?;

    let tasks = cli::filter_and_sort(tasks, due, limit);

    for task in &tasks {
        println!("{}", cli::format_task(task));
    }
    Ok(())
}

fn run_watch(args: &cli::WatchArgs) -> Result<()> {
    let cfg = load_watch_config(args)?;

    if cfg.watch.is_empty() {
        bail!("no watch paths configured\nUsage: mindtape watch <path>\n   or: mindtape watch --config <file>");
    }

    let db_path = config::resolve_db_path(&cfg);
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create database directory {}", parent.display()))?;
        }
    }

    let store = SqliteStore::open(&db_path)
        .with_context(|| format!("failed to open database at {}", db_path.display()))?;

    let mut watcher = Watcher::new(store, &cfg.watch)
        .context("failed to set up file watcher")?;

    let scan = watcher.initial_scan();
    eprintln!(
        "initial scan: {} found, {} indexed, {} skipped, {} errors",
        scan.found, scan.indexed, scan.skipped, scan.errors,
    );

    watcher.run().context("watcher error")?;
    Ok(())
}

fn run_list(args: &cli::ListArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    let filter = TaskFilter {
        done: args.done_filter(),
        tag: args.tag.clone(),
        due_before: args.due_before.clone(),
        file_path: args.file.clone(),
        folder: args.folder.clone(),
        limit: args.limit,
    };

    let tasks = store.query_tasks(&filter)
        .context("failed to query tasks")?;

    match format {
        OutputFormat::Json => print_json(&tasks)?,
        OutputFormat::Csv => print!("{}", cli::format_tasks_csv(&tasks)?),
        OutputFormat::Table => {
            if tasks.is_empty() {
                eprintln!("no tasks found");
                return Ok(());
            }
            let mut current_file = String::new();
            for task in &tasks {
                let file_str = task.file_path.to_string_lossy();
                if file_str != current_file {
                    if !current_file.is_empty() {
                        println!();
                    }
                    let header = task.file_title.as_deref().unwrap_or(&file_str);
                    println!("{header} ({file_str})");
                    current_file = file_str.to_string();
                }
                println!("  {}", cli::format_task_view(task));
            }
        }
    }
    Ok(())
}

fn run_search(args: &cli::SearchArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    let results = store.search(&args.keyword, args.limit)
        .context("failed to search")?;

    match format {
        OutputFormat::Json => print_json(&results)?,
        OutputFormat::Csv => print!("{}", cli::format_search_csv(&results)?),
        OutputFormat::Table => print!("{}", cli::format_search_results(&results)),
    }
    Ok(())
}

fn run_agenda(args: &cli::AgendaArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let agenda = store.query_agenda(&today)
        .context("failed to query agenda")?;

    match format {
        OutputFormat::Json => print_json(&agenda)?,
        OutputFormat::Csv => print!("{}", cli::format_agenda_csv(&agenda)?),
        OutputFormat::Table => {
            print!(
                "{}",
                cli::format_agenda(
                    &agenda,
                    args.show_overdue(),
                    args.show_today(),
                    args.show_week(),
                )
            );
        }
    }
    Ok(())
}

fn run_status(args: &StatusArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    let stats = store.get_stats()
        .context("failed to query stats")?;

    match format {
        OutputFormat::Json => print_json(&stats)?,
        OutputFormat::Csv => print!("{}", cli::format_stats_csv(&stats)?),
        OutputFormat::Table => println!("{}", cli::format_stats(&stats)),
    }
    Ok(())
}

fn run_files(args: &FilesArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    let files = store.list_files()
        .context("failed to list files")?;

    match format {
        OutputFormat::Json => print_json(&files)?,
        OutputFormat::Csv => print!("{}", cli::format_files_csv(&files)?),
        OutputFormat::Table => {
            if files.is_empty() {
                eprintln!("no indexed files");
                return Ok(());
            }
            for file in &files {
                println!("{}", cli::format_file_view(file));
            }
        }
    }
    Ok(())
}

fn run_deps(args: &cli::DepsArgs) -> Result<()> {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref())?;

    if let Some(file) = &args.file {
        let deps = store.get_file_dependencies(file)
            .context("failed to query dependencies")?
            .ok_or_else(|| anyhow::anyhow!("file not found in index: {}", file.display()))?;

        match format {
            OutputFormat::Json => print_json(&deps)?,
            OutputFormat::Csv => print!("{}", cli::format_deps_csv(&deps)?),
            OutputFormat::Table => print!("{}", cli::format_deps(&deps)),
        }
    } else {
        let all_deps = store.list_file_dependencies()
            .context("failed to list dependencies")?;

        match format {
            OutputFormat::Json => print_json(&all_deps)?,
            OutputFormat::Csv => print!("{}", cli::format_all_deps_csv(&all_deps)?),
            OutputFormat::Table => print!("{}", cli::format_all_deps(&all_deps)),
        }
    }
    Ok(())
}

fn run_check(args: &cli::CheckArgs) -> Result<()> {
    let store = open_query_db(args.db.as_deref())?;

    let task = store.find_task_by_id(&args.task_id)
        .context("failed to find task")?;

    let current_hash = mindtape::store::hash_file(&task.file_path)
        .with_context(|| format!("failed to read file {}", task.file_path.display()))?;

    if current_hash != task.file_hash {
        bail!(
            "file {} has changed since last index\nRun 'mindtape watch' to re-index, then try again.",
            task.file_path.display()
        );
    }

    let source = mindtape::eval::load_source(&task.file_path)
        .with_context(|| format!("failed to load file {}", task.file_path.display()))?;

    let new_content = mindtape::eval::toggle_task_checkbox(&source, &task.task_id)
        .context("failed to toggle checkbox")?;

    std::fs::write(&task.file_path, new_content)
        .with_context(|| format!("failed to write file {}", task.file_path.display()))?;

    let status = if task.is_done { "unchecked" } else { "checked" };
    println!("Task {} {}: {}", task.task_id, status, task.task_title);
    Ok(())
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .context("failed to serialize JSON")?;
    println!("{json}");
    Ok(())
}

/// Open the `SQLite` database for query commands.
///
/// Uses the explicit `--db` override if given, otherwise resolves from
/// config auto-discovery or the default path.
fn open_query_db(db_override: Option<&Path>) -> Result<SqliteStore> {
    let db_path = if let Some(p) = db_override {
        p.to_path_buf()
    } else {
        let cfg = config::find_config()
            .and_then(|p| config::load_config(&p).ok())
            .unwrap_or(config::Config {
                database: None,
                watch: vec![],
            });
        config::resolve_db_path(&cfg)
    };

    if !db_path.exists() {
        bail!(
            "database not found: {}\nRun 'mindtape watch' first to create the index.",
            db_path.display()
        );
    }

    SqliteStore::open(&db_path)
        .with_context(|| format!("failed to open database at {}", db_path.display()))
}

/// Build a Config from CLI args: --config file, path argument, or auto-discovery.
fn load_watch_config(args: &cli::WatchArgs) -> Result<config::Config> {
    if let Some(ref config_path) = args.config {
        return config::load_config(config_path)
            .with_context(|| format!("failed to load config from {}", config_path.display()));
    }

    if let Some(ref path) = args.path {
        return Ok(config::Config {
            database: None,
            watch: vec![config::WatchEntry {
                path: path.to_string_lossy().to_string(),
                recursive: true,
            }],
        });
    }

    if let Some(config_path) = config::find_config() {
        return config::load_config(&config_path)
            .with_context(|| format!("failed to load config from {}", config_path.display()));
    }

    Ok(config::Config {
        database: None,
        watch: vec![config::WatchEntry {
            path: ".".to_string(),
            recursive: true,
        }],
    })
}
