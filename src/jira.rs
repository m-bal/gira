use reqwest::blocking::Client;
use reqwest::Url;
use serde::{Deserialize, Deserializer};

use crate::config::JiraConfig;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub key: String,
    #[serde(rename = "fields", deserialize_with = "nested_fields_summary")]
    pub title: String,
}

fn nested_fields_summary<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: for<'a> Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Fields {
        summary: String,
    }
    Fields::deserialize(deserializer).map(|fields| fields.summary)
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JiraSearchOutput {
    pub error_messages: Option<Vec<String>>,
    pub issues: Option<Vec<Issue>>,
}

pub struct JiraClient {
    client: reqwest::blocking::Client,
    config: JiraConfig,
}

#[derive(Default)]
pub struct SearchArgs {
    pub project: Option<String>,
    pub assignee: Option<String>,
    pub id: Option<String>,
    pub filter: Option<String>,
}

impl std::fmt::Display for SearchArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut jql_search = "".to_string();
        //TODO: Do this with a macro?
        if self.project.is_some() {
            jql_search.push_str(&format!("project='{}'", self.project.as_ref().unwrap()));
        }
        if self.assignee.is_some() {
            jql_search.push_str(&format!(
                "{}assignee='{}'",
                if jql_search.is_empty() { "" } else { "AND " },
                self.assignee.as_ref().unwrap(),
            ));
        }
        if self.id.is_some() {
            jql_search.push_str(&format!(
                "{}id='{}'",
                if jql_search.is_empty() { "" } else { "AND " },
                self.id.as_ref().unwrap()
            ));
        }
        if self.filter.is_some() {
            jql_search.push_str(&format!(
                "{}text ~ '{}'",
                if jql_search.is_empty() { "" } else { "AND " },
                self.filter.as_ref().unwrap()
            ));
        }

        if jql_search.is_empty() {
            Ok(())
        } else {
            write!(f, "?jql={}", jql_search)
        }
    }
}

impl JiraClient {
    pub fn new(config: JiraConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn issues(self, args: SearchArgs) -> JiraSearchOutput {
        let jql = format!("{}", args);

        let url = Url::parse(&format!(
            "{}{}{}",
            self.config.host,
            String::from("/rest/api/3/search"),
            jql
        ))
        .expect("unable to parse url");
        let resp = self
            .client
            .get(url)
            .basic_auth(&self.config.email, Some(&self.config.api_token))
            .send()
            .expect("Unable to send request to jira");

        serde_json::from_slice(&resp.bytes().unwrap())
            .unwrap_or_else(|err| panic!("Unable to parse response to json {}", err))
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::BufReader};

    use super::*;
    use httpmock::MockServer;
    use serde_json::json;

    fn mock_search_results() -> String {
        let file = File::open("tests/data/search_res.json").unwrap();
        let reader = BufReader::new(file);
        let data = serde_json::from_reader::<_, serde_json::Value>(reader).unwrap();
        data.to_string()
    }

    #[track_caller]
    fn check(
        mock_path: String,
        mock_query: String,
        result_code: u16,
        result_body: String,
        search_args: impl Into<Option<SearchArgs>>,
        expected_response: JiraSearchOutput,
    ) {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            if !mock_query.is_empty() {
                when.path(mock_path).query_param("jql", mock_query);
            } else {
                when.path(mock_path);
            }
            then.status(result_code).body(result_body);
        });

        let config = JiraConfig {
            host: server.base_url(),
            ..Default::default()
        };
        let client = JiraClient::new(config);
        let client_args = search_args.into().unwrap_or(SearchArgs::default());
        let res = client.issues(client_args);
        mock.assert();
        for expected_issue in expected_response.issues.iter().flatten() {
            assert!(
                res.issues
                    .iter()
                    .flatten()
                    .any(|issue| issue.title == expected_issue.title
                        && issue.key == expected_issue.key),
                "Could not find {:?} in response {:#?}",
                expected_issue,
                res
            );
        }
        for expected_error in expected_response.error_messages.iter().flatten() {
            assert!(
                res.error_messages
                    .iter()
                    .flatten()
                    .any(|error| error == expected_error),
                "Could not find {:?} in response {:#?}",
                expected_error,
                res
            );
        }
    }

    #[test]
    fn test_serach_issues() {
        let expected_json = json!({"issues":
            [
                {"key": "CLOUD-2", "fields": {"summary": "test again"}},
                {"key": "CLOUD-1", "fields": {"summary": "test"}},
                {"key": "TEST-1", "fields": {"summary": "Test in Test Project"}},
            ]
        });
        let expected_res: JiraSearchOutput = serde_json::from_value(expected_json).unwrap();
        check(
            "/rest/api/3/search".to_string(),
            "".to_string(),
            200,
            mock_search_results(),
            SearchArgs::default(),
            expected_res,
        );
    }

    #[test]
    fn test_serach_issues_by_id() {
        let issue_id = "CLOUD-1".to_string();
        let search_args = SearchArgs {
            id: Some(issue_id.clone()),
            ..Default::default()
        };
        let expected_json = json!({"issues":
            [
                {"key": "CLOUD-1", "fields": {"summary": "test"}},
            ]
        });
        let expected_res: JiraSearchOutput = serde_json::from_value(expected_json).unwrap();
        check(
            "/rest/api/3/search".to_string(),
            format!("id='{}'", issue_id),
            200,
            mock_search_results(),
            search_args,
            expected_res,
        );
    }

    #[test]
    fn test_serach_issues_with_nonexistent_id() {
        let issue_id = "CLOUD-2".to_string();
        let search_args = SearchArgs {
            id: Some(issue_id.clone()),
            ..Default::default()
        };
        let error_message =
            "Issue does not exist or you do not have permission to set it".to_string();

        let mock_json_body = json!({
            "errorMessages": [error_message]
        });
        let expected_res: JiraSearchOutput =
            serde_json::from_value(mock_json_body.clone()).unwrap();

        check(
            "/rest/api/3/search".to_string(),
            format!("id='{}'", issue_id),
            200,
            mock_json_body.to_string(),
            search_args,
            expected_res,
        );
    }
}
