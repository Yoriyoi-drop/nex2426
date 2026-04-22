`include "nex2426_pkg.svh"

module ctr_mode (
    input logic clk,
    input logic rst_n,
    input logic enable,
    input logic [255:0] key,
    input logic [63:0] nonce,
    input logic [511:0] plaintext,
    output logic [511:0] ciphertext,
    output logic valid
);
    import nex2426_pkg::*;
    
    logic [63:0] counter;
    logic [255:0] ctr_key;
    logic [511:0] keystream;
    logic processing;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            counter <= 0;
            processing <= 0;
            valid <= 0;
        end else if (enable && !processing) begin
            processing <= 1;
            counter <= nonce;
            // Generate CTR key
            ctr_key <= key ^ {nonce, nonce, nonce, nonce};
        end else if (processing) begin
            counter <= counter + 1;
            processing <= 0;
            valid <= 1;
            ciphertext <= plaintext ^ keystream;
        end else if (valid) begin
            valid <= 0;
        end
    end
    
    // Generate keystream
    hash_core ctr_hash (
        .clk(clk),
        .rst_n(rst_n),
        .start(processing),
        .mode(MODE_ENCRYPT),
        .key(ctr_key),
        .input_data({counter, counter, counter, counter, counter, counter, counter, counter}),
        .cost(1),
        .done(),
        .hash_out(keystream),
        .status()
    );
    
endmodule
