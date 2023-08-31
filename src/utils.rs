use std::process::{self, Command};

use regex::Regex;

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

pub fn current_branch_name() -> Result<String, String> {
    let command = "git branch --show-current";
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
    Ok(String::from_utf8(output.stdout)
        .expect("unable to convert stdout to String")
        .trim()
        .to_string())
}

pub fn bump_branch(name: String) -> String {
    let split_name: Vec<&str> = name.split(".").collect();
    if split_name.len() < 2 {
        return name + ".v1";
    }
    let re = Regex::new(r"v(?<num>[0-9]+)$").unwrap();
    let Some(cap) = re.captures(split_name.last().unwrap()) else {
        return name + ".v1";
    };
    let num = cap["num"]
        .parse::<u8>()
        .expect("Unable to parse branch name");
    return split_name[0..split_name.len() - 1].join(".") + format!(".v{}", num + 1).as_str();
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    #[test_case("test_branch", "test_branch.v1"; "initial bump")]
    #[test_case("test.branch", "test.branch.v1"; "initial bump with dot")]
    #[test_case("test_branch.v1", "test_branch.v2"; "bump from v1 to v2")]
    #[test_case("test.branch.v1", "test.branch.v2"; "bump from v1 to v2 with dot")]
    fn test_bumpping(branch_name: &str, expected: &str) {
        assert_eq!(super::bump_branch(branch_name.to_string()), expected)
    }
}
