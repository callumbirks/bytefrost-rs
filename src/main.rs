use std::{net::{self, SocketAddr, TcpStream}, time::Duration, collections::HashMap, io, cmp::Ordering};

struct Discovery {
    sock_in: net::UdpSocket,
    sock_out: net::UdpSocket,
    username: String,
}

impl Discovery {
    fn new(username: &str) -> Self {
        
        let sock_in = net::UdpSocket::bind(Discovery::sock_addr()).expect("Failed to bind socket");
        sock_in
            .join_multicast_v4(&Discovery::ip_addr(), &net::Ipv4Addr::new(0, 0, 0, 0))
            .expect("Failed to join multicast");
        sock_in.set_read_timeout(Some(Duration::from_millis(200))).expect("Error setting socket timeout.");

        let sock_out = net::UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

        return Discovery {
            sock_in,
            sock_out,
            username: username.to_owned(),
        };
    }

    fn ip_addr() -> net::Ipv4Addr {
        net::Ipv4Addr::new(239, 255, 73, 5)
    }

    fn sock_addr() -> net::SocketAddrV4 {
        net::SocketAddrV4::new(Discovery::ip_addr(), 26372)
    }

    fn send(&self) {
        println!("Broadcasting...");
        if let Err(err) = self.sock_out.send_to(self.username.as_bytes(), Discovery::sock_addr()) {
            println!("Error sending multicast: {}", err);
        }
    }

    fn recv(&self) -> Option<(String, SocketAddr)> {
        println!("Listening...");
        let mut buf = [0u8; 64];
        match self.sock_in.recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                let data = &buf[..len];
                return Some((String::from_utf8_lossy(data).to_string(), remote_addr));
            },
            Err(_) => None
        }
    }
}

struct Messaging {
    username: String,
    peers: HashMap<String, net::TcpStream>,
    sock_in: net::UdpSocket
}

impl Messaging {
    fn new(username: &str) -> Self {
        let sock = net::UdpSocket::bind("0.0.0.0:39635").expect("Failed to bind listening socket");
        
        Messaging { username: username.to_string(), peers: HashMap::new(), sock_in: sock }
    }

    fn add_peer(&mut self, username: &str, sock_addr: SocketAddr) {
        if self.peers.contains_key(username) {
            return;
        }

        let stream = match self.username.as_str().cmp(username) {
            Ordering::Less => self.accept_connection(sock_addr),
            Ordering::Greater => self.initiate_connection(sock_addr),
            Ordering::Equal => return
        };
        
        if let Err(err) = stream {
            println!("Error adding peer: {}", err);
            return;
        }

        self.peers.insert(username.to_string(), stream.unwrap());
        println!("Peer added!");
    }

    fn initiate_connection(&self, sock_addr: SocketAddr) -> Result<TcpStream, io::Error> {
        TcpStream::connect_timeout(&sock_addr, Duration::from_millis(500))
    }

    fn accept_connection(&self, sock_addr: SocketAddr) -> Result<TcpStream, io::Error> {
        let listener = net::TcpListener::bind(sock_addr)?;
        let (stream, _) = listener.accept()?;
        Ok(stream)
    }
}

fn main() {
    println!("Hello, world!");
    
    let username = "borkmac";
    let disc = Discovery::new(username);
    let mut msgs = Messaging::new(username);

    loop {
        disc.send();
        let found = disc.recv();
        if let Some((found_name, found_addr)) = found {
            if found_name != username {
                msgs.add_peer(&found_name, found_addr);
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

#[test]
fn ip_is_multicast() {
    assert!(Discovery::ip_addr().is_multicast())
}
