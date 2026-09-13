//! Mirrors `cmd/runcli` (Go): drives AppOrchestrator::run_cli.
use capy_core::orchestrator::app::AppOrchestrator;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let code = AppOrchestrator.run_cli(&a[0], &a[1]);
    println!("[exit {}]", code);
}
