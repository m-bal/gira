use headers::authorization::Basic;
use headers::{Authorization, HeaderMapExt};
use hyper::client::HttpConnector;
use hyper::{Client, Request, Uri};
use hyper_tls::HttpsConnector;
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
    client: Client<HttpsConnector<HttpConnector>>,
    config: JiraConfig,
    basic_auth: Authorization<Basic>,
}

#[derive(Default)]
pub struct SearchArgs {
    pub project: Option<String>,
    pub assignee: Option<String>,
    pub id: Option<String>,
}

impl SearchArgs {
    fn to_string(self) -> String {
        let mut jql_search = "".to_string();
        //TODO: Do this with a macro?
        if self.project.is_some() {
            jql_search.push_str(format!("project='{}'", self.project.unwrap()).as_ref());
        }
        if self.assignee.is_some() {
            jql_search.push_str(format!("assignee='{}'", self.assignee.unwrap()).as_ref());
        }
        if self.id.is_some() {
            jql_search.push_str(format!("id='{}'", self.id.unwrap()).as_ref());
        }
        jql_search
    }
}

impl JiraClient {
    pub fn new(config: JiraConfig) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder().build(https);
        let basic_auth = Authorization::basic(&config.email, &config.api_token);
        Self {
            client,
            config,
            basic_auth,
        }
    }

    fn extract_search_args(&self, args: SearchArgs) -> String {
        let query = args.to_string();
        if query.is_empty() {
            return "".to_string();
        }
        "?jql=".to_string() + &query
    }

    pub async fn issues(self, args: SearchArgs) -> JiraSearchOutput {
        let jql = self.extract_search_args(args);

        let url = self.config.host + &String::from("/rest/api/3/search") + &jql;
        let request_uri = url
            .parse::<Uri>()
            .unwrap_or_else(|err| panic!("cannot parse uri {:?}", err));
        let mut req = Request::builder()
            .uri(request_uri)
            .method("GET")
            .body(hyper::Body::empty())
            .expect("Request not built");

        let headers = req.headers_mut();
        headers.typed_insert(self.basic_auth);

        let res = self
            .client
            .request(req)
            .await
            .unwrap_or_else(|err| panic!("Unable to make request: {}", err));
        let buf = hyper::body::to_bytes(res)
            .await
            .unwrap_or_else(|err| panic!("Unable to turn body into bytes {}", err));
        serde_json::from_slice(&buf)
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

    async fn check(
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
        let res = client.issues(client_args).await;
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

    #[tokio::test]
    async fn test_serach_issues() {
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
        )
        .await;
    }

    #[tokio::test]
    async fn test_serach_issues_by_id() {
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
            format!("id={}", issue_id),
            200,
            mock_search_results(),
            search_args,
            expected_res,
        )
        .await;
    }

    #[tokio::test]
    async fn test_serach_issues_with_nonexistent_id() {
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
            format!("id={}", issue_id),
            200,
            mock_json_body.to_string(),
            search_args,
            expected_res,
        )
        .await;
    }
}
