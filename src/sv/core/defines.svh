//==============================================================================
// NEX2426 SystemVerilog Defines
// Global definitions and constants
//==============================================================================

`ifndef NEX2426_DEFINES_SVH
`define NEX2426_DEFINES_SVH

// Version information
`define NEX2426_VERSION_MAJOR 6
`define NEX2426_VERSION_MINOR 0
`define NEX2426_VERSION_PATCH 0
`define NEX2426_VERSION_STRING "6.0.0"

// Debug macros
`ifdef NEX2426_DEBUG
    `define DEBUG_PRINT(fmt, args) $display("[DEBUG] " + fmt, args)
`else
    `define DEBUG_PRINT(fmt, args)
`endif

// Assertions
`ifdef NEX2426_ASSERTIONS
    `define ASSERT(cond, msg) assert(cond) else $error(msg)
`else
    `define ASSERT(cond, msg)
`endif

// Coverage
`ifdef NEX2426_COVERAGE
    `define COVER(name) covergroup name_cg; coverpoint name; endgroup name_cg = new();
`else
    `define COVER(name)
`endif

// Timing parameters
`define NEX2426_CLK_PERIOD 10ns
`define NEX2426_RESET_TIME 100ns

// Memory parameters
`define NEX2426_MEM_SIZE 4096
`define NEX2426_MEM_ADDR_WIDTH 12
`define NEX2426_MEM_DATA_WIDTH 64

// Crypto parameters
`define NEX2426_KEY_SIZE 256
`define NEX2426_BLOCK_SIZE 512
`define NEX2426_HASH_SIZE 512
`define NEX2426_MAX_COST 1000

`endif // NEX2426_DEFINES_SVH
