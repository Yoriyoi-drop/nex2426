use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};


/// Stream Buffer Size (High Performance for Large Files)
/// 16 MB Chunk Size reduces syscall overhead for 1 TB files.
const CHUNK_SIZE: usize = 16 * 1024 * 1024;

pub struct LargeFileProcessor;

impl LargeFileProcessor {
    /// Copies data from Input to Output in large chunks.
    /// Closure `process_fn` is applied to each chunk in-place.
    pub fn process_stream<F>(
        input_path: &str,
        output_path: &str,
        mut process_fn: F
    ) -> io::Result<u64>
    where F: FnMut(u64, &mut [u8]) 
    {
        let input_file = File::open(input_path)?;
        let output_file = File::create(output_path)?;
        
        // 1MB standard buffer for the Reader/Writer wrapping
        let mut reader = BufReader::with_capacity(1024 * 1024, input_file);
        let mut writer = BufWriter::with_capacity(1024 * 1024, output_file);
        
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut total_processed: u64 = 0;

        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            
            // Apply transformation to the valid part of the buffer
            process_fn(total_processed, &mut buffer[0..n]);
            
            writer.write_all(&buffer[0..n])?;
            total_processed += n as u64;
            
            // Optional: Print progress every ~1GB
            if total_processed % (1024 * 1024 * 1024) == 0 {
                 println!(" [IO] Processed: {} GB", total_processed / (1024 * 1024 * 1024));
            }
        }
        
        writer.flush()?;
        
        // Critical: Zeroize the huge 16MB buffer before releasing memory
        crate::security::memory::secure_clean(&mut buffer);
        
        Ok(total_processed)
    }
}
