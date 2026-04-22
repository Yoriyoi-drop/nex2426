`include "nex2426_pkg.svh"

module enc_module_3 (
    input logic clk,
    input logic rst_n,
    input logic enable,
    input logic [255:0] key,
    input logic [511:0] data_in,
    output logic [511:0] data_out,
    output logic valid
);
    import nex2426_pkg::*;
    
    logic [511:0] mixed_data;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            mixed_data <= 512'h0;
            data_out <= 512'h0;
            valid <= 1'b0;
        end else if (enable) begin
            mixed_data <= data_in ^ {key, key} ^ {key[127:0], key[255:128], key[63:0], key[191:128]};
            data_out <= mixed_data;
            valid <= 1'b1;
        end else begin
            valid <= 1'b0;
        end
    end
endmodule
