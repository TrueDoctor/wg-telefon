use audio::Options;
use protocol::{
    audio_buffer::AudioBuffer,
    cowconnect::{ControlType, Datagram, DatagramType},
};
use std::{net::UdpSocket, time::Duration};

fn main() -> std::io::Result<()> {
    let input = std::env::args().nth(1);
    let socket = UdpSocket::bind("0.0.0.0:1312")?;
    let mut audio_buffer = AudioBuffer::new();

    let addr = input.unwrap_or("192.168.4.131:1312".to_string());

    socket.set_nonblocking(true)?;

    println!("Connectiong to: {addr}");
    socket.connect(addr)?;
    println!("Connected");

    let mut seq = 0;
    let mut send_datagram = |datagram_type| {
        let datagram = Datagram::new(seq, datagram_type);
        seq = seq.wrapping_add(1);
        while socket.send(&datagram.to_bytes()).is_err() {
            std::thread::sleep(Duration::from_millis(10));
        }
    };

    println!("Sending Connect");
    send_datagram(DatagramType::Control(ControlType::Connect));

    let options = Options {
        buffer_length: 100.,
        ..Default::default()
    };
    let mut context = audio::create_context(options).expect("Failed to create audio context");

    let mut send_buf = [0f32; 2048];
    let mut send_buf_offset = 0;
    let mut buf = [0u8; 2048];
    loop {
        // Send Heartbeat
        send_datagram(DatagramType::Control(ControlType::Heartbeat));

        // Receive Audio
        if let Ok(amt) = socket.recv(&mut buf) {
            if amt == 0 {
                continue;
            }
            //println!("Received {} bytes ", amt);
            let Some(datagram) = Datagram::from_bytes(&buf[..amt])  else { continue };
            if let DatagramType::Audio(data) = datagram.datagram_type {
                // Save Audio to Buffer
                audio_buffer.submit(datagram.seq, data);
                //println!("available_samples: {}", audio_buffer.available_samples());
            }
        }

        // Get Audio Samples
        let mut samples = [0f32; 1024];
        let mut index = 0;
        for sample in samples.iter_mut() {
            if let Some(s) = audio_buffer.next_sample() {
                *sample = s;
            } else {
                break;
            }
            index += 1;
        }

        // Play Audio
        context.write_samples(&samples[..index]);

        // Read Audio Samples
        let sample_count = context.read_samples(&mut send_buf[send_buf_offset..]);
        send_buf_offset += sample_count;
        if send_buf_offset > 512 {
            // Send Audio
            send_buf_offset = 0;
            send_datagram(DatagramType::Audio(send_buf[..sample_count].to_vec()));
        }

        std::thread::sleep(Duration::from_micros(100));
    }

    //send_datagram(DatagramType::Control(ControlType::Disconnect));
}
