pub struct CombFilter {
    // TODO: your code here
    gain: f32,
    delay: f32,
    delay_line: Vec<f32>,
    filter_type: FilterType,
    sample_rate_hz: f32,
    num_channels: usize,
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

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize) -> Self {
        let delay_time = (max_delay_secs * sample_rate_hz) as usize;
        let delay_line = vec![0.0; delay_time];
        return CombFilter { gain: 0.0,
                            delay: max_delay_secs,
                            delay_line, 
                            filter_type, 
                            sample_rate_hz, 
                            num_channels 
                            };
    }

    pub fn reset(&mut self) {
        for x in &mut self.delay_line {
            *x = 0.0;
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        // input/output: 2D array

        for channel in 0..input.len() {
            // for n = 1:length(x)
            for sample in 0..input[channel].len() {
                let input_sample = input[channel][sample];
                let delayed_sample = self.delay_line[self.delay_line.len() - 1];

                let output_sample = match self.filter_type {
                    FilterType::FIR => {
                        let output_sample = input_sample + self.gain * delayed_sample;
                        self.delay_line.rotate_right(1);
                        self.delay_line[0] = input_sample;
                        output_sample
                    }
                    FilterType::IIR => {
                        let output_sample = input_sample + self.gain * delayed_sample;
                        self.delay_line.rotate_right(1);
                        self.delay_line[0] = output_sample;
                        output_sample
                    }
                };
                output[channel][sample] = output_sample;   
            }
        }
    }    

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => {
                self.gain = value;
                Ok(())
            }
            FilterParam::Delay => {
                self.delay_line = vec![0.0; value as usize];
                Ok(())
            }
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => self.delay,
        }
    }

}


