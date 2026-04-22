`include "nex2426_pkg.svh"

module enc_module_4 (
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
    logic [31:0] round_count;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            temp_data <= 512'h0;
            data_out <= 512'h0;
            round_count <= 0;
            valid <= 1'b0;
        end else if (enable) begin
            temp_data <= data_in;
            round_count <= 0;
        end else if (round_count < 4) begin
            temp_data <= temp_data ^ {key, key} ^ {round_count, round_count, round_count, round_count, 
                                           round_count, round_count, round_count, round_count,
                                           round_count, round_count, round_count, round_count,
                                           round_count, round_count, round_count, round_count};
            round_count <= round_count + 1;
        end else if (round_count == 4) begin
            data_out <= temp_data;
            valid <= 1'b1;
            round_count <= round_count + 1;
        end else if (valid) begin
            valid <= 1'b0;
        end
    end
endmodule
