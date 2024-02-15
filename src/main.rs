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
    let max_delay_secs = 0.01;
    let sample_rate_hz = spec.sample_rate as f32;

    let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
    comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();

    //------- Block loading failed -------//
    // let block_size = 1024; // 1024;
    // let mut input_buffer = vec![vec![0.0; block_size]; channels as usize]; 
    // let mut output_buffer = vec![vec![0.0; block_size]; channels as usize];
    // let mut input_buffer: Vec<&[f32]> = input_buffer.iter_mut().map(|v| v.as_slice()).collect();
    // let mut output_buffer: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();
    // println!("Input Buffers: {:?}", input_buffer);
    // println!("Output Buffers: {:?}", output_buffer);
    // println!("Input Buffer len: {:?}", input_buffer[0].len());

    // for (i, sample) in reader.samples::<i32>().enumerate() {
    //     let sample = sample.unwrap() as f32 / (1 << 15) as f32;
    //     write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    // }

    //------- Sample loading -------//
    let mut left_channel: Vec<f32> = Vec::new();
    let mut right_channel: Vec<f32> = Vec::new();
    for sample in reader.samples::<i32>() {
        let sample_value = sample.unwrap();

        if left_channel.len() <= right_channel.len() {
            left_channel.push(sample_value as f32 / 32768.0);
        } else {
            right_channel.push(sample_value as f32 / 32768.0);
        }        
    }

    let audio = [left_channel, right_channel];
    let input: &[&[f32]] = &[&audio[0], &audio[1]];
    let mut output_buffer = vec![vec![0.0; input[0].len()]; channels];

    // Calculate the output with comb filter
    let effect = comb_filter.process(&input, &mut output_buffer);

    // Write the output to txt
    let mut out = File::create("output.txt").expect("Unable to create file");
    for (l, r) in effect[0].iter().zip(effect[1].iter()) {
        write!(out, "{}, {}\n", l, r).unwrap();
    }

    // Write the output to a WAV file
    let spec = hound::WavSpec {
        channels: channels as u16,
        sample_rate: 44100 as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };    

    let mut writer = hound::WavWriter::create(&args[2], spec).expect("Failed to create WAV writer");

    for frame in effect.iter() {
        for &sample in frame.iter() {
            writer.write_sample(sample as f32).expect("Failed to write sample");
        }
    }    
 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fir_cancellation() {
        let mut comb_filter = CombFilter::new(FilterType::FIR, 1.0, 44100.0, 1);
        comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();
        let input_freq = 1000.0; // Choose an input frequency
        let sample_rate = 44100.0;

        // Calculate the delay time that corresponds to the input frequency
        let delay_time = (sample_rate / input_freq) as usize;

        // Create an input signal with a single cycle of the input frequency
        let input_signal: Vec<f32> = (0..delay_time).map(|i| (2.0 * std::f32::consts::PI * input_freq * (i as f32) / sample_rate).sin()).collect();

        // Process the input signal
        let mut output = vec![0.0; delay_time];
        let effect = comb_filter.process(&[&input_signal], &mut vec![output.clone()]);

        // Check that the output is close to zero due to cancellation
        for sample in effect[0].iter() {
            assert!(sample.abs() < 1.0, "output is too large");
        }
    }

    #[test]
    fn test_zero_input_signal() {
        let max_delay_secs = 0.1;
        let sample_rate_hz = 44100.0;
        let channels = 2;
        let mut comb_filter_fir = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
        let mut comb_filter_iir = CombFilter::new(FilterType::IIR, max_delay_secs, sample_rate_hz, channels);

        comb_filter_fir.set_param(FilterParam::Gain, 0.5).unwrap();
        comb_filter_iir.set_param(FilterParam::Gain, 0.5).unwrap();

        // Create zero input signal
        let input_other = vec![vec![0.0; 10], vec![0.0; 10]];
        let input: &[&[f32]] = &[&input_other[0], &input_other[1]];
        let mut output_buffer_fir = vec![vec![0.0; 10]; channels];
        let mut output_buffer_iir = vec![vec![0.0; 10]; channels];

        let result_fir = comb_filter_fir.process(&input, &mut output_buffer_fir);
        println!("{:?}", result_fir);

        let result_iir = comb_filter_iir.process(&input, &mut output_buffer_iir);
        println!("{:?}", result_iir);
    }

}