use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    /// List all issues assigned to you (uses configured git.email to filter)
    #[arg(short, long)]
    pub list: bool,

    /// Creates a branch based on the jira issue id (PROJECT-ID)
    #[arg(short, long)]
    pub start: String,
}
