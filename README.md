# Gira


## Install Binary

#### Requires Cargo to be install. [Installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```sh
git clone https://github.com/m-bal/gira.git
cd gira/
cargo install --path .
```

Cargo installs the binary into `${HOME}/.cargo/bin`

Add the path to your `.bashrc` if it doesn't exist:

```bash
export PATH="$PATH:<path-to-.cargo/bin>"
```

## Configure Gira

Gira needs to know the name of the jira site you are using:
```sh
git config --global jira.host https://<company_name>.atlassian.net
```

In order for Gira to connect to Jira, you need to create an API token using this [guide](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/#Create-an-API-token).

```sh
git config --global jira.token <api_token>
```

### verify
`jira.host` and `jira.token` should appear in this list with the values you provided.
```sh
git config --global --list
```
If you don't see the token and host in the list, run `git config --global --edit` and add them to the file:
```sh
[jira]
    token=<api_token>
    host=<host>
```


#### Jira Email
If your jira account email is different from your git configured `user.email`, then you can set
`jira.email`
```sh
git config --global jira.email <jira-account-email>
```


## Usage

```sh
Usage: gira [COMMAND]

Commands:
  start     Creates a branch based on the jira issue id (PROJECT-ID)
  list      List issues assigned to you (uses jira.email or user.email to filter)
  list-all  List all issues
  bump      bump the branch version (creating a new branch for the current issue)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```sh
Usage: gira start <ISSUE_ID>

Arguments:
  <ISSUE_ID

Options:
  -h, --help  Print help
```

```sh
Usage: gira list [OPTIONS]

Options:
  -p, --project <PROJECT>  Filter by project name
  -f, --filter <FILTER>    Search an issue's Summary, Description, Environment and Comme
nts field
  -h, --help               Print help
```

```sh
Usage: gira list-all [OPTIONS]

Options:
  -p, --project <PROJECT>  Search by project name
  -f, --filter <FILTER>    Search an issue's Summary, Description, Environment and Comme
nts field
  -h, --help               Print help
```
