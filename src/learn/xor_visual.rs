//! State machine for the step-by-step XOR cipher visualization.

use super::steps::EncryptionStep;
use crate::entropy::{EntropySource, SystemEntropy};

/// Manages the state of the XOR learning visualization.
pub struct EncryptionProcess {
    pub input_text: String,
    pub steps: Vec<EncryptionStep>,
    pub current_step_index: usize,
    pub is_playing: bool,
    /// Steps per second (approximate).
    pub speed: f32,
    /// Timestamp of the last animation update.
    pub last_update: f64,
}

impl Default for EncryptionProcess {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            steps: Vec::new(),
            current_step_index: 0,
            is_playing: false,
            speed: 1.0,
            last_update: 0.0,
        }
    }
}

impl EncryptionProcess {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the visualization with the given text.
    ///
    /// Iterates over UTF-8 bytes (`text.as_bytes()`), so multi-byte Turkish
    /// characters and emoji are represented exactly.
    pub fn start(&mut self, text: &str) {
        self.input_text = text.to_string();
        self.steps.clear();
        self.current_step_index = 0;
        self.is_playing = false;

        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return;
        }

        let mut entropy = SystemEntropy::new();
        let mut keystream = vec![0u8; bytes.len()];
        entropy.fill_bytes(&mut keystream);

        for (byte_index, character) in text.char_indices() {
            let width = character.len_utf8();
            for offset in 0..width {
                let index = byte_index + offset;
                self.steps.push(EncryptionStep::new(
                    bytes[index],
                    keystream[index],
                    character,
                    offset,
                ));
            }
        }
    }

    pub fn current_step(&self) -> Option<&EncryptionStep> {
        self.steps.get(self.current_step_index)
    }

    pub fn next_step(&mut self) {
        if self.current_step_index + 1 < self.steps.len() {
            self.current_step_index += 1;
        } else {
            self.is_playing = false;
        }
    }

    pub fn prev_step(&mut self) {
        if self.current_step_index > 0 {
            self.current_step_index -= 1;
        }
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
    }

    /// Advance the animation based on the current time.
    pub fn update(&mut self, time: f64) {
        if self.is_playing {
            let interval = 1.0 / self.speed.max(0.1) as f64;
            if time - self.last_update > interval {
                self.next_step();
                self.last_update = time;
            }
        } else {
            self.last_update = time;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_multibyte_text_produces_one_step_per_byte() {
        let text = "şğİ";
        let mut process = EncryptionProcess::new();
        process.start(text);

        assert_eq!(process.steps.len(), text.len());
        assert_eq!(process.steps.len(), 6);

        for (step, &expected_byte) in process.steps.iter().zip(text.as_bytes()) {
            assert_eq!(step.input_byte, expected_byte);
            assert_eq!(step.input_byte ^ step.keystream_byte, step.result_byte);
        }
    }

    #[test]
    fn emoji_is_preserved_byte_by_byte() {
        let text = "🔐";
        let mut process = EncryptionProcess::new();
        process.start(text);

        assert_eq!(process.steps.len(), 4);
        assert_eq!(process.steps[0].character, '🔐');
        assert_eq!(process.steps[3].byte_offset, 3);
    }

    #[test]
    fn navigation_is_bounded_and_empty_input_has_no_steps() {
        let mut process = EncryptionProcess::new();
        process.start("AB");

        process.prev_step();
        assert_eq!(process.current_step_index, 0, "prev must not underflow");
        process.next_step();
        assert_eq!(process.current_step_index, 1);
        process.next_step();
        assert_eq!(process.current_step_index, 1, "next must not overflow");
        assert!(!process.is_playing);

        process.toggle_play();
        assert!(process.is_playing);
        process.toggle_play();
        assert!(!process.is_playing);

        process.start("");
        assert!(process.steps.is_empty());
        assert!(process.current_step().is_none());
        assert_eq!(process.current_step_index, 0);
    }

    #[test]
    fn update_advances_playing_process_only_after_the_interval() {
        let mut process = EncryptionProcess::new();
        process.start("ABC");
        process.speed = 1.0;

        process.update(10.0);
        assert_eq!(process.current_step_index, 0);

        process.toggle_play();
        process.update(10.5);
        assert_eq!(process.current_step_index, 0, "too early to advance");
        process.update(11.5);
        assert_eq!(process.current_step_index, 1);

        process.toggle_play();
        process.update(20.0);
        assert_eq!(
            process.current_step_index, 1,
            "paused process must not advance"
        );
    }

    #[test]
    fn every_step_satisfies_the_xor_identity() {
        let mut process = EncryptionProcess::new();
        process.start("ş🔐Z");

        for step in &process.steps {
            assert_eq!(step.input_byte ^ step.keystream_byte, step.result_byte);
            assert_eq!(step.input_binary().len(), 8);
            assert_eq!(step.result_binary().len(), 8);
        }
    }
}
