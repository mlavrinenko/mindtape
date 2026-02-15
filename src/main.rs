use std::path::Path;
use std::process;

use clap::Parser;

use mindtape::cli::{self, Cli, Command, OutputFormat, StatusArgs, FilesArgs};
use mindtape::config;
use mindtape::eval;
use mindtape::store::{SqliteStore, Store, TaskFilter};
use mindtape::watcher::Watcher;
use mindtape::world;

fn main() {
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
                let _ = Cli::parse_from(["mindtape", "--help"]);
                process::exit(1);
            };
            run_eval(&file, cli.due, cli.limit);
        }
    }
}

fn run_eval(file: &Path, due: bool, limit: Option<usize>) {
    if !file.exists() {
        eprintln!("Error: file not found: {}", file.display());
        process::exit(1);
    }

    let world = match world::MindTapeWorld::new(file) {
        Ok(world) => world,
        Err(err) => {
            eprintln!("Error creating world: {err}");
            process::exit(1);
        }
    };

    let tasks = match eval::eval_file(&world) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error evaluating file: {err}");
            process::exit(1);
        }
    };

    let tasks = cli::filter_and_sort(tasks, due, limit);

    for task in &tasks {
        println!("{}", cli::format_task(task));
    }
}

fn run_watch(args: &cli::WatchArgs) {
    let cfg = load_watch_config(args);

    if cfg.watch.is_empty() {
        eprintln!("Error: no watch paths configured");
        eprintln!("Usage: mindtape watch <path>");
        eprintln!("   or: mindtape watch --config <file>");
        process::exit(1);
    }

    let db_path = config::resolve_db_path(&cfg);
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Error creating database directory: {e}");
                process::exit(1);
            }
        }
    }

    let store = match SqliteStore::open(&db_path) {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Error opening database: {err}");
            process::exit(1);
        }
    };

    let mut watcher = match Watcher::new(store, &cfg.watch) {
        Ok(watcher) => watcher,
        Err(err) => {
            eprintln!("Error setting up watcher: {err}");
            process::exit(1);
        }
    };

    let scan = watcher.initial_scan();
    eprintln!(
        "initial scan: {} found, {} indexed, {} skipped, {} errors",
        scan.found, scan.indexed, scan.skipped, scan.errors,
    );

    if let Err(e) = watcher.run() {
        eprintln!("Watcher error: {e}");
        process::exit(1);
    }
}

