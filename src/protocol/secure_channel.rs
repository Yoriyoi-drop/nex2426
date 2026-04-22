use crate::protocol::kx::{NexKeyExchange, NetworkPacket};
use crate::standards::hmac::HmacNex;
use crate::transform::stage_chaos::ChaosEngine;
use crate::security::memory::Protected;
use crate::kernel::NexKernel;
use crate::error::NexResult;

/// Secure Channel Protocol
/// Integrates Post-Quantum Key Exchange with Chaos Stream Encyption 
/// and HMAC-SHA3-like Integrity.

pub struct SecureSession {
    session_id: u64,
    shared_secret: Protected<[u8; 32]>,
    tx_counter: u64,
    rx_counter: u64,
    rx_engine: ChaosEngine,
    tx_engine: ChaosEngine,
}

#[derive(Debug)]
pub enum ChannelError {
    HandshakeFailed,
    IntegrityViolation,
    ReplayDetected,
    DecryptionError
}

impl SecureSession {
    /// Perform a Client-Side Handshake (Bob role in KEM)
    /// Input: My Key Exchange Engine, Server's Public Packet
    /// Output: (Established Session, Ciphertext to send back)
    pub fn handshake(client_kx: &mut NexKeyExchange, server_pub: &NetworkPacket) -> NexResult<(Self, NetworkPacket)> {
        // Encap: Generate Shared Secret and Ciphertext
        let (ciphertext, secret) = client_kx.encapsulate(server_pub)?;
        
        let session = Self::from_shared_secret(secret, 0x12345678); // Random Session ID in valid impl
        Ok((session, ciphertext))
    }

    /// Establishes a session from the perspective of the Initiator (Alice)
    /// This assumes the exchange has already happened and we have the shared secret.
    pub fn from_shared_secret(secret: [u8; 32], session_id: u64) -> Self {
        // Expand the 32-byte secret into TWO 256-bit seeds (one for TX, one for RX)
        // using the NexKernel KDF
        let kernel = NexKernel::new(1);
        
        // Seed derivation for TX
        let mut seed_ctx_tx = Vec::from(secret);
        seed_ctx_tx.extend_from_slice(b"TX-STREAM");
        let tx_hash = kernel.hash_bytes(&seed_ctx_tx, "Session-KDF");
        
        // Seed derivation for RX
        // In a real protocol, Alice's TX is Bob's RX. So Bob would swap these contexts.
        // For simplicity, we assume this is a symmetric setup where A -> B uses one key
        // and B -> A uses another.
        let mut seed_ctx_rx = Vec::from(secret);
        seed_ctx_rx.extend_from_slice(b"RX-STREAM");
        let rx_hash = kernel.hash_bytes(&seed_ctx_rx, "Session-KDF");
        
        // Convert hashes to [u64; 4] seeds
        let tx_seed = Self::bytes_to_seed(&tx_hash);
        let rx_seed = Self::bytes_to_seed(&rx_hash);

        Self {
            session_id,
            shared_secret: Protected::new(secret),
            tx_counter: 0,
            rx_counter: 0,
            tx_engine: ChaosEngine::new(tx_seed),
            rx_engine: ChaosEngine::new(rx_seed),
        }
    }
    
    fn bytes_to_seed(bytes: &[u8]) -> [u64; 4] {
        let mut seed = [0u64; 4];
        for (i, seed_val) in seed.iter_mut().enumerate() {
            let start = i * 8;
            let end = start + 8;
            if end <= bytes.len() {
                *seed_val = u64::from_le_bytes(bytes[start..end].try_into().unwrap_or([0u8; 8]));
            }
        }
        seed
    }

    /// Encrypts a message payload.
    /// Format: [Header: SessionID(8) | Counter(8)] [Payload: Encrypted] [Mac: HMAC(32)]
    pub fn send_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(16 + payload.len() + 32);
        
        // 1. Header
        packet.extend_from_slice(&self.session_id.to_le_bytes());
        packet.extend_from_slice(&self.tx_counter.to_le_bytes());
        
        // 2. Encrypt Payload
        // We use the Chaos Engine stateful stream
        let mut encrypted_payload = payload.to_vec();
        for byte in encrypted_payload.iter_mut() {
            let key_byte = (self.tx_engine.next_u64() & 0xFF) as u8;
            *byte ^= key_byte;
        }
        packet.extend_from_slice(&encrypted_payload);
        
        // 3. Authenticate (Encrypt-then-MAC)
        // HMAC Covers Header + Encrypted Payload
        let mac_key = self.shared_secret.access();
        let hmac = HmacNex::new(mac_key)?;
        let tag = hmac.sign(&packet); // Packet so far is Header + Ciphertext
        
        packet.extend_from_slice(&tag);
        
        self.tx_counter += 1;
        
        packet
    }
    
    /// Decrypts a received packet.
    pub fn receive_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>, ChannelError> {
        if packet.len() < 48 { // 16 header + 32 MAC + 0 payload
            return Err(ChannelError::IntegrityViolation);
        }
        
        let split_point = packet.len() - 32; // Last 32 bytes are MAC
        let (data_part, mac_part) = packet.split_at(split_point);
        
        // 1. Verify HMAC
        let mac_key = self.shared_secret.access();
        let hmac = HmacNex::new(mac_key)?;
        if !hmac.verify(data_part, mac_part) {
            return Err(ChannelError::IntegrityViolation);
        }
        
        // 2. Parse Header
        let session_id_bytes: [u8; 8] = if data_part.len() >= 8 {
            data_part[0..8].try_into().unwrap_or([0u8; 8])
        } else {
            [0u8; 8]
        };
        let counter_bytes: [u8; 8] = if data_part.len() >= 16 {
            data_part[8..16].try_into().unwrap_or([0u8; 8])
        } else {
            [0u8; 8]
        };
        
        let rcv_session_id = u64::from_le_bytes(session_id_bytes);
        let rcv_counter = u64::from_le_bytes(counter_bytes);
        
        if rcv_session_id != self.session_id {
            // Wrong session
             return Err(ChannelError::DecryptionError);
        }
        
        // Simple Replay Protection
        if rcv_counter <= self.rx_counter && self.rx_counter != 0 {
             return Err(ChannelError::ReplayDetected);
        }
        self.rx_counter = rcv_counter;
        
        // 3. Decrypt Payload
        let ciphertext = &data_part[16..];
        let mut plaintext = ciphertext.to_vec();
        
        // Sync the stream?
        // Issue: If packets are lost (UDP), Stream Cipher loses sync.
        // For TCP/Reliable, this stateful engine is fine.
        // For Datagrams, we would need to Seek/Rekey based on Counter.
        // Assuming RELIABLE transport here (stateful stream).
        
        for byte in plaintext.iter_mut() {
            let key_byte = (self.rx_engine.next_u64() & 0xFF) as u8;
            *byte ^= key_byte;
        }
        
        Ok(plaintext)
    }
}
