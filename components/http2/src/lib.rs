use api::serde::Deserialize;

struct Component;

pub struct Actor {
    domain: String,
    path: String,
    port: u16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(crate="api::serde")]
struct Config {
    domain: String,

    #[serde(default)]
    path: String,

    #[serde(default)]
    port: u16,

    outputs: Vec<String>,
    function_name: String,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let config: Config = api::parse_args(&arguments).unwrap();

        Box::new(Actor::new(config))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["http2_client"]
    }

    fn name(&'static self) -> &'static str {
        "http2"
    }
}

impl Actor {
    fn new(config: Config) -> Self {
        Actor {
            domain: config.domain,
            path: if !config.path.starts_with("/") {
                format!("/{}", config.path)
            } else {
                config.path
            },
            port: if config.port == 0 {
                443
            } else {
                config.port
            },
        }
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        runtime.spawn_task_with_runtime(move |runtime| self.connect(runtime, address.unwrap(), mailbox.unwrap()));
    }
}

impl Actor {
    async fn connect(&self, runtime: impl api::Runtime, mut address: impl api::Address, mut mailbox: impl api::Mailbox) {
        let tls_connector = tokio_native_tls::TlsConnector::from(
            tokio_native_tls::native_tls::TlsConnector::builder()
                .request_alpns(&["h2"])
                .build().unwrap()
        );
        let tcp_stream = tokio::net::TcpStream::connect((&self.domain[..], self.port)).await.unwrap();
        let tls_stream = tls_connector.connect(&self.domain, tcp_stream).await.unwrap();

        assert!(tls_stream.get_ref().negotiated_alpn().unwrap().unwrap() == b"h2");

        let (h2_handle, h2_connection) = h2::client::handshake(tls_stream).await.unwrap();
        runtime.spawn_task(h2_connection); // todo: handle error

        let mut h2_handle = h2_handle.ready().await.unwrap();
        let request = http::Request::builder()
            .version(http::Version::HTTP_2)
            .method(http::Method::GET)
            .uri(format!("https://{}{}", self.domain, self.path))
            .body(())
            .unwrap();
        let (response, mut send_stream) = h2_handle.send_request(request, false).unwrap();
        let mut recv_stream = response.await.unwrap().into_body();

        runtime.spawn_task(async move {
            while let Some(data) = recv_stream.data().await {
                let data = data.unwrap();
                address.send(data.iter().cloned().collect()).await.unwrap();
                recv_stream.flow_control().release_capacity(data.len()).unwrap();
            }
        });

        runtime.spawn_task(async move {
            while let Some(msg) = mailbox.recv().await {
                send_stream.send_data(msg.into(), false).unwrap();
            }

            send_stream.reserve_capacity(0);
            send_stream.send_data(vec![].into(), true).unwrap();
        });
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}

// TODO: this component does not support static linking, perhaps due to TLS dependency. Try rustls?
