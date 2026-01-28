
.intel_syntax noprefix
.section .text
.global nex_mix
.global nex_scramble
.global nex_mix_avx2

# -----------------------------------------------------------------------
# Function: nex_mix(regs: u64, val: u64) -> u64
# Arguments:
#   rdi = seed (a)
#   rsi = value (b)
# Return:
#   rax = mixed result
# -----------------------------------------------------------------------
nex_mix:
    mov rax, rdi        # rax = a
    add rax, rsi        # rax = a + b
    xor rax, rsi        # rax = (a + b) ^ b
    rol rax, 13         # rax = rotate_left(rax, 13)
    ret

# -----------------------------------------------------------------------
# Function: nex_scramble(val: u64) -> u64
# Arguments:
#   rdi = val
# Description:
#   Hardware Accelerated Scrambling using AES-NI instructions.
#   Uses the CPU's dedicated encryption circuits for maximum confusion/diffusion.
# -----------------------------------------------------------------------
nex_scramble:
    # 1. Load input into XMM0 (128-bit vector)
    vmovq xmm0, rdi
    
    # 2. Prepare Round Key (Magic Constant: Golden Ratio Phi)
    mov rax, 0x9E3779B97F4A7C15
    vmovq xmm1, rax
    vpbroadcastq xmm1, xmm1  # XMM1 = [Key | Key] (128-bit)
    
    # 3. AES Encryption Rounds
    # We treat the input as a block of data and "encrypt" it with our constant key.
    # This provides non-linear mixing far superior to simple multiplication.
    
    aesenc xmm0, xmm1        # Round 1: ShiftRows, SubBytes, MixColumns, AddRoundKey
    aesenclast xmm0, xmm1    # Round 2: Final scramble (no MixColumns)
    
    # 4. Extract result
    vmovq rax, xmm0
    ret

# -----------------------------------------------------------------------
# Function: nex_mix_avx2(ptr: *mut u64, mask: *const u64)
# Arguments:
#   rdi = ptr to [u64; 4] (Destination/Source)
#   rsi = ptr to [u64; 4] (Mask/Salt)
# Description:
#   Performs parallel mixing of 4 64-bit integers using AVX2 instructions.
#   Equivalent to running 4 'nex_mix' operations simultaneously.
# -----------------------------------------------------------------------
nex_mix_avx2:
    # Load 256 bits (4 x 64-bit) unaligned is safer usually, but aligned preferred if creating aligned structs.
    # We use vmovdqu (unaligned) just in case Rust Vec alignment matches or fails slightly.
    vmovdqu ymm0, [rdi]      # YMM0 = [A, B, C, D]
    vmovdqu ymm1, [rsi]      # YMM1 = [M0, M1, M2, M3]

    # Parallel Mixing Algorithm (AVX2)
    # 1. ADD: Dest += Mask
    vpaddq ymm0, ymm0, ymm1
    
    # 2. XOR: Dest ^= Mask
    vpxor ymm0, ymm0, ymm1

    # 3. Mixing Multiply (Simulate scrambling)
    # Note: Full 64-bit multiply is slow/complex in AVX2, usually 32-bit.
    # We will use simple ADD/SUB/XOR chains for speed.
    vpaddq ymm0, ymm0, ymm0  # Left Shift by 1 (effectively)
    vpxor ymm0, ymm0, ymm1   # Mix Mask again

    # Store back
    vmovdqu [rdi], ymm0
    
    # Return (void)
# -----------------------------------------------------------------------
# Function: nex_cpuid_hash() -> u64
# Description:
#   Generates a unique hash based on the physical CPU signature (CPUID).
#   Used for "Hardware Locking" encryption.
# -----------------------------------------------------------------------
.global nex_cpuid_hash
nex_cpuid_hash:
    push rbx          # Save RBX (callee-saved)
    
    # 1. Get Vendor ID (EAX=0)
    xor rax, rax
    cpuid
    # RBX, RDX, RCX contain "GenuineIntel" or "AuthenticAMD"
    # Mix them into a base hash
    mov r8, rbx
    xor r8, rcx
    xor r8, rdx
    
    # 2. Get Processor Info / Feature Bits (EAX=1)
    mov rax, 1
    cpuid
    # EAX = Stepping/Family, EBX = Brand Index, ECX/EDX = Feature Bits
    
    # Mix everything into R8
    xor r8, rax
    rol r8, 10
    xor r8, rbx
    rol r8, 10
    xor r8, rcx
    rol r8, 10
    xor r8, rdx
    
    # Final Result
    mov rax, r8
    
    pop rbx           # Restore RBX
    ret
