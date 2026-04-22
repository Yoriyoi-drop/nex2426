//==============================================================================
// NEX2426 SystemVerilog Package
// Quantum-Resistant Chaos Encryption Engine - Hardware Implementation
//==============================================================================

package nex2426_pkg;
    // Constants
    parameter BLOCK_SIZE = 512;
    parameter KEY_SIZE = 256;
    parameter HASH_SIZE = 512;
    parameter COST_MAX = 1000;
    parameter COST_MIN = 1;
    
    // Data types
    typedef logic [511:0] block_t;
    typedef logic [255:0] key_t;
    typedef logic [511:0] hash_t;
    typedef logic [63:0] word_t;
    typedef logic [31:0] dword_t;
    
    // Lorenz attractor parameters
    typedef struct packed {
        logic signed [63:0] x;
        logic signed [63:0] y;
        logic signed [63:0] z;
    } lorenz_state_t;
    
    // Chaos engine parameters
    parameter real SIGMA = 10.0;
    parameter real RHO = 28.0;
    parameter real BETA = 8.0/3.0;
    parameter real SCALE = 1000000.0;
    
    // Memory hardening parameters
    parameter MEMORY_SIZE = 1024;
    parameter LANE_COUNT = 8;
    
    // Operation modes
    typedef enum logic [2:0] {
        MODE_ENCRYPT = 3'b001,
        MODE_DECRYPT = 3'b010,
        MODE_HASH = 3'b011,
        MODE_BENCHMARK = 3'b100,
        MODE_KEYGEN = 3'b101,
        MODE_STEALTH = 3'b110,
        MODE_BIOLOCK = 3'b111
    } operation_mode_t;
    
    // Status codes
    typedef enum logic [3:0] {
        STATUS_IDLE = 4'b0000,
        STATUS_BUSY = 4'b0001,
        STATUS_SUCCESS = 4'b0010,
        STATUS_ERROR_KEY = 4'b0100,
        STATUS_ERROR_COST = 4'b0101,
        STATUS_ERROR_MODE = 4'b0110,
        STATUS_ERROR_MEMORY = 4'b0111,
        STATUS_ERROR_CRYPTO = 4'b1000
    } status_code_t;
    
    // Function prototypes
    function automatic word_t asm_scramble(input word_t seed);
        asm_scramble = (seed ^ (seed << 13)) ^ (seed ^ (seed >> 7));
    endfunction
    
    function automatic word_t asm_pseudo_rand(input word_t seed);
        asm_pseudo_rand = asm_scramble(seed) + 0x9E3779B97F4A7C15;
    endfunction
    
    // Utility functions
    function automatic logic [63:0] get_timestamp();
        get_timestamp = 64'(time() / 1000); // Convert to milliseconds
    endfunction
    
    function automatic logic [31:0] crc32(input logic [31:0] crc, input logic [7:0] data);
        logic [31:0] i;
        crc32 = crc ^ {24'b0, data};
        for (i = 0; i < 32; i++) begin
            if (crc32[31]) 
                crc32 = (crc32 << 1) ^ 0xEDB88320;
            else
                crc32 = crc32 << 1;
        end
    endfunction
    
endpackage
