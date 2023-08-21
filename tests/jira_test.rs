use std::time::Duration;

use crate::jira_api::{JiraApi, JiraState};
use crate::mock_writer::MockWriter;
use hyper::{Client, StatusCode};
use serde_json::Value;
use std::ascii::escape_default;
use std::sync::{Arc, Mutex};
use tokio;

mod jira_api;
mod mock_writer;
use gitra::jira::{JiraClient, SearchArgs};

fn wait_for<F>(mut f: F, mut timeout: Option<Duration>, mut snooze: Option<Duration>) -> bool
where
    F: FnMut() -> bool,
{
    snooze = Some(snooze.unwrap_or(Duration::from_millis(100)));
    timeout = Some(timeout.unwrap_or(Duration::from_secs(5)));

    let deadline = std::time::Instant::now() + timeout.unwrap();
    while std::time::Instant::now() <= deadline {
        if f() {
            return true;
        }
        std::thread::sleep(snooze.unwrap());
    }
    f()
}

async fn start_server(host: String, port: u16) {
    let api = JiraApi::new(host, port, "/".to_string());
    let server_state = api.state.clone();
    let is_server_running = || -> bool {
        matches!(
            Arc::try_unwrap(server_state.clone()).unwrap_or(JiraState::NotRunning),
            JiraState::Running
        )
    };
    tokio::spawn(async { api.run().await });

    wait_for(is_server_running, None, None);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dummy_server() {
    use hyper::Uri;

    let host = "127.0.0.1".to_string();
    let port = 8080;
    let url = "http://".to_string() + &host + &":".to_string() + &port.to_string();
    start_server(host, port).await;

    let client: Client<hyper::client::HttpConnector> = Client::builder().build_http();
    let req_url: Uri = (url + "/alive").parse().unwrap();
    let res = client.get(req_url).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = hyper::body::to_bytes(res).await.unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();
    let json_body = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    assert_eq!(json_body["Alive"].as_bool(), Some(true));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_serach_issues() {
    let host = "127.0.0.1".to_string();
    let port = 8081;
    let url = "http://".to_string() + &host + &":".to_string() + &port.to_string();
    start_server(host, port).await;

    let config = gitra::config::JiraConfig {
        host: url,
        api_token: "".to_string(),
    };
    let client = JiraClient::new(config);
    let _res = client.issues(None).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_issue_by_id() {
    let storage: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let writer = MockWriter::new(storage.clone());
    let _sub = tracing_subscriber::fmt()
        .pretty()
        .compact()
        .with_writer(writer)
        .init();
    let host = "127.0.0.1".to_string();
    let port = 8082;
    let url = "http://".to_string() + &host + &":".to_string() + &port.to_string();
    start_server(host, port).await;

    let config = gitra::config::JiraConfig {
        host: url,
        api_token: "".to_string(),
    };
    let client = JiraClient::new(config);
    let args = SearchArgs {
        project: None,
        assignee: None,
        id: Some("CLOUD-1".to_string()),
    };
    let _res = client.issues(Some(args)).await;
    let shared_data = storage.lock().unwrap();
    let expected = "/rest/api/3/search?jql=id=CLOUD-1".to_string();

    assert!(shared_data.iter().any(|event| event.contains(&expected)));
}
