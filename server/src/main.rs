mod con_man;

pub use protocol::{audio_buffer, cowconnect};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    time::Duration,
};

use con_man::ConnectionManager;
use cowconnect::{ControlType, DatagramType};

const ISAAK_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 156)), 1313);

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:1312")?;
    let mut connection_man = ConnectionManager::new(socket);
    connection_man.connect(ISAAK_PEER);

    let mut buf = [0; 1024];

    loop {
        // Receive Audio
        let (amt, src) = connection_man.local_socket.recv_from(&mut buf)?;

        if let Some(datagram) = cowconnect::Datagram::from_bytes(&buf[..amt]) {
            println!("Received {} bytes from {}", amt, src);
            println!("Received {:?}", datagram);
            match datagram.datagram_type {
                DatagramType::Control(ControlType::Connect) => {
                    println!("Received Connect");
                    connection_man.connect(src);
                }
                DatagramType::Control(ControlType::Disconnect) => {
                    println!("Received Connect");
                    connection_man.connect(src);
                }
                DatagramType::Control(ControlType::Heartbeat) => {
                    connection_man.heartbeat(src);
                }
                DatagramType::Audio(data) => {
                    connection_man.update_audio_buffer(src, datagram.seq, &data);
                }
            }
        }

        // Purge Connections
        connection_man.purge_connections(Duration::from_secs(5));

        // Send Audio
        connection_man.send_audio();
    }
}
