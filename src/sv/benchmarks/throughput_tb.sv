`include "nex2426_pkg.svh"

module throughput_tb;
    
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
    int hash_count;
    real throughput;
    
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
        key = 256'h123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        input_data = 512'hCAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABE;
        cost = 1;
        hash_count = 0;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Throughput Benchmark ===");
        start_time = $realtime;
        
        for (int i = 0; i < 1000; i++) begin
            input_data = input_data + 1;
            start = 1;
            #10;
            start = 0;
            wait(done);
            hash_count++;
            #10;
        end
        
        end_time = $realtime;
        throughput = real'(hash_count) / ((end_time - start_time) / 1000.0) * 1000.0;
        
        $display("Processed %d hashes in %0.3f ms", hash_count, (end_time - start_time) / 1000.0);
        $display("Throughput: %0.2f hashes/sec", throughput);
        
        $finish;
    end
    
endmodule
