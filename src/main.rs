mod cli;
mod manifest;
mod tags;

use clap::Parser;

fn main() {
    let args = cli::CLI::parse();
    println!("{:?}", args);
}
