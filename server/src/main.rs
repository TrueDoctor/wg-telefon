mod con_man;

pub use protocol::{audio_buffer, cowconnect};

use std::{net::UdpSocket, thread, time::Duration};

use con_man::ConnectionManager;
use cowconnect::{ControlType, DatagramType};

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:1312")?;
    socket.set_nonblocking(true)?;
    let mut connection_man = ConnectionManager::new(socket);

    let mut buf = [0; u16::MAX as usize];

    loop {
        // Receive Audio
        if let Ok((amt, src)) = connection_man.local_socket.recv_from(&mut buf) {
            if let Some(datagram) = cowconnect::Datagram::from_bytes(&buf[..amt]) {
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
                        connection_man.heartbeat(src);
                        connection_man.update_audio_buffer(src, datagram.seq, &data);
                    }
                }
            }
        }
        thread::sleep(Duration::from_micros(100));

        // Purge Connections
        connection_man.purge_connections(Duration::from_secs(5));

        // Send Audio
        connection_man.send_audio();
    }
}
