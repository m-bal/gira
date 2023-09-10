mod cli;
mod config;
pub mod jira;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Command, Opts};
use config::JiraConfig;
use gira::utils;
use jira::{JiraClient, JiraSearchOutput, SearchArgs};
use std::process::ExitCode;

fn main() {
    let _ = run();
}

fn run() -> Result<ExitCode> {
    let opts = Opts::parse();

    let mut s = SearchArgs::default();
    let email = utils::retrive_git_config("jira.email").unwrap_or_else(|_| {
        utils::retrive_git_config("user.email")
            .expect("Unable to retrieve jira.email or user.email")
    });
    let jira_config = JiraConfig {
        host: utils::retrive_git_config("jira.host").unwrap(),
        email: email.clone(),
        api_token: utils::retrive_git_config("jira.token").unwrap(),
    };
    let client = JiraClient::new(jira_config);
    if opts.subcommands.is_none() {
        let _ = Opts::command().print_help();
        return Ok(ExitCode::FAILURE);
    }

    match opts.subcommands.unwrap() {
        Command::Start { issue_id } => {
            s.id = Some(issue_id.clone());
            //TODO: block with a timer.
            let res = client.issues(s);
            if let Some(err) = res.error_messages {
                eprintln!("Failed to get any data from jira {}", err.join("\n"));
                return Ok(ExitCode::FAILURE);
            }
            if res.issues.is_none() {
                eprintln!("Could not find any issues related to id {}", issue_id);
                return Ok(ExitCode::FAILURE);
            }
            if res.issues.as_ref().is_some_and(|issues| issues.len() > 1) {
                eprintln!("Found more than one issue:");
                for issue in res.issues.as_ref().unwrap() {
                    eprintln!("{}-{}", issue.key, utils::normalize_title(&issue.title));
                }
                return Ok(ExitCode::FAILURE);
            }
            let first_issue = &res.issues.unwrap()[0];
            let branch_created = utils::git_make_branch(
                format!(
                    "{}-{}",
                    first_issue.key,
                    utils::normalize_title(&first_issue.title)
                )
                .as_str(),
            );

            Ok(branch_created.unwrap_or_else(|err| {
                eprintln!("{}", err);
                ExitCode::FAILURE
            }))
        }
        Command::List { project, filter } => {
            s.assignee = Some(email);
            s.project = project;
            s.filter = filter;
            let res = client.issues(s);
            list_issues(res)
        }
        Command::ListAll { project, filter } => {
            s.project = project;
            s.filter = filter;
            let res = client.issues(s);
            list_issues(res)
        }
        Command::Bump => {
            let branch_name = utils::current_branch_name();
            match branch_name {
                Ok(name) => {
                    let bumped_branch_name = utils::bump_branch(&name);
                    let branch_created = utils::git_make_branch(&bumped_branch_name);
                    Ok(branch_created.unwrap_or_else(|err| {
                        eprintln!("{}", err);
                        ExitCode::FAILURE
                    }))
                }
                Err(err) => {
                    eprintln!("{}", err);
                    Ok(ExitCode::FAILURE)
                }
            }
        }
    }
}

fn list_issues(res: JiraSearchOutput) -> Result<ExitCode> {
    if let Some(err) = res.error_messages {
        eprintln!("Failed to get any data from jira {}", err.join("\n"));
        return Ok(ExitCode::FAILURE);
    }
    if res.issues.is_none() {
        eprintln!("You do not have any assigned issues");
        return Ok(ExitCode::SUCCESS);
    }
    for issue in res.issues.iter().flatten() {
        let text = format!("{}-{}", issue.key, utils::normalize_title(&issue.title));
        let url = format!(
            "{}/browse/{}",
            utils::retrive_git_config("jira.host").unwrap(),
            issue.key
        );
        eprintln!("{}", utils::convert_str_to_link(&url, &text))
    }
    Ok(ExitCode::SUCCESS)
}
