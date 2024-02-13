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

    // TODO: Create a comb filter
    let max_delay_secs = 0.05;
    let sample_rate_hz = spec.sample_rate as f32;
    let mut comb_filter = CombFilter::new(FilterType::FIR, max_delay_secs, sample_rate_hz, channels);
    comb_filter.set_param(FilterParam::Gain, 0.5).unwrap();    

    // TODO: Modify this to process audio in blocks using your comb filter and write the result to an audio file.
    //       Use the following block size:
    let block_size = 5; // 1024;

    // Read audio data and write it to the output text file (one column per channel)
    let mut out = File::create(&args[2]).expect("Unable to create file");
    // let mut input_buffer = vec![vec![0.0; block_size]; channels as usize]; 
    let mut input_buffer = vec![vec![0.0; channels]; block_size as usize]; 
    let mut output_buffer = vec![vec![0.0; channels]; block_size as usize];

    println!("Input Buffers: {:?}", input_buffer);
    println!("Input Buffer len: {:?}", input_buffer[0].len());

    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        
        for channel in 0..channels as usize {
            input_buffer[channel].push(sample);
        }

        // if input_buffer.len() == block_size {}

        // write!(out, "{}{}", sample, if i % channels as usize == (channels - 1).into() { "\n" } else { " " }).unwrap();
    }
}
