//==============================================================================
// NEX2426 Top Level Module
// Complete NEX2426 encryption engine with all subsystems
//==============================================================================

`include "nex2426_pkg.svh"

module nex2426_top #(
    parameter KEY_SIZE = 256,
    parameter BLOCK_SIZE = 512,
    parameter PIPELINE_STAGES = 5
)(
    // Clock and reset
    input  logic                    clk,
    input  logic                    rst_n,
    
    // Control interface
    input  logic                    start,
    input  operation_mode_t         mode,
    input  logic [KEY_SIZE-1:0]      key,
    input  logic [31:0]             cost,
    input  logic                    bio_lock_enable,
    input  logic                    stealth_mode,
    input  logic [63:0]             hardware_id,
    
    // Data interface
    input  logic [7:0]              data_in,
    input  logic                    data_valid,
    output logic [7:0]              data_out,
    output logic                    data_ready,
    
    // Status interface
    output logic                    done,
    output status_code_t           status,
    output logic [BLOCK_SIZE-1:0]   hash_output,
    output logic [31:0]             performance_cycles
);
    
    import nex2426_pkg::*;
    
    // Internal signals
    logic [KEY_SIZE-1:0] session_key;
    logic [BLOCK_SIZE-1:0] hash_core_input;
    logic [BLOCK_SIZE-1:0] hash_core_output;
    logic hash_core_done;
    status_code_t hash_core_status;
    
    logic [7:0] encrypted_data;
    logic encrypt_done;
    status_code_t encrypt_status;
    
    logic bio_lock_verified;
    logic stealth_timestamp_valid;
    
    // Performance counter
    logic [31:0] cycle_counter;
    logic [31:0] start_cycle;
    logic [31:0] end_cycle;
    
    // Cycle counter
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            cycle_counter <= 0;
        end else begin
            cycle_counter <= cycle_counter + 1;
        end
    end
    
    // Performance measurement
    always_ff @(posedge clk) begin
        if (start && !done) begin
            start_cycle <= cycle_counter;
        end else if (done) begin
            end_cycle <= cycle_counter;
            performance_cycles <= end_cycle - start_cycle;
        end
    end
    
    // Bio-lock verification
    bio_lock_verifier bio_lock (
        .clk(clk),
        .rst_n(rst_n),
        .enable(bio_lock_enable),
        .hardware_id(hardware_id),
        .key(key),
        .verified(bio_lock_verified),
        .error()
    );
    
    // Stealth timestamp generator
    stealth_timestamp_gen stealth_ts (
        .clk(clk),
        .rst_n(rst_n),
        .enable(stealth_mode),
        .key(key),
        .valid(stealth_timestamp_valid),
        .timestamp()
    );
    
    // Key derivation
    always_comb begin
        if (bio_lock_enable && bio_lock_verified) begin
            session_key = key ^ {hardware_id, hardware_id, hardware_id, hardware_id};
        end else begin
            session_key = key;
        end
    end
    
    // Hash core for hashing operations
    hash_core hash_engine (
        .clk(clk),
        .rst_n(rst_n),
        .start(start && (mode == MODE_HASH || mode == MODE_BENCHMARK)),
        .mode(mode),
        .key(session_key),
        .input_data(hash_core_input),
        .cost(cost),
        .done(hash_core_done),
        .hash_out(hash_core_output),
        .status(hash_core_status)
    );
    
    // Encryption engine for encryption/decryption
    encryption_engine encrypt_engine (
        .clk(clk),
        .rst_n(rst_n),
        .start(start && (mode == MODE_ENCRYPT || mode == MODE_DECRYPT)),
        .mode(mode),
        .key(session_key),
        .cost(cost),
        .bio_lock_enable(bio_lock_enable),
        .stealth_mode(stealth_mode),
        .hardware_id(hardware_id),
        .data_in(data_in),
        .data_valid(data_valid),
        .data_out(encrypted_data),
        .data_ready(data_ready),
        .done(encrypt_done),
        .status(encrypt_status)
    );
    
    // Hash core input preparation
    always_comb begin
        case (mode)
            MODE_HASH: begin
                hash_core_input = {data_in, 504'b0}; // Pad single byte
            end
            MODE_BENCHMARK: begin
                hash_core_input = 512'h0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
            end
            default: begin
                hash_core_input = 512'h0;
            end
        endcase
    end
    
    // Main control logic
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            done <= 1'b0;
            status <= STATUS_IDLE;
            hash_output <= 512'b0;
            data_out <= 8'h0;
        end else begin
            case (mode)
                MODE_HASH, MODE_BENCHMARK: begin
                    if (hash_core_done) begin
                        done <= 1'b1;
                        status <= hash_core_status;
                        hash_output <= hash_core_output;
                    end else if (start) begin
                        done <= 1'b0;
                        status <= STATUS_BUSY;
                    end
                end
                
                MODE_ENCRYPT, MODE_DECRYPT: begin
                    if (encrypt_done) begin
                        done <= 1'b1;
                        status <= encrypt_status;
                        data_out <= encrypted_data;
                    end else if (start) begin
                        done <= 1'b0;
                        status <= STATUS_BUSY;
                    end
                end
                
                default: begin
                    if (start) begin
                        done <= 1'b1;
                        status <= STATUS_ERROR_MODE;
                    end
                end
            endcase
        end
    end
    
endmodule
