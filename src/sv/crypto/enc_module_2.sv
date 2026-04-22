`include "nex2426_pkg.svh"

module enc_module_2 (
    input logic clk,
    input logic rst_n,
    input logic enable,
    input logic [255:0] key,
    input logic [511:0] data_in,
    output logic [511:0] data_out,
    output logic valid
);
    import nex2426_pkg::*;
    
    logic [511:0] temp_data;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            temp_data <= 512'h0;
            data_out <= 512'h0;
            valid <= 1'b0;
        end else if (enable) begin
            temp_data <= data_in ^ {key, key};
            data_out <= asm_scramble(temp_data[63:0]) | (asm_scramble(temp_data[127:64]) << 64) |
                       (asm_scramble(temp_data[191:128]) << 128) | (asm_scramble(temp_data[255:192]) << 192) |
                       (asm_scramble(temp_data[319:256]) << 256) | (asm_scramble(temp_data[383:320]) << 320) |
                       (asm_scramble(temp_data[447:384]) << 384) | (asm_scramble(temp_data[511:448]) << 448);
            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end
endmodule
