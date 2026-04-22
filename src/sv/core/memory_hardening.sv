//==============================================================================
// NEX2426 Memory Hardening - Hardware Implementation
// Argon2-inspired memory hardening with parallel lanes
//==============================================================================

`include "nex2426_pkg.svh"

module memory_hardening #(
    parameter LANE_COUNT = 8,
    parameter MEMORY_SIZE = 1024,
    parameter ITERATIONS = 100
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    start,
    input  logic [511:0]           input_blocks,
    input  logic [31:0]            iterations,
    output logic                    done,
    output logic [511:0]           output_blocks,
    output logic                    error
);
    
    import nex2426_pkg::*;
    
    // Lane memory arrays
    logic [63:0] lane_mem [LANE_COUNT-1:0][MEMORY_SIZE-1:0];
    logic [31:0] lane_iter [LANE_COUNT-1:0];
    logic [63:0] lane_seed [LANE_COUNT-1:0];
    logic lane_active [LANE_COUNT-1:0];
    logic lane_done [LANE_COUNT-1:0];
    
    // Control signals
    logic [2:0] active_lanes;
    logic [31:0] current_iter;
    logic [63:0] base_seed;
    
    // Base seed calculation
    assign base_seed = input_blocks[511:448] ^ input_blocks[447:384] ^ 
                      input_blocks[383:320] ^ input_blocks[319:256] ^
                      input_blocks[255:192] ^ input_blocks[191:128] ^
                      input_blocks[127:64] ^ input_blocks[63:0];
    
    // Generate unique seeds for each lane
    genvar i;
    generate
        for (i = 0; i < LANE_COUNT; i++) begin : lane_seeds
            assign lane_seed[i] = base_seed ^ (64'(i) * 64'h9E3779B97F4A7C15);
        end
    endgenerate
    
    // Lane initialization and processing
    generate
        for (i = 0; i < LANE_COUNT; i++) begin : lane_processors
            logic [63:0] current_seed;
            logic [9:0] mem_addr;
            logic [63:0] mem_data;
            logic mem_we;
            logic [31:0] lane_counter;
            
            // Lane state machine
            always_ff @(posedge clk or negedge rst_n) begin
                if (!rst_n) begin
                    lane_active[i] <= 1'b0;
                    lane_done[i] <= 1'b0;
                    lane_counter <= 0;
                    current_seed <= lane_seed[i];
                    mem_we <= 1'b1;
                    mem_addr <= 0;
                end else if (start && !lane_active[i]) begin
                    lane_active[i] <= 1'b1;
                    lane_done[i] <= 1'b0;
                    lane_counter <= 0;
                    current_seed <= lane_seed[i];
                    mem_we <= 1'b1;
                    mem_addr <= 0;
                end else if (lane_active[i]) begin
                    if (mem_addr < MEMORY_SIZE && mem_we) begin
                        // Initialize memory
                        mem_data <= asm_scramble(current_seed ^ 64'(mem_addr));
                        current_seed <= mem_data;
                        mem_addr <= mem_addr + 1;
                        if (mem_addr == MEMORY_SIZE - 1) begin
                            mem_we <= 1'b0;
                            mem_addr <= 0;
                        end
                    end else if (!mem_we && lane_counter < iterations) begin
                        // Memory mixing walk
                        logic [63:0] addr1, addr2, data1, data2;
                        addr1 = asm_scramble(current_seed ^ 64'(lane_counter)) % MEMORY_SIZE;
                        addr2 = asm_scramble(current_seed ^ 64'(lane_counter + 1)) % MEMORY_SIZE;
                        
                        data1 = lane_mem[i][addr1];
                        data2 = lane_mem[i][addr2];
                        
                        lane_mem[i][addr1] <= asm_scramble(data1 ^ data2);
                        lane_mem[i][addr2] <= asm_scramble(data2 ^ data1);
                        
                        current_seed <= data1 ^ data2;
                        lane_counter <= lane_counter + 1;
                        
                        if (lane_counter == iterations - 1) begin
                            lane_done[i] <= 1'b1;
                            lane_active[i] <= 1'b0;
                        end
                    end
                end
            end
            
            // Memory interface
            always_ff @(posedge clk) begin
                if (mem_we && mem_addr < MEMORY_SIZE) begin
                    lane_mem[i][mem_addr] <= mem_data;
                end
            end
        end
    endgenerate
    
    // Collect and mix results
    logic [511:0] final_blocks;
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            final_blocks <= input_blocks;
            done <= 1'b0;
            error <= 1'b0;
        end else if (start) begin
            done <= 1'b0;
            error <= 1'b0;
        end else if (&lane_done) begin
            // Mix results from all lanes
            final_blocks <= input_blocks;
            for (int j = 0; j < LANE_COUNT; j++) begin
                final_blocks[63:0] <= final_blocks[63:0] ^ lane_mem[j][0];
                final_blocks[127:64] <= final_blocks[127:64] ^ lane_mem[j][1];
                final_blocks[191:128] <= final_blocks[191:128] ^ lane_mem[j][2];
                final_blocks[255:192] <= final_blocks[255:192] ^ lane_mem[j][3];
                final_blocks[319:256] <= final_blocks[319:256] ^ lane_mem[j][4];
                final_blocks[383:320] <= final_blocks[383:320] ^ lane_mem[j][5];
                final_blocks[447:384] <= final_blocks[447:384] ^ lane_mem[j][6];
                final_blocks[511:448] <= final_blocks[511:448] ^ lane_mem[j][7];
            end
            done <= 1'b1;
        end
    end
    
    assign output_blocks = final_blocks;
    
endmodule
