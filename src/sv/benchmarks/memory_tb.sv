`include "nex2426_pkg.svh"

module memory_tb;
    
    import nex2426_pkg::*;
    
    logic clk;
    logic rst_n;
    logic start;
    logic [511:0] input_blocks;
    logic [31:0] iterations;
    logic done;
    logic [511:0] output_blocks;
    logic error;
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    memory_hardening dut (
        .clk(clk),
        .rst_n(rst_n),
        .start(start),
        .input_blocks(input_blocks),
        .iterations(iterations),
        .output_blocks(output_blocks),
        .done(done),
        .error(error)
    );
    
    initial begin
        rst_n = 0;
        start = 0;
        input_blocks = 512'hCAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABE;
        iterations = 10;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Memory Hardening Test ===");
        start = 1;
        #10;
        start = 0;
        wait(done);
        $display("Output: %h", output_blocks);
        $display("Error: %d", error);
        
        #20;
        $finish;
    end
    
endmodule
