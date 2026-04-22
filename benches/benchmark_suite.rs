//! Comprehensive benchmarking suite for NEX2426
//! 
//! Provides performance testing for all major components and operations
//! to measure and validate performance improvements.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

use nex2426::{
    kernel::NexKernel,
    protocol::kx::NexKeyExchange,
    standards::hmac::HmacNex,
    standards::modes::ctr::CNTMode,
    quantum::lattice::LatticeEngine,
    transform::stage_memory::apply_memory_hardening_parallel,
    memory_opt::{StreamingProcessor, ZeroCopyBuffer},
    utils::entropy::{SecureRng, secure_random_bytes},
};

/// Benchmark configuration
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub sample_size: usize,
    pub warmup_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            sample_size: 1024 * 1024, // 1MB
            warmup_iterations: 10,
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub operation: String,
    pub throughput_mbps: f64,
    pub latency_ms: f64,
    pub memory_mb: f64,
    pub cpu_cycles: Option<u64>,
}

impl BenchmarkResults {
    pub fn new(operation: &str, duration: Duration, bytes_processed: usize) -> Self {
        let seconds = duration.as_secs_f64();
        let throughput_mbps = (bytes_processed as f64 / 1024.0 / 1024.0) / seconds;
        let latency_ms = duration.as_millis() as f64;
        
        Self {
            operation: operation.to_string(),
            throughput_mbps,
            latency_ms,
            memory_mb: bytes_processed as f64 / 1024.0 / 1024.0,
            cpu_cycles: None, // Would need hardware counters
        }
    }
}

fn bench_kernel_hashing(c: &mut Criterion) {
    let config = BenchmarkConfig::default();
    
    c.bench_function("kernel_hash_1kb", |b| {
        b.iter(|| {
            let kernel = NexKernel::new(1);
            let data = vec![0u8; 1024];
            let mut cursor = std::io::Cursor::new(&data);
            black_box(kernel.execute(&mut cursor, "test_key"));
        })
    });
    
    c.bench_function("kernel_hash_1mb", |b| {
        b.iter(|| {
            let kernel = NexKernel::new(1);
            let data = vec![0u8; 1024 * 1024];
            let mut cursor = std::io::Cursor::new(&data);
            black_box(kernel.execute(&mut cursor, "test_key"));
        })
    });
    
    c.bench_function("kernel_hash_10mb", |b| {
        b.iter(|| {
            let kernel = NexKernel::new(1);
            let data = vec![0u8; 10 * 1024 * 1024];
            let mut cursor = std::io::Cursor::new(&data);
            black_box(kernel.execute(&mut cursor, "test_key"));
        })
    });
}

fn bench_key_exchange(c: &mut Criterion) {
    c.bench_function("key_exchange_generate", |b| {
        b.iter(|| {
            let mut alice = NexKeyExchange::new();
            black_box(alice.generate_keypair().unwrap());
        })
    });
    
    c.bench_function("key_exchange_encapsulate", |b| {
        b.iter_with_setup(
            || {
                let mut alice = NexKeyExchange::new();
                let alice_pub = alice.generate_keypair().unwrap();
                alice_pub
            },
            |alice_pub| {
                let mut bob = NexKeyExchange::new();
                black_box(bob.encapsulate(&alice_pub).unwrap());
            }
        )
    });
    
    c.bench_function("key_exchange_decapsulate", |b| {
        b.iter_with_setup(
            || {
                let mut alice = NexKeyExchange::new();
                let alice_pub = alice.generate_keypair().unwrap();
                let mut bob = NexKeyExchange::new();
                let (ciphertext, _) = bob.encapsulate(&alice_pub).unwrap();
                (alice_pub, ciphertext)
            },
            |(alice_pub, ciphertext)| {
                let mut alice = NexKeyExchange::new();
                black_box(alice.decapsulate(&ciphertext).unwrap());
            }
        )
    });
}

fn bench_hmac(c: &mut Criterion) {
    let key = secure_random_bytes(32).unwrap();
    let message = b"Benchmark message for HMAC performance testing";
    
    c.bench_function("hmac_sign_1kb", |b| {
        let hmac = HmacNex::new(&key).unwrap();
        b.iter(|| {
            black_box(hmac.sign(message));
        })
    });
    
    c.bench_function("hmac_sign_64kb", |b| {
        let hmac = HmacNex::new(&key).unwrap();
        let large_message = vec![0u8; 64 * 1024];
        b.iter(|| {
            black_box(hmac.sign(&large_message));
        })
    });
    
    c.bench_function("hmac_verify", |b| {
        let hmac = HmacNex::new(&key).unwrap();
        let signature = hmac.sign(message);
        b.iter(|| {
            black_box(hmac.verify(message, &signature));
        })
    });
}

