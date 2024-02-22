use std::{fs::File, io::Write};

mod comb_filter;
use comb_filter::{CombFilter, FilterType, FilterParam, Error};

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input wave filename> <output text filename>", args[0]);
        return
    }

    // Open the input wave file
    let mut reader = hound::WavReader::open(&args[1]).unwrap();
    
    let spec = reader.spec();
    let channels = spec.channels as usize;
    let max_delay_secs = 0.5; // max_delay_sec sets the length of delayline, which determine the delay time
    let sample_rate_hz = spec.sample_rate as f32;
    let delay_time = (max_delay_secs * sample_rate_hz) as f32;
    dbg!(spec);

    // create wav writer
    let mut writer = hound::WavWriter::create(&args[2], spec).expect("Failed to create writer");
    
    let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
    comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();
    comb_filter.set_param(FilterParam::Delay, delay_time).unwrap();

    //-------------- Block loading --------------//
    let block_size = 44100;
    let mut input_buffer = vec![vec![0.0 as f32; block_size]; channels as usize]; 
    let mut output_buffer = vec![vec![0.0 as f32; block_size]; channels as usize];
    let mut sample_size = 0;

    for (i, sample) in reader.samples::<i32>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        let channel = i % channels;
        let position = i / channels;

        input_buffer[channel][position % block_size] = sample; // fill the input buffer

        if (i + 1) % (block_size * channels) == 0 {
            let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
            let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

            comb_filter.process(&input, &mut output);

            // Write the processed samples to the output WAV file
            for pos in 0..block_size {
                for ch in 0..channels {
                    let processed_sample = output[ch][pos];
                    let sample_i16 = (processed_sample * (i16::MAX as f32)) as i16;
                    writer.write_sample(sample_i16).unwrap();
                }
            }

            // Clear the input buffer after processing
            input_buffer = vec![vec![0.0 as f32; block_size]; channels as usize];
        }
        sample_size += 1
    }

    // Process the remaining samples in the last block
    let remaining_samples = (sample_size / channels as usize) % block_size;

    if remaining_samples > 0 {
        println!("Total sample of {}. Processing the last block with {} samples", sample_size / channels, remaining_samples);

        let input: Vec<&[f32]> = input_buffer.iter().map(|v| &v[..remaining_samples]).collect();
        let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| &mut v[..remaining_samples]).collect();

        comb_filter.process(&input, &mut output);

        // Write the processed samples to the output WAV file
        for pos in 0..remaining_samples {
            for ch in 0..channels {
                let processed_sample = output[ch][pos];
                let sample_i16 = (processed_sample * (i16::MAX as f32)) as i16;
                writer.write_sample(sample_i16).unwrap();
            }
        }        

    }  

    writer.finalize().unwrap();
 
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPSILON: f32 = 1e-2;

    #[test]
    fn test_fir_cancellation() {
        let max_delay_secs = 0.1;
        let sample_rate_hz = 10.0;
        let channels = 1;
        let delay_time = (max_delay_secs * sample_rate_hz) as f32;

        // Create a CombFilter with FIR type
        let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
        comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();
        comb_filter.set_param(FilterParam::Delay, delay_time).unwrap();

        // Generate a periodic input signal with a period of 2 seconds
        let period = 2.0;
        let input_signal: Vec<f32> = (0..(sample_rate_hz as usize * period as usize))
            .map(|t| ((t as f32 / sample_rate_hz) * 2.0 * std::f32::consts::PI).sin())
            .collect();

        // Process the input signal
        let mut output_signal = vec![0.0; input_signal.len()];
        let output_signal_slice = output_signal.as_mut_slice();
        comb_filter.process(&[input_signal.as_slice()], &mut [output_signal_slice]);

        // Assert that the output signal is close to zero
        for &sample in output_signal_slice.iter() {
            assert!(sample.abs() < EPSILON);
        }
    }

    #[test]
    fn test_zero_input_signal() {
        let max_delay_secs = 0.1;
        let sample_rate_hz = 44100.0;
        let channels = 2;

        // Create CombFilters with FIR and IIR types
        let mut comb_filter_fir = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
        let mut comb_filter_iir = CombFilter::new(FilterType::IIR, max_delay_secs, sample_rate_hz, channels);

        // Set parameters for both filters
        comb_filter_fir.set_param(FilterParam::Gain, 0.5).unwrap();
        comb_filter_iir.set_param(FilterParam::Gain, 0.5).unwrap();

        // Generate a zero input signal
        let input_signal: Vec<f32> = vec![0.0; 44100]; // Assuming a 1-second zero signal

        // Process the zero input signal using FIR filter
        let mut output_signal_fir = vec![0.0; input_signal.len()];
        let mut output_signal_fir_slice = output_signal_fir.as_mut_slice();
        comb_filter_fir.process(&[input_signal.as_slice()], &mut [output_signal_fir_slice]);

        // Assert that the output signal from FIR filter is close to zero
        for &sample in output_signal_fir.iter() {
            assert!(sample.abs() < EPSILON);
        }

        // Process the zero input signal using IIR filter
        let mut output_signal_iir = vec![0.0; input_signal.len()];
        let mut output_signal_iir_slice = output_signal_iir.as_mut_slice();
        comb_filter_iir.process(&[input_signal.as_slice()], &mut [output_signal_iir_slice]);

        // Assert that the output signal from IIR filter is close to zero
        for &sample in output_signal_iir.iter() {
            assert!(sample.abs() < EPSILON);
        }
    }

    #[test]
    fn test_iir_magnitude_increase() {
        // Assuming you have a known frequency that matches the feedback
        let input_frequency = 1000.0; // Adjust as needed

        let max_delay_secs = 0.1;
        let sample_rate_hz = 44100.0;
        let channels = 2;

        // Calculate the delay time for the given input frequency
        let delay_time = (1.0 / input_frequency) * sample_rate_hz;

        let mut comb_filter_iir = CombFilter::new(FilterType::IIR, max_delay_secs, sample_rate_hz, channels);
        comb_filter_iir.set_param(FilterParam::Gain, 0.5).unwrap();
        comb_filter_iir.set_param(FilterParam::Delay, delay_time as f32).unwrap();

        // Generate a sinusoidal input signal at the specified frequency
        let input_signal: Vec<f32> = (0..44100).map(|t| (2.0 * std::f32::consts::PI * input_frequency * t as f32 / sample_rate_hz).sin()).collect();

        // Process the input signal
        let mut output_iir = vec![0.0; input_signal.len()];
        comb_filter_iir.process(&[input_signal.as_slice()], &mut [output_iir.as_mut_slice()]);

        // Assert that the magnitude of the output signal is greater than the input signal
        for (&input, &output) in input_signal.iter().zip(output_iir.iter()) {
            assert!(output.abs() > input.abs());
        }
    }

    #[test]
    fn test_fir_iir_varying_block_sizes() {
        let max_delay_secs = 0.0;
        let sample_rate_hz = 1000.0; // Use a higher sample rate for better accuracy
        let channels = 1;
        let delay_time = (max_delay_secs * sample_rate_hz) as f32;

        // Create a CombFilter with FIR type
        let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
        comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();
        comb_filter.set_param(FilterParam::Delay, delay_time).unwrap();

        // Generate a sine wave input signal with a period of 2 seconds
        let period = 2.0;
        let input_signal: Vec<f32> = (0..(sample_rate_hz as usize * period as usize))
            .map(|t| ((t as f32 / sample_rate_hz) * 2.0 * std::f32::consts::PI).sin())
            .collect();

        // Define different block sizes to test
        let block_sizes = vec![100, 200, 500, 1000];

        for &block_size in block_sizes.iter() {
            // Process the input signal with varying block sizes
            let mut output_signal = vec![0.0; input_signal.len()];
            let mut output_signal_slice = output_signal.as_mut_slice();

            for i in (0..input_signal.len()).step_by(block_size) {
                let input_block = &input_signal[i..std::cmp::min(i + block_size, input_signal.len())];
                let output_block = &mut output_signal_slice[i..std::cmp::min(i + block_size, input_signal.len())];

                comb_filter.process(&[input_block], &mut [output_block]);
            }

            // Assert that the output signal is close to the input signal
            for (input_sample, output_sample) in input_signal.iter().zip(output_signal.iter()) {
                assert!((input_sample - output_sample).abs() < EPSILON);
            }
        }
    }

    #[test]
    fn test_clipping() {
        let max_delay_secs = 0.1;
        let sample_rate_hz = 44100.0;
        let channels = 2;
    
        // Create a CombFilter with FIR type
        let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
        comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();
    
        // Generate a sine wave input signal with a frequency of 440 Hz
        let input_amplitude = 0.9; // Adjust as needed
        let input_frequency = 440.0;
        let input_signal: Vec<f32> = (0..100).map(|t| input_amplitude * (2.0 * std::f32::consts::PI * input_frequency * t as f32 / sample_rate_hz).sin()).collect();
    
        // Process the input signal
        let mut output_signal = vec![0.0; input_signal.len()];
        let mut output_signal_slice = output_signal.as_mut_slice();
        comb_filter.process(&[input_signal.as_slice()], &mut [output_signal_slice]);
    
        // Check for clipping in the output signal
        for &sample in output_signal_slice.iter() {
            dbg!((sample.abs() - 1.0).abs());
            assert!((sample.abs() - 1.0).abs() > 0.1); // Check if the sample is not close to 1.0 (clipping threshold)
        }
    }

}

