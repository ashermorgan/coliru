//! A minimal, flexible, dotfile installer

mod cli;
mod core;
mod local;
mod manifest;
mod ssh;

#[cfg(test)]
#[path = "../tests/test_utils/mod.rs"]
mod test_utils; // Re-use E2E test utils for integration tests

fn main() {
    cli::run();
}
