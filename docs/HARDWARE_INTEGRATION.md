# NEX2426 Hardware Integration Guide

## Overview

NEX2426 supports hardware acceleration through SystemVerilog implementation and Rust interface. This allows for FPGA/ASIC acceleration of cryptographic operations.

## Architecture

### SystemVerilog Components

The hardware implementation consists of:

1. **Core Modules** (`src/sv/core/`)
   - `nex2426_pkg.svh` - Package with constants and types
   - `chaos_engine.sv` - Lorenz attractor chaos engine
   - `memory_hardening.sv` - Argon2-inspired memory hardening
   - `nex2426_top.sv` - Top-level core module

2. **Crypto Modules** (`src/sv/crypto/`)
   - `hash_core.sv` - Multi-stage hashing pipeline
   - `encryption_engine.sv` - Stream cipher encryption
   - `ctr_mode.sv` - Counter mode encryption
   - `kdf.sv` - Key derivation function
   - `enc_module_*.sv` - Additional encryption modules

3. **Blockchain Modules** (`src/sv/blockchain/`)
   - `merkle_tree.sv` - Merkle tree implementation
   - `consensus.sv` - Proof of work consensus

4. **Utility Modules** (`src/sv/utils/`)
   - `asm_ops.sv` - Assembly-style operations
   - `entropy_gen.sv` - Entropy generation
   - `fifo.sv` - FIFO buffer
   - `clock_divider.sv` - Clock division
   - `shift_reg.sv` - Shift register
   - `debounce.sv` - Signal debouncing

5. **Interface Modules** (`src/sv/interfaces/`)
   - `nex2426_axi.sv` - AXI4-Lite interface
   - `nex2426_wishbone.sv` - Wishbone interface
   - `crypto_if.svh` - Crypto interface definition
   - `blockchain_if.svh` - Blockchain interface definition
   - `memory_if.svh` - Memory interface definition
   - `uart_if.svh` - UART interface definition

6. **Rust Interface** (`src/sv/rust_interface/`)
   - `hardware_bridge.sv` - Bridge between Rust and SystemVerilog
   - `nex2426_rust_top.sv` - Top-level Rust interface module

### Rust Hardware Interface

The Rust hardware interface provides:

1. **Memory-mapped registers** for hardware control
2. **Hardware bridge** for clock domain crossing
3. **Hardware accelerator** for high-level operations
4. **Performance monitoring** and benchmarking

## Usage

### Command Line Options

```bash
# Hardware acceleration
nex2426 --hardware [cost]

# Hardware testing
nex2426 --hw-test

# Hardware benchmarking
nex2426 --hw-bench [cost]
```

### Rust API

```rust
use nex2426::hardware::{HardwareAccelerator, OperationMode};

// Create hardware accelerator
let accelerator = HardwareAccelerator::new(5000); // 5 second timeout

// Hash data with hardware acceleration
let data = b"Hello, hardware!";
let key = [0x01; 32];
let hash = accelerator.hash_data(data, &key, 5)?;

// Encrypt data with hardware
let encrypted = accelerator.encrypt_data(data, &key, 3)?;

// Run hardware tests
let results = accelerator.run_comprehensive_test()?;
```

### Memory Map

| Register | Address | Description |
|----------|---------|-------------|
| CONTROL  | 0x00    | Control register (start bit) |
| STATUS   | 0x01    | Status register (done, error codes) |
| MODE     | 0x02    | Operation mode (hash, encrypt, etc.) |
| KEY0-3   | 0x10-13 | 256-bit key (4x64-bit registers) |
| COST     | 0x20    | Cost parameter |
| CONFIG   | 0x21    | Configuration (bio-lock, stealth) |
| HW_ID    | 0x22    | Hardware ID |
| DATA_IN  | 0x30    | Input data |
| DATA_OUT | 0x31    | Output data |
| HASH0-7  | 0x40-47 | 512-bit hash output (8x64-bit) |
| VERSION  | 0xFF    | Hardware version |

## Implementation Details

### Clock Domain Crossing

The hardware bridge handles clock domain crossing between Rust software clock and hardware clock using synchronizers and proper CDC techniques.

