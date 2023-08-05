use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::Duration,
};
mod cowconnect;
const ISAAK: std::net::SocketAddrV4 =
    std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(192, 168, 12, 94), 1312);

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("0.0.0.0:1312")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = b"hello world\n";
        for i in 0..100 {
            socket.send_to(buf, ISAAK)?;
            println!("Sent {} bytes", buf.len());
            println!("Sent {}", std::str::from_utf8(buf).unwrap());
            thread::sleep(Duration::from_millis(100));
        }
        //let (amt, src) = socket.recv_from(&mut buf[])?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        //let buf = &mut buf[..amt];
        //buf.reverse();
        //socket.send_to(buf, &src)?;
    } // the socket is closed here
    Ok(())
}