fn bench_ctr_mode(c: &mut Criterion) {
    let kernel = NexKernel::new(1);
    let nonce = [0u8; 32];
    let ctr_key = "benchmark_ctr_key_123456789";
    
    c.bench_function("ctr_encrypt_1kb", |b| {
        let mut ctr = CNTMode::new(kernel.clone(), nonce, ctr_key.to_string());
        let plaintext = vec![0u8; 1024];
        b.iter(|| {
            black_box(ctr.process(&plaintext));
        })
    });
    
    c.bench_function("ctr_encrypt_1mb", |b| {
        let mut ctr = CNTMode::new(kernel.clone(), nonce, ctr_key.to_string());
        let plaintext = vec![0u8; 1024 * 1024];
        b.iter(|| {
            black_box(ctr.process(&plaintext));
        })
    });
}

fn bench_quantum_lattice(c: &mut Criterion) {
    c.bench_function("lattice_diffuse", |b| {
        b.iter(|| {
            let mut lattice = LatticeEngine::new();
            let test_input = vec![1u32, 2, 3, 4, 5];
            lattice.inject(&test_input);
            let seed = [0x12345678, 0x9ABCDEF0, 0xFEDCBA98, 0x76543210];
            black_box(lattice.diffuse(seed));
        })
    });
    
    c.bench_function("lattice_inject", |b| {
        b.iter(|| {
            let mut lattice = LatticeEngine::new();
            let test_input = vec![1u32, 2, 3, 4, 5];
            black_box(lattice.inject(&test_input));
        })
    });
}

fn bench_memory_hardening(c: &mut Criterion) {
    let blocks = vec![0u64; 8];
    
    c.bench_function("memory_hardening_100", |b| {
        b.iter(|| {
            black_box(apply_memory_hardening_parallel(blocks.clone(), 100));
        })
    });
    
    c.bench_function("memory_hardening_1000", |b| {
        b.iter(|| {
            black_box(apply_memory_hardening_parallel(blocks.clone(), 1000));
        })
    });
    
    c.bench_function("memory_hardening_10000", |b| {
        b.iter(|| {
            black_box(apply_memory_hardening_parallel(blocks.clone(), 10000));
        })
    });
}

fn bench_memory_optimization(c: &mut Criterion) {
    c.bench_function("zero_copy_buffer", |b| {
        b.iter(|| {
            let mut buffer = ZeroCopyBuffer::new(1024);
            let slice = buffer.get_mut(512).unwrap();
            black_box(slice.len());
        })
    });
    
    c.bench_function("streaming_processor", |b| {
        let data = vec![0u8; 1024 * 1024];
        let cursor = std::io::Cursor::new(data);
        b.iter(|| {
            let mut processor = StreamingProcessor::new(cursor.clone(), 8192);
            let mut total = 0;
            processor.process_chunks(|chunk| {
                total += chunk.len();
                Ok(())
            }).unwrap();
            black_box(total);
        })
    });
}

fn bench_entropy_generation(c: &mut Criterion) {
    c.bench_function("secure_random_1kb", |b| {
        b.iter(|| {
            black_box(secure_random_bytes(1024).unwrap());
        })
    });
    
    c.bench_function("secure_random_1mb", |b| {
        b.iter(|| {
            black_box(secure_random_bytes(1024 * 1024).unwrap());
        })
    });
    
    c.bench_function("rng_operations", |b| {
        b.iter(|| {
            let mut rng = SecureRng::new().unwrap();
            black_box(rng.next_u64().unwrap());
        })
    });
}

fn bench_parallel_performance(c: &mut Criterion) {
    use rayon::prelude::*;
    
    let data: Vec<u64> = (0..10000).collect();
    
    c.bench_function("sequential_processing", |b| {
        b.iter(|| {
            let sum: u64 = data.iter().sum();
            black_box(sum);
        })
    });
    
    c.bench_function("parallel_processing", |b| {
        b.iter(|| {
            let sum: u64 = data.par_iter().sum();
            black_box(sum);
        })
    });
}

fn bench_error_handling(c: &mut Criterion) {
    use nex2426::error::{NexError, NexResult};
    
    c.bench_function("error_creation", |b| {
        b.iter(|| {
            let result: NexResult<()> = Err(NexError::validation("test error"));
            black_box(result.is_err());
        })
    });
    
    c.bench_function("validation_macros", |b| {
        b.iter(|| {
            let key = [0x12, 0x34, 0x56, 0x78];
            let is_valid = nex2426::validation::validate_key_material(&key, "test").is_ok();
            black_box(is_valid);
        })
    });
}

criterion_group!(
    benches,
    bench_kernel_hashing,
    bench_key_exchange,
    bench_hmac,
    bench_ctr_mode,
    bench_quantum_lattice,
    bench_memory_hardening,
    bench_memory_optimization,
    bench_entropy_generation,
    bench_parallel_performance,
    bench_error_handling
);

criterion_main!(benches);
