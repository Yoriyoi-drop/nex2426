`include "nex2426_pkg.svh"

module clock_divider #(
    parameter DIVIDE_BY = 4
)(
    input  logic clk_in,
    input  rst_n,
    output logic clk_out
);
    
    logic [$clog2(DIVIDE_BY)-1:0] counter;
    
    always_ff @(posedge clk_in or negedge rst_n) begin
        if (!rst_n) begin
            counter <= 0;
            clk_out <= 0;
        end else begin
            counter <= counter + 1;
            if (counter == DIVIDE_BY/2 - 1) begin
                clk_out <= ~clk_out;
                counter <= 0;
            end
        end
    end
    
endmodule
