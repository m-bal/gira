use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub subcommands: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Creates a branch based on the jira issue id (PROJECT-ID)
    Start {
        #[arg(value_parser = issue_validator)]
        issue_id: String,
    },
    /// List issues assigned to you (uses jira.email or user.email to filter)
    List {
        /// Filter by project name
        #[arg(short, long)]
        project: Option<String>,
        /// Search an issue's Summary, Description, Environment and Comments field
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// List all issues
    ListAll {
        /// Search by project name
        #[arg(short, long)]
        project: Option<String>,
        /// Search an issue's Summary, Description, Environment and Comments field
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// bump the branch version (creating a new branch for the current issue)
    Bump,
}

fn issue_validator(s: &str) -> Result<String, String> {
    let split_issue_id = s.split_once('-');
    if split_issue_id.is_none() {
        return Err("Must be of form <team-name>-<id> example: CLOUD-1".to_string());
    }
    let (first, sec) = split_issue_id.unwrap();
    if first.chars().all(|c| c.is_alphabetic()) && sec.parse::<u64>().is_ok() {
        return Ok(s.to_string());
    }
    Err("Must be of form <team-name>-<id> example: CLOUD-1".to_string())
}
