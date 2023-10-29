mod con_man;

pub use protocol::{audio_buffer, cowconnect};

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use con_man::ConnectionManager;
use cowconnect::{ControlType, DatagramType};

const ISAAK_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 156)), 1313);

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:1312")?;
    let mut connection_man = ConnectionManager::new(socket);
    connection_man.connect(ISAAK_PEER);

    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let buf = b"hello world\n";
    connection_man.send_to(buf, &ISAAK_PEER);
    //for i in 0..100 {
    //    socket.send_to(buf, ISAAK)?;
    //    println!("Sent {} bytes", buf.len());
    //    println!("Sent {}", std::str::from_utf8(buf).unwrap());
    //    thread::sleep(Duration::from_millis(100));
    //}
    //let (amt, src) = socket.recv_from(&mut buf[])?;
    let mut buf = [0; 1024];
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
            _ => println!("Received something else"),
        }
    }
    Ok(())
}
