// Constants for lattice-based operations
pub const LATTICE_DIMENSION: usize = 100;
pub const MIXING_CONSTANT: u32 = 0x9e3779b9;
pub const NOISE_CONSTANT: u32 = 0x6a09e667;

// Quantum lattice constants
pub const LATTICE_STATE_SIZE: usize = 100;
pub const LATTICE_ROUNDS: usize = 16;
pub const LATTICE_DIFFUSION_ROUNDS: usize = 2;
pub const LATTICE_MIXING_PARTNER: usize = 31; // Coprime with 100

// Quantum security parameters
pub const QUANTUM_SECURITY_LEVEL: u32 = 128; // bits
pub const QUANTUM_MODULUS: u64 = 0x7fffffff; // Large prime for lattice operations
pub const QUANTUM_ERROR_BOUND: u64 = 1000; // Error tolerance for quantum operations

// Lattice transformation constants
pub const LATTICE_ROTATE_LEFT: u32 = 13;
pub const LATTICE_ROTATE_RIGHT_BASE: u32 = 32;
pub const LATTICE_CHAOS_SEED_SIZE: usize = 4; // 4 x u64 seeds

// Performance constants
pub const LATTICE_PARALLEL_THRESHOLD: usize = 50;
pub const LATTICE_CACHE_SIZE: usize = 1024; // bytes