fn run_list(args: &cli::ListArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    let filter = TaskFilter {
        done: args.done_filter(),
        tag: args.tag.clone(),
        due_before: args.due_before.clone(),
        file_path: args.file.clone(),
        folder: args.folder.clone(),
        limit: args.limit,
    };

    let tasks = match store.query_tasks(&filter) {
        Ok(tasks) => tasks,
        Err(err) => {
            eprintln!("Error querying tasks: {err}");
            process::exit(1);
        }
    };

    match format {
        OutputFormat::Json => {
            print_json(&tasks);
        }
        OutputFormat::Csv => print_csv(cli::format_tasks_csv(&tasks)),
        OutputFormat::Table => {
            if tasks.is_empty() {
                eprintln!("no tasks found");
                return;
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
}

fn run_search(args: &cli::SearchArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    let results = match store.search(&args.keyword, args.limit) {
        Ok(results) => results,
        Err(err) => {
            eprintln!("Error searching: {err}");
            process::exit(1);
        }
    };

    match format {
        OutputFormat::Json => print_json(&results),
        OutputFormat::Csv => print_csv(cli::format_search_csv(&results)),
        OutputFormat::Table => print!("{}", cli::format_search_results(&results)),
    }
}

fn run_agenda(args: &cli::AgendaArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let agenda = match store.query_agenda(&today) {
        Ok(agenda) => agenda,
        Err(err) => {
            eprintln!("Error querying agenda: {err}");
            process::exit(1);
        }
    };

    match format {
        OutputFormat::Json => print_json(&agenda),
        OutputFormat::Csv => print_csv(cli::format_agenda_csv(&agenda)),
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
}

fn run_status(args: &StatusArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    let stats = match store.get_stats() {
        Ok(stats) => stats,
        Err(err) => {
            eprintln!("Error querying stats: {err}");
            process::exit(1);
        }
    };

    match format {
        OutputFormat::Json => print_json(&stats),
        OutputFormat::Csv => print_csv(cli::format_stats_csv(&stats)),
        OutputFormat::Table => println!("{}", cli::format_stats(&stats)),
    }
}

fn run_files(args: &FilesArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    let files = match store.list_files() {
        Ok(files) => files,
        Err(err) => {
            eprintln!("Error listing files: {err}");
            process::exit(1);
        }
    };

    match format {
        OutputFormat::Json => print_json(&files),
        OutputFormat::Csv => print_csv(cli::format_files_csv(&files)),
        OutputFormat::Table => {
            if files.is_empty() {
                eprintln!("no indexed files");
                return;
            }
            for file in &files {
                println!("{}", cli::format_file_view(file));
            }
        }
    }
}

fn run_deps(args: &cli::DepsArgs) {
    let format = args.query.output_format();
    let store = open_query_db(args.query.db.as_deref());

    if let Some(file) = &args.file {
        let deps = match store.get_file_dependencies(file) {
            Ok(Some(deps)) => deps,
            Ok(None) => {
                eprintln!("Error: file not found in index: {}", file.display());
                process::exit(1);
            }
            Err(err) => {
                eprintln!("Error querying dependencies: {err}");
                process::exit(1);
            }
        };

        match format {
            OutputFormat::Json => print_json(&deps),
            OutputFormat::Csv => print_csv(cli::format_deps_csv(&deps)),
            OutputFormat::Table => print!("{}", cli::format_deps(&deps)),
        }
    } else {
        let all_deps = match store.list_file_dependencies() {
            Ok(all_deps) => all_deps,
            Err(err) => {
                eprintln!("Error listing dependencies: {err}");
                process::exit(1);
            }
        };

        match format {
            OutputFormat::Json => print_json(&all_deps),
            OutputFormat::Csv => print_csv(cli::format_all_deps_csv(&all_deps)),
            OutputFormat::Table => print!("{}", cli::format_all_deps(&all_deps)),
        }
    }
}

fn run_check(args: &cli::CheckArgs) {
    let store = open_query_db(args.db.as_deref());

    let task = match store.find_task_by_id(&args.task_id) {
        Ok(task) => task,
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(1);
        }
    };

    let current_hash = match mindtape::store::hash_file(&task.file_path) {
        Ok(hash) => hash,
        Err(err) => {
            eprintln!("Error reading file {}: {err}", task.file_path.display());
            process::exit(1);
        }
    };

    if current_hash != task.file_hash {
        eprintln!(
            "Error: file {} has changed since last index",
            task.file_path.display()
        );
        eprintln!("Run 'mindtape watch' to re-index, then try again.");
        process::exit(1);
    }

    let source = match mindtape::eval::load_source(&task.file_path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("Error loading file: {err}");
            process::exit(1);
        }
    };

    let new_content = match mindtape::eval::toggle_task_checkbox(&source, &task.task_id) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error toggling checkbox: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = std::fs::write(&task.file_path, new_content) {
        eprintln!("Error writing file: {err}");
        process::exit(1);
    }

    let status = if task.is_done { "unchecked" } else { "checked" };
    println!("Task {} {}: {}", task.task_id, status, task.task_title);
}

fn print_csv(result: Result<String, csv::Error>) {
    match result {
        Ok(csv) => print!("{csv}"),
        Err(err) => {
            eprintln!("Error formatting CSV: {err}");
            process::exit(1);
        }
    }
}

fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            eprintln!("Error serializing JSON: {err}");
            process::exit(1);
        }
    }
}

/// Open the `SQLite` database for query commands.
///
/// Uses the explicit `--db` override if given, otherwise resolves from
/// config auto-discovery or the default path.
fn open_query_db(db_override: Option<&Path>) -> SqliteStore {
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
        eprintln!("Error: database not found: {}", db_path.display());
        eprintln!("Run 'mindtape watch' first to create the index.");
        process::exit(1);
    }

    match SqliteStore::open(&db_path) {
        Ok(store) => store,
        Err(err) => {
            eprintln!("Error opening database: {err}");
            process::exit(1);
        }
    }
}

/// Build a Config from CLI args: --config file, path argument, or auto-discovery.
fn load_watch_config(args: &cli::WatchArgs) -> config::Config {
    if let Some(ref config_path) = args.config {
        match config::load_config(config_path) {
            Ok(cfg) => return cfg,
            Err(err) => {
                eprintln!("Error loading config {}: {err}", config_path.display());
                process::exit(1);
            }
        }
    }

    if let Some(ref path) = args.path {
        return config::Config {
            database: None,
            watch: vec![config::WatchEntry {
                path: path.to_string_lossy().to_string(),
                recursive: true,
            }],
        };
    }

    if let Some(config_path) = config::find_config() {
        match config::load_config(&config_path) {
            Ok(cfg) => return cfg,
            Err(err) => {
                eprintln!("Error loading config {}: {err}", config_path.display());
                process::exit(1);
            }
        }
    }

    config::Config {
        database: None,
        watch: vec![config::WatchEntry {
            path: ".".to_string(),
            recursive: true,
        }],
    }
}
