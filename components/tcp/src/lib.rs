use std::sync::atomic::AtomicU64;
use api::serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

struct Component;

mod passive;
mod active;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(crate="api::serde")]
struct Config {
    #[serde(default)]
    _pos_arg_1: api::Argument,

    #[serde(default)]
    _pos_arg_2: api::Argument,

    port: Option<u16>,
    addr: Option<String>,

    #[serde(default)]
    active: bool,

    outputs: Vec<String>,
    function_name: String,
}

impl Config {
    fn get_addr_and_port(&self) -> (Option<std::net::IpAddr>, Option<u16>) {
        let mut addr: Option<std::net::IpAddr> = None;
        let mut port: Option<u16> = self.port;

        let mut try_fill_addr = |x: std::net::IpAddr| {
            if addr.is_some() {
                panic!("specified address multiple times")
            }
            addr = Some(x);
        };

        let mut try_fill_port = |x: u16| {
            if port.is_some() {
                panic!("specified portess multiple times")
            }
            port = Some(x);
        };

        if let Some(s) = &self.addr {
            if let Ok(ip_addr) = s.parse::<std::net::IpAddr>() {
                try_fill_addr(ip_addr);
            } else {
                panic!("wrong address format")
            }
        }

        for pos_arg in [&self._pos_arg_1, &self._pos_arg_2] {
            match pos_arg {
                api::Argument::String(s) => {
                    if let Ok(socket_addr) = s.parse::<std::net::SocketAddr>() {
                        try_fill_addr(socket_addr.ip());
                        try_fill_port(socket_addr.port());
                    } else if let Ok(ip_addr) = s.parse::<std::net::IpAddr>() {
                        try_fill_addr(ip_addr);
                    } else {
                        panic!("wrong address format: {}", s)
                    }
                },
                api::Argument::Int(i) => {
                    try_fill_port(*i as _)
                },
                api::Argument::Vec(_) => panic!("wrong argument type"),
                api::Argument::None => {},
            }
        }

        (addr, port)
    }
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let config: Config = api::parse_args(&arguments).unwrap();

        if config.active {
            todo!()
        } else {
            Box::new(passive::Actor::new(config))
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

async fn read_tcp(mut stream: impl AsyncReadExt + Unpin, mut addr: impl api::Address) {
    let mut buffer = vec![0; 1024].into_boxed_slice();
    loop {
        match stream.read(&mut buffer[..]).await {
            Ok(0) => return, // EOF
            Ok(n) => if addr.send(buffer[..n].iter().copied().collect()).await.is_err() {
                return
            }
            Err(e) => {
                eprintln!("IO error: {}", e);
                return
            }
        }
    }
}

async fn write_tcp(mut stream: impl AsyncWriteExt + Unpin, mut mail: impl api::Mailbox) {
    while let Some(msg) = mail.recv().await {
        if stream.write_all(&msg).await.is_err() {
            break
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_addr_and_port() {
        let args = [
            ("".into(), "127.0.0.1".to_string().into()),
            ("outputs".into(), ["".to_string()].into_iter().collect()),
            ("function_name".into(), "tcp".to_string().into())
        ];
        let config: Config = api::parse_args(&args).unwrap();
        let (addr, port) = config.get_addr_and_port();
        assert_eq!(addr, Some("127.0.0.1".parse().unwrap()));
        assert_eq!(port, None);

        let args = [
            ("".into(), "127.0.0.1:8888".to_string().into()),
            ("outputs".into(), ["".to_string()].into_iter().collect()),
            ("function_name".into(), "tcp".to_string().into())
        ];
        let config: Config = api::parse_args(&args).unwrap();
        let (addr, port) = config.get_addr_and_port();
        assert_eq!(addr, Some("127.0.0.1".parse().unwrap()));
        assert_eq!(port, Some(8888));

        let args = [
            ("".into(), 8888.into()),
            ("outputs".into(), ["".to_string()].into_iter().collect()),
            ("function_name".into(), "tcp".to_string().into())
        ];
        let config: Config = api::parse_args(&args).unwrap();
        let (addr, port) = config.get_addr_and_port();
        assert_eq!(addr, None);
        assert_eq!(port, Some(8888));

        let args = [
            ("port".into(), 8888.into()),
            ("".into(), "127.0.0.1".to_string().into()),
            ("outputs".into(), ["".to_string()].into_iter().collect()),
            ("function_name".into(), "tcp".to_string().into())
        ];
        let config: Config = api::parse_args(&args).unwrap();
        let (addr, port) = config.get_addr_and_port();
        assert_eq!(addr, Some("127.0.0.1".parse().unwrap()));
        assert_eq!(port, Some(8888));

        let args = [
            ("".into(), 8888.into()),
            ("".into(), "127.0.0.1".to_string().into()),
            ("outputs".into(), ["".to_string()].into_iter().collect()),
            ("function_name".into(), "tcp".to_string().into())
        ];
        let config: Config = api::parse_args(&args).unwrap();
        let (addr, port) = config.get_addr_and_port();
        assert_eq!(addr, Some("127.0.0.1".parse().unwrap()));
        assert_eq!(port, Some(8888));
    }
}
