use crate::ring_buffer::RingBuffer;

pub struct CombFilter {
    filter_type: FilterType,
    sample_rate: f32,
    gain: f32,
    delay_lines: Vec<RingBuffer<f32>>,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

// Helper functions for core of FIR and IIR comb filters.
fn process_fir(gain: f32, delay_line: &mut RingBuffer<f32>, input: &[f32], output: &mut [f32]) {
    for (x, y) in input.iter().zip(output) {
        // NOTE: We push first to ensure correct handling of zero-delay case.
        delay_line.push(*x);
        *y = x + gain * delay_line.pop();
    }
}

fn process_iir(gain: f32, delay_line: &mut RingBuffer<f32>, input: &[f32], output: &mut [f32]) {
    for (x, y) in input.iter().zip(output) {
        *y = x + gain * delay_line.pop();
        delay_line.push(*y);
    }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let mut delay_lines = Vec::with_capacity(num_channels);
        let delay_line_size = (max_delay_secs * sample_rate_hz).ceil() as usize + 1;
        for _ in 0..num_channels {
            let delay_line = RingBuffer::new(delay_line_size);
            delay_lines.push(delay_line);
        };
        CombFilter {
            filter_type,
            sample_rate: sample_rate_hz,
            gain: 0.5,
            delay_lines,
        }
    }

