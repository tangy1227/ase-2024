use crate::ring_buffer::RingBuffer;

pub struct CombFilter {
    filters: Vec<Box<dyn Filter>>
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
        let make_filter = || match filter_type {
            FilterType::FIR => Box::new(FIRFilter::new(max_delay_secs, sample_rate_hz)) as Box<dyn Filter>,
            FilterType::IIR => Box::new(IIRFilter::new(max_delay_secs, sample_rate_hz)) as Box<dyn Filter>,
        };
        let mut filters = Vec::with_capacity(num_channels);
        for _ in 0..num_channels {
            filters.push(make_filter());
        };
        CombFilter { filters }
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset()
        }
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        // Pass each input/output channel to the corresponding filter.
        for i in 0..self.filters.len() {
            self.filters[i].process(input[i], output[i]);
        }
    }

    // A reasonable question: what units should set_param/get_param use for Gain & Delay?
    // Another reasonable question: what if this is called with a delay that's not an integer number of samples?
    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        for filter in &mut self.filters {
            match param {
                FilterParam::Gain => filter.set_gain(value),
                FilterParam::Delay => filter.set_delay(value)?,
            };
        }
        Ok(())
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        if self.filters.is_empty() {
            // Per the interface, this function can't return an error, so we can't indicate the error here (without panicking).
            return 0.0;
        }
        // Just get the parameter from the first filter in the array, since all the filters have the same parameters.
        match param {
            FilterParam::Gain => self.filters[0].get_gain(),
            FilterParam::Delay => self.filters[0].get_delay(),
        }
    }
}

trait Filter {
    fn reset(&mut self);
    fn get_gain(&self) -> f32;
    fn get_delay(&self) -> f32;
    fn set_gain(&mut self, gain: f32);
    fn set_delay(&mut self, delay: f32) -> Result<(), Error>;
    fn process(&mut self, input: &[f32], output: &mut [f32]);
}

struct FilterBase {
    sample_rate: f32,
    delay_line: RingBuffer<f32>,
    gain: f32,
}

impl FilterBase {
    fn new(max_delay: f32, sample_rate: f32) -> Self {
        let delay_line_size = (max_delay * sample_rate).ceil() as usize + 1;
        let delay_line = RingBuffer::new(delay_line_size);
        FilterBase { sample_rate, delay_line, gain: 0.5 }
    }

    fn reset(&mut self) {
        self.delay_line.reset()
    }

    fn get_gain(&self) -> f32 {
        self.gain
    }

    fn get_delay(&self) -> f32 {
        self.delay_line.len() as f32 / self.sample_rate
    }

    fn set_gain(&mut self, gain: f32) {
        self.gain = gain
    }

    fn set_delay(&mut self, delay: f32) -> Result<(), Error> {
        let delay_in_samples = (delay * self.sample_rate).round() as usize;
        if delay < 0.0 || delay_in_samples > (self.delay_line.capacity() - 1) {
            Err(Error::InvalidValue { param: FilterParam::Delay, value: delay })
        } else {
            self.delay_line.set_read_index(self.delay_line.capacity() + self.delay_line.get_write_index() - delay_in_samples);
            Ok(())
        }
    }
}

struct FIRFilter(FilterBase);

impl FIRFilter {
    fn new(max_delay: f32, sample_rate: f32) -> Self { FIRFilter(FilterBase::new(max_delay, sample_rate)) }
}

impl Filter for FIRFilter {
    fn reset(&mut self) { self.0.reset() }
    fn get_gain(&self) -> f32 { self.0.get_gain() }
    fn get_delay(&self) -> f32 { self.0.get_delay() }
    fn set_gain(&mut self, gain: f32) { self.0.set_gain(gain) }
    fn set_delay(&mut self, delay: f32) -> Result<(), Error> { self.0.set_delay(delay) }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (x, y) in input.into_iter().zip(output) {
            // NOTE: We push first to ensure correct handling of zero-delay case.
            self.0.delay_line.push(*x);
            *y = x + self.0.gain * self.0.delay_line.pop();
        }
    }
}

struct IIRFilter(FilterBase);

impl IIRFilter {
    fn new(max_delay: f32, sample_rate: f32) -> Self { IIRFilter(FilterBase::new(max_delay, sample_rate)) }
}

impl Filter for IIRFilter {
    fn reset(&mut self) { self.0.reset() }
    fn get_gain(&self) -> f32 { self.0.get_gain() }
    fn get_delay(&self) -> f32 { self.0.get_delay() }
    fn set_gain(&mut self, gain: f32) { self.0.set_gain(gain) }
    fn set_delay(&mut self, delay: f32) -> Result<(), Error> {
        if (delay * self.0.sample_rate).round() < 1.0 {
            Err(Error::InvalidValue { param: FilterParam::Delay, value: delay })
        } else {
            self.0.set_delay(delay)
        }
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (x, y) in input.into_iter().zip(output) {
            *y = *x + self.0.gain * self.0.delay_line.pop();
            self.0.delay_line.push(*y);
        }
    }
}
