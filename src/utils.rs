use std::process::{self, Command};

pub fn git_email() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git config user.email")
        .output()
        .expect("Failed to run git config");

    if !output.status.success() {
        panic!(
            "git config user.email failed with exit code {}",
            output.status.code().expect("No status code")
        );
    }

    if output.stdout.is_empty() {
        panic!("git config user.email returned nothing please config git email",);
    }
    String::from_utf8(output.stdout)
        .expect("unable to convert stdout to String")
        .trim()
        .to_string()
}

pub fn git_jira_token() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git config --global jira.token")
        .output()
        .expect("Failed to run git config");

    if !output.status.success() {
        panic!(
            "git config jira.token failed with exit code {}",
            output.status.code().expect("No status code")
        );
    }

    if output.stdout.is_empty() {
        panic!("git config --global jira.token returned nothing please config it with git config --global --edit",);
    }
    String::from_utf8(output.stdout)
        .expect("unable to convert stdout to String")
        .trim()
        .to_string()
}

pub fn git_jira_host() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("git config --global jira.host")
        .output()
        .expect("Failed to run git config");

    if !output.status.success() {
        panic!(
            "git config jira.host failed with exit code {}",
            output.status.code().expect("No status code")
        );
    }

    if output.stdout.is_empty() {
        panic!("git config --global jira.host returned nothing please config it with git config --global --edit",);
    }
    String::from_utf8(output.stdout)
        .expect("unable to convert stdout to String")
        .trim()
        .to_string()
}

pub fn normalize_title(title: &String) -> String {
    title
        .trim()
        .split_whitespace()
        .map(|word| word.to_string())
        .collect::<Vec<String>>()
        .join("-")
}

pub fn git_make_branch(branch_name: String) -> Result<process::ExitCode, String> {
    let command = format!("git checkout -b {}", branch_name);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command.clone())
        .output()
        .expect("Failed to run git checkout");

    if !output.status.success() {
        return Err(format!(
            "{} failed with status code {} and message {}",
            command,
            output.status.code().expect("No status code"),
            String::from_utf8(output.stderr).unwrap_or("".to_string()),
        ));
    }

    Ok(process::ExitCode::SUCCESS)
}
