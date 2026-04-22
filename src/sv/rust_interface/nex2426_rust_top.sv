//==============================================================================
// NEX2426 Rust Interface Top Module
// Complete SystemVerilog implementation with Rust interface
//==============================================================================

`include "nex2426_pkg.svh"

module nex2426_rust_top #(
    parameter ADDR_WIDTH = 32,
    parameter DATA_WIDTH = 64
)(
    // Clock and reset
    input  logic                    clk,
    input  logic                    rst_n,
    
    // Rust interface signals
    input  logic                    rust_clk,
    input  logic                    rust_rst_n,
    input  logic                    rust_enable,
    input  logic [ADDR_WIDTH-1:0]   rust_addr,
    input  logic                    rust_write,
    input  logic [DATA_WIDTH-1:0]   rust_wdata,
    output logic [DATA_WIDTH-1:0]   rust_rdata,
    output logic                    rust_ready,
    output logic                    rust_valid,
    
    // Debug and status outputs
    output logic [31:0]             debug_status,
    output logic                    debug_error,
    output logic [63:0]             performance_counters
);
    
    import nex2426_pkg::*;
    
    // Internal signals
    logic core_start;
    operation_mode_t core_mode;
    logic [255:0] core_key;
    logic [31:0] core_cost;
    logic core_bio_lock;
    logic core_stealth;
    logic [63:0] core_hw_id;
    logic [7:0] core_data_in;
    logic core_data_valid;
    logic [7:0] core_data_out;
    logic core_data_ready;
    logic core_done;
    status_code_t core_status;
    logic [511:0] core_hash_out;
    logic [31:0] core_performance_cycles;
    
    // Hardware bridge instance
    hardware_bridge #(
        .ADDR_WIDTH(ADDR_WIDTH),
        .DATA_WIDTH(DATA_WIDTH)
    ) hw_bridge (
        .clk(clk),
        .rst_n(rst_n),
        .rust_clk(rust_clk),
        .rust_rst_n(rust_rst_n),
        .rust_enable(rust_enable),
        .rust_addr(rust_addr),
        .rust_write(rust_write),
        .rust_wdata(rust_wdata),
        .rust_rdata(rust_rdata),
        .rust_ready(rust_ready),
        .rust_valid(rust_valid),
        .core_start(core_start),
        .core_mode(core_mode),
        .core_key(core_key),
        .core_cost(core_cost),
        .core_bio_lock(core_bio_lock),
        .core_stealth(core_stealth),
        .core_hw_id(core_hw_id),
        .core_data_in(core_data_in),
        .core_data_valid(core_data_valid),
        .core_data_out(core_data_out),
        .core_data_ready(core_data_ready),
        .core_done(core_done),
        .core_status(core_status),
        .core_hash_out(core_hash_out),
        .bridge_status(debug_status),
        .bridge_error(debug_error)
    );
    
    // NEX2426 top-level core
    nex2426_top nex2426_core (
        .clk(clk),
        .rst_n(rst_n),
        .start(core_start),
        .mode(core_mode),
        .key(core_key),
        .cost(core_cost),
        .bio_lock_enable(core_bio_lock),
        .stealth_mode(core_stealth),
        .hardware_id(core_hw_id),
        .data_in(core_data_in),
        .data_valid(core_data_valid),
        .data_out(core_data_out),
        .data_ready(core_data_ready),
        .done(core_done),
        .status(core_status),
        .hash_output(core_hash_out),
        .performance_cycles(core_performance_cycles)
    );
    
    // Performance counters
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            performance_counters <= 64'b0;
        end else begin
            performance_counters[31:0] <= core_performance_cycles;
            performance_counters[63:32] <= debug_status[15:0]; // Bridge cycle count
        end
    end
    
    // Debug status monitoring
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            debug_error <= 1'b0;
        end else begin
            debug_error <= debug_error | (core_status != STATUS_IDLE && core_status != STATUS_SUCCESS);
        end
    end
    
endmodule
