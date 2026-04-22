`include "nex2426_pkg.svh"

module entropy_gen #(
    parameter OUTPUT_WIDTH = 256
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    enable,
    output logic [OUTPUT_WIDTH-1:0] entropy_out,
    output logic                    valid,
    output logic                    ready
);
    import nex2426_pkg::*;
    
    logic [63:0] chaos_entropy;
    logic [63:0] timestamp_entropy;
    logic [63:0] hw_entropy;
    logic generating;
    
    chaos_engine #(.SEED_WIDTH(256)) chaos_gen (
        .clk(clk),
        .rst_n(rst_n),
        .enable(enable),
        .seed(256'h0),
        .ready(),
        .entropy_out(chaos_entropy),
        .valid()
    );
    
    hardware_id_gen hw_id_gen (
        .clk(clk),
        .rst_n(rst_n),
        .hardware_id(hw_entropy),
        .valid()
    );
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            generating <= 1'b0;
            valid <= 1'b0;
            ready <= 1'b1;
            entropy_out <= 256'h0;
        end else if (enable && ready) begin
            generating <= 1'b1;
            ready <= 1'b0;
            timestamp_entropy <= get_timestamp();
        end else if (generating) begin
            generating <= 1'b0;
            valid <= 1'b1;
            entropy_out <= {chaos_entropy, timestamp_entropy, hw_entropy, chaos_entropy ^ timestamp_entropy};
        end else if (valid) begin
            valid <= 1'b0;
            ready <= 1'b1;
        end
    end
    
endmodule
