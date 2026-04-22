//==============================================================================
// NEX2426 Hardware Bridge - Rust Interface
// Bridge between SystemVerilog hardware and Rust software
//==============================================================================

`include "nex2426_pkg.svh"

module hardware_bridge #(
    parameter ADDR_WIDTH = 32,
    parameter DATA_WIDTH = 64
)(
    // Clock and reset
    input  logic                    clk,
    input  logic                    rst_n,
    
    // Rust interface signals (memory-mapped)
    input  logic                    rust_clk,
    input  logic                    rust_rst_n,
    input  logic                    rust_enable,
    input  logic [ADDR_WIDTH-1:0]   rust_addr,
    input  logic                    rust_write,
    input  logic [DATA_WIDTH-1:0]   rust_wdata,
    output logic [DATA_WIDTH-1:0]   rust_rdata,
    output logic                    rust_ready,
    output logic                    rust_valid,
    
    // NEX2426 core interface
    output logic                    core_start,
    output operation_mode_t         core_mode,
    output logic [255:0]            core_key,
    output logic [31:0]             core_cost,
    output logic                    core_bio_lock,
    output logic                    core_stealth,
    output logic [63:0]             core_hw_id,
    output logic [7:0]              core_data_in,
    output logic                    core_data_valid,
    input  logic [7:0]              core_data_out,
    input  logic                    core_data_ready,
    input  logic                    core_done,
    input  status_code_t           core_status,
    input  logic [511:0]            core_hash_out,
    
    // Status and debug
    output logic [31:0]             bridge_status,
    output logic                    bridge_error
);
    
    import nex2426_pkg::*;
    
    // Internal registers for Rust interface
    logic [DATA_WIDTH-1:0] reg_array [255:0];
    logic [7:0] reg_addr;
    logic reg_write;
    logic [DATA_WIDTH-1:0] reg_wdata;
    logic [DATA_WIDTH-1:0] reg_rdata;
    logic reg_valid;
    
    // Register map for Rust interface
    localparam REG_CONTROL       = 8'h00;
    localparam REG_STATUS        = 8'h01;
    localparam REG_MODE          = 8'h02;
    localparam REG_KEY0          = 8'h10;
    localparam REG_KEY1          = 8'h11;
    localparam REG_KEY2          = 8'h12;
    localparam REG_KEY3          = 8'h13;
    localparam REG_COST          = 8'h20;
    localparam REG_CONFIG        = 8'h21;
    localparam REG_HW_ID         = 8'h22;
    localparam REG_DATA_IN       = 8'h30;
    localparam REG_DATA_OUT      = 8'h31;
    localparam REG_HASH0         = 8'h40;
    localparam REG_HASH1         = 8'h41;
    localparam REG_HASH2         = 8'h42;
    localparam REG_HASH3         = 8'h43;
    localparam REG_HASH4         = 8'h44;
    localparam REG_HASH5         = 8'h45;
    localparam REG_HASH6         = 8'h46;
    localparam REG_HASH7         = 8'h47;
    localparam REG_BRIDGE_STATUS = 8'hFE;
    localparam REG_VERSION       = 8'hFF;
    
    // Rust interface state machine
    typedef enum logic [2:0] {
        RUST_IDLE,
        RUST_READ,
        RUST_WRITE,
        RUST_RESPONSE
    } rust_state_t;
    
    rust_state_t rust_state;
    logic [31:0] rust_cycle_count;
    
    // Clock domain crossing (CDC)
    logic rust_enable_sync;
    logic [7:0] rust_addr_sync;
    logic rust_write_sync;
    logic [DATA_WIDTH-1:0] rust_wdata_sync;
    
    // Synchronizers for clock domain crossing
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rust_enable_sync <= 1'b0;
            rust_addr_sync <= 8'h00;
            rust_write_sync <= 1'b0;
            rust_wdata_sync <= 64'h0;
        end else begin
            rust_enable_sync <= rust_enable;
            rust_addr_sync <= rust_addr[7:0];
            rust_write_sync <= rust_write;
            rust_wdata_sync <= rust_wdata;
        end
    end
    
    // Main Rust interface logic
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            rust_state <= RUST_IDLE;
            rust_ready <= 1'b1;
            rust_valid <= 1'b0;
            rust_rdata <= 64'h0;
            bridge_status <= 32'h0;
            bridge_error <= 1'b0;
            rust_cycle_count <= 0;
        end else begin
            rust_cycle_count <= rust_cycle_count + 1;
            
            case (rust_state)
                RUST_IDLE: begin
                    if (rust_enable_sync && rust_ready) begin
                        rust_state <= rust_write_sync ? RUST_WRITE : RUST_READ;
                        rust_ready <= 1'b0;
                        reg_addr <= rust_addr_sync;
                        reg_write <= rust_write_sync;
                        reg_wdata <= rust_wdata_sync;
                    end
                end
                
                RUST_WRITE: begin
                    register_write(reg_addr, reg_wdata);
                    rust_state <= RUST_RESPONSE;
                    rust_valid <= 1'b1;
                    bridge_status[31:16] <= rust_cycle_count;
                end
                
                RUST_READ: begin
                    reg_rdata <= register_read(reg_addr);
                    rust_state <= RUST_RESPONSE;
                    rust_valid <= 1'b1;
                    bridge_status[31:16] <= rust_cycle_count;
                end
                
                RUST_RESPONSE: begin
                    rust_rdata <= reg_rdata;
                    if (!rust_enable_sync) begin
                        rust_state <= RUST_IDLE;
                        rust_ready <= 1'b1;
                        rust_valid <= 1'b0;
                    end
                end
                
                default: begin
                    rust_state <= RUST_IDLE;
                end
            endcase
        end
    end
    
    // Register write task
    task register_write;
        input [7:0] addr;
        input [63:0] data;
    begin
        case (addr)
            REG_CONTROL: begin
                reg_array[addr] <= data;
                core_start <= data[0];
            end
            REG_MODE: begin
                reg_array[addr] <= data;
                core_mode <= operation_mode_t'(data[2:0]);
            end
            REG_KEY0: begin
                reg_array[addr] <= data;
                core_key[31:0] <= data[31:0];
            end
            REG_KEY1: begin
                reg_array[addr] <= data;
                core_key[63:32] <= data[31:0];
            end
            REG_KEY2: begin
                reg_array[addr] <= data;
                core_key[95:64] <= data[31:0];
            end
            REG_KEY3: begin
                reg_array[addr] <= data;
                core_key[127:96] <= data[31:0];
            end
            REG_COST: begin
                reg_array[addr] <= data;
                core_cost <= data[31:0];
            end
            REG_CONFIG: begin
                reg_array[addr] <= data;
                core_bio_lock <= data[0];
                core_stealth <= data[1];
            end
            REG_HW_ID: begin
                reg_array[addr] <= data;
                core_hw_id[31:0] <= data[31:0];
            end
            REG_DATA_IN: begin
                reg_array[addr] <= data;
                core_data_in <= data[7:0];
                core_data_valid <= 1'b1;
            end
            REG_BRIDGE_STATUS: begin
                reg_array[addr] <= bridge_status;
            end
            default: begin
                // Read-only registers
                bridge_error <= 1'b1;
            end
        endcase
    end
    endtask
    
    // Register read function
    function logic [63:0] register_read;
        input [7:0] addr;
    begin
        case (addr)
            REG_STATUS: register_read = {28'b0, core_status, 3'b0, core_done};
            REG_MODE: register_read = reg_array[REG_MODE];
            REG_KEY0: register_read = {32'b0, core_key[31:0]};
            REG_KEY1: register_read = {32'b0, core_key[63:32]};
            REG_KEY2: register_read = {32'b0, core_key[95:64]};
            REG_KEY3: register_read = {32'b0, core_key[127:96]};
            REG_COST: register_read = {32'b0, core_cost};
            REG_CONFIG: register_read = {30'b0, core_stealth, core_bio_lock, 30'b0};
            REG_HW_ID: register_read = {32'b0, core_hw_id[31:0]};
            REG_DATA_OUT: register_read = {56'b0, core_data_out};
            REG_HASH0: register_read = {32'b0, core_hash_out[31:0]};
            REG_HASH1: register_read = {32'b0, core_hash_out[63:32]};
            REG_HASH2: register_read = {32'b0, core_hash_out[95:64]};
            REG_HASH3: register_read = {32'b0, core_hash_out[127:96]};
            REG_HASH4: register_read = {32'b0, core_hash_out[159:128]};
            REG_HASH5: register_read = {32'b0, core_hash_out[191:160]};
            REG_HASH6: register_read = {32'b0, core_hash_out[223:192]};
            REG_HASH7: register_read = {32'b0, core_hash_out[255:224]};
            REG_BRIDGE_STATUS: register_read = bridge_status;
            REG_VERSION: register_read = {32'h00060000, 32'h4E455832}; // "NEX2" version 6.0.0
            default: register_read = 64'h0;
        endcase
    end
    endfunction
    
    // Update bridge status
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            bridge_status[15:0] <= 16'h0000;
        end else begin
            bridge_status[15:8] <= rust_state;
            bridge_status[7:0] <= {bridge_error, 7'b0};
        end
    end
    
    // Clear data valid after one cycle
    always_ff @(posedge clk) begin
        if (core_data_valid) begin
            core_data_valid <= 1'b0;
        end
    end
    
endmodule
