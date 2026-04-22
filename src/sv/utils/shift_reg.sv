`include "nex2426_pkg.svh"

module shift_reg #(
    parameter DATA_WIDTH = 64,
    parameter LENGTH = 8
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    enable,
    input  logic [DATA_WIDTH-1:0]   data_in,
    output logic [DATA_WIDTH-1:0]   data_out,
    output logic                    valid
);
    
    logic [DATA_WIDTH-1:0] reg_array [LENGTH-1:0];
    logic [3:0] shift_count;
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            for (int i = 0; i < LENGTH; i++) begin
                reg_array[i] <= 0;
            end
            shift_count <= 0;
            valid <= 1'b0;
        end else if (enable) begin
            reg_array[0] <= data_in;
            for (int i = 1; i < LENGTH; i++) begin
                reg_array[i] <= reg_array[i-1];
            end
            shift_count <= shift_count + 1;
            if (shift_count == LENGTH-1) begin
                valid <= 1'b1;
            end
        end else if (valid) begin
            valid <= 1'b0;
        end
    end
    
    assign data_out = reg_array[LENGTH-1];
    
endmodule
