use crate::ring_buffer::RingBuffer;
use std::{f32::consts::PI, ops::DerefMut};
use crate::lfo::Lfo;

pub struct Vibrato {
    DELAY: f32,
    // Delayline: RingBuffer<f32>,
    Delayline: Vec<f32>,
    WIDTH: f32,     // width_samples
    MODFREQ: f32,   // mod_freq_samples
    num_channels: usize,
    mod_amplitude: f32
}

#[derive(Debug, Clone, Copy)]
pub enum fxParam {
    ModFreq,
    Width,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: fxParam, value: f32 },
}

impl Vibrato {
    pub fn new(modfreq: f32, width: f32, mod_amplitude: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let Delay = width;
        let DELAY = (Delay * sample_rate_hz).round();
        let WIDTH = (width * sample_rate_hz).round();

        if WIDTH > DELAY {
            panic!("Error: delay greater than basic delay !!!");
        }

        let MODFREQ = modfreq / sample_rate_hz;
        let L = 2.0 + DELAY + WIDTH * 2.0;
        // let Delayline = RingBuffer::new(L as usize);
        let Delayline = vec![0.0; L as usize];
        
        Vibrato {
            DELAY,
            Delayline,
            WIDTH,
            MODFREQ,
            num_channels,
            mod_amplitude
        }

    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let M = self.MODFREQ;
        let frequency = self.MODFREQ;

        for channel in 0..input.len() {
            for sample in 0..input[channel].len() {
                let amplitude = self.mod_amplitude;
                let lfo = Lfo::new(frequency, sample, amplitude);
                let MOD = lfo.generate();
                // let MOD = (M * 2.0 * PI * sample as f32).sin();

                let TAP = 1.0 + self.DELAY + self.WIDTH * MOD;
                let i = TAP.floor() as usize;
                let frac = TAP - i as f32;

                let input_sample = input[channel][sample];

                let i_plus_1 = (i + 1) % self.Delayline.len();
                self.Delayline[0] = input_sample;
                self.Delayline.rotate_right(1);
                
                // linear interpolation
                let output_sample = self.Delayline[i_plus_1]*frac + self.Delayline[i] * (1.0-frac);

                output[channel][sample] = output_sample
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test() {
        // This test is for comparing the output to matlab code with the same sinusoid input
        let freq = 10.0;
        let amplitude = 1.0;
        let duration = 0.1;
        let sampling_rate = 1000.0;
        let channels = 1;

        let num_samples = (duration * sampling_rate) as usize;

        let mut input_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize]; 
        let mut output_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize];
        for i in 0 .. channels {
            for j in 0 .. num_samples {
                let t = j as f32 / sampling_rate;        
                let cur = amplitude * (2.0 * PI * freq * t).sin();
                input_buffer[i][j] = cur;
            }
        }

        let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
        let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

        let modfreq = 5.0;
        let width = 0.001;
        let mod_amplitude = 1.0;
        let sample_rate_hz = sampling_rate;
        let num_channels = 1;
        let mut vibrato = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);
        vibrato.process(&input, &mut output);
        dbg!(output);
    }
}