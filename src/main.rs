mod cli;
mod config;
pub mod jira;

use anyhow::Result;
use clap::Parser;
use cli::Opts;
use std::process::ExitCode;

fn main() {
    let _ = run();
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();
    print!("{}\n", opts.list);
    return Ok(ExitCode::SUCCESS);
}
