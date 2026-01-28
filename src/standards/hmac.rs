use crate::kernel::NexKernel;

/// Industry Standard HMAC Construction
/// Adapted for Nex2426 Kernel (512-bit block size presumed for inner hashing, though Nex is stream-based).
/// We treat the NexKernel as a Hash Function H(key, msg).
/// 
/// HMAC(K, m) = H( (K' ^ opad) || H( (K' ^ ipad) || m ) )
/// 
/// Since NexKernel takes a "Key" argument separately from "Data", 
/// we can simplify or stick to the strict construction.
/// NexKernel is effectively H_keyed(key, msg).
/// 
/// Standard HMAC is for unkeyed primitives.
/// However, to integrate with "Standard Industry" flows using a keyed hash:
/// We will use the standard construction where the "Key" arg of NexKernel is fixed (e.g. "Salt")
/// and we implement the HMAC logic on the data stream itself.

pub struct HmacNex {
    kernel: NexKernel,
    key: Vec<u8>,
}

impl HmacNex {
    pub fn new(key: &[u8]) -> Self {
        Self {
            kernel: NexKernel::new(1),
            key: key.to_vec(),
        }
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let block_size = 64; // 512 bits
        
        // 1. Process Key
        let mut k_prime = self.key.clone();
        if k_prime.len() > block_size {
            k_prime = self.kernel.hash_bytes(&k_prime, "HMAC-Key-Reduction");
        }
        if k_prime.len() < block_size {
            k_prime.resize(block_size, 0);
        }

        // 2. Prepare Pads
        let mut o_pad = vec![0x5c; block_size];
        let mut i_pad = vec![0x36; block_size];

        for i in 0..block_size {
            o_pad[i] ^= k_prime[i];
            i_pad[i] ^= k_prime[i];
        }

        // 3. Inner Hash: H(i_pad || message)
        let mut inner_data = i_pad;
        inner_data.extend_from_slice(message);
        let inner_hash = self.kernel.hash_bytes(&inner_data, "HMAC-Inner");

        // 4. Outer Hash: H(o_pad || inner_hash)
        let mut outer_data = o_pad;
        outer_data.extend_from_slice(&inner_hash);
        let outer_hash = self.kernel.hash_bytes(&outer_data, "HMAC-Outer");

        outer_hash
    }

    pub fn verify(&self, message: &[u8], tag: &[u8]) -> bool {
        let computed = self.sign(message);
        // Constant time comparison to prevent timing attacks
        if computed.len() != tag.len() { return false; }
        
        let mut diff = 0;
        for i in 0..computed.len() {
            diff |= computed[i] ^ tag[i];
        }
        
        diff == 0
    }
}
