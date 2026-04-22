//==============================================================================
// NEX2426 Hash Core - Hardware Implementation
// Multi-stage hashing pipeline with chaos-based mixing
//==============================================================================

`include "nex2426_pkg.svh"

module hash_core #(
    parameter PIPELINE_STAGES = 5,
    parameter HASH_ROUNDS = 64
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    start,
    input  operation_mode_t         mode,
    input  logic [255:0]            key,
    input  logic [511:0]            input_data,
    input  logic [31:0]             cost,
    output logic                    done,
    output logic [511:0]            hash_out,
    output status_code_t           status
);
    
    import nex2426_pkg::*;
    
    // Pipeline stages
    logic [511:0] stage1_out, stage2_out, stage3_out, stage4_out, stage5_out;
    logic stage1_valid, stage2_valid, stage3_valid, stage4_valid, stage5_valid;
    logic stage1_ready, stage2_ready, stage3_ready, stage4_ready, stage5_ready;
    
    // Control signals
    logic [31:0] round_count;
    logic processing;
    logic error_detected;
    
    // Stage 1: Input Expansion
    stage1_expansion stage1 (
        .clk(clk),
        .rst_n(rst_n),
        .start(start),
        .input_data(input_data),
        .key(key),
        .output_data(stage1_out),
        .valid(stage1_valid),
        .ready(stage1_ready)
    );
    
    // Stage 2: Chaos Mixing
    chaos_mixer stage2 (
        .clk(clk),
        .rst_n(rst_n),
        .enable(stage1_valid),
        .input_data(stage1_out),
        .key(key),
        .round_count(round_count[7:0]),
        .output_data(stage2_out),
        .valid(stage2_valid),
        .ready(stage2_ready)
    );
    
    // Stage 3: Memory Hardening
    memory_hardening stage3 (
        .clk(clk),
        .rst_n(rst_n),
        .start(stage2_valid),
        .input_blocks(stage2_out),
        .iterations(cost),
        .output_blocks(stage3_out),
        .done(stage3_valid),
        .error(error_detected)
    );
    
    // Stage 4: Virtual Machine Processing
    vm_processor stage4 (
        .clk(clk),
        .rst_n(rst_n),
        .enable(stage3_valid),
        .input_data(stage3_out),
        .key(key),
        .output_data(stage4_out),
        .valid(stage4_valid),
        .ready(stage4_ready)
    );
    
    // Stage 5: Temporal Binding
    temporal_binding stage5 (
        .clk(clk),
        .rst_n(rst_n),
        .enable(stage4_valid),
        .input_data(stage4_out),
        .output_data(stage5_out),
        .valid(stage5_valid),
        .ready(stage5_ready)
    );
    
    // Main control logic
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            round_count <= 0;
            processing <= 1'b0;
            done <= 1'b0;
            status <= STATUS_IDLE;
            hash_out <= 512'b0;
        end else if (start) begin
            if (cost < COST_MIN || cost > COST_MAX) begin
                status <= STATUS_ERROR_COST;
                done <= 1'b1;
            end else if (mode == MODE_ENCRYPT || mode == MODE_DECRYPT || mode == MODE_HASH) begin
                processing <= 1'b1;
                round_count <= 0;
                status <= STATUS_BUSY;
            end else begin
                status <= STATUS_ERROR_MODE;
                done <= 1'b1;
            end
        end else if (processing) begin
            if (round_count < HASH_ROUNDS) begin
                round_count <= round_count + 1;
            end else if (stage5_valid) begin
                processing <= 1'b0;
                done <= 1'b1;
                status <= STATUS_SUCCESS;
                hash_out <= stage5_out;
            end else if (error_detected) begin
                processing <= 1'b0;
                done <= 1'b1;
                status <= STATUS_ERROR_CRYPTO;
            end
        end else if (done) begin
            done <= 1'b0;
            status <= STATUS_IDLE;
        end
    end
    
endmodule

//==============================================================================
// Stage 1: Input Expansion Module
//==============================================================================

module stage1_expansion (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        start,
    input  logic [511:0] input_data,
    input  logic [255:0] key,
    output logic [511:0] output_data,
    output logic        valid,
    output logic        ready
);
    import nex2426_pkg::*;
    
    logic [31:0] part_count;
    logic [511:0] expanded_data;
    logic processing;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            part_count <= 0;
            processing <= 1'b0;
            valid <= 1'b0;
            ready <= 1'b1;
            expanded_data <= 512'b0;
        end else if (start && ready) begin
            processing <= 1'b1;
            ready <= 1'b0;
            part_count <= 0;
            
            // Mix key with input data
            expanded_data[511:384] <= input_data[511:384] ^ {key[255:128], key[255:128]};
            expanded_data[383:256] <= input_data[383:256] ^ {key[127:0], key[127:0]};
            expanded_data[255:128] <= input_data[255:128] ^ {key[255:128], key[255:128]};
            expanded_data[127:0] <= input_data[127:0] ^ {key[127:0], key[127:0]};
        end else if (processing) begin
            // Apply scrambling to each part
            for (int i = 0; i < 8; i++) begin
                expanded_data[64*i +: 64] <= asm_scramble(expanded_data[64*i +: 64]);
            end
            processing <= 1'b0;
            valid <= 1'b1;
        end else if (valid) begin
            valid <= 1'b0;
            ready <= 1'b1;
        end
    end
    
    assign output_data = expanded_data;
    
endmodule

//==============================================================================
// Chaos Mixing Module
//==============================================================================

module chaos_mixer (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        enable,
    input  logic [511:0] input_data,
    input  logic [255:0] key,
    input  logic [7:0]   round_count,
    output logic [511:0] output_data,
    output logic        valid,
    output logic        ready
);
    import nex2426_pkg::*;
    
    chaos_engine #(.SEED_WIDTH(256)) chaos_gen (
        .clk(clk),
        .rst_n(rst_n),
        .enable(enable),
        .seed({key[255:192], key[191:128], key[127:64], key[63:0]}),
        .ready(ready),
        .entropy_out(),
        .valid(valid)
    );
    
    logic [511:0] mixed_data;
    logic [63:0] chaos_entropy;
    
    // Generate chaos entropy for mixing
    chaos_engine #(.SEED_WIDTH(256)) chaos_entropy_gen (
        .clk(clk),
        .rst_n(rst_n),
        .enable(enable),
        .seed({input_data[511:448], input_data[383:320], input_data[255:192], input_data[127:64]}),
        .ready(),
        .entropy_out(chaos_entropy),
        .valid()
    );
    
    // Mix input data with chaos entropy
    assign mixed_data = input_data ^ {chaos_entropy, chaos_entropy, chaos_entropy, chaos_entropy,
                                     chaos_entropy, chaos_entropy, chaos_entropy, chaos_entropy};
    
    assign output_data = mixed_data;
    
endmodule

//==============================================================================
// Virtual Machine Processor Module
//==============================================================================

module vm_processor (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        enable,
    input  logic [511:0] input_data,
    input  logic [255:0] key,
    output logic [511:0] output_data,
    output logic        valid,
    output logic        ready
);
    import nex2426_pkg::*;
    
    logic [31:0] pc;
    logic [255:0] program_memory [255:0];
    logic [63:0] registers [7:0];
    logic [511:0] vm_data;
    logic processing;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            pc <= 0;
            processing <= 1'b0;
            valid <= 1'b0;
            ready <= 1'b1;
            vm_data <= 512'b0;
        end else if (enable && ready) begin
            processing <= 1'b1;
            ready <= 1'b0;
            pc <= 0;
            vm_data <= input_data;
            
            // Initialize registers with key-derived values
            registers[0] <= {key[255:192]};
            registers[1] <= {key[191:128]};
            registers[2] <= {key[127:64]};
            registers[3] <= {key[63:0]};
            registers[4] <= 64'h5555555555555555;
            registers[5] <= 64'hAAAAAAAAAAAAAAAA;
            registers[6] <= 64'h3333333333333333;
            registers[7] <= 64'hCCCCCCCCCCCCCCCC;
        end else if (processing) begin
            // Simple VM execution
            logic [7:0] opcode = program_memory[pc][7:0];
            logic [2:0] reg_idx = program_memory[pc][10:8];
            logic [63:0] operand = registers[reg_idx];
            
            case (opcode)
                8'h00: registers[reg_idx] <= asm_scramble(registers[reg_idx]); // SCRAMBLE
                8'h01: registers[reg_idx] <= registers[reg_idx] ^ operand;    // XOR
                8'h02: registers[reg_idx] <= registers[reg_idx] + operand;    // ADD
                8'h03: registers[reg_idx] <= registers[reg_idx] - operand;    // SUB
                8'h04: registers[reg_idx] <= registers[reg_idx] << 1;          // SHL
                8'h05: registers[reg_idx] <= registers[reg_idx] >> 1;          // SHR
                default: registers[reg_idx] <= registers[reg_idx];
            endcase
            
            pc <= pc + 1;
            
            if (pc >= 255) begin
                processing <= 1'b0;
                valid <= 1'b1;
                
                // Mix VM results back into data
                vm_data <= input_data ^ {registers[0], registers[1], registers[2], registers[3],
                                       registers[4], registers[5], registers[6], registers[7]};
            end
        end else if (valid) begin
            valid <= 1'b0;
            ready <= 1'b1;
        end
    end
    
    assign output_data = vm_data;
    
endmodule

//==============================================================================
// Temporal Binding Module
//==============================================================================

module temporal_binding (
    input  logic        clk,
    input  logic        rst_n,
    input  logic        enable,
    input  logic [511:0] input_data,
    output logic [511:0] output_data,
    output logic        valid,
    output logic        ready
);
    import nex2426_pkg::*;
    
    logic [63:0] timestamp;
    logic [511:0] temporal_data;
    logic processing;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            processing <= 1'b0;
            valid <= 1'b0;
            ready <= 1'b1;
            temporal_data <= 512'b0;
        end else if (enable && ready) begin
            processing <= 1'b1;
            ready <= 1'b0;
            timestamp <= get_timestamp();
        end else if (processing) begin
            // Bind timestamp to hash
            temporal_data <= input_data ^ {timestamp, timestamp, timestamp, timestamp,
                                         timestamp, timestamp, timestamp, timestamp};
            processing <= 1'b0;
            valid <= 1'b1;
        end else if (valid) begin
            valid <= 1'b0;
            ready <= 1'b1;
        end
    end
    
    assign output_data = temporal_data;
    
endmodule
