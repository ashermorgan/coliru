mod cli;
mod core;
mod local;
mod manifest;
mod ssh;
mod tags;

#[cfg(test)]
#[path = "../tests/common/mod.rs"]
mod common; // Re-use e2e test utils for integration tests

fn main() {
    cli::run();
}
