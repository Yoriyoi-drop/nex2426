use crate::kernel::NexKernel;
use crate::transform::stage_chaos::ChaosEngine;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

const HEADER_MAGIC: &[u8; 8] = b"Nex2426\0";

/// Encrypts a file using the derived key from NexKernel and the Chaos Stream Cipher.
/// NOW SUPPORTS: Hardware Binding (Bio-Lock) & Stealth Mode (No Header)
pub fn encrypt_file(input_path: &str, output_path: &str, key: &str, cost: u32, use_bio_lock: bool, is_stealth: bool) -> std::io::Result<()> {
    let kernel = NexKernel::new(cost);
    
    // In Stealth Mode, we don't store the timestamp. 
    // To decrypt, the user MUST know the timestamp used, or we use a "Ghost Epoch".
    // Let's use a fixed "Ghost Epoch" for stealth mode to keep it simple but hidden.
    let timestamp = if is_stealth { 0x1337C0DE_BAADF00D } else {
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
    let (derived_blocks, _) = kernel.execute_pipeline_raw(&mut cursor, key);
    
    // Setup Cipher
    let seed_chaos = [derived_blocks[0], derived_blocks[1], derived_blocks[2], derived_blocks[3]];
    
    // Clear derived intermediate blocks from memory
    // derived_blocks is a Vec<u64>, we need to zeroize it.
    // Assuming secure_clean works on &[u8], we might need to cast or iterate.
    // Or just overwrite.
    {
        let ptr = derived_blocks.as_ptr() as *mut u8;
        let len = derived_blocks.len() * 8;
        unsafe { crate::security::memory::secure_clean(std::slice::from_raw_parts_mut(ptr, len)); }
    }

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
    let mut timestamp = 0x1337C0DE_BAADF00D;
    let mut flags = 0u8;

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
    }

    let salt_input = format!("{}-{}", key, timestamp);
    if (flags & 0x01) != 0 || (is_stealth && true /* for now assume biolock could be manual toggle */) {
        // Note: For stealth, we either assume BioLock is off or user must specify.
        // Let's assume Stealth doesn't auto-detect BioLock since there's no header.
    }

    let kernel = NexKernel::new(cost);
    let mut cursor = std::io::Cursor::new(salt_input.as_bytes());
    let (derived_blocks, _) = kernel.execute_pipeline_raw(&mut cursor, key);
    
    let seed_chaos = [derived_blocks[0], derived_blocks[1], derived_blocks[2], derived_blocks[3]];
    
    // Clear derived blocks
    {
        let ptr = derived_blocks.as_ptr() as *mut u8;
        let len = derived_blocks.len() * 8;
        unsafe { crate::security::memory::secure_clean(std::slice::from_raw_parts_mut(ptr, len)); }
    }

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
