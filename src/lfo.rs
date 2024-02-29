use std::f32::consts::PI;

pub struct Lfo {
    frequency: f32,
    sample: usize,
    amplitude: f32,
}

impl Lfo {
    pub fn new(frequency: f32, sample: usize, amplitude: f32) -> Self {
        Lfo {
            frequency,
            sample,
            amplitude,
        }
    }

    pub fn generate(&self) -> f32 {
        let value = self.amplitude * (self.frequency * 2.0 * PI * self.sample as f32).sin();
        value
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
    // Example usage
    let frequency = 1.0; // Set the frequency of the LFO
    let sample = 1;    // Set the sample value
    let amplitude = 0.8; // Set the amplitude

    // Create a new SinusoidalLFO instance
    let lfo = Lfo::new(frequency, sample, amplitude);

    // Generate the LFO value
    let lfo_value = lfo.generate();

    // Print or use the generated LFO value
    println!("LFO Value: {}", lfo_value);
    }
}
