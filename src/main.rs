mod eval;
mod world;

use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut file: Option<PathBuf> = None;
    let mut due = false;
    let mut limit: Option<usize> = None;

    for arg in &args {
        if arg == "--due" {
            due = true;
        } else if arg.starts_with('-') && arg[1..].parse::<usize>().is_ok() {
            limit = Some(arg[1..].parse().unwrap());
        } else {
            file = Some(PathBuf::from(arg));
        }
    }

    let file = match file {
        Some(f) => f,
        None => {
            eprintln!("Usage: mindtape <file.typ> [--due] [-N]");
            process::exit(1);
        }
    };

    if !file.exists() {
        eprintln!("Error: file not found: {}", file.display());
        process::exit(1);
    }

    let world = match world::MindTapeWorld::new(&file) {
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

    // Filter out completed tasks
    let mut tasks: Vec<_> = tasks.into_iter().filter(|t| !t.done).collect();

    // If --due: keep only tasks with due dates, sort by due date ascending
    if due {
        tasks.retain(|t| t.due.is_some());
        tasks.sort_by(|a, b| {
            let a_due = a.due.as_ref().unwrap();
            let b_due = b.due.as_ref().unwrap();
            due_sort_key(a_due).cmp(&due_sort_key(b_due))
        });
    }

    // Apply limit
    if let Some(n) = limit {
        tasks.truncate(n);
    }

    // Print results
    for task in &tasks {
        match &task.due {
            Some(dt) => {
                println!("- (due {}) {}", format_due(dt), task.title);
            }
            None => {
                println!("- {}", task.title);
            }
        }
    }
}

fn format_due(dt: &typst::foundations::Datetime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}

fn due_sort_key(dt: &typst::foundations::Datetime) -> (i32, u8, u8) {
    (
        dt.year().unwrap_or(0),
        dt.month().unwrap_or(0),
        dt.day().unwrap_or(0),
    )
}
