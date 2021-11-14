use api::{MetaData, Address, Mailbox, Runtime};

pub struct Actor;

impl<R: Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, mut metadata: MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut address = address.expect("socks5 no address to return");
        let mut mailbox = mailbox.expect("socks5 no input");

        runtime.spawn_task_with_runtime(move |runtime| async move {
            let mut buf: Vec<u8> = vec![];

            macro_rules! try_split_at {
                // `()` indicates that the macro takes no argument.
                ($slice: ident, $pos: expr) => {{
                    if $slice.len() < $pos {
                        continue
                    }
                    $slice.split_at($pos)
                }};
            }

            // handshake
            let consumed = loop {
                if let Some(packet) = mailbox.recv().await {
                    buf.extend(&*packet)
                } else {
                    return
                }

                let slice = &buf[..];

                let (header, slice) = try_split_at!(slice, 2);

                if header[0] != 5 {
                    let hint = "if the version is 71, the client might have used it as an HTTP proxy";
                    eprintln!("unsupported socks version {}. Hint: {}", header[0], hint);
                    return
                }

                let n_methods = header[1] as usize;

                let (methods, slice) = try_split_at!(slice, n_methods);

                if !methods.contains(&0) {
                    let _ = address.send(Box::from([5, 0xff])).await; // we will return anyway
                    eprintln!("client do not support NO AUTH method");
                    return
                }

                match address.send(Box::from([5, 0])).await {
                    Ok(_) => break buf.len() - slice.len(),
                    Err(_) => return
                }
            };

            // read request
            let (addr, port, consumed) = loop {
                if let Some(packet) = mailbox.recv().await {
                    buf.extend(&*packet)
                } else {
                    return
                }

                let slice = &buf[consumed..];

                let (header, slice) = try_split_at!(slice, 4);
                let [ver, cmd, _rev, atyp]: [u8; 4] = header.try_into().unwrap();

                if ver != 5 {
                    eprintln!("unsupported socks version {}", ver);
                    return
                }

                if cmd != 1 {
                    eprintln!("unsupported command type {}", cmd);
                }

                let (addr, slice) = match atyp {
                    0x01 => {
                        let (addr, slice) = try_split_at!(slice, 4);
                        let addr: [u8; 4] = addr.try_into().unwrap();
                        let addr = std::net::Ipv4Addr::from(addr);
                        let addr = format!("{}", addr);
                        (addr, slice)
                    },
                    0x04 => {
                        let (addr, slice) = try_split_at!(slice, 16);
                        let addr: [u8; 16] = addr.try_into().unwrap();
                        let addr = std::net::Ipv6Addr::from(addr);
                        let addr = format!("{}", addr);
                        (addr, slice)
                    },
                    0x03 => {
                        let (len, slice) = try_split_at!(slice, 1);
                        let (addr, slice) = try_split_at!(slice, len[0] as usize);
                        let addr = std::str::from_utf8(addr).unwrap().to_string();
                        (addr, slice)
                    },
                    _ => {
                        eprintln!("unknown ATYP");
                        return
                    }
                };

                let (port, slice) = try_split_at!(slice, 2);
                let port = (port[0] as u16) << 8 | port[1] as u16;

                break (addr, port, buf.len() - slice.len())
            };

            // write initial reply
            let reply = Box::<[u8]>::from([5, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
            if address.send(reply).await.is_err() {
                return
            }

            // start forwarding data
            metadata.set("destination_addr".into(), addr);
            metadata.set("destination_port".into(), port);

            let (mut forward_address, forward_mailbox) = runtime.channel();
            let (backward_address, mut backward_mailbox) = runtime.channel();
            runtime.spawn_next(0, metadata, backward_address, forward_mailbox);
            runtime.spawn_task(async move {
                #[allow(clippy::collapsible_if)]
                if buf.len() > consumed {
                    if forward_address.send(Box::from(&buf[consumed..])).await.is_err() {
                        return
                    }
                }
                while let Some(msg) = mailbox.recv().await {
                    if forward_address.send(msg).await.is_err() {
                        return
                    }
                }
            });
            runtime.spawn_task(async move {
                while let Some(msg) = backward_mailbox.recv().await {
                    if address.send(msg).await.is_err() {
                        return
                    }
                }
            });
        })
    }
}
