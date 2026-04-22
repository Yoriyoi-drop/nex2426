//==============================================================================
// NEX2426 Encryption Engine - Hardware Implementation
// Stream cipher based on chaos engine with hardware binding support
//==============================================================================

`include "nex2426_pkg.svh"

module encryption_engine #(
    parameter KEY_SIZE = 256,
    parameter BLOCK_SIZE = 512,
    parameter STREAM_WIDTH = 64
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    start,
    input  operation_mode_t         mode,
    input  logic [KEY_SIZE-1:0]      key,
    input  logic [31:0]             cost,
    input  logic                    bio_lock_enable,
    input  logic                    stealth_mode,
    input  logic [63:0]             hardware_id,
    input  logic [7:0]              data_in,
    input  logic                    data_valid,
    output logic [7:0]              data_out,
    output logic                    data_ready,
    output logic                    done,
    output status_code_t           status
);
    
    import nex2426_pkg::*;
    
    // Internal state
    logic [KEY_SIZE-1:0] session_key;
    logic [511:0] key_material;
    logic [63:0] keystream;
    logic [63:0] chaos_entropy;
    logic encrypting;
    logic bio_lock_verified;
    logic stealth_timestamp_valid;
    
    // Chaos engine for keystream generation
    chaos_engine #(.SEED_WIDTH(KEY_SIZE)) chaos_gen (
        .clk(clk),
        .rst_n(rst_n),
        .enable(start),
        .seed(key_material[KEY_SIZE-1:0]),
        .ready(),
        .entropy_out(chaos_entropy),
        .valid()
    );
    
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
    
    // Stealth mode timestamp generator
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
            // Include hardware ID in key material
            session_key = key ^ {hardware_id, hardware_id, hardware_id, hardware_id};
        end else begin
            session_key = key;
        end
        
        // Derive 512-bit key material from 256-bit session key
        key_material = {session_key, session_key ^ chaos_entropy};
    end
    
    // Keystream generation
    logic [63:0] keystream_counter;
    logic [63:0] keystream_seed;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            keystream_counter <= 0;
            keystream_seed <= 0;
            encrypting <= 1'b0;
            done <= 1'b0;
            status <= STATUS_IDLE;
        end else if (start) begin
            if (mode == MODE_ENCRYPT || mode == MODE_DECRYPT) begin
                if (bio_lock_enable && !bio_lock_verified) begin
                    status <= STATUS_ERROR_KEY;
                    done <= 1'b1;
                end else if (stealth_mode && !stealth_timestamp_valid) begin
                    status <= STATUS_ERROR_CRYPTO;
                    done <= 1'b1;
                end else begin
                    encrypting <= 1'b1;
                    keystream_counter <= 0;
                    keystream_seed <= key_material[63:0];
                    status <= STATUS_BUSY;
                end
            end else begin
                status <= STATUS_ERROR_MODE;
                done <= 1'b1;
            end
        end else if (encrypting && data_valid) begin
            // Generate next keystream byte
            keystream_counter <= keystream_counter + 1;
            keystream_seed <= asm_scramble(keystream_seed ^ keystream_counter);
        end else if (encrypting && !data_valid && done) begin
            encrypting <= 1'b0;
            status <= STATUS_SUCCESS;
        end
    end
    
    // Data encryption/decryption
    always_ff @(posedge clk) begin
        if (encrypting && data_valid) begin
            keystream <= asm_scramble(keystream_seed ^ keystream_counter);
            data_out <= data_in ^ keystream[7:0];
            data_ready <= 1'b1;
        end else begin
            data_ready <= 1'b0;
        end
    end
    
endmodule

//==============================================================================
// Bio-Lock Verifier Module
//==============================================================================

module bio_lock_verifier (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        enable,
    input  logic [63:0]  hardware_id,
    input  logic [255:0] key,
    output logic        verified,
    output logic        error
);
    import nex2426_pkg::*;
    
    logic [255:0] hw_binding_key;
    logic [255:0] computed_hash;
    logic verifying;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            verified <= 1'b0;
            error <= 1'b0;
            verifying <= 1'b0;
        end else if (enable && !verifying) begin
            verifying <= 1'b1;
            verified <= 1'b0;
            error <= 1'b0;
            
            // Derive hardware binding key
            hw_binding_key <= key ^ {hardware_id, hardware_id, hardware_id, hardware_id};
        end else if (verifying) begin
            // Simple hash verification (in real implementation, use proper crypto)
            computed_hash <= {hw_binding_key[127:0], hw_binding_key[255:128]};
            verified <= 1'b1; // Simplified - always verify for demo
            verifying <= 1'b0;
        end
    end
    
endmodule

//==============================================================================
// Stealth Timestamp Generator Module
//==============================================================================

module stealth_timestamp_gen (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        enable,
    input  logic [255:0] key,
    output logic        valid,
    output logic [63:0]  timestamp
);
    import nex2426_pkg::*;
    
    logic [63:0] derived_timestamp;
    logic generating;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            valid <= 1'b0;
            timestamp <= 0;
            generating <= 1'b0;
        end else if (enable && !generating) begin
            generating <= 1'b1;
            valid <= 1'b0;
            
            // Derive timestamp from key (deterministic but secure)
            derived_timestamp <= key[63:0] ^ key[127:64] ^ key[191:128] ^ key[255:192];
        end else if (generating) begin
            timestamp <= derived_timestamp;
            valid <= 1'b1;
            generating <= 1'b0;
        end
    end
    
endmodule
