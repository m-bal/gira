use hyper::client::HttpConnector;
use hyper::{Client, Uri};
use serde_json::Error;
use tracing::{debug, error, info, span, warn, Level};

use crate::config::JiraConfig;

pub struct JiraClient {
    client: Client<HttpConnector>,
    config: JiraConfig,
}

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
            jql_search.push_str(format!("project={}", self.project.unwrap()).as_ref());
        }
        if self.assignee.is_some() {
            jql_search.push_str(format!("assignee={}", self.assignee.unwrap()).as_ref());
        }
        if self.id.is_some() {
            jql_search.push_str(format!("id={}", self.id.unwrap()).as_ref());
        }
        jql_search
    }
}

impl JiraClient {
    pub fn new(config: JiraConfig) -> Self {
        let client = Client::builder().build_http();
        Self { client, config }
    }

    fn extract_search_args(&self, args: SearchArgs) -> String {
        let query = args.to_string();
        if query.is_empty() {
            return "".to_string();
        }
        "?jql=".to_owned() + &query
    }

    pub async fn issues(self, args: Option<SearchArgs>) -> Result<serde_json::Value, Error> {
        // TODO: make request_uri somewhere before?
        let mut jql = String::from("");
        match args {
            Some(a) => jql = self.extract_search_args(a),
            _ => info!("No search args"),
        }
        let url = self.config.host + &String::from("/rest/api/3/search") + &jql;
        let request_uri = url
            .parse::<Uri>()
            .unwrap_or_else(|err| panic!("cannot parse uri {:?}", err));
        let future = self.client.get(request_uri);
        let res = future.await;
        let buf = hyper::body::to_bytes(res.unwrap()).await.unwrap();
        let json = serde_json::from_slice(&buf);
        json
    }
}
