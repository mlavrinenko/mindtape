use std::path::Path;
use std::process;

use mindtape::cli::{self, Command};
use mindtape::config;
use mindtape::eval;
use mindtape::store::{SqliteStore, Store, TaskFilter};
use mindtape::watcher::Watcher;
use mindtape::world;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let command = match cli::parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    match command {
        Command::Eval(args) => run_eval(args),
        Command::Watch(args) => run_watch(args),
        Command::List(args) => run_list(args),
        Command::Status(args) => run_status(args),
        Command::Files(args) => run_files(args),
    }
}

fn run_eval(args: cli::EvalArgs) {
    if !args.file.exists() {
        eprintln!("Error: file not found: {}", args.file.display());
        process::exit(1);
    }

    let world = match world::MindTapeWorld::new(&args.file) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error creating world: {e}");
            process::exit(1);
        }
    };

    let tasks = match eval::eval_file(&world) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error evaluating file: {e}");
            process::exit(1);
        }
    };

    let tasks = cli::filter_and_sort(tasks, args.due, args.limit);

    for task in &tasks {
        println!("{}", cli::format_task(task));
    }
}

fn run_watch(args: cli::WatchArgs) {
    let cfg = load_watch_config(&args);

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
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {e}");
            process::exit(1);
        }
    };

    let mut watcher = match Watcher::new(store, &cfg.watch) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error setting up watcher: {e}");
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

fn run_list(args: cli::ListArgs) {
    let store = open_query_db(args.db.as_deref());

    let filter = TaskFilter {
        // Default: show pending only. --status all overrides to show everything.
        done: if args.status_all { None } else { args.done.or(Some(false)) },
        tag: args.tag,
        due_before: args.due_before,
        file_path: args.file,
        folder: args.folder,
        limit: args.limit,
    };

    let tasks = match store.query_tasks(&filter) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error querying tasks: {e}");
            process::exit(1);
        }
    };

    if tasks.is_empty() {
        eprintln!("no tasks found");
        return;
    }

    // Group by file
    let mut current_file = String::new();
    for task in &tasks {
        let file_str = task.file_path.to_string_lossy();
        if file_str != current_file {
            if !current_file.is_empty() {
                println!();
            }
            let header = task.file_title.as_deref().unwrap_or(&file_str);
            println!("{header} ({})", file_str);
            current_file = file_str.to_string();
        }
        println!("  {}", cli::format_task_view(task));
    }
}

fn run_status(args: cli::QueryArgs) {
    let store = open_query_db(args.db.as_deref());

    let stats = match store.get_stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error querying stats: {e}");
            process::exit(1);
        }
    };

    println!("{}", cli::format_stats(&stats));
}

fn run_files(args: cli::QueryArgs) {
    let store = open_query_db(args.db.as_deref());

    let files = match store.list_files() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error listing files: {e}");
            process::exit(1);
        }
    };

    if files.is_empty() {
        eprintln!("no indexed files");
        return;
    }

    for file in &files {
        println!("{}", cli::format_file_view(file));
    }
}

/// Open the SQLite database for query commands.
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
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {e}");
            process::exit(1);
        }
    }
}

/// Build a Config from CLI args: --config file, path argument, or auto-discovery.
fn load_watch_config(args: &cli::WatchArgs) -> config::Config {
    // Explicit --config flag takes priority.
    if let Some(ref config_path) = args.config {
        match config::load_config(config_path) {
            Ok(c) => return c,
            Err(e) => {
                eprintln!("Error loading config {}: {e}", config_path.display());
                process::exit(1);
            }
        }
    }

    // If a path argument was given, build a minimal config from it.
    if let Some(ref path) = args.path {
        return config::Config {
            database: None,
            watch: vec![config::WatchEntry {
                path: path.to_string_lossy().to_string(),
                recursive: true,
            }],
        };
    }

    // Try auto-discovery.
    if let Some(config_path) = config::find_config() {
        match config::load_config(&config_path) {
            Ok(c) => return c,
            Err(e) => {
                eprintln!("Error loading config {}: {e}", config_path.display());
                process::exit(1);
            }
        }
    }

    // Build config watching current directory.
    config::Config {
        database: None,
        watch: vec![config::WatchEntry {
            path: ".".to_string(),
            recursive: true,
        }],
    }
}
