use api::{Address, serde::Deserialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{ToSocketAddrs, UdpSocket};

struct Component;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(crate = "api::serde")]
struct Config {
    #[serde(default)]
    addr: api::Argument,
    port: Option<u16>,

    outputs: Vec<String>,
    function_name: String,
}

pub struct Actor {
    addr: Option<String>,
    port: Option<u16>,
    has_output: bool,
}

impl Config {
    fn get_addr_and_port(&self) -> (Option<String>, Option<u16>) {
        let mut addr: Option<String> = None;
        let mut port: Option<u16> = self.port;

        match self.addr.clone() {
            api::Argument::String(s) => {
                addr = Some(s);
            }
            api::Argument::Int(i) => port = Some(i as _),
            api::Argument::Vec(_) => panic!("wrong argument type"),
            api::Argument::None => {}
        }

        (addr, port)
    }
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        Box::new(Actor::new(api::parse_args(&arguments).unwrap()))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["udp"]
    }
}

impl Actor {
    pub(crate) fn new(config: Config) -> Self {
        let (addr, port) = config.get_addr_and_port();

        Actor {
            addr, port,
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("tcp can only accept one output"),
            },
        }
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, mut metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        assert!(!self.has_output);

        let mut addr = metadata.take::<String>("destination_addr").map(|x| *x);
        let mut port = metadata.take::<u16>("destination_port").map(|x| *x);

        if addr.is_some() || port.is_some() {
            if self.addr.is_some() || self.port.is_some() {
                panic!("The stream already contains destination information")
            }
        } else {
            addr = self.addr.clone();
            port = self.port;
        }

        if let Some(port) = port {
            runtime.spawn_task_with_runtime(move |runtime| {
                self.connect(runtime, (addr.unwrap(), port), address.unwrap(), mailbox.unwrap())
            })
        } else {
            runtime.spawn_task_with_runtime(move |runtime| {
                self.connect(runtime, addr.unwrap(), address.unwrap(), mailbox.unwrap())
            })
        }
    }

    fn spawn_source(&'static self, runtime: R) {
        assert!(self.has_output);
        runtime.spawn_task_with_runtime(move |runtime| self.listen(runtime))
    }
}

impl Actor {
    async fn connect(&self, runtime: impl api::Runtime, dest: impl ToSocketAddrs, address: impl api::Address, mailbox: impl api::Mailbox) {
        let socket = Arc::new(UdpSocket::bind(("::", 0)).await.unwrap());
        socket.connect(dest).await.unwrap();

        runtime.spawn_task(read_udp(socket.clone(), address));
        runtime.spawn_task(write_udp(socket, mailbox));
    }

    async fn listen(&self, runtime: impl api::Runtime) {
        let addr = self.addr.as_deref().unwrap_or("::");
        let listener = if let Some(port) = self.port {
            UdpSocket::bind((addr, port)).await.unwrap()
        } else {
            UdpSocket::bind(addr).await.unwrap()
        };

        while let api::RunLevel::Init = runtime.get_runlevel() {
            tokio::time::sleep(Duration::from_millis(20)).await
        }

        let (mut address, mailbox) = runtime.channel();
        let mut meta = api::MetaData::default();
        meta.set("stream_type".into(), "UDP".to_string());
        runtime.spawn_next(0, meta, None, mailbox);

        while let api::RunLevel::Run = runtime.get_runlevel() {
            let mut buffer = vec![0; 65536].into_boxed_slice();
            match tokio::time::timeout(Duration::from_secs(1), listener.recv_from(&mut buffer[..])).await {
                Ok(Ok((n, origin))) => {
                    eprintln!("Recieved UDP packet from {:?}", origin);

                    if address.send(Box::from(&buffer[..n])).await.is_err() {
                        return
                    }
                }
                Ok(Err(err)) => {
                    eprintln!("accept error = {}", err)
                }
                Err(_) => {} // timeout, check runlevel and listen again
            }
        }
    }
}

async fn read_udp(socket: Arc<UdpSocket>, mut addr: impl api::Address) {
    let mut buffer = vec![0; 65536].into_boxed_slice();
    loop {
        match tokio::time::timeout(Duration::from_secs(5), socket.recv(&mut buffer[..])).await {
            Ok(Ok(n)) => {
                if addr.send(buffer[..n].iter().copied().collect()).await.is_err() {
                    return;
                }
            }
            Ok(Err(e)) => {
                eprintln!("IO error: {}", e);
                return;
            }
            Err(_) => return // timeout, assume the UDP session is end
        }
    }
}

async fn write_udp(socket: Arc<UdpSocket>, mut mail: impl api::Mailbox) {
    while let Some(msg) = mail.recv().await {
        if socket.send(&msg).await.is_err() {
            break;
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
