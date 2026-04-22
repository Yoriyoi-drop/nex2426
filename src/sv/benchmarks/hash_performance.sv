//==============================================================================
// NEX2426 Hash Performance Benchmark
// Performance measurement and analysis of hash core throughput
//==============================================================================

`include "nex2426_pkg.svh"

module hash_performance_bench;
    
    import nex2426_pkg::*;
    
    // Benchmark signals
    logic                    clk;
    logic                    rst_n;
    logic                    start;
    operation_mode_t         mode;
    logic [255:0]            key;
    logic [511:0]            input_data;
    logic [31:0]             cost;
    logic                    done;
    logic [511:0]            hash_out;
    status_code_t           status;
    
    // Performance counters
    logic [31:0]             cycle_count;
    logic [31:0]             start_cycle;
    logic [31:0]             end_cycle;
    logic [31:0]             total_hashes;
    real                     throughput_mhz;
    real                     avg_latency_ns;
    
    // Clock generation (100MHz)
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    // DUT instantiation
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
    
    // Cycle counter
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            cycle_count <= 0;
        end else begin
            cycle_count <= cycle_count + 1;
        end
    end
    
    // Performance measurement
    always_ff @(posedge clk) begin
        if (start && !done) begin
            start_cycle <= cycle_count;
        end else if (done && !start) begin
            end_cycle <= cycle_count;
            total_hashes <= total_hashes + 1;
            
            // Calculate metrics
            logic [31:0] latency_cycles = end_cycle - start_cycle;
            avg_latency_ns = real'(latency_cycles) * 10.0; // 100MHz = 10ns per cycle
            throughput_mhz = 100.0 / real'(latency_cycles); // Hashes per second at 100MHz
        end
    end
    
    // Benchmark stimulus
    initial begin
        // Initialize
        rst_n = 0;
        start = 0;
        mode = MODE_HASH;
        key = 256'hDEADBEEFCAFEBABE123456789ABCDEF0DEADBEEFCAFEBABE123456789ABCDEF0;
        input_data = 512'h0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        cost = 1;
        total_hashes = 0;
        #20;
        rst_n = 1;
        #10;
        
        // Benchmark 1: Different costs
        $display("=== Benchmark 1: Performance vs Cost ===");
        for (int c = 1; c <= 10; c++) begin
            cost = c;
            input_data = input_data + 1; // Change input data
            start = 1;
            wait(done);
            #10;
            start = 0;
            wait(!done);
            $display("Cost %2d: Latency = %0.1f ns, Throughput = %0.2f MHz", 
                     c, avg_latency_ns, throughput_mhz);
            #50;
        end
        
        // Benchmark 2: Throughput test
        $display("=== Benchmark 2: Sustained Throughput Test ===");
        real start_time, end_time;
        int sustained_hashes = 0;
        
        start_time = $realtime;
        for (int i = 0; i < 1000; i++) begin
            input_data = input_data + i;
            start = 1;
            wait(done);
            #10;
            start = 0;
            wait(!done);
            sustained_hashes++;
        end
        end_time = $realtime;
        
        real total_time = (end_time - start_time) / 1000.0; // Convert to ms
        real sustained_throughput = real'(sustained_hashes) / total_time * 1000.0; // Hashes per second
        
        $display("Sustained throughput: %0.2f hashes/sec over %d hashes", 
                 sustained_throughput, sustained_hashes);
        
        // Benchmark 3: Key variation test
        $display("=== Benchmark 3: Key Variation Impact ===");
        cost = 5; // Fixed cost
        
        for (int k = 0; k < 10; k++) begin
            key = key + 256'h1111111111111111111111111111111111111111111111111111111111111111;
            input_data = 512'hCAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABECAFEBABE;
            start = 1;
            wait(done);
            #10;
            start = 0;
            wait(!done);
            $display("Key %2d: Latency = %0.1f ns", k + 1, avg_latency_ns);
            #50;
        end
        
        // Benchmark 4: Data pattern test
        $display("=== Benchmark 4: Data Pattern Impact ===");
        key = 256'h123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        cost = 3;
        
        logic [511:0] test_patterns [8];
        test_patterns[0] = 512'h0000000000000000000000000000000000000000000000000000000000000000;
        test_patterns[1] = 512'hFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
        test_patterns[2] = 512'hAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA;
        test_patterns[3] = 512'h5555555555555555555555555555555555555555555555555555555555555555555;
        test_patterns[4] = 512'h0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        test_patterns[5] = 512'hFEDCBA9876543210FEDCBA9876543210FEDCBA9876543210FEDCBA9876543210;
        test_patterns[6] = 512'hDEADBEEFCAFEBABEDEADBEEFCAFEBABEDEADBEEFCAFEBABEDEADBEEFCAFEBABE;
        test_patterns[7] = 512'h123456789ABCDEF123456789ABCDEF123456789ABCDEF123456789ABCDEF1234;
        
        for (int p = 0; p < 8; p++) begin
            input_data = test_patterns[p];
            start = 1;
            wait(done);
            #10;
            start = 0;
            wait(!done);
            $display("Pattern %2d: Latency = %0.1f ns", p + 1, avg_latency_ns);
            #50;
        end
        
        // Summary
        $display("=== Performance Summary ===");
        $display("Total hashes processed: %d", total_hashes);
        $display("Average latency: %0.1f ns", avg_latency_ns);
        $display("Peak throughput: %0.2f MHz", throughput_mhz);
        $display("Sustained throughput: %0.2f hashes/sec", sustained_throughput);
        
        $display("=== Hash Performance Benchmark Complete ===");
        $finish;
    end
    
    // Assertions for performance validation
    property max_latency;
        @(posedge clk) disable iff (!rst_n)
        start |-> ##[1000:10000] done; // Should complete within 10,000 cycles (100us at 100MHz)
    endproperty
    
    property min_throughput;
        @(posedge clk) disable iff (!rst_n)
        done |-> throughput_mhz >= 0.01; // Minimum 10 KHz throughput
    endproperty
    
    assert property (max_latency) else $error("Hash operation took too long");
    assert property (min_throughput) else $error("Throughput too low");
    
    // Performance coverage
    covergroup perf_cg @(posedge clk);
        cp_cost: coverpoint cost {
            bins low = {[1:3]};
            bins medium = {[4:7]};
            bins high = {[8:10]};
        }
        cp_latency: coverpoint avg_latency_ns {
            bins fast = {[0:1000]};
            bins medium = {[1001:5000]};
            bins slow = {[5001:$]};
        }
        cp_throughput: coverpoint throughput_mhz {
            bins high = {[10:$]};
            bins medium = {[1:9.99]};
            bins low = {[0.01:0.99]};
        }
    endgroup
    
    perf_cg perf_cov = new();
    
endmodule
