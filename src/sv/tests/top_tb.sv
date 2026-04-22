`include "nex2426_pkg.svh"

module top_tb;
    
    import nex2426_pkg::*;
    
    logic clk;
    logic rst_n;
    logic start;
    operation_mode_t mode;
    logic [255:0] key;
    logic [31:0] cost;
    logic bio_lock_enable;
    logic stealth_mode;
    logic [63:0] hardware_id;
    logic [7:0] data_in;
    logic data_valid;
    logic [7:0] data_out;
    logic data_ready;
    logic done;
    status_code_t status;
    logic [511:0] hash_output;
    logic [31:0] performance_cycles;
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    nex2426_top dut (
        .clk(clk),
        .rst_n(rst_n),
        .start(start),
        .mode(mode),
        .key(key),
        .cost(cost),
        .bio_lock_enable(bio_lock_enable),
        .stealth_mode(stealth_mode),
        .hardware_id(hardware_id),
        .data_in(data_in),
        .data_valid(data_valid),
        .data_out(data_out),
        .data_ready(data_ready),
        .done(done),
        .status(status),
        .hash_output(hash_output),
        .performance_cycles(performance_cycles)
    );
    
    initial begin
        rst_n = 0;
        start = 0;
        mode = MODE_HASH;
        key = 256'hTEST123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF;
        cost = 1;
        bio_lock_enable = 0;
        stealth_mode = 0;
        hardware_id = 64'hDEADBEEFCAFEBABE;
        data_in = 8'hAA;
        data_valid = 0;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Top Level Test ===");
        start = 1;
        #10;
        start = 0;
        wait(done);
        $display("Hash output: %h", hash_output);
        $display("Status: %d", status);
        $display("Performance cycles: %d", performance_cycles);
        
        #20;
        $finish;
    end
    
endmodule
