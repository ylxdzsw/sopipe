use api::{Address, Mailbox};
use crypto::{digest::Digest, mac::Mac, symmetriccipher::BlockEncryptor};

struct BufReader<R: api::Runtime> {
    mailbox: R::Mailbox,
    buffer: Box<[u8]>,
    pos: usize
}

impl<R: api::Runtime> BufReader<R> {
    fn new(mailbox: R::Mailbox) -> Self {
        Self { mailbox, buffer: Box::new([]), pos: 0 }
    }

    // take the buffer and return the rest. If the buffer is empty, wait for the next non-empty message
    async fn take(&mut self) -> Option<Box<[u8]>> {
        while self.pos == self.buffer.len() {
            self.pos = 0;
            self.buffer = self.mailbox.recv().await?;
        }

        let buffer = core::mem::replace(&mut self.buffer, Box::new([]));
        let pos = core::mem::replace(&mut self.pos, 0);

        if pos == 0 {
            Some(buffer)
        } else {
            Some(buffer[pos..].into())
        }
    }

    /// read a slice of data of specified length, wait for more data when necessary.
    async fn read_exact(&mut self, len: usize) -> Option<&[u8]> {
        if self.pos + len > self.buffer.len() {
            let mut buffer: Vec<u8> = self.buffer[self.pos..].iter().copied().collect();
            while buffer.len() < len {
                if let Some(mail) = self.mailbox.recv().await {
                    buffer.extend_from_slice(&mail);
                } else {
                    // ensure no data loss and future calls still returns None (if with the same length)
                    self.buffer = buffer.into_boxed_slice();
                    self.pos = 0;
                    return None
                }
            }
            self.buffer = buffer.into_boxed_slice();
            self.pos = 0;
        }

        self.pos += len;
        Some(&self.buffer[self.pos-len..self.pos])
    }
}

#[derive(Debug, Clone)]
pub enum Addr {
    V4([u8; 4]),
    V6([u8; 16]),
    Domain(Box<[u8]>)
}

impl Addr {
    fn parse(x: String) -> Self {
        if let Ok(addr) = x.parse::<std::net::IpAddr>() {
            match addr {
                std::net::IpAddr::V4(x) => Addr::V4(x.octets()),
                std::net::IpAddr::V6(x) => Addr::V6(x.octets())
            }
        } else {
            Addr::Domain(x.into_bytes().into())
        }
    }
}


#[allow(clippy::unreadable_literal)]
fn fnv1a(x: &[u8]) -> u32 {
    let prime = 16777619;
    let mut hash = 0x811c9dc5;
    for byte in x.iter() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(prime);
    }
    hash
}

#[derive(Debug)]
struct AES128CFB {
    key: [u8; 16],
    state: [u8; 16],
    p: usize,
}

impl AES128CFB {
    #[allow(non_snake_case)]
    fn new(key: [u8; 16], IV: [u8; 16]) -> AES128CFB {
        AES128CFB { key, state: IV, p: 16 }
    }

    fn encode(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            if self.p == 16 {
                crypto::aessafe::AesSafe128Encryptor::new(&self.key).encrypt_block(&self.state.clone(), &mut self.state);
                self.p = 0;
            }
            *byte ^= self.state[self.p];
            self.state[self.p] = *byte;
            self.p += 1;
        }
    }

    fn decode(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            if self.p == 16 {
                crypto::aessafe::AesSafe128Encryptor::new(&self.key).encrypt_block(&self.state.clone(), &mut self.state); // yes it's encrypt
                self.p = 0;
            }
            let temp = *byte;
            *byte ^= self.state[self.p];
            self.state[self.p] = temp;
            self.p += 1;
        }
    }
}

macro_rules! md5 {
    ($($x:expr),*) => {{
        let mut digest = crypto::md5::Md5::new();
        let mut result = [0; 16];
        $(digest.input($x);)*
        digest.result(&mut result);
        result
    }}
}

pub struct Client {
    user_id: [u8; 16]
}

impl Client {
    pub fn new(user_id: [u8; 16]) -> Self {
        Self { user_id }
    }
}

#[allow(non_snake_case)]
impl<R: api::Runtime> api::Actor<R> for Client {
    fn spawn(&'static self, runtime: R, mut metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (mut forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();

        let (key, IV): ([u8; 16], [u8; 16]) = rand::random();
        let addr = Addr::parse(*metadata.take::<String>("destination_addr").unwrap());
        let port = *metadata.take::<u16>("destination_port").unwrap();

        runtime.spawn_next(0, metadata.clone(), backward_address, forward_mailbox);

        let mut address = address.unwrap();
        let mut mailbox = mailbox.unwrap();

        // forward (mailbox -> forward_address)
        runtime.spawn_task(async move {
            let mut encoder = AES128CFB::new(key, IV);
            let handshake_msg = forward_handshake_msg(self.user_id, addr, port, key, IV);

            if forward_address.send(handshake_msg).await.is_err() {
                return
            }

            while let Some(data) = mailbox.recv().await {
                let data = forward_msg(&mut encoder, &data);
                if forward_address.send(data).await.is_err() {
                    return
                }
            }
        });

        // backward (backward_mailbox -> address)
        runtime.spawn_task(async move {
            let mut decoder = AES128CFB::new(md5!(&key), md5!(&IV));
            let mut reader = BufReader::<R>::new(backward_mailbox);
            if backward_handshake(&mut reader, &mut decoder).await.is_none() {
                return
            }

            while let Some(data) = backward_read(&mut reader, &mut decoder).await {
                if address.send(data).await.is_err() {
                    return
                }
            }
        });
    }
}

