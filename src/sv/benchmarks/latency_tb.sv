`include "nex2426_pkg.svh"

module latency_tb;
    
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
    
    real start_time, end_time;
    real latency_ns;
    
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
        key = 256'hLATENCY123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF;
        input_data = 512'hLATENCY_TEST_DATA_LATENCY_TEST_DATA_LATENCY_TEST_DATA_LATENCY_TEST_DATA;
        cost = 5;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Latency Benchmark ===");
        
        for (int i = 1; i <= 10; i++) begin
            cost = i;
            start_time = $realtime;
            start = 1;
            #10;
            start = 0;
            wait(done);
            end_time = $realtime;
            latency_ns = end_time - start_time;
            $display("Cost %2d: Latency = %0.1f ns", i, latency_ns);
            #50;
        end
        
        $finish;
    end
    
endmodule
