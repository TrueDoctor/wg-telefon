use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use protocol::cowconnect::SampleType;

use crate::audio_buffer::AudioBuffer;

pub struct Client {
    addr: SocketAddr,
    last_message: Instant,
    audio_buffer: AudioBuffer,
    seq: u16,
}

impl Client {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            last_message: Instant::now(),
            audio_buffer: AudioBuffer::new(),
            seq: 0,
        }
    }
}

pub struct ConnectionManager {
    pub(crate) local_socket: UdpSocket,
    connections: Vec<Client>,
}

impl ConnectionManager {
    pub fn new(socket: UdpSocket) -> Self {
        ConnectionManager {
            connections: vec![],
            local_socket: socket,
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) {
        self.connections.push(Client::new(addr));
        println!("Added Connection");
    }

    pub fn client_mut(&mut self, addr: &SocketAddr) -> Option<&mut Client> {
        self.connections.iter_mut().find(|c| c.addr == *addr)
    }

    pub fn update_audio_buffer(&mut self, src: SocketAddr, seq: u16, data: &[f32]) {
        if let Some(client) = self.client_mut(&src) {
            client.audio_buffer.submit(seq, data);
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

    pub fn send_audio(&mut self) {
        let samples = self.gather_samples();
        let mixed_samples = self.mix_samples(samples);
        for (client, samples) in self.connections.iter_mut().zip(mixed_samples) {
            let datagram = protocol::cowconnect::Datagram::new(
                client.seq,
                protocol::cowconnect::DatagramType::Audio(samples),
            );
            client.seq = client.seq.wrapping_add(1);
            if self
                .local_socket
                .send_to(&datagram.to_bytes(), client.addr)
                .is_err()
            {
                println!("Failed to send audio to {}", client.addr);
            };
        }
    }

    pub fn gather_samples(&mut self) -> Vec<Vec<SampleType>> {
        let mut client_samples = Vec::with_capacity(self.connections.len());
        for connection in self.connections.iter_mut() {
            let mut samples = Vec::new();
            while let Some(sample) = connection.audio_buffer.next_sample() {
                samples.push(sample);
            }
            client_samples.push(samples);
        }
        client_samples
    }

    pub fn mix_samples(&self, samples: Vec<Vec<SampleType>>) -> Vec<Vec<SampleType>> {
        let max_len = samples.iter().map(|s| s.len()).max().unwrap_or(0);
        let empty = vec![0.0; max_len];
        let mut mixed_samples = vec![empty; samples.len()];
        for (client, mixed) in mixed_samples.iter_mut().enumerate() {
            for (stream_id, samples) in samples.iter().enumerate() {
                if stream_id == client {
                    continue;
                }
                for (i, sample) in samples.iter().enumerate() {
                    mixed[i] += *sample;
                }
            }
        }
        mixed_samples
    }
}
