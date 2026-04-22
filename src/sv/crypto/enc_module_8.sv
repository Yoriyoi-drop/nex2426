`include "nex2426_pkg.svh"

module enc_module_8 (
    input logic clk,
    input logic rst_n,
    input logic enable,
    input logic [255:0] key,
    input logic [511:0] data_in,
    output logic [511:0] data_out,
    output logic valid
);
    import nex2426_pkg::*;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            data_out <= 512'h0;
            valid <= 1'b0;
        end else if (enable) begin
            data_out <= data_in ^ {key, key} ^ 512'h8;
            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end
endmodule
