use std::net::ToSocketAddrs;
use std::{convert::Infallible, fs::File, io::BufReader, net::SocketAddr, sync::Arc};
use tracing::{debug, error, info, span, warn, Level};

use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use hyper::{Body, Error, Request, Response, Server};

#[derive(Clone, Debug)]
pub enum JiraState {
    Running,
    NotRunning,
}

#[derive(Clone)]
pub struct JiraApi {
    endpoint: String,
    addr: SocketAddr,
    pub state: Arc<JiraState>,
}

impl JiraApi {
    pub fn new(addr: String, port: u16, ep: String) -> Self {
        let addr = SocketAddr::new(addr.parse().unwrap(), port);
        Self {
            endpoint: ep,
            addr,
            state: Arc::new(JiraState::NotRunning),
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        let service = make_service_fn(|_conn: &AddrStream| async {
            Ok::<_, Infallible>(service_fn(Self::handle))
        });
        let server = Server::bind(&self.addr).serve(service);
        self.state = JiraState::Running.into();
        server.await
    }

    pub fn ep(self) -> String {
        self.endpoint
    }

    #[tracing::instrument]
    async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
        info!(request = req.uri().to_string(), "Request handler");
        match req.uri().path() {
            "/alive" => Ok(Response::new(Body::from(
                serde_json::to_string(&serde_json::json!({"Alive": true})).unwrap(),
            ))),
            "/rest/api/3/search" => Ok(Self::handle_search(req)),
            _ => return Ok(Response::new(Body::from(""))),
        }
    }

    fn handle_search(_req: Request<Body>) -> Response<Body> {
        let file = File::open("tests/data/search_res.json").unwrap();
        let reader = BufReader::new(file);
        let data = serde_json::from_reader::<_, serde_json::Value>(reader).unwrap();
        return Response::new(Body::from(data.to_string()));
    }
}
