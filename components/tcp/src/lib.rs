use std::{collections::BTreeMap, error::Error};
use thiserror::Error;

struct Spec {

}

struct Forward(u16);

impl api::Component for Forward {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

}

struct ForwardActor {}

#[api::async_trait]
impl api::Actor for ForwardActor {
    async fn run(self: Box<Self>, ) {}
}

struct Backward;

impl api::Component for Backward {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

}

struct BackwardActor {}

#[api::async_trait]
impl api::Actor for BackwardActor {
    async fn run(self: Box<Self>, ) {}
}

#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Invalid arguments. Detail: {0}")]
    InvalidArgument(&'static str),
}

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> Result<Box<dyn api::Component>, Box<dyn Error + Send + Sync>> {
        let comp: Box<dyn api::Component> = match &arguments.iter().find(|x| x.0 == "direction").unwrap().1.as_string().unwrap()[..] {
            "forward" => {
                let port = arguments.iter().find(|x| x.0.is_empty() || x.0 == "port").unwrap().1.as_int().unwrap();
                Box::new(Forward(*port as _))
            },
            "backward" => {
                Box::new(Backward)
            }
            _ => unreachable!()
        };

        Ok(comp)
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

pub fn init() -> &'static dyn api::ComponentSpec {
    println!("Hello, world from tcp");
    &Spec {}
}


// impl Component for TcpListener {
//     // async fn init(&'static self, _args: &ByteDict) -> *mut c_void {
//     //     let handle = net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await.unwrap();
//     //     leak(handle) as *mut _ as _
//     // }

//     async fn creat_forward_context(&'static self, args: &ByteDict) -> *mut c_void {
//         let mut listener = net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await.unwrap();

//         while let Some(socket_res) = listener.next().await {
//             match socket_res {
//                 Ok(mut socket) => {
//                     println!("Accepted connection from {:?}", socket.peer_addr());

//                     // unsafe {
//                     //     let forward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"forward\0").unwrap();
//                     //     let backward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"backward\0").unwrap();

//                     //     let mut buf = [0; 2048];
//                     //     let forward_handle = async {
//                     //         while let Ok(l) = socket.read(&mut buf).await {
//                     //             if l == 0 {
//                     //                 break
//                     //             }

//                     //             let mut stream = SopipeStream::new();
//                     //             forward(&mut stream as *mut _ as *mut _, l as _, buf.as_ptr());
//                     //         }
//                     //     };

//                     //     forward_handle.await
//                     // }

//                     // io::copy(&mut socket, &mut io::stdout()).await.unwrap();
//                 }
//                 Err(err) => {
//                     println!("accept error = {:?}", err)
//                 }
//             }
//         }

//         unreachable!()
//     }

//     async fn creat_backward_context(&'static self, args: &ByteDict) -> *mut c_void {
//         todo!()
//     }
// }