async fn backward_handshake<R: api::Runtime>(reader: &mut BufReader<R>, decoder: &mut AES128CFB) -> Option<()> {
    let mut head = reader.read_exact(4).await?.to_owned();
    decoder.decode(&mut head);

    assert!(head[0] == 39); // match the number provided at request handshaking
    let mut cmd = reader.read_exact(head[3] as usize).await?.to_owned();
    decoder.decode(&mut cmd);
    Some(())
}

async fn backward_read<R: api::Runtime>(reader: &mut BufReader<R>, decoder: &mut AES128CFB) -> Option<Box<[u8]>> {
    let mut temp = [0; 4];

    // 1. read and decode length
    temp[..2].copy_from_slice(&reader.read_exact(2).await?);
    decoder.decode(&mut temp[..2]);
    let len = (temp[0] as usize) << 8 | temp[1] as usize;

    // 2. read and decode checksum
    temp.copy_from_slice(reader.read_exact(4).await?);
    decoder.decode(&mut temp);

    // 3. read and decode data
    let mut data = reader.read_exact(len-4).await?.to_owned();
    decoder.decode(&mut data);

    // 4. verify checksum
    let checksum = fnv1a(&data);
    if checksum.to_be_bytes() != temp {
        panic!("invalid checksum!")
    }

    Some(data.into_boxed_slice())
}

#[allow(non_snake_case, non_upper_case_globals)]
fn forward_handshake_msg(user_id: [u8; 16], addr: Addr, port: u16, key: [u8; 16], IV: [u8; 16]) -> Box<[u8]> {
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_be_bytes();
    let mut hmac = crypto::hmac::Hmac::new(crypto::md5::Md5::new(), &user_id);
    hmac.input(&time);
    let mut auth = [0; 16];
    hmac.raw_result(&mut auth);

    let mut buffer: Vec<_> = auth.into();

    let version = 1;
    buffer.push(version);

    buffer.extend_from_slice(&IV);
    buffer.extend_from_slice(&key);

    let V = 39; // should be random but who bother
    buffer.push(V);

    let opt = 0b0000_0001;
    buffer.push(opt);

    const P_len: u8 = 0;
    let sec = 1; // AES-128-CFB
    buffer.push((P_len << 4) | (sec & 0x0f));

    let rev = 0; // reserved
    buffer.push(rev);

    let cmd = 1; // tcp
    buffer.push(cmd);

    let port = port.to_be_bytes();
    buffer.extend_from_slice(&port);

    match addr {
        Addr::V4(x) => {
            buffer.push(1);
            buffer.extend_from_slice(&x);
        }
        Addr::V6(x) => {
            buffer.push(3);
            buffer.extend_from_slice(&x);
        },
        Addr::Domain(x) => {
            buffer.push(2);
            buffer.push(x.len() as u8);
            buffer.extend_from_slice(&x);
        }
    }

    let P = [0; P_len as usize];
    buffer.extend_from_slice(&P);

    let F = fnv1a(&buffer[16..]);
    buffer.extend_from_slice(&F.to_be_bytes());

    let header_key = md5!(&user_id, b"c48619fe-8f02-49e0-b9e9-edf763e17e21");
    let header_IV = md5!(&time, &time, &time, &time);

    AES128CFB::new(header_key, header_IV).encode(&mut buffer[16..]);

    buffer.into()
}

fn forward_msg(encoder: &mut AES128CFB, data: &[u8]) -> Box<[u8]> {
    let len = data.len() + 4;
    let mut buf = Vec::with_capacity(len + 2);
    buf.extend_from_slice(&(len as u16).to_be_bytes());
    buf.extend_from_slice(&fnv1a(data).to_be_bytes());
    buf.extend_from_slice(data);
    encoder.encode(&mut buf);
    buf.into()
}

// TODO: VMessAEAD? it is not documneted anywhere but seems to be simple
// search "isAEAD" in https://github.com/v2fly/v2ray-core/blob/master/proxy/vmess/encoding/client.go
// basically it replaces the md5+timestamp with aead
// otherwise we need to start the server with V2RAY_VMESS_AEAD_FORCED=false environment variable
