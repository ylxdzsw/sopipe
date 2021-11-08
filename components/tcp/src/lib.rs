use std::{collections::BTreeMap, error::Error};

struct Component {

}

struct Forward(u16);

struct Backward;



impl api::Component for Component {
    fn create(&self, arguments: Vec<api::Argument>) -> api::Result<Box<api::Actor>> {
        todo!()
        // let comp: Box<dyn api::Component> = match &arguments.iter().find(|x| x.0 == "direction").unwrap().1.as_string().unwrap()[..] {
        //     "forward" => {
        //         let port = arguments.iter().find(|x| x.0.is_empty() || x.0 == "port").unwrap().1.as_int().unwrap();
        //         Box::new(Forward(*port as _))
        //     },
        //     "backward" => {
        //         Box::new(Backward)
        //     }
        //     _ => unreachable!()
        // };

        // Ok(comp)
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

pub fn init() -> &'static dyn api::Component {
    &Component {}
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
