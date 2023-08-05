#[derive(Debug, PartialEq)]
pub struct Datagram {
    seq: u16,
    datagram_type: DatagramType,
}

const AudioDatagramID: u8 = 0;
const ControlDatagramID: u8 = 1;
const ConnectDatagramID: u8 = 0;
const DisconnectDatagramID: u8 = 1;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum DatagramType {
    Audio(Vec<f32>) = AudioDatagramID,
    Control(ControlType) = ControlDatagramID,
}

impl Datagram {
    pub fn new(seq: u16, datagram_type: DatagramType) -> Self {
        Self {
            seq,
            datagram_type,
        }
    }

    fn id(&self) -> u8 {
        match &self.datagram_type {
            DatagramType::Audio(_) => AudioDatagramID,
            DatagramType::Control(_) => ControlDatagramID,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.seq.to_be_bytes());
        bytes.push(self.id());
        match &self.datagram_type {
            DatagramType::Audio(audio) => {
                for sample in audio {
                    bytes.extend_from_slice(&sample.to_be_bytes());
                }
            },
            DatagramType::Control(control) => {
                bytes.extend_from_slice(&control.to_bytes());
            },
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 3 {
            return None;
        }
        let seq = u16::from_be_bytes([bytes[0], bytes[1]]);
        let datagram_type = match bytes[2] {
            AudioDatagramID => {
                if bytes.len() < 3 + 4 {
                    return None;
                }
                let mut audio = Vec::new();
                for i in 3..bytes.len() {
                    audio.push(f32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]));
                }
                DatagramType::Audio(audio)
            },
            ControlDatagramID => {
                if bytes.len() < 3 + 1 {
                    return None;
                }
                let control_type = ControlType::from_bytes(&bytes[3..]).unwrap();
                DatagramType::Control(control_type)
            },
            _ => return None,
        };
        Some(Self::new(seq, datagram_type))
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ControlType {
    Connect() = ConnectDatagramID,
    Disconnect() = DisconnectDatagramID,
}

impl ControlType {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            ConnectDatagramID => Some(Self::Connect()),
            DisconnectDatagramID => Some(Self::Disconnect()),
            _ => None,
        }
    }
    fn id(&self) -> u8 {
        match self {
            Self::Connect() => ConnectDatagramID,
            Self::Disconnect() => DisconnectDatagramID,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.id()]
    }
}

fn filter(list: &[Datagram]) -> Vec<&Datagram> {
    let mut seq = list[0].seq;
    let mut output = Vec::new();
    for datagram in list {
        if (datagram.seq >= seq) {
            output.push(datagram);
        } else {
            continue;
        }
        seq = seq.wrapping_add(1);
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_filter() {
        let from_id = |seq: u16| Datagram {
            seq,
            datagram_type: DatagramType::Audio(vec![0.0]),
        };

        let list = vec![from_id(0), from_id(1), from_id(1), from_id(2), from_id(std::u16::MAX), from_id(0)];
        let filtered = filter(&list);

        let result = vec![from_id(0), from_id(1), from_id(2), from_id(std::u16::MAX), from_id(0)];
        
        for (a, b) in filtered.iter().zip(result.iter()) {
            assert_eq!(*a, b);
        }
    }
}