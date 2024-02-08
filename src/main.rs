use std::{fs::File, io::Write};
use hound;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
   show_info();

    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    let args: Vec<String> = std::env::args().collect();

    // TODO: your code here
    // command line arg: target/debug/ase sweep.wav output.txt
    let signal_path = args.get(1).unwrap();
    let output_path = args.get(2).unwrap();

    // Open the input wave file and determine number of channels
    // TODO: your code here; see `hound::WavReader::open`.
    let mut reader = hound::WavReader::open(signal_path).unwrap();

    // Read audio data and write it to the output text file (one column per channel)
    // TODO: your code here; we suggest using `hound::WavReader::samples`, `File::create`, and `write!`.
    // Remember to convert the samples to floating point values and respect the number of channels!
    let mut left_channel: Vec<f32> = Vec::new();
    let mut right_channel: Vec<f32> = Vec::new();
    for sample in reader.samples::<i16>() {
        let sample_value = sample.unwrap();

        if left_channel.len() <= right_channel.len() {
            left_channel.push(sample_value as f32 / 32768.0);
        } else {
            right_channel.push(sample_value as f32 / 32768.0);
        }        
    }

    let mut file = File::create(output_path).unwrap();
    for (l, r) in left_channel.iter().zip(right_channel.iter()) {
        write!(file, "{}, {}\n", l, r).unwrap();
    }

}