    pub fn reset(&mut self) {
        for delay_line in &mut self.delay_lines {
            delay_line.reset()
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        let process_mono = match self.filter_type {
            FilterType::FIR => process_fir,
            FilterType::IIR => process_iir,
        };
        // Process each input/output channel using the corresponding delay line.
        for i in 0..self.delay_lines.len() {
            process_mono(self.gain, &mut self.delay_lines[i], input[i], output[i]);
        }
    }

    fn set_delay(&mut self, delay: f32) -> Result<(), Error> {
        // Convert from seconds to samples.
        let delay = (delay * self.sample_rate).round();
        let min_delay = match self.filter_type {
            FilterType::FIR => 0.0,
            // Can't have a zero-delay cycle.
            FilterType::IIR => 1.0,
        };
        let max_delay = (self.delay_lines[0].capacity() - 1) as f32;
        if delay < min_delay || delay > max_delay {
            Err(Error::InvalidValue { param: FilterParam::Delay, value: delay })
        } else {
            // Change delay by adjusting gap between read and write indices in ring buffers.
            let read_index = self.delay_lines[0].capacity() + self.delay_lines[0].get_write_index() - delay as usize;
            for delay_line in self.delay_lines.iter_mut() {
                delay_line.set_read_index(read_index);
            }
            Ok(())
        }
    }

    // A reasonable question: what units should set_param/get_param use for Gain & Delay?
    // Another reasonable question: what if this is called with a delay that's not an integer number of samples?
    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => { self.gain = value; Ok(()) },
            FilterParam::Delay => self.set_delay(value),
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.delay_lines[0].len() as f32 / self.sample_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // "Close enough" value for floating point assertions.
    const EPSILON: f32 = 1e-5;

    #[test]
    fn test_fir_interference() {
        // Feed in sine wave with period = delay * 2; should cancel with itself.
        let sample_rate = 44100.0;
        let delay = 0.1;
        let freq = 1.0 / (delay * 2.0);

        let mut filter = CombFilter::new(FilterType::FIR, delay, sample_rate, 1);
        filter.set_param(FilterParam::Delay, delay).unwrap();
        filter.set_param(FilterParam::Gain, 1.0).unwrap();

        for i in 0..(delay * sample_rate * 10.0).ceil() as usize {
            let inp = [(2.0*PI*freq*i as f32/sample_rate).sin()];
            let ins: &[&[f32]] = &[&inp];
            let mut out = [0.0_f32];
            let outs: &mut [&mut [f32]] = &mut[&mut out];
            filter.process(ins, outs);
            if i as f32 >= delay * sample_rate {
                assert!(out[0].abs() < EPSILON);
            }
        }
    }

    #[test]
    fn test_multichannel() {
        // Test multichannel support. Ensure that filter output is correct for each channel.
        let input = [
            [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0],
            [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        ];
        let mut output = [[0.0_f32; 10]; 2];
        let ins: &[&[f32]] = &[&input[0], &input[1]];
        let (out0, out1) = output.split_at_mut(1);
        let outs: &mut[&mut [f32]] = &mut [&mut out0[0], &mut out1[0]];

        let sample_rate = 10.0;
        let gain = 0.1;
        // Delay of one sample.
        let delay = 1.0 / sample_rate;

        let mut filter = CombFilter::new(FilterType::FIR, delay, sample_rate, 2);
        filter.set_param(FilterParam::Delay, delay).unwrap();
        filter.set_param(FilterParam::Gain, gain).unwrap();

        filter.process(ins, outs);
        for i in 0..2 {
            for j in 1..10 {
                let expected = input[i][j] + gain * input[i][j-1];
                assert!((output[i][j] - expected).abs() < EPSILON);
            }
        }
    }

    #[test]
    fn test_iir_interference() {
        // Feed in sine wave with period = delay * 2; should cancel with itself.
        let sample_rate = 44100.0;
        let delay = 0.1;
        let freq = 1.0 / delay;

        let mut filter = CombFilter::new(FilterType::IIR, delay, sample_rate, 1);
        filter.set_param(FilterParam::Delay, delay).unwrap();
        filter.set_param(FilterParam::Gain, 1.0).unwrap();

        for i in 0..(delay * sample_rate * 10.0).ceil() as usize {
            let inp = [(2.0*PI*freq*i as f32/sample_rate).sin()];
            let ins: &[&[f32]] = &[&inp];
            let mut out = [0.0_f32];
            let outs: &mut [&mut [f32]] = &mut[&mut out];
            filter.process(ins, outs);
            if i as f32 >= delay * sample_rate {
                // Due to feedback loop, signal should keep constructively interfering with delay, producing larger and larger values.
                // Output should be delay_cycle * input, where delay_cycle is how many times we've gone around delay (starting from 1).
                let delay_cycle = i / ((delay * sample_rate) as usize) + 1;
                assert!((out[0] / (delay_cycle as f32) - inp[0]).abs() < EPSILON);
            }
        }
    }

    #[test]
    fn test_block_size() {
        // For simplicity, the other tests process one sample at a time.
        // In contrast, this test specifically checks for handling of variable block sizes.
        let sample_rate = 12345.0;
        let delay = 0.1;

        for filter_type in [FilterType::FIR, FilterType::IIR] {
            const LENGTH: usize = 8192;
            let mut input = [0.0; LENGTH];
            let mut output_a = [0.0; LENGTH];
            let mut output_b = [0.0; LENGTH];

            // Generate random input signal.
            use rand::{Rng, SeedableRng};
            let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
            for x in input.iter_mut() {
                *x = rng.gen();
            }

            // First, compute the output in one go.
            let ins: &[&[f32]] = &[&input];
            let outs: &mut [&mut [f32]] = &mut[&mut output_a];
            let mut filter = CombFilter::new(filter_type, delay, sample_rate, 1);
            filter.set_param(FilterParam::Delay, delay).unwrap();
            filter.process(ins, outs);

            // Then, compute the output in many smaller blocks with variable size (from 0-1024).
            filter.reset();
            filter.set_param(FilterParam::Delay, delay).unwrap();
            let mut i = 0;
            while i < LENGTH {
                let block_size = rng.gen_range(0..=std::cmp::min(LENGTH - i, 1024));
                let ins = &[&input[i..i + block_size]];
                let outs = &mut[&mut output_b[i..i + block_size]];
                filter.process(ins, outs);
                i += block_size;
            }

            // Ensure outputs are identical.
            for (&a, b) in output_a.iter().zip(output_b) {
                assert_eq!(a, b);
            }
        }
    }

    #[test]
    fn test_silence() {
        // Test that output is silent for silent input (signal of zeros).
        // NOTE: Choice of delay, gain, sample rate should be irrelevant for this test.
        let sample_rate = 12345.0;
        let delay = 0.1;

        for filter_type in [FilterType::FIR, FilterType::IIR] {
            let mut filter = CombFilter::new(filter_type, delay, sample_rate, 1);
            for _ in 0..(delay * sample_rate * 10.0).ceil() as usize {
                let inp = [0.0];
                let ins: &[&[f32]] = &[&inp];
                let mut out = [0.0_f32];
                let outs: &mut [&mut [f32]] = &mut[&mut out];
                filter.process(ins, outs);
                assert_eq!(out[0], 0.0);
            }
        }
    }


    #[test]
    fn test_no_delay() {
        // Test behavior when delay is set to zero.
        let sample_rate = 12345.0;
        let max_delay = 0.1;

        // For FIR filter, output should be merely scaled when delay is zero.
        let mut filter = CombFilter::new(FilterType::FIR, max_delay, sample_rate, 1);
        // Set delay to 0.
        filter.set_param(FilterParam::Delay, 0.0).unwrap();
        let expected_gain = 1.0 + filter.get_param(FilterParam::Gain);

        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
        for _ in 0..(max_delay * sample_rate * 10.0).ceil() as usize {
            let inp = [rng.gen()];
            let ins: &[&[f32]] = &[&inp];
            let mut out = [0.0_f32];
            let outs: &mut [&mut [f32]] = &mut[&mut out];
            filter.process(ins, outs);
            // Since delay is 0, we expect `out = in + gain * in`.
            assert!((out[0] / expected_gain - inp[0]).abs() < EPSILON);
        }

        // For IIR filter, a delay of zero is invalid, as it would create a zero-delay feedback loop.
        // Check for the expected error.
        let mut filter = CombFilter::new(FilterType::IIR, max_delay, sample_rate, 1);
        let result = filter.set_param(FilterParam::Delay, 0.0);
        let matches = if let Err(Error::InvalidValue { param: FilterParam::Delay, value }) = result {
            value == 0.0
        } else {
            false
        };
        assert!(matches);
    }
}
