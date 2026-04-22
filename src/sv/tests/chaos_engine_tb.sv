//==============================================================================
// NEX2426 Chaos Engine Testbench
// Test and verification of Lorenz attractor based PRNG
//==============================================================================

`include "nex2426_pkg.svh"

module chaos_engine_tb;
    
    import nex2426_pkg::*;
    
    // Testbench signals
    logic                    clk;
    logic                    rst_n;
    logic                    enable;
    logic [255:0]            seed;
    logic                    ready;
    logic [63:0]             entropy_out;
    logic                    valid_out;
    
    // Clock generation
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    // DUT instantiation
    chaos_engine #(
        .SEED_WIDTH(256),
        .STATE_WIDTH(64)
    ) dut (
        .clk(clk),
        .rst_n(rst_n),
        .enable(enable),
        .seed(seed),
        .ready(ready),
        .entropy_out(entropy_out),
        .valid_out(valid_out)
    );
    
    // Test stimulus
    initial begin
        // Reset sequence
        rst_n = 0;
        enable = 0;
        seed = 256'h0;
        #20;
        rst_n = 1;
        #10;
        
        // Test 1: Basic functionality
        $display("=== Test 1: Basic Functionality ===");
        seed = 256'hDEADBEEFCAFEBABE123456789ABCDEF0DEADBEEFCAFEBABE123456789ABCDEF0;
        enable = 1;
        wait(ready);
        #10;
        enable = 0;
        wait(valid_out);
        $display("Entropy output: %h", entropy_out);
        #20;
        
        // Test 2: Different seeds produce different outputs
        $display("=== Test 2: Seed Uniqueness ===");
        logic [63:0] entropy1, entropy2, entropy3;
        
        // Seed 1
        seed = 256'h1111111111111111222222222222222333333333333333344444444444444444;
        enable = 1;
        wait(ready);
        #10;
        enable = 0;
        wait(valid_out);
        entropy1 = entropy_out;
        $display("Seed 1 entropy: %h", entropy1);
        #20;
        
        // Seed 2
        seed = 256'hAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA;
        enable = 1;
        wait(ready);
        #10;
        enable = 0;
        wait(valid_out);
        entropy2 = entropy_out;
        $display("Seed 2 entropy: %h", entropy2);
        #20;
        
        // Seed 3
        seed = 256'h5555555555555555555555555555555555555555555555555555555555555555555;
        enable = 1;
        wait(ready);
        #10;
        enable = 0;
        wait(valid_out);
        entropy3 = entropy_out;
        $display("Seed 3 entropy: %h", entropy3);
        #20;
        
        // Verify uniqueness
        if (entropy1 != entropy2 && entropy1 != entropy3 && entropy2 != entropy3) begin
            $display("PASS: All entropy values are unique");
        end else begin
            $display("FAIL: Entropy values are not unique");
        end
        
        // Test 3: Statistical randomness test
        $display("=== Test 3: Statistical Randomness Test ===");
        logic [63:0] entropy_samples [100];
        logic [31:0] bit_counts [64];
        
        // Initialize bit counts
        for (int i = 0; i < 64; i++) begin
            bit_counts[i] = 0;
        end
        
        // Generate 100 samples
        for (int i = 0; i < 100; i++) begin
            seed = 256'h123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0 + i;
            enable = 1;
            wait(ready);
            #10;
            enable = 0;
            wait(valid_out);
            entropy_samples[i] = entropy_out;
            
            // Count bits
            for (int bit = 0; bit < 64; bit++) begin
                if (entropy_out[bit]) begin
                    bit_counts[bit]++;
                end
            end
            #20;
        end
        
        // Analyze bit distribution
        logic [31:0] min_count, max_count;
        min_count = 32'hFFFFFFFF;
        max_count = 0;
        
        for (int i = 0; i < 64; i++) begin
            if (bit_counts[i] < min_count) min_count = bit_counts[i];
            if (bit_counts[i] > max_count) max_count = bit_counts[i];
        end
        
        $display("Bit count range: %d to %d", min_count, max_count);
        $display("Expected range: 25 to 75 (for 100 samples)");
        
        if (min_count >= 25 && max_count <= 75) begin
            $display("PASS: Bit distribution is reasonably uniform");
        end else begin
            $display("FAIL: Bit distribution is not uniform enough");
        end
        
        // Test 4: Consecutive values are different
        $display("=== Test 4: Consecutive Value Test ===");
        logic [63:0] prev_entropy;
        int consecutive_same = 0;
        
        for (int i = 0; i < 50; i++) begin
            seed = 256'h9999999999999999999999999999999999999999999999999999999999999999 + i;
            enable = 1;
            wait(ready);
            #10;
            enable = 0;
            wait(valid_out);
            
            if (i > 0 && entropy_out == prev_entropy) begin
                consecutive_same++;
            end
            prev_entropy = entropy_out;
            #20;
        end
        
        $display("Consecutive identical values: %d", consecutive_same);
        
        if (consecutive_same == 0) begin
            $display("PASS: No consecutive identical values");
        end else begin
            $display("FAIL: Found %d consecutive identical values", consecutive_same);
        end
        
        $display("=== Chaos Engine Test Complete ===");
        $finish;
    end
    
    // Assertions
    property ready_after_reset;
        @(posedge clk) disable iff (!rst_n)
        ##[5] ready;
    endproperty
    
    property valid_after_enable;
        @(posedge clk) disable iff (!rst_n)
        enable && ready |-> ##[10:20] valid_out;
    endproperty
    
    assert property (ready_after_reset) else $error("Ready signal not asserted after reset");
    assert property (valid_after_enable) else $error("Valid signal not asserted after enable");
    
    // Coverage
    covergroup cg @(posedge clk);
        cp_enable: coverpoint enable;
        cp_ready: coverpoint ready;
        cp_valid: coverpoint valid_out;
        cp_entropy: coverpoint entropy_out {
            bins low = {[0:16'hFFFF]};
            bins mid = {[16'h1000:16'hF000]};
            bins high = {[16'hF001:16'hFFFF]};
        }
    endgroup
    
    cg cov = new();
    
endmodule
