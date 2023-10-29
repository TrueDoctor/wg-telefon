use std::collections::BTreeMap;

use crate::cowconnect::SampleType;

#[derive(Default, Debug)]
pub struct AudioBuffer {
    last_seq: u64,
    audio_buffer: BTreeMap<u64, Vec<SampleType>>,
}

impl AudioBuffer {
    const WARP_THRESHOLD: i64 = i16::MAX as i64 / 2;

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn submit<S: AsRef<[f32]>>(&mut self, seq: u16, audio: S) {
        let seq = match self.last_seq as i64 - seq as i64 {
            x if x > Self::WARP_THRESHOLD => {
                let mask = x & !0xffff;
                let offset = mask + u16::MAX as i64 + 1;
                offset + seq as i64
            }
            x if x < -Self::WARP_THRESHOLD => {
                let mask = x & !0xffff;
                let offset = mask - u16::MAX as i64;
                offset + seq as i64
            }
            _ => seq as i64,
        };
        self.audio_buffer
            .insert(seq as u64, audio.as_ref().to_vec());
    }
}