### Operation Modes

- **Hash (0x03)**: Cryptographic hashing
- **Encrypt (0x01)**: Stream cipher encryption
- **Decrypt (0x02)**: Stream cipher decryption
- **Benchmark (0x04)**: Performance benchmarking
- **KeyGen (0x05)**: Key generation
- **Stealth (0x06)**: Stealth mode encryption
- **BioLock (0x07)**: Bio-lock encryption

### Error Handling

The hardware interface provides comprehensive error handling:

- **BridgeError**: Communication errors
- **AcceleratorError**: Hardware accelerator errors
- **Timeout**: Operation timeout
- **InvalidOperation**: Invalid parameters
- **HardwareFault**: Hardware fault detection

### Performance Monitoring

The implementation includes:

- **Cycle counters**: Operation timing
- **Status monitoring**: Real-time status
- **Performance metrics**: Throughput and latency
- **Error tracking**: Error statistics

## FPGA/ASIC Integration

### Target Platforms

- **Xilinx FPGAs**: Artix-7, Kintex-7, Virtex-7, Zynq
- **Intel FPGAs**: Cyclone V, Arria 10, Stratix
- **ASIC**: Custom silicon implementations

### Synthesis Guidelines

1. **Clock frequency**: 100-200 MHz typical
2. **Resource utilization**: ~10k-50k LUTs
3. **Power consumption**: 100-500mW typical
4. **Latency**: 100-1000 cycles per operation

### Interface Options

- **AXI4-Lite**: Standard ARM interface
- **Wishbone**: Open-source bus standard
- **Custom**: Memory-mapped interface

## Testing

### SystemVerilog Testbenches

```bash
# Run chaos engine test
vlog chaos_engine_tb.sv && vsim -c chaos_engine_tb

# Run hash core test
vlog hash_core_tb.sv && vsim -c hash_core_tb

# Run Rust interface test
vlog rust_interface_tb.sv && vsim -c rust_interface_tb
```

### Rust Hardware Tests

```bash
# Run comprehensive hardware tests
nex2426 --hw-test

# Run hardware benchmark
nex2426 --hw-bench 5
```

## Performance

### Expected Performance

- **Hash throughput**: 1-10 GB/s (depending on implementation)
- **Encryption throughput**: 1-5 GB/s
- **Latency**: 100-1000 cycles
- **Power efficiency**: 10-100 MB/s/mW

### Comparison with Software

Typical speedup: 2-10x compared to software implementation, depending on:

- Hardware platform
- Operation type
- Data size
- Cost parameter

## Security Features

### Hardware Security

- **Secure key storage**: On-chip key storage
- **Side-channel resistance**: Constant-time operations
- **Physical security**: Tamper detection
- **Secure boot**: Hardware-based authentication

### Bio-Lock

Hardware binding using:

- **Hardware ID**: Unique device identifier
- **Physical unclonable functions (PUFs)**
- **Secure element integration**

## Troubleshooting

### Common Issues

1. **Bridge initialization failure**
   - Check clock domain crossing
   - Verify reset sequence
   - Check interface signals

2. **Operation timeout**
   - Increase timeout value
   - Check hardware status
   - Verify cost parameter

3. **Performance issues**
   - Check clock frequency
   - Verify pipeline depth
   - Optimize resource usage

### Debug Tools

- **Signal tap**: Real-time signal monitoring
- **Logic analyzer**: Interface debugging
- **Performance counters**: Performance analysis
- **Status registers**: Error tracking

## Future Enhancements

### Planned Features

- **Multi-core support**: Parallel processing
- **Advanced crypto**: Post-quantum algorithms
- **AI acceleration**: Machine learning operations
- **Network interface**: Direct network connectivity

### Roadmap

1. **Phase 1**: Basic hardware acceleration
2. **Phase 2**: Advanced crypto features
3. **Phase 3**: Multi-core and AI support
4. **Phase 4**: Network and cloud integration

## Conclusion

The NEX2426 hardware integration provides significant performance improvements while maintaining security and compatibility. The SystemVerilog implementation offers flexibility for different FPGA/ASIC platforms, while the Rust interface provides seamless software integration.
