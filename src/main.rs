use std::fs::File;

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

    let out = File::create(&args[2]).expect("Unable to create file");
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
