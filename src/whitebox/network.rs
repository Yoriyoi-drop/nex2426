use super::tables::t1::T1_BOX;
use super::tables::t2::T2_BOX;
use super::tables::t3::T3_BOX;
use super::tables::t4::T4_BOX;

/// The White-Box Network executes a series of table lookups to obscure the key.
/// In a full implementation, this would involve thousands of tables.
/// Here we demonstrate the core mechanism with 4 distinct tables cycled over many rounds.
pub struct NetworkEngine {
    pub state: [u32; 16],
}

impl NetworkEngine {
    pub fn new(seed: [u32; 16]) -> Self {
        Self { state: seed }
    }

    /// Executes 1024 rounds of table lookups.
    /// This is where the code volume comes from in a fully moved whitebox (unrolled).
    pub fn execute(&mut self) {
        for _ in 0..1024 {
            self.round();
        }
    }

    #[inline(always)]
    fn round(&mut self) {
        // Unrolling the state update roughly for demonstration
        for i in 0..4 {
            let idx0 = (self.state[i*4 + 0] & 0xFF) as usize;
            let idx1 = (self.state[i*4 + 1] & 0xFF) as usize;
            let idx2 = (self.state[i*4 + 2] & 0xFF) as usize;
            let idx3 = (self.state[i*4 + 3] & 0xFF) as usize;

            self.state[i*4 + 0] ^= T1_BOX[idx0];
            self.state[i*4 + 1] ^= T2_BOX[idx1];
            self.state[i*4 + 2] ^= T3_BOX[idx2];
            self.state[i*4 + 3] ^= T4_BOX[idx3];
        }

        // Shuffle
        let temp = self.state[0];
        for i in 0..15 {
            self.state[i] = self.state[i+1];
        }
        self.state[15] = temp;
    }
}
