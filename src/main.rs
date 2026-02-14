use std::process;

use mindtape::cli::{self, Command};
use mindtape::config;
use mindtape::eval;
use mindtape::store::SqliteStore;
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
