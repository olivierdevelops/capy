//! Differential-testing aid: mirrors `cmd/runcmd` (Go).
use capy_core::orchestrator::commands::run_command;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 2 {
        eprintln!("usage: runcmd <lib> <cmd> [args...]");
        std::process::exit(2);
    }
    if let Err(e) = run_command(&a[0], &a[1], &a[2..]) {
        println!("RUNCMDERR\t{}", e);
        std::process::exit(1);
    }
}
