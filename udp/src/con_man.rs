use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

pub struct Client {
    addr: SocketAddr,
    last_message: Instant,
    audio_buffer: (),
}

impl Client {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            last_message: Instant::now(),
            audio_buffer: (),
        }
    }
}

pub struct ConnectionManager {
    pub(crate) local_socket: UdpSocket,
    connections: Vec<Client>,
}

impl ConnectionManager {
    pub fn new(socket: UdpSocket) -> ConnectionManager {
        ConnectionManager {
            connections: vec![],
            local_socket: socket,
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) {
        self.connections.push(Client::new(addr));
        println!("Added Connection");
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) {
        println!("Sending");
        self.local_socket.send_to(buf, addr).expect("Weird :(");
    }

    pub fn broadcast(&self, buf: &[u8]) {
        for client in &self.connections {
            self.send_to(buf, &client.addr);
        }
    }
    pub fn client(&self, addr: &SocketAddr) -> Option<&Client> {
        self.connections.iter().find(|c| c.addr == *addr)
    }

    pub fn client_mut(&mut self, addr: &SocketAddr) -> Option<&mut Client> {
        self.connections.iter_mut().find(|c| c.addr == *addr)
    }

    pub fn update_audio_buffer(&mut self, src: SocketAddr, seq: u16, data: &[f32]) {
        if let Some(client) = self.client_mut(&src) {
            //client.audio_buffer.submit(seq, data);
        }
    }

    pub(crate) fn heartbeat(&mut self, src: SocketAddr) {
        if let Some(client) = self.client_mut(&src) {
            client.last_message = Instant::now();
        }
    }

    pub fn purge_connections(&mut self, timeout: Duration) {
        let mut i = 0;
        while i < self.connections.len() {
            if self.connections[i].last_message.elapsed() > timeout {
                println!("Purging Connection");
                self.connections.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
