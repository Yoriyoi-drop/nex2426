/// Run-Length Encoding (RLE) Implementation.
/// Efficient for sparse files (Zero-byte padding) common in disk images.

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    if input.is_empty() { return output; }

    let mut count = 1;
    let mut prev = input[0];

    for &byte in &input[1..] {
        if byte == prev && count < 255 {
            count += 1;
        } else {
            // Flush
            output.push(count);
            output.push(prev);
            
            // Reset
            prev = byte;
            count = 1;
        }
    }
    // Flush last
    output.push(count);
    output.push(prev);

    output
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let count = input[i];
        let byte = input[i+1];
        for _ in 0..count {
            output.push(byte);
        }
        i += 2;
    }
    output
}
