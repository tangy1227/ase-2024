#[derive(Debug)]
pub struct RingBuffer<T> {
    // TODO: fill this in.
    head: usize,
    tail: usize,
    capacity: usize,
    ringbuff: Vec<T>
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.

        RingBuffer {
            head: 0,
            tail: 0,
            capacity: length,
            ringbuff: vec![T::default(); length],
        }

    }

    pub fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
        for element in &mut self.ringbuff {
            *element = T::default();
        }
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        self.ringbuff[self.tail] = value;
    }

    pub fn peek(&self) -> T {
        self.ringbuff[self.head]
    }

    pub fn get(&self, offset: usize) -> T {
        let ind = (self.head + offset) % self.capacity;
        self.ringbuff[ind]
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        self.put(value);
        self.tail = (self.tail + 1) % self.capacity;
    }

    pub fn pop(&mut self) -> T {
        let value = self.peek();
        self.head = (self.head + 1) % self.capacity;
        return value;
    }

    pub fn get_read_index(&self) -> usize {
        self.head
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.head = index % self.capacity;
    }

    pub fn get_write_index(&self) -> usize {
        self.tail
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.tail = index % self.capacity;
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.capacity - (self.head - self.tail)
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_func() {
        let mut ring_buffer = RingBuffer::new(5);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);
        assert_eq!(ring_buffer.len(), 3);
    }

    #[test]
    fn test_reset_func() {
        let mut ring_buffer = RingBuffer::new(5);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);        
        ring_buffer.reset();
        assert_eq!(ring_buffer.len(), 0);
    }

    #[test]
    fn test_peek_func() {
        let mut ring_buffer = RingBuffer::new(5);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);        
        assert_eq!(ring_buffer.peek(), 3);
    }

    #[test]
    fn test_put_func() {
        let mut ring_buffer = RingBuffer::new(5);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);
        ring_buffer.push(4);
        ring_buffer.push(5);   
        ring_buffer.put(6);
        assert_eq!(ring_buffer.peek(), 6);

        ring_buffer.push(7);
        ring_buffer.push(8);
        ring_buffer.put(9);
        assert_eq!(ring_buffer.peek(), 7);
        assert_eq!(ring_buffer.get(2), 9);
    }
}
