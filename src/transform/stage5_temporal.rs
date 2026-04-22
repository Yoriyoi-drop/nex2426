use std::time::{SystemTime, UNIX_EPOCH};
use crate::utils::asm_ops;

pub struct TemporalBinding {
    pub timestamp: u64,
}

impl TemporalBinding {
    pub fn new() -> Self {
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| {
                // Fallback to 0 if time goes backwards
                std::time::Duration::from_secs(0)
            })
            .as_secs();
        Self { timestamp }
    }

    /// Mixes the timestamp into the hash blocks to bind them to this specific time.
    /// This makes the hash unique to WHEN it was created, preventing Replay Attacks.
    pub fn bind(&self, mut blocks: Vec<u64>) -> Vec<u64> {
        let ts_mix = asm_ops::asm_scramble(self.timestamp);
        
        for (i, block) in blocks.iter_mut().enumerate() {
            // Mix timestamp into every block
            *block = asm_ops::asm_scramble(*block ^ ts_mix ^ (i as u64));
        }
        
        blocks
    }
}
