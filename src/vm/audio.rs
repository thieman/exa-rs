use std::f64;

use fastrand;
use synthrs::filter::*;
use synthrs::synthesizer::quantize_samples;

use super::VM;

pub trait AudioSample {
    // Set frequency from register value. 60 = middle C, semitone
    // increments from there. Bounded 1-99.
    fn set_frequency(&mut self, value: i32);

    /// Fill audio_buffer for next frame and return borrow of it
    fn sample(&mut self) -> &[f64; 44100 / 60];
}

pub trait WaveForm: Default + AudioSample {}

#[derive(Debug)]
pub struct SquareWave {
    samples: Vec<f64>,
    pos: f64,
    audio_buffer: [f64; 44100 / 60],
    pub frequency: f64,
}

impl AudioSample for SquareWave {
    fn set_frequency(&mut self, value: i32) {
        let steps = value as f64 - 60.0;
        self.frequency = f64::powf(2.0, steps / 12.0) * 261.63;
    }

    fn sample(&mut self) -> &[f64; 44100 / 60] {
        for idx in 0..44100 / 60 {
            self.pos += self.frequency;
            if self.pos > 44100.0 {
                self.pos -= 44100.0;
            }

            let sample_idx = self.pos.floor() as usize;
            self.audio_buffer[idx] = self.samples[sample_idx];
        }
        &self.audio_buffer
    }
}

impl Default for SquareWave {
    fn default() -> Self {
        let mut samples = vec![0.0; 44100];
        for idx in 0..44100 {
            let t = (2.0 * f64::consts::PI * idx as f64 / 44100.0).sin();
            samples[idx] = if t > 0.0 { 1.0 } else { -1.0 };
        }

        SquareWave {
            samples,
            pos: 0.0,
            audio_buffer: [0.0; 44100 / 60],
            frequency: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct TriangleWave {
    samples: Vec<f64>,
    pos: f64,
    audio_buffer: [f64; 44100 / 60],
    frequency: f64,
}

impl AudioSample for TriangleWave {
    fn set_frequency(&mut self, value: i32) {
        let steps = value as f64 - 60.0;
        self.frequency = f64::powf(2.0, steps / 12.0) * 261.63;
    }

    fn sample(&mut self) -> &[f64; 44100 / 60] {
        for idx in 0..44100 / 60 {
            self.pos += self.frequency;
            if self.pos > 44100.0 {
                self.pos -= 44100.0;
            }

            let sample_idx = self.pos.floor() as usize;
            self.audio_buffer[idx] = self.samples[sample_idx];
        }
        &self.audio_buffer
    }
}

impl Default for TriangleWave {
    fn default() -> Self {
        let mut samples = vec![0.0; 44100];
        for idx in 0..44100 {
            let t = (2.0 / f64::consts::PI)
                * (2.0 * f64::consts::PI * idx as f64 / 44100.0).sin().asin();
            samples[idx] = t;
        }

        TriangleWave {
            samples,
            pos: 0.0,
            audio_buffer: [0.0; 44100 / 60],
            frequency: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct Noise {
    samples: Vec<f64>,
    pos: f64,
    audio_buffer: [f64; 44100 / 60],
    frequency: f64,
}

impl AudioSample for Noise {
    fn set_frequency(&mut self, value: i32) {
        let steps = value as f64 - 60.0;
        self.frequency = f64::powf(2.0, steps / 12.0) * 261.63;
    }

    fn sample(&mut self) -> &[f64; 44100 / 60] {
        for idx in 0..44100 / 60 {
            self.pos += self.frequency;
            if self.pos > 44100.0 {
                self.pos -= 44100.0;
            }

            let sample_idx = self.pos.floor() as usize;
            self.audio_buffer[idx] = self.samples[sample_idx];
        }
        &self.audio_buffer
    }
}

impl Default for Noise {
    fn default() -> Self {
        let mut samples = vec![0.0; 44100];
        for idx in 0..44100 {
            let value = (fastrand::f64() - 0.5) * 2.0;
            samples[idx] = value;
        }

        Noise {
            samples,
            pos: 0.0,
            audio_buffer: [0.0; 44100 / 60],
            frequency: 0.0,
        }
    }
}

impl<'a> VM<'a> {
    /// Return interleaved stereo audio stream for a single
    /// 60hz frame of the VM.
    pub fn audio_frame(&mut self) -> &[i16; (44100 / 60) * 2] {
        if self.redshift.is_none() {
            return &self.audio_buffer;
        }

        let sqr0_value = self.redshift.as_ref().unwrap().sqr0.borrow().value;
        let sqr1_value = self.redshift.as_ref().unwrap().sqr1.borrow().value;
        let tri0_value = self.redshift.as_ref().unwrap().tri0.borrow().value;
        let nse0_value = self.redshift.as_ref().unwrap().nse0.borrow().value;

        let mut sqr0_wave = self.redshift.as_ref().unwrap().sqr0_wave.borrow_mut();
        let mut sqr1_wave = self.redshift.as_ref().unwrap().sqr1_wave.borrow_mut();
        let mut tri0_wave = self.redshift.as_ref().unwrap().tri0_wave.borrow_mut();
        let mut nse0_wave = self.redshift.as_ref().unwrap().nse0_wave.borrow_mut();

        sqr0_wave.set_frequency(sqr0_value);
        sqr1_wave.set_frequency(sqr1_value);
        tri0_wave.set_frequency(tri0_value);
        nse0_wave.set_frequency(nse0_value);

        let mut waves: Vec<&[i16]> = vec![];
        if sqr0_value > 0 {
            // waves.push(sqr0_wave.sample());
        }

        if sqr1_value > 0 {
            // waves.push(sqr1_wave.sample());
        }

        if tri0_value > 0 {
            // waves.push(tri0_wave.sample());
        }

        let bandpass = bandpass_filter(
            cutoff_from_frequency(nse0_wave.frequency * 0.95, 44100),
            cutoff_from_frequency(nse0_wave.frequency * 1.05, 44100),
            0.01,
        );
        let nse0_i16 = quantize_samples::<i16>(&convolve(&bandpass, nse0_wave.sample()));
        if nse0_value > 0 {
            waves.push(&nse0_i16);
        }

        if waves.len() == 0 {
            self.audio_buffer.iter_mut().for_each(|m| *m = 0);
            return &self.audio_buffer;
        }

        for idx in 0..44100 / 60 {
            let mut value = 0;
            for wave in waves.iter() {
                value += wave[idx] / waves.len() as i16;
            }
            self.audio_buffer[idx * 2] = value;
            self.audio_buffer[idx * 2 + 1] = value;
        }

        &self.audio_buffer
    }
}
