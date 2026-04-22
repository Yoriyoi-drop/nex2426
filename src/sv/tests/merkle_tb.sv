`include "nex2426_pkg.svh"

module merkle_tb;
    
    import nex2426_pkg::*;
    
    logic clk;
    logic rst_n;
    logic start;
    logic [511:0] leaf_data;
    logic [7:0] leaf_index;
    logic leaf_valid;
    logic ready;
    logic [511:0] root_hash;
    logic root_valid;
    logic verification_result;
    logic verification_valid;
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    merkle_tree dut (
        .clk(clk),
        .rst_n(rst_n),
        .start(start),
        .leaf_data(leaf_data),
        .leaf_index(leaf_index),
        .leaf_valid(leaf_valid),
        .ready(ready),
        .root_hash(root_hash),
        .root_valid(root_valid),
        .verification_result(verification_result),
        .verification_valid(verification_valid)
    );
    
    initial begin
        rst_n = 0;
        start = 0;
        leaf_data = 512'h0;
        leaf_index = 0;
        leaf_valid = 0;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Merkle Tree Test ===");
        
        // Add some leaves
        for (int i = 0; i < 4; i++) begin
            leaf_data = 512'hMERKLE_TEST_DATA_0 + i;
            leaf_index = i;
            leaf_valid = 1;
            #10;
            leaf_valid = 0;
            wait(ready);
            #10;
        end
        
        #100;
        if (root_valid) begin
            $display("Root hash: %h", root_hash);
        end
        
        #20;
        $finish;
    end
    
endmodule
