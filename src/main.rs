use std::process;

use mindtape::cli;
use mindtape::eval;
use mindtape::world;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let args = match cli::parse_args(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

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
