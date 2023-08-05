use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

pub struct ConnectionManager {
    local_socket: UdpSocket,
    connections: Vec<std::net::SocketAddr>,
}

impl ConnectionManager {
    pub fn new(socket:UdpSocket) -> ConnectionManager {
        ConnectionManager {
            connections: vec![],
            local_socket: socket,
        }
    }

    pub fn connect(&mut self, addr:SocketAddr) {
        self.connections.push(addr);
        println!("Added Connection");
    }

    pub fn send_to(&self, buf: &[u8], addr:&SocketAddr){
        println!("Sending");
        self.local_socket.send_to(buf, addr).expect("Weird :(");
    }
}
