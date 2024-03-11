use core::num;
use std::{fs::File, io::Write};
use std::{f32::consts::PI, ops::DerefMut};

mod ring_buffer;
mod vibrato;
mod lfo;
// pub mod LFO;
use vibrato::Vibrato;

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
    let sample_rate_hz = spec.sample_rate as f32;
    let num_channels = spec.channels as usize;

    // vibrato param
    let modfreq = 5.0;
    let width = 0.005;
    let mod_amplitude = 1.0;
    let mut vibrato_efx = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);

    // create wav writer
    let mut writer = hound::WavWriter::create(&args[2], spec).unwrap();

    //--------------- block loading ----------------//
    let blocksize = reader.duration() as usize; // Small blocksize will have artifacts in the output..
    // let mut processed_samples = vec![vec![0.0 as i16; reader.duration() as usize]; num_channels as usize];
    // let remainder = (reader.duration() as f32 % blocksize as f32) as usize;

    let mut input_buffer = vec![vec![0.0 as f32; blocksize]; num_channels as usize];
    let mut output_buffer = vec![vec![0.0 as f32; blocksize]; num_channels as usize];
    let mut sample_size = 0;

    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32;
        let channel = i % num_channels;
        let position = i / num_channels;

        input_buffer[channel][position % blocksize] = sample;

        if (i + 1) % (blocksize * num_channels) == 0 {
            let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
            let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

            vibrato_efx.process(&input, &mut output);
            // dbg!(&output);

            for pos in 0..blocksize {
                for ch in 0..num_channels {
                    let processed_sample = output[ch][pos];
                    let sample_i16 = (processed_sample * (i16::MAX as f32)) as i16;
                    writer.write_sample(sample_i16).unwrap();
                }
            }
            input_buffer[channel][position % blocksize] = sample;
        }
        sample_size += 1;     
    }

    // Process remaining samples
    let remaining_samples = (sample_size / num_channels as usize) % blocksize;
    if remaining_samples > 0 {
        println!("Total sample of {}. Process the remaining {} samples", sample_size/num_channels, remaining_samples);

        let input: Vec<&[f32]> = input_buffer.iter().map(|v| &v[..remaining_samples]).collect();
        let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| &mut v[..remaining_samples]).collect();

        vibrato_efx.process(&input, &mut output);

        for pos in 0..remaining_samples {
            for ch in 0..num_channels {
                let processed_sample = output[ch][pos];
                let sample_i16 = (processed_sample * (i16::MAX as f32)) as i16;
                writer.write_sample(sample_i16).unwrap();
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        // test if output equals 1 sample delayed input when modulation amplitude is 0

        let freq = 10.0;
        let amplitude = 1.0;
        let duration = 0.1;
        let sampling_rate = 1000.0;
        let channels = 1;

        let num_samples = (duration * sampling_rate) as usize;

        let mut input_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize]; 
        let mut output_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize];
        
        // create sinusoid input
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
        let width = 0.001; // delayed by 1 sample
        let mod_amplitude = 0.0;
        let sample_rate_hz = sampling_rate;
        let num_channels = 1;
        let mut vibrato = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);
        vibrato.process(&input, &mut output);

        assert_eq!(&input[0][2], &output[0][3]);
        assert_eq!(&input[0][10], &output[0][11]);
        assert_eq!(&input[0][26], &output[0][27]);
        assert_eq!(&input[0][35], &output[0][36]);
    }

    #[test]
    fn test_2() {
        // test if DC input results in DC output, regardless of parameters
        let dc_level = 0.5;
        let duration = 0.1;
        let sampling_rate = 1000.0;
        let channels = 1;

        let num_samples = (duration * sampling_rate) as usize;

        let mut input_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize]; 
        let mut output_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize];
        for i in 0 .. channels {
            for j in 0 .. num_samples {    
                let cur = dc_level;
                input_buffer[i][j] = cur;
            }
        }

        let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
        let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

        let modfreq = 5.0;
        let width = 0.001; // delayed by 1 sample
        let mod_amplitude = 1.0;
        let sample_rate_hz = sampling_rate;
        let num_channels = 1;
        let mut vibrato = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);
        vibrato.process(&input, &mut output);

        assert_eq!(&input[0][5], &output[0][5]);
        assert_eq!(&input[0][35], &output[0][35]);
        assert_eq!(&input[0][25], &output[0][25]);
        assert_eq!(&input[0][45], &output[0][45]);
    }

    #[test]
    fn test_3() {
        // test varying block size

        let input_file_path = "sweep.wav";
        
        let mut reader = hound::WavReader::open(input_file_path).unwrap();
        let spec = reader.spec();
        let sample_rate_hz = spec.sample_rate as f32;
        let num_channels = spec.channels as usize;
        let sample_length = reader.duration() as usize;
        dbg!(sample_length);

        // vibrato param
        let modfreq = 5.0;
        let width = 0.005;
        let mod_amplitude = 1.0;
        let mut vibrato_efx = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);        

        // Varying block sizes to test
        let block_sizes = [16, 32, 64, 256];
        for &block_size in block_sizes.iter() {     
            let blocksize = block_size as usize;

            let mut input_buffer = vec![vec![0.0 as f32; blocksize]; num_channels as usize];
            let mut output_buffer = vec![vec![0.0 as f32; blocksize]; num_channels as usize];
            let mut sample_size = 0;  

            let mut sample_i16_vectors: Vec<Vec<i16>> = Vec::new();

            for (i, sample) in reader.samples::<i16>().enumerate() {
                let sample = sample.unwrap() as f32 / (1 << 15) as f32;
                let channel = i % num_channels;
                let position = i / num_channels;

                input_buffer[channel][position % blocksize] = sample;

                if (i + 1) % (blocksize * num_channels) == 0 {
                    let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
                    let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

                    vibrato_efx.process(&input, &mut output);
                    let mut current_sample_i16_vector: Vec<i16> = Vec::new();

                    for pos in 0..blocksize {
                        for ch in 0..num_channels {
                            let processed_sample = output[ch][pos];
                            let sample_i16 = (processed_sample * (i16::MAX as f32)) as i16;
                            current_sample_i16_vector.push(sample_i16);
                        }
                    }

                    sample_i16_vectors.push(current_sample_i16_vector);

                    input_buffer[channel][position % blocksize] = sample;
                }
                sample_size += 1;
            }

            let remaining_samples = (sample_size / num_channels as usize) % blocksize;
            assert_eq!(remaining_samples, sample_length % blocksize);
        }

    }    

    #[test]
    fn test_4() {
        // test if zero input results

        let dc_level = 0.0; 
        let duration = 0.1;
        let sampling_rate = 1000.0;
        let channels = 1;

        let num_samples = (duration * sampling_rate) as usize;

        let mut input_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize]; 
        let mut output_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize];
        for i in 0 .. channels {
            for j in 0 .. num_samples {    
                let cur = dc_level;
                input_buffer[i][j] = cur;
            }
        }

        let input: Vec<&[f32]> = input_buffer.iter().map(|v| v.as_slice()).collect();
        let mut output: Vec<&mut [f32]> = output_buffer.iter_mut().map(|v| v.as_mut_slice()).collect();

        let modfreq = 5.0;
        let width = 0.001; // delayed by 1 sample
        let mod_amplitude = 1.0;
        let sample_rate_hz = sampling_rate;
        let num_channels = 1;
        let mut vibrato = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);
        vibrato.process(&input, &mut output);

        assert_eq!(&input[0][5], &output[0][5]);
        assert_eq!(&input[0][35], &output[0][35]);
        assert_eq!(&input[0][25], &output[0][25]);
        assert_eq!(&input[0][45], &output[0][45]);        
    }

    #[test]
    fn test_5() {
        // test if there is clipping in the output

        let freq = 10.0;
        let amplitude = 1.0;
        let duration = 0.1;
        let sampling_rate = 1000.0;
        let channels = 1;

        let num_samples = (duration * sampling_rate) as usize;

        let mut input_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize]; 
        let mut output_buffer = vec![vec![0.0 as f32; num_samples]; channels as usize];
        
        // create sinusoid input
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
        let width = 0.001; // delayed by 1 sample
        let mod_amplitude = 0.0;
        let sample_rate_hz = sampling_rate;
        let num_channels = 1;
        let mut vibrato = Vibrato::new(modfreq, width, mod_amplitude, sample_rate_hz, num_channels);
        vibrato.process(&input, &mut output);   

        // Check for clipping in the output
        for channel in output.iter() {
            for &sample in channel.iter() {
                assert!(
                    sample >= -1.0 - 1e-3 && sample <= 1.0 + 1e-3,
                    "Clipping detected in the output: {}",
                    sample
                );
            }
        }          
    }

}

