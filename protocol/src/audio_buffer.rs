use std::collections::BTreeMap;

use crate::cowconnect::SampleType;

pub struct AudioBuffer {
    last_seq: u64,
    curr_sample: Box<dyn Iterator<Item = SampleType>>,
    audio_buffer: BTreeMap<u64, Vec<SampleType>>,
}

impl AudioBuffer {
    const WARP_THRESHOLD: i64 = i16::MAX as i64 / 2;

    pub fn new() -> Self {
        Self {
            last_seq: 0,
            curr_sample: Box::new(Vec::new().into_iter()),
            audio_buffer: BTreeMap::new(),
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

    /// Resets the AudioBuffer to the initial state
    pub fn clear(&mut self) {
        self.audio_buffer = BTreeMap::new();
        self.curr_sample = Box::new(vec![].into_iter());
        self.last_seq = 0;
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        self.curr_sample.next().or_else(|| {
            let maybe_iter = self
                .audio_buffer
                .pop_first()
                .map(|(_, v)| Box::new(v.into_iter()));

            if let Some(iter) = maybe_iter {
                self.curr_sample = iter;
                return self.curr_sample.next();
            }

            None
        })
    }

    pub fn available_samples(&self) -> usize {
        self.audio_buffer.values().map(|x| x.len()).sum()
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}
