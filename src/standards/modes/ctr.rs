use crate::kernel::NexKernel;
use std::io::Cursor;

/// Industry Standard CTR (Counter) Mode
/// Turns the NexKernel (PRF) into a Stream Cipher.
/// 
/// Formula:
/// Keystream = Kernel(Nonce || Counter)
/// Ciphertext = Plaintext ^ Keystream

pub struct CNTMode {
    kernel: NexKernel,
    nonce: [u8; 32], // 256-bit Nonce
    counter: u64,
    ctr_key: String, // Consistent key for CTR mode
}

impl CNTMode {
    pub fn new(kernel: NexKernel, nonce: [u8; 32], ctr_key: String) -> Self {
        Self {
            kernel,
            nonce,
            counter: 0,
            ctr_key,
        }
    }

    /// Process data (Encrypt/Decrypt are the same in CTR mode)
    pub fn process(&mut self, data: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(data.len());
        let mut keystream_buffer = Vec::new();
        
        let block_size = 64; // 512-bit blocks from Kernel
        
        for (i, byte) in data.iter().enumerate() {
            if i % block_size == 0 {
                // Generate new keystream block
                keystream_buffer = self.generate_next_block();
            }
            
            let key_byte = keystream_buffer[i % block_size];
            output.push(byte ^ key_byte);
        }
        
        output
    }

    fn generate_next_block(&mut self) -> Vec<u8> {
        // Construct input seed: Nonce (32) + Counter (8) + Padding (24)
        let mut input_seed = Vec::with_capacity(64);
        input_seed.extend_from_slice(&self.nonce);
        input_seed.extend_from_slice(&self.counter.to_be_bytes());
        // Pad with zeros to fill 64 bytes
        while input_seed.len() < 64 {
            input_seed.push(0);
        }
        
        self.counter += 1;
        
        // Execute Kernel to get 512-bit hash (Keystream)
        let mut cursor = std::io::Cursor::new(input_seed);
        
        // Use the consistent CTR key (SECURITY FIX: No more random keys per block!)
        let (blocks, _) = self.kernel.execute_pipeline_raw(&mut cursor, &self.ctr_key);
        
        // Convert [u64; 8] blocks to [u8; 64]
        let mut bytes = Vec::with_capacity(64);
        for b in blocks {
            bytes.extend_from_slice(&b.to_be_bytes());
        }
        
        bytes
    }
}
