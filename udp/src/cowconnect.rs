#[derive(Debug, PartialEq)]
pub struct Datagram {
    pub seq: u16,
    pub datagram_type: DatagramType,
}

pub type SampleType = f32;

// Datagram Type Ids
const AUDIO_DATAGRAM_ID: u8 = 0;
const CONTROL_DATAGRAM_ID: u8 = 1;

// Control Type Ids
const CONNECT_DATAGRAM_ID: u8 = 0;
const DISCONNECT_DATAGRAM_ID: u8 = 1;
const HEARTBEAT_DATAGRAM_ID: u8 = 2;

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum DatagramType {
    Audio(Vec<SampleType>) = AUDIO_DATAGRAM_ID,
    Control(ControlType) = CONTROL_DATAGRAM_ID,
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ControlType {
    Connect = CONNECT_DATAGRAM_ID,
    Disconnect = DISCONNECT_DATAGRAM_ID,
    Heartbeat = HEARTBEAT_DATAGRAM_ID,
}

impl Datagram {
    pub fn new(seq: u16, datagram_type: DatagramType) -> Self {
        Self { seq, datagram_type }
    }

    fn id(&self) -> u8 {
        match &self.datagram_type {
            DatagramType::Audio(_) => AUDIO_DATAGRAM_ID,
            DatagramType::Control(_) => CONTROL_DATAGRAM_ID,
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
            }
            DatagramType::Control(control) => {
                bytes.extend_from_slice(&control.to_bytes());
            }
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        const SEQ_HEADER_BYTES: usize = 3;

        if bytes.len() < SEQ_HEADER_BYTES {
            return None;
        }

        let seq = u16::from_be_bytes([bytes[0], bytes[1]]);
        let datagram_type = match bytes[2] {
            AUDIO_DATAGRAM_ID if bytes.len() >= SEQ_HEADER_BYTES + 4 => {
                let take4 = |i| {
                    SampleType::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]])
                };
                let remaining_bytes = SEQ_HEADER_BYTES..bytes.len();
                let audio = remaining_bytes.map(take4).collect();
                DatagramType::Audio(audio)
            }
            CONTROL_DATAGRAM_ID if bytes.len() >= SEQ_HEADER_BYTES + 1 => {
                let control_type = ControlType::from_bytes(&bytes[SEQ_HEADER_BYTES..]).unwrap();
                DatagramType::Control(control_type)
            }
            _ => return None,
        };
        Some(Self::new(seq, datagram_type))
    }
}

impl ControlType {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            CONNECT_DATAGRAM_ID => Some(Self::Connect),
            DISCONNECT_DATAGRAM_ID => Some(Self::Disconnect),
            _ => None,
        }
    }

    #[inline]
    fn id(&self) -> u8 {
        match self {
            Self::Connect => CONNECT_DATAGRAM_ID,
            Self::Disconnect => DISCONNECT_DATAGRAM_ID,
            Self::Heartbeat => HEARTBEAT_DATAGRAM_ID,
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
        if datagram.seq < seq {
            continue;
        }

        output.push(datagram);
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

        let list = vec![
            from_id(0),
            from_id(1),
            from_id(1),
            from_id(2),
            from_id(std::u16::MAX),
            from_id(0),
        ];
        let filtered = filter(&list);

        let result = vec![
            from_id(0),
            from_id(1),
            from_id(2),
            from_id(std::u16::MAX),
            from_id(0),
        ];

        for (a, b) in filtered.iter().zip(result.iter()) {
            assert_eq!(*a, b);
        }
    }
}
