`include "nex2426_pkg.svh"

module kdf #(
    parameter ITERATIONS = 1000,
    parameter OUTPUT_SIZE = 256
)(
    input logic clk,
    input logic rst_n,
    input logic start,
    input logic [255:0] password,
    input logic [255:0] salt,
    output logic [OUTPUT_SIZE-1:0] derived_key,
    output logic done
);
    import nex2426_pkg::*;
    
    logic [31:0] iter_count;
    logic [255:0] current_hash;
    logic [511:0] hash_input;
    logic processing;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            iter_count <= 0;
            current_hash <= 256'h0;
            processing <= 0;
            done <= 0;
        end else if (start && !processing) begin
            processing <= 1;
            iter_count <= 0;
            current_hash <= password ^ salt;
        end else if (processing) begin
            if (iter_count < ITERATIONS) begin
                iter_count <= iter_count + 1;
                hash_input <= {current_hash, salt};
            end else begin
                processing <= 0;
                done <= 1;
                derived_key <= current_hash;
            end
        end else if (done) begin
            done <= 0;
        end
    end
    
    hash_core kdf_hash (
        .clk(clk),
        .rst_n(rst_n),
        .start(processing && iter_count < ITERATIONS),
        .mode(MODE_HASH),
        .key(256'h0),
        .input_data(hash_input),
        .cost(1),
        .done(),
        .hash_out(current_hash),
        .status()
    );
    
endmodule
