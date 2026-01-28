# NEX2426 - Quantum-Resistant Chaos Encryption Engine

![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build](https://img.shields.io/badge/build-passing-green.svg)

## Overview

NEX2426 adalah mesin enkripsi berbasis chaos yang dirancang untuk tahan terhadap serangan komputer kuantum. Sistem ini mengimplementasikan pipeline enkripsi 6 tahap dengan kombinasi kriptografi post-quantum, white-box obfuscation, dan temporal binding.

## Fitur Utama

### 🔐 Keamanan Tingkat Tinggi
- **Quantum-Resistant**: Implementasi lattice-based cryptography
- **White-Box Obfuscation**: Melindungi logika internal dari reverse engineering
- **Memory-Hardened**: 8MB per lane untuk mencegah serangan hardware
- **Temporal Binding**: Timestamp integration untuk proof-of-existence

### ⚡ Performa Optimal
- **AVX2 Vectorization**: Optimasi SIMD untuk prosesor modern
- **Parallel Processing**: Auto-scaling ke semua core yang tersedia
- **Assembly Integration**: Operasi kritis menggunakan assembly

### 🛠️ Mode Operasi
- **File Encryption**: Enkripsi/dekripsi file dengan opsi bio-lock & stealth
- **Interactive Shell**: Mode interaktif untuk hashing real-time
- **Benchmark Mode**: Pengujian performa sistem
- **Blockchain Demo**: Implementasi quantum ledger

## Quick Start

### Installation

```bash
# Clone repository
git clone <repository-url>
cd nex2426

# Build project
cargo build --release

# Run binary
./target/release/nex2426
```

### Basic Usage

```bash
# Hash string sederhana
./nex2426 "Hello World"

# File encryption
./nex2426 --encrypt secret.txt "MySecretKey" 3 --bio-lock --stealth

# File decryption
./nex2426 --decrypt secret.txt.nex2426 "MySecretKey" --stealth

# Interactive mode
./nex2426

# Benchmark performance
./nex2426 --bench 3
```

## Architecture

Pipeline enkripsi NEX2426 terdiri dari 6 tahap utama:

1. **Stage 1 - Expansion**: Stream input expansion dengan key mixing
2. **Stage 2 - Binary**: Konversi ke binary blocks
3. **Stage 3 - Reduction**: Block reduction dengan memory hardening
4. **Stage 4 - Finalization**: Cross-mixing dan diffusion
5. **Stage 5 - Temporal Binding**: Timestamp integration (opsional)
6. **Stage 6 - Integrity Seal**: Final hash formatting

## Security Features

### Post-Quantum Resistance
- Lattice-based cryptography dengan 100-dimensional state space
- Chaos-based stream cipher untuk non-linear transformation
- Quantum-safe key derivation

### Advanced Obfuscation
- White-box network execution (1024 rounds)
- Dynamic polymorphic VM generation
- Assembly-level scrambling operations

### Integrity & Authenticity
- SHA-256 inspired integrity seals
- Temporal binding untuk non-repudiation
- Blockchain-compatible signature scheme

## Performance

### Benchmark Results
- **Cost 1**: ~50,000 hashes/second
- **Cost 3**: ~15,000 hashes/second  
- **Cost 5**: ~5,000 hashes/second

### Memory Usage
- **Base**: 8MB per hashing lane
- **Scaling**: Auto-adjusts based on available cores
- **Peak**: 8MB × CPU cores

## API Reference

### Core Components

```rust
// Main kernel
let kernel = NexKernel::new(cost);
kernel.enable_temporal_binding();
let result = kernel.execute(&mut input, "secret_key");

// File operations
utils::file_ops::encrypt_file(input, output, key, cost, bio_lock, stealth)?;
utils::file_ops::decrypt_file(input, output, key, stealth)?;

// Blockchain
let mut ledger = NexLedger::new();
let block = ledger.create_block(transactions, prev_hash, index, &private_key);
```

## Security Considerations

### ⚠️ Important Notes
- **Experimental**: Sistem ini masih dalam tahap eksperimental
- **Audit Required**: Memerlukan security audit oleh kriptographer ahli
- **Non-Standard**: Menggunakan format hash custom, bukan standar industri

### 🛡️ Threat Model
- **Quantum Attacks**: Dilindungi melalui lattice-based cryptography
- **Side-Channel**: Memory-hardening dan constant-time operations
- **Reverse Engineering**: White-box obfuscation dan assembly scrambling

## Development Status

### ✅ Completed
- Core encryption pipeline
- File encryption/decryption
- Interactive shell
- Basic blockchain implementation
- Performance benchmarking

### 🚧 In Progress
- Comprehensive test suite
- Security documentation
- Standardization efforts

### 📋 Planned
- Hardware acceleration (GPU/FPGA)
- Network protocol implementation
- Mobile platform support

## Contributing

1. Fork repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## License

Project ini dilisensikan under MIT License - lihat file [LICENSE](../LICENSE) untuk detail.

## Disclaimer

NEX2426 adalah project eksperimental untuk tujuan penelitian. Tidak direkomendasikan untuk production use tanpa security audit komprehensif.

---

**Version**: 6.0-Compressed  
**Last Updated**: 2025-01-28  
**Language**: Rust 2024 Edition
