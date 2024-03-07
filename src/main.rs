use std::{f32::consts::PI, fs::File, io::Write, ops::Div};

use comb_filter::{CombFilter, FilterParam, FilterType};

mod ring_buffer;
mod comb_filter;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wav> <output wav> [fir|iir] <delay in seconds> <feedforward/back gain>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    let spec = reader.spec();
    let channels = spec.channels as usize;

    let block_size = 1024;

    let filter_type = if args[3] == "fir" { FilterType::FIR } else { FilterType::IIR };
    let delay: f32 = args[4].parse().unwrap();
    let gain: f32 = args[5].parse().unwrap();
    let sample_rate = spec.sample_rate as f32;
    let mut filter = CombFilter::new(filter_type, delay, sample_rate, channels);
    filter.set_param(FilterParam::Delay, delay).unwrap();
    filter.set_param(FilterParam::Gain, gain).unwrap();

    let mut out = File::create(&args[2]).expect("Unable to create file");
    let mut writer = hound::WavWriter::new(out, spec).unwrap();

    // Read audio data and write it to the output text file (one column per channel)
    let mut block = vec![Vec::<f32>::with_capacity(block_size); channels];
    let mut output_block = vec![vec![0.0_f32; block_size]; channels];
    let num_samples = reader.len() as usize;
    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        block[i % channels].push(sample);
        if (i % (channels * 1024) == 0) || (i == num_samples - 1) {
            // Process block
            let ins = block.iter().map(|c| c.as_slice()).collect::<Vec<&[f32]>>();
            let mut outs = output_block.iter_mut().map(|c| c.as_mut_slice()).collect::<Vec<&mut [f32]>>();
            filter.process(ins.as_slice(), outs.as_mut_slice());
            for j in 0..(channels * block[0].len()) {
                writer.write_sample((output_block[j % channels][j / channels] * (1 << 15) as f32) as i32).unwrap();
            }
            for channel in block.iter_mut() {
                channel.clear();
            }
        }
    }
}


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
        const length: usize = 8192;
        let mut input = [0.0; length];
        let mut output_a = [0.0; length];
        let mut output_b = [0.0; length];

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
        while i < length {
            let block_size = rng.gen_range(0..=std::cmp::min(length - i, 1024));
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
    let matches = if let Err(comb_filter::Error::InvalidValue { param: FilterParam::Delay, value }) = result {
        value == 0.0
    } else {
        false
    };
    assert!(matches);
}
