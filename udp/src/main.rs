mod con_man;
mod cowconnect;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

const ISAAK_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 4, 156)), 1313);

fn main() -> std::io::Result<()> {
    {
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
    } // the socket is closed here
    Ok(())
}
