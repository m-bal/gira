mod cli;
mod config;
pub mod jira;

use anyhow::Result;
use clap::Parser;
use cli::{Command, Opts};
use config::JiraConfig;
use gira::utils;
use jira::{JiraClient, SearchArgs};
use std::process::ExitCode;
use tokio::runtime::Runtime;

fn main() {
    let _ = run();
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();

    let rt = Runtime::new().unwrap();
    let mut s = SearchArgs::default();
    let jira_config = JiraConfig {
        host: utils::git_jira_host(),
        email: utils::git_email(),
        api_token: utils::git_jira_token(),
    };
    let client = JiraClient::new(jira_config);

    match &opts.command {
        Command::Start { issue_id } => {
            s.id = Some(issue_id.to_string());
            //TODO: block with a timer.
            let res = rt.block_on(client.issues(s));
            match res.error_messages {
                Some(err) => {
                    println!("Failed to get any data from jira {}", err.join("\n"));
                    return Ok(ExitCode::FAILURE);
                }
                _ => (),
            }
            match res.issues {
                None => {
                    println!("Could not find any issues related to id {}", issue_id);
                    return Ok(ExitCode::FAILURE);
                }
                _ => (),
            }
            if res.issues.as_ref().is_some_and(|issues| issues.len() > 1) {
                println!("Found more than one issue:");
                for issue in res.issues.as_ref().unwrap() {
                    println!("{}-{}", issue.key, utils::normalize_title(&issue.title));
                }
            }
            let first_issue = &res.issues.unwrap()[0];
            let branch_created = utils::git_make_branch(format!(
                "{}-{}",
                first_issue.key,
                utils::normalize_title(&first_issue.title)
            ));

            return Ok(branch_created.unwrap_or_else(|err| {
                println!("{}", err);
                return std::process::ExitCode::FAILURE;
            }));
        }
        Command::List => {
            s.assignee = Some(utils::git_email());
            let res = rt.block_on(client.issues(s));
            match res.error_messages {
                Some(err) => {
                    println!("Failed to get any data from jira {}", err.join("\n"));
                    return Ok(ExitCode::FAILURE);
                }
                _ => (),
            }
            match res.issues {
                None => {
                    println!("You do not have any assigned issues");
                    return Ok(ExitCode::FAILURE);
                }
                _ => (),
            }
            for issue in res.issues.iter().flatten() {
                println!("{}-{}", issue.key, utils::normalize_title(&issue.title));
            }
        }
        Command::Bump => {
            let branch_name = utils::current_branch_name();
            match branch_name {
                Ok(name) => {
                    let bumped_branch_name = utils::bump_branch(name);
                    let branch_created = utils::git_make_branch(bumped_branch_name);
                    return Ok(branch_created.unwrap_or_else(|err| {
                        println!("{}", err);
                        return std::process::ExitCode::FAILURE;
                    }));
                }
                Err(err) => {
                    println!("{}", err);
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
    }
    return Ok(ExitCode::SUCCESS);
}
