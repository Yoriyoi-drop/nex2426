# NEX2426 API Documentation

## Table of Contents

- [Core API](#core-api)
- [File Operations](#file-operations)
- [Blockchain Module](#blockchain-module)
- [Quantum Module](#quantum-module)
- [White-Box Module](#white-box-module)
- [Utilities](#utilities)
- [Command Line Interface](#command-line-interface)

---

## Core API

### NexKernel

Main encryption engine that orchestrates the entire pipeline.

```rust
pub struct NexKernel {
    pub cost: u32,
    pub version: &'static str,
    pub deterministic: bool,
}
```

#### Methods

##### `new(cost: u32) -> Self`
Creates new kernel instance with specified cost parameter.

```rust
let kernel = NexKernel::new(3);
```

**Parameters:**
- `cost`: Memory hardness factor (1-10 recommended)

**Returns:** New NexKernel instance

##### `enable_temporal_binding(&mut self)`
Enables temporal binding for non-deterministic hashing.

```rust
kernel.enable_temporal_binding();
```

**Effect:** Makes hashes time-dependent for proof-of-existence

##### `execute(&self, input: &mut dyn Read, key: &str) -> KernelResult`
Executes full encryption pipeline.

```rust
let mut cursor = std::io::Cursor::new(data);
let result = kernel.execute(&mut cursor, "secret_key");
```

**Parameters:**
- `input`: Any type implementing Read trait
- `key`: Encryption key string

**Returns:** `KernelResult` containing formatted hash

##### `execute_pipeline_raw(&self, input: &mut dyn Read, key: &str) -> (Vec<u64>, u64)`
Executes pipeline and returns raw 512-bit hash.

```rust
let (blocks, timestamp) = kernel.execute_pipeline_raw(&mut input, key);
```

**Returns:** Tuple of (8 u64 blocks, timestamp)

##### `hash_bytes(&self, data: &[u8], key: &str) -> Vec<u8>`
Convenience method for hashing byte arrays.

```rust
let digest = kernel.hash_bytes(b"hello", "key");
```

**Returns:** 64-byte hash digest

##### `benchmark(&self) -> u32`
Runs performance benchmark for 1 second.

```rust
let hashes_per_second = kernel.benchmark();
```

**Returns:** Number of hashes computed per second

### KernelResult

```rust
pub struct KernelResult {
    pub full_formatted_string: String,  // $nex6$v=... format
    pub hash_hex: String,              // Hexadecimal representation
    pub hash_base58: String,           // Base58 encoded
    pub timestamp: u64,                // Unix timestamp
}
```

---

## File Operations

### Encryption Functions

```rust
pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    key: &str,
    cost: u32,
    use_bio_lock: bool,
    is_stealth: bool
) -> Result<(), Box<dyn std::error::Error>>
```

Encrypts a file with specified security options.

**Parameters:**
- `input_path`: Source file path
- `output_path`: Destination file path
- `key`: Encryption password
- `cost`: Memory hardness factor
- `use_bio_lock`: Enable biometric locking
- `is_stealth`: Remove file headers

```rust
utils::file_ops::encrypt_file(
    "secret.txt",
    "secret.txt.nex2426",
    "password123",
    3,
    true,
    false
)?;
```

### Decryption Functions

```rust
pub fn decrypt_file(
    input_path: &str,
    output_path: &str,
    key: &str,
    is_stealth: bool
) -> Result<(), Box<dyn std::error::Error>>
```

Decrypts a NEX2426 encrypted file.

**Parameters:**
- `input_path`: Encrypted file path
- `output_path`: Decrypted output path
- `key`: Decryption password
- `is_stealth`: File was encrypted in stealth mode

---

## Blockchain Module

### NexLedger

Quantum-resistant blockchain implementation.

```rust
pub struct NexLedger {
    pub chain: Vec<NexBlock>,
    pub difficulty: u32,
}
```

#### Methods

##### `new() -> Self`
Creates new ledger with genesis block.

```rust
let ledger = NexLedger::new();
```

##### `create_block(&mut self, transactions: Vec<&[u8]>, prev_hash: String, index: u32, private_key: &[u8]) -> NexBlock`
Creates new block with given transactions.

```rust
let block = ledger.create_block(
    vec![b"transaction data"],
    prev_hash,
    1,
    &private_key
);
```

##### `add_block(&mut self, block: NexBlock, public_key: &[u8]) -> bool`
Adds block to chain after verification.

```rust
let success = ledger.add_block(block, &public_key);
```

### NexBlock

```rust
pub struct NexBlock {
    pub index: u32,
    pub timestamp: u64,
    pub transactions_root: String,
    pub prev_hash: String,
    pub hash: String,
    pub signature: NexSignature,
}
```

---

## Quantum Module

### LatticeEngine

Post-quantum lattice-based cryptography.

```rust
pub struct LatticeEngine {
    pub state: [u32; 100],  // 100-dimensional lattice
}
```

#### Methods

##### `new() -> Self`
Creates new lattice engine with random state.

```rust
let lattice = LatticeEngine::new();
```

##### `inject(&mut self, data: &[u32])`
Injects data into lattice state.

```rust
lattice.inject(&whitebox_state);
```

##### `diffuse(&mut self, seed: [u64; 4])`
Applies quantum diffusion transformation.

```rust
lattice.diffuse([1, 2, 3, 4]);
```

---

## White-Box Module

### NetworkEngine

White-box obfuscation network.

```rust
pub struct NetworkEngine {
    pub state: [u32; 16],
    pub tables: Vec<Vec<u32>>,
}
```

#### Methods

##### `new(initial_state: [u32; 16]) -> Self`
Creates new network with given initial state.

```rust
let network = NetworkEngine::new([0; 16]);
```

##### `execute(&mut self)`
Runs 1024 rounds of white-box transformation.

```rust
network.execute();
```

---

## Utilities

### Encoding Functions

#### Base58 Encoding

```rust
pub fn encode_blocks(blocks: &[u64]) -> String
pub fn decode_blocks(encoded: &str) -> Result<Vec<u64>, EncodingError>
```

#### Hex Encoding

```rust
pub fn blocks_to_hex(blocks: &[u64]) -> String
pub fn hex_to_blocks(hex: &str) -> Result<Vec<u64>, HexError>
```

### Assembly Operations

```rust
pub fn asm_scramble(value: u64) -> u64
pub fn asm_mix(a: u64, b: u64) -> u64
```

Low-level assembly optimizations for critical operations.

---

## Command Line Interface

### Basic Usage

```bash
# Hash string
nex2426 "input string" [key] [cost]

# File operations
nex2426 --encrypt <file> <key> [cost] [--bio-lock] [--stealth]
nex2426 --decrypt <file.nex2426> <key> [--stealth]

# Interactive mode
nex2426

# Benchmark
nex2426 --bench [cost]

# Blockchain demo
nex2426 --blockchain

# Generate signature
nex2426 --sign [message]
```

### Exit Codes

- `0`: Success
- `1`: General error
- `2`: File not found
- `3`: Invalid key
- `4`: Corrupted encrypted file

### Environment Variables

- `NEX2426_COST`: Default cost parameter
- `NEX2426_THREADS`: Number of threads to use
- `NEX2426_VERBOSE`: Enable verbose output

---

## Error Handling

### Error Types

```rust
pub enum NexError {
    InvalidInput(String),
    KeyDerivationError,
    MemoryError,
    IoError(std::io::Error),
    EncryptionError,
    DecryptionError,
}
```

### Result Type

```rust
pub type NexResult<T> = Result<T, NexError>;
```

---

## Performance Considerations

### Memory Usage

- **Base**: 8MB per hashing lane
- **Scaling**: `8MB × cost_factor × num_threads`
- **Peak**: Depends on parallel execution

### CPU Usage

- **Single Thread**: ~50,000 hashes/sec (cost=1)
- **Multi Thread**: Scales linearly with CPU cores
- **AVX2 Required**: For optimal performance

### Recommendations

1. Use cost factor 1-3 for interactive applications
2. Use cost factor 5-10 for file encryption
3. Enable AVX2 for best performance
4. Limit concurrent operations based on available RAM

---

## Security Guidelines

### Key Management

- Use strong, random keys (>16 characters)
- Different keys for different purposes
- Rotate keys regularly
- Never hardcode keys in source code

### Best Practices

1. Always verify file integrity after decryption
2. Use bio-lock for sensitive files
3. Enable stealth mode when header metadata is sensitive
4. Combine with other security measures (encryption at rest, access control)

### Common Pitfalls

- Don't reuse keys across different contexts
- Avoid low cost factors for sensitive data
- Don't assume temporal binding for key derivation
- Always handle errors properly

---

## Examples

### Basic Hashing

```rust
use nex2426::kernel::{NexKernel, KernelResult};

fn hash_data(data: &[u8], key: &str) -> KernelResult {
    let kernel = NexKernel::new(3);
    kernel.enable_temporal_binding();
    
    let mut cursor = std::io::Cursor::new(data);
    kernel.execute(&mut cursor, key)
}
```

### File Encryption

```rust
use nex2426::utils::file_ops;

fn encrypt_sensitive_file(path: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = format!("{}.nex2426", path);
    
    file_ops::encrypt_file(
        path,
        &output,
        password,
        5,           // High cost for security
        true,        // Enable bio-lock
        false        // No stealth mode
    )?;
    
    println!("File encrypted: {}", output);
    Ok(())
}
```

### Blockchain Integration

```rust
use nex2426::integrity::ledger::{NexLedger, NexBlock};
use nex2426::standards::signatures::NexSigner;

fn create_quantum_transaction() -> Result<NexBlock, Box<dyn std::error::Error>> {
    let mut ledger = NexLedger::new();
    let signer = NexSigner::new();
    let (sk, pk) = signer.generate_keypair();
    
    let tx_data = b"Transfer 100 NEX from Alice to Bob";
    let prev_hash = ledger.chain.last().unwrap().hash.clone();
    
    let block = ledger.create_block(
        vec![tx_data],
        prev_hash,
        ledger.chain.len() as u32,
        &sk
    );
    
    Ok(block)
}
```

---

*This API documentation covers the core functionality of NEX2426. For implementation details, refer to the source code and inline documentation.*
