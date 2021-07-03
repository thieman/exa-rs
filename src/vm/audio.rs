use std::f64;

use synthrs::filter::*;
use synthrs::synthesizer::{make_samples, quantize_samples};
use synthrs::wave::sine_wave;

use super::VM;

pub trait AudioSample {
    // Set frequency from register value. 60 = middle C, semitone
    // increments from there. Bounded 1-99.
    fn set_frequency(&mut self, value: i32);

    /// Fill audio_buffer for next frame and return borrow of it
    fn sample(&mut self) -> &[f64; 44100 / 30];
}

pub trait WaveForm: Default + AudioSample {}

#[derive(Debug)]
pub struct SquareWave {
    samples: [f64; 44100],
    pos: f64,
    audio_buffer: [f64; 44100 / 30],
    pub frequency: f64,
}

impl AudioSample for SquareWave {
    fn set_frequency(&mut self, value: i32) {
        let steps = value as f64 - 60.0;
        self.frequency = f64::powf(2.0, steps as f64 / 12.0) * 261.63;
    }

    fn sample(&mut self) -> &[f64; 44100 / 30] {
        for idx in 0..44100 / 30 {
            self.pos += self.frequency;
            if self.pos > 44100.0 {
                self.pos -= 44100.0;
            }

            let sample_idx = self.pos.floor() as usize;
            self.audio_buffer[idx] = self.samples[sample_idx];
        }

        // antialiasing pass, this is necessary since the frequency
        // might not be divisible by the sampling rate
        // let mut idx = 1;
        // let mut current = i16::MAX;
        // while idx < (44100 / 30) - 1 {
        //     if self.audio_buffer[idx] != current {
        //         current = self.audio_buffer[idx];
        //         if current == i16::MAX {
        //             self.audio_buffer[idx - 1] = -16383;
        //             self.audio_buffer[idx] = 0;
        //             self.audio_buffer[idx + 1] = 16383;
        //             idx += 1;
        //         } else if current == i16::MIN {
        //             self.audio_buffer[idx - 1] = 16383;
        //             self.audio_buffer[idx] = 0;
        //             self.audio_buffer[idx + 1] = -16383;
        //             idx += 1;
        //         }
        //     }
        //     idx += 1;
        // }
        &self.audio_buffer
    }
}

impl Default for SquareWave {
    fn default() -> Self {
        let mut samples = [0.0; 44100];
        for idx in 0..44100 {
            let t = (2.0 * f64::consts::PI * idx as f64 / 44100.0).sin();
            // samples[idx] = if t > 0.0 { i16::MAX } else { i16::MIN };
            samples[idx] = t;
        }

        SquareWave {
            samples,
            pos: 0.0,
            audio_buffer: [0.0; 44100 / 30],
            frequency: 0.0,
        }
    }
}

impl<'a> VM<'a> {
    /// Return interleaved stereo audio stream for a single
    /// 30hz frame of the VM.
    pub fn audio_frame(&mut self) -> &[i16; (44100 / 30) * 2] {
        if self.redshift.is_none() {
            return &self.audio_buffer;
        }

        let sqr0_value = self.redshift.as_ref().unwrap().sqr0.borrow().value;
        let _sqr1 = self.redshift.as_ref().unwrap().sqr1.borrow().value;
        let _tri0 = self.redshift.as_ref().unwrap().tri0.borrow().value;
        let _nse0 = self.redshift.as_ref().unwrap().nse0.borrow().value;

        if sqr0_value == 0 {
            self.audio_buffer.iter_mut().for_each(|m| *m = 0);
            return &self.audio_buffer;
        }

        let sqr0_wave = &mut self.redshift.as_mut().unwrap().sqr0_wave;
        sqr0_wave.set_frequency(sqr0_value);

        let high_pass = bandpass_filter(
            cutoff_from_frequency(90.0, 44100),
            cutoff_from_frequency(14000.0, 44100),
            0.01,
        );
        // let samples = quantize_samples::<i16>(&convolve(&high_pass, sqr0_wave.sample()));
        let samples = quantize_samples::<i16>(sqr0_wave.sample());
        // println!("{}", samples.len());

        for (idx, value) in samples.iter().enumerate() {
            if idx * 2 >= 2940 {
                break;
            }
            self.audio_buffer[idx * 2] = *value;
            self.audio_buffer[idx * 2 + 1] = *value;
        }

        println!("{:?}", &self.audio_buffer);
        &self.audio_buffer
    }
}
