use super::*;

pub struct TcpComponent;
pub enum TcpInstance { Forward { port: u16}, Backward }
pub struct TcpActor {

}

impl Component for TcpComponent {
    fn create(&'static self, args: &rhai::Map) -> *const InstanceState {
        match &args["direction"].clone().into_string().unwrap()[..] {
            "forward" => {
                assert!(args["n_outputs"].as_int().unwrap() == 1);
                TcpInstance::Forward { port: args["port"].as_int().unwrap().try_into().unwrap() }.box_and_leak() as *mut _ as _
            }
            "backward" => {
                assert!(args["n_outputs"].as_int().unwrap() == 0);
                TcpInstance::Backward.box_and_leak() as *mut _ as _
            }
            _ => unreachable!()
        }
    }

    fn spawn(&'static self, instance: *const InstanceState) -> &'static mut dyn Actor {
        todo!()
    }
}

impl Actor for TcpActor {
    fn poll(&'static self, arg: *mut c_void) -> Request {
        Request::Yield
    }
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
