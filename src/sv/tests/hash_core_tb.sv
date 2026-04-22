`include "nex2426_pkg.svh"

module hash_core_tb;
    
    import nex2426_pkg::*;
    
    logic clk;
    logic rst_n;
    logic start;
    operation_mode_t mode;
    logic [255:0] key;
    logic [511:0] input_data;
    logic [31:0] cost;
    logic done;
    logic [511:0] hash_out;
    status_code_t status;
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    hash_core dut (
        .clk(clk),
        .rst_n(rst_n),
        .start(start),
        .mode(mode),
        .key(key),
        .input_data(input_data),
        .cost(cost),
        .done(done),
        .hash_out(hash_out),
        .status(status)
    );
    
    initial begin
        rst_n = 0;
        start = 0;
        mode = MODE_HASH;
        key = 256'hDEADBEEFCAFEBABE123456789ABCDEF0DEADBEEFCAFEBABE123456789ABCDEF0;
        input_data = 512'h0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        cost = 1;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Hash Core Test ===");
        start = 1;
        wait(done);
        $display("Hash output: %h", hash_out);
        $display("Status: %d", status);
        
        #20;
        $finish;
    end
    
endmodule
