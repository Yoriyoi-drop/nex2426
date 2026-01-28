/// Simple Fletcher-64 Checksum for fast error detection.
/// Not cryptographic, but extremely fast for 1TB data scanning.

pub struct Fletcher64 {
    sum1: u32,
    sum2: u32,
}

impl Fletcher64 {
    pub fn new() -> Self {
        Self { sum1: 0, sum2: 0 }
    }

    pub fn update(&mut self, data: &[u32]) {
        for &word in data {
            self.sum1 = self.sum1.wrapping_add(word);
            self.sum2 = self.sum2.wrapping_add(self.sum1);
        }
    }

    pub fn finish(&self) -> u64 {
        ((self.sum2 as u64) << 32) | (self.sum1 as u64)
    }
}
