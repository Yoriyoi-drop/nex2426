`include "nex2426_pkg.svh"

module proof_of_work #(
    parameter DIFFICULTY_BITS = 20
)(
    input logic clk,
    input logic rst_n,
    input logic start,
    input logic [511:0] block_header,
    input logic [31:0] difficulty,
    output logic [63:0] nonce,
    output logic valid,
    output logic done
);
    import nex2426_pkg::*;
    
    logic [63:0] current_nonce;
    logic [511:0] hash_result;
    logic mining;
    logic [511:0] target;
    
    assign target = (512'h1 << (512 - DIFFICULTY_BITS)) - 1;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            current_nonce <= 0;
            mining <= 0;
            valid <= 0;
            done <= 0;
        end else if (start && !mining) begin
            mining <= 1;
            current_nonce <= 0;
        end else if (mining) begin
            current_nonce <= current_nonce + 1;
            if (hash_result < target) begin
                mining <= 0;
                valid <= 1;
                done <= 1;
                nonce <= current_nonce;
            end
        end else if (done) begin
            done <= 0;
        end
    end
    
    hash_core pow_hash (
        .clk(clk),
        .rst_n(rst_n),
        .start(mining),
        .mode(MODE_HASH),
        .key(256'h0),
        .input_data({block_header[447:0], current_nonce}),
        .cost(1),
        .done(),
        .hash_out(hash_result),
        .status()
    );
    
endmodule
