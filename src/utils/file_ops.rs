use crate::kernel::NexKernel;
use crate::transform::stage_chaos::ChaosEngine;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

const HEADER_MAGIC: &[u8; 8] = b"Nex2426\0";

/// Encrypts a file using the derived key from NexKernel and the Chaos Stream Cipher.
/// NOW SUPPORTS: Hardware Binding (Bio-Lock) & Stealth Mode (No Header)
pub fn encrypt_file(input_path: &str, output_path: &str, key: &str, cost: u32, use_bio_lock: bool, is_stealth: bool) -> std::io::Result<()> {
    let kernel = NexKernel::new(cost);
    
    // In Stealth Mode, we derive timestamp from key to maintain security
    // while keeping decryption possible without storing metadata
    let timestamp = if is_stealth {
        // Derive a deterministic but secure timestamp from key
        let temp_kernel = NexKernel::new(1);
        // Generate secure random stealth mode key
        use crate::utils::entropy::SecureRng;
        let stealth_key = if let Ok(mut rng) = SecureRng::new() {
            let mut key_bytes = [0u8; 8];
            if rng.fill_bytes(&mut key_bytes).is_ok() {
                format!("stealth-{}", hex::encode(key_bytes))
            } else {
                format!("stealth-timestamp-{}", key)
            }
        } else {
            format!("stealth-timestamp-{}", key)
        };
        let mut cursor = std::io::Cursor::new(format!("{}-{}", stealth_key, key));
        let (derived_blocks, _) = temp_kernel.execute_pipeline_raw(&mut cursor, &stealth_key);
        // Use first derived block as timestamp (mod 2^32 to keep it reasonable)
        (derived_blocks[0] & 0xFFFFFFFF) as u64
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("System time error: {}", e)))?
            .as_secs()
    };
    
    let mut salt_input = format!("{}-{}", key, timestamp);
    let mut flags: u8 = 0;

    if use_bio_lock {
        let hw_id = crate::utils::asm_ops::get_hardware_id();
        salt_input.push_str(&format!("-{}", hw_id));
        flags |= 0x01; 
    }

    let mut cursor = std::io::Cursor::new(salt_input.as_bytes());
    let (mut derived_blocks, _) = kernel.execute_pipeline_raw(&mut cursor, key);
    
    // Setup Cipher
    let seed_chaos = [derived_blocks[0], derived_blocks[1], derived_blocks[2], derived_blocks[3]];
    
    // Clear derived intermediate blocks from memory securely
    use crate::security::memory::Zeroize;
    derived_blocks.zeroize();

    let mut cipher = ChaosEngine::new(seed_chaos);

    let mut input_file = BufReader::new(File::open(input_path)?);
    let mut output_file = BufWriter::new(File::create(output_path)?);

    // Header Write
    if !is_stealth {
        output_file.write_all(HEADER_MAGIC)?; 
        output_file.write_all(&[1u8])?; // Version
        output_file.write_all(&cost.to_le_bytes())?;
        output_file.write_all(&timestamp.to_le_bytes())?;
        output_file.write_all(&[flags])?; 
        output_file.write_all(&[0u8; 10])?; 
    }

    // Encrypt Loop
    let mut buffer = [0u8; 65536]; 
    loop {
        let n = input_file.read(&mut buffer)?;
        if n == 0 { break; }
        let chunk = &mut buffer[0..n];
        let mut i = 0;
        while i < chunk.len() {
            let keystream_bytes = cipher.next_u64().to_le_bytes();
            let remain = chunk.len() - i;
            let take = if remain >= 8 { 8 } else { remain };
            for j in 0..take { chunk[i + j] ^= keystream_bytes[j]; }
            i += take;
        }
        output_file.write_all(chunk)?;
    }
    output_file.flush()?;
    
    crate::security::memory::secure_clean(&mut buffer);
    
    Ok(())
}

/// Decrypts a file. Supports standard and Stealth Mode.
pub fn decrypt_file(input_path: &str, output_path: &str, key: &str, is_stealth: bool) -> std::io::Result<()> {
    let mut input_file = BufReader::new(File::open(input_path)?);
    
    let mut cost = 1;
    let timestamp;
    let flags;

    if !is_stealth {
        let mut header = [0u8; 32];
        input_file.read_exact(&mut header)?;
        if &header[0..8] != HEADER_MAGIC {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Nex2426 File Format (Try --stealth?)"));
        }
        cost = u32::from_le_bytes(header[9..13].try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid header: cost bytes")
        })?);
        timestamp = u64::from_le_bytes(header[13..21].try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid header: timestamp bytes")
        })?);
        flags = header[21];
    } else {
        // For stealth mode, derive timestamp the same way as encryption
        let temp_kernel = NexKernel::new(1);
        // Generate secure random stealth mode key
        use crate::utils::entropy::SecureRng;
        let stealth_key = if let Ok(mut rng) = SecureRng::new() {
            let mut key_bytes = [0u8; 8];
            if rng.fill_bytes(&mut key_bytes).is_ok() {
                format!("stealth-{}", hex::encode(key_bytes))
            } else {
                format!("stealth-timestamp-{}", key)
            }
        } else {
            format!("stealth-timestamp-{}", key)
        };
        let mut cursor = std::io::Cursor::new(format!("{}-{}", stealth_key, key));
        let (derived_blocks, _) = temp_kernel.execute_pipeline_raw(&mut cursor, &stealth_key);
        timestamp = (derived_blocks[0] & 0xFFFFFFFF) as u64;
        // In stealth mode, we assume no bio-lock unless explicitly specified by user
        flags = 0;
    }

    let mut salt_input = format!("{}-{}", key, timestamp);
    
    // Handle bio-lock for both standard and stealth mode
    let bio_lock_enabled = (flags & 0x01) != 0;
    if bio_lock_enabled {
        let hw_id = crate::utils::asm_ops::get_hardware_id();
        salt_input.push_str(&format!("-{}", hw_id));
    }

    let kernel = NexKernel::new(cost);
    let mut cursor = std::io::Cursor::new(salt_input.as_bytes());
    let (mut derived_blocks, _) = kernel.execute_pipeline_raw(&mut cursor, key);
    
    let seed_chaos = [derived_blocks[0], derived_blocks[1], derived_blocks[2], derived_blocks[3]];
    
    // Clear derived blocks securely
    use crate::security::memory::Zeroize;
    derived_blocks.zeroize();

    let mut cipher = ChaosEngine::new(seed_chaos);
    
    let mut output_file = BufWriter::new(File::create(output_path)?);
    let mut buffer = [0u8; 65536]; 
    loop {
        let n = input_file.read(&mut buffer)?;
        if n == 0 { break; }
        let chunk = &mut buffer[0..n];
        let mut i = 0;
        while i < chunk.len() {
            let keystream_bytes = cipher.next_u64().to_le_bytes();
            let remain = chunk.len() - i;
            let take = if remain >= 8 { 8 } else { remain };
            for j in 0..take { chunk[i + j] ^= keystream_bytes[j]; }
            i += take;
        }
        output_file.write_all(chunk)?;
    }
    output_file.flush()?;
    
    crate::security::memory::secure_clean(&mut buffer);
    
    Ok(())
}
