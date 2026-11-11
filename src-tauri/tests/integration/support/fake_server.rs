use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Call {
    pub method: String,
    pub path: String,
    pub form: String,
    pub authorization: Option<String>,
}

impl Call {
    pub fn query_value(&self, key: &str) -> Option<String> {
        let query = self.path.split_once('?')?.1;
        let prefix = format!("{key}=");
        query
            .split('&')
            .find_map(|pair| pair.strip_prefix(prefix.as_str()))
            .and_then(|raw| urlencoding::decode(raw).ok())
            .map(|value| value.into_owned())
    }

    pub fn route(&self) -> &str {
        self.path.split('?').next().unwrap_or(&self.path)
    }

    pub fn form_value(&self, key: &str) -> Option<String> {
        let prefix = format!("{key}=");
        self.form
            .split('&')
            .find_map(|pair| pair.strip_prefix(prefix.as_str()))
            .and_then(|raw| urlencoding::decode(raw).ok())
            .map(|value| value.into_owned())
    }
}

pub struct Reply {
    pub status: u16,
    pub body: String,
    pub headers: Vec<(&'static str, String)>,
}

impl Reply {
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
            headers: Vec::new(),
        }
    }

    pub fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }
}

pub struct FakeServer {
    pub base_url: String,
    calls: Arc<Mutex<Vec<Call>>>,
    server: Arc<tiny_http::Server>,
}

impl FakeServer {
    pub fn start(reply: impl Fn(&Call) -> (u16, String) + Send + 'static) -> Self {
        Self::start_replying(move |call| {
            let (status, body) = reply(call);
            Reply::new(status, body)
        })
    }

    pub fn start_replying(reply: impl Fn(&Call) -> Reply + Send + 'static) -> Self {
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").expect("port local"));
        let port = server.server_addr().to_ip().expect("adresse IP").port();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = calls.clone();
        let listener = server.clone();
        std::thread::spawn(move || {
            for mut request in listener.incoming_requests() {
                let mut form = String::new();
                request
                    .as_reader()
                    .read_to_string(&mut form)
                    .expect("corps lisible");
                let call = Call {
                    method: request.method().to_string(),
                    path: request.url().to_string(),
                    form,
                    authorization: request
                        .headers()
                        .iter()
                        .find(|h| h.field.equiv("Authorization"))
                        .map(|h| h.value.to_string()),
                };
                seen.lock().unwrap().push(call.clone());
                let planned = reply(&call);
                let mut response = tiny_http::Response::from_string(planned.body)
                    .with_status_code(tiny_http::StatusCode(planned.status))
                    .with_header(
                        "Content-Type: application/json"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    );
                for (name, value) in planned.headers {
                    response = response.with_header(
                        format!("{name}: {value}")
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    );
                }
                request.respond(response).expect("reponse envoyee");
            }
        });
        Self {
            base_url: format!("http://127.0.0.1:{port}"),
            calls,
            server,
        }
    }

    pub fn calls_on(&self, route: &str) -> Vec<Call> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.route() == route)
            .cloned()
            .collect()
    }

    pub fn calls_to(&self, path: &str) -> Vec<Call> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.path == path)
            .cloned()
            .collect()
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

pub fn fixture(dir: &str, name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(dir)
            .join(name),
    )
    .unwrap_or_else(|e| panic!("fixture {dir}/{name} : {e}"))
}
