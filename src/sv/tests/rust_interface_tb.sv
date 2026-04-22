`include "nex2426_pkg.svh"

module rust_interface_tb;
    
    import nex2426_pkg::*;
    
    // Clock and reset
    logic clk;
    logic rst_n;
    
    // Rust interface signals
    logic rust_clk;
    logic rust_rst_n;
    logic rust_enable;
    logic [31:0] rust_addr;
    logic rust_write;
    logic [63:0] rust_wdata;
    logic [63:0] rust_rdata;
    logic rust_ready;
    logic rust_valid;
    
    // Debug outputs
    logic [31:0] debug_status;
    logic debug_error;
    logic [63:0] performance_counters;
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    initial begin
        rust_clk = 0;
        forever #10 rust_clk = ~rust_clk;
    end
    
    nex2426_rust_top dut (
        .clk(clk),
        .rst_n(rst_n),
        .rust_clk(rust_clk),
        .rust_rst_n(rust_rst_n),
        .rust_enable(rust_enable),
        .rust_addr(rust_addr),
        .rust_write(rust_write),
        .rust_wdata(rust_wdata),
        .rust_rdata(rust_rdata),
        .rust_ready(rust_ready),
        .rust_valid(rust_valid),
        .debug_status(debug_status),
        .debug_error(debug_error),
        .performance_counters(performance_counters)
    );
    
    // Task to write to register
    task write_register;
        input [7:0] addr;
        input [63:0] data;
    begin
        rust_addr = {24'b0, addr};
        rust_wdata = data;
        rust_write = 1;
        rust_enable = 1;
        wait(rust_ready);
        #10;
        rust_enable = 0;
        wait(rust_valid);
        #10;
        rust_write = 0;
    end
    endtask
    
    // Task to read from register
    task read_register;
        input [7:0] addr;
        output [63:0] data;
    begin
        rust_addr = {24'b0, addr};
        rust_write = 0;
        rust_enable = 1;
        wait(rust_ready);
        #10;
        rust_enable = 0;
        wait(rust_valid);
        data = rust_rdata;
        #10;
    end
    endtask
    
    initial begin
        rst_n = 0;
        rust_rst_n = 0;
        rust_enable = 0;
        rust_write = 0;
        rust_addr = 0;
        rust_wdata = 0;
        #20;
        rst_n = 1;
        rust_rst_n = 1;
        #10;
        
        $display("=== Rust Interface Test ===");
        
        // Test 1: Read version register
        $display("Test 1: Reading version register...");
        read_register(8'hFF, rust_rdata);
        $display("Version: %h", rust_rdata);
        
        // Test 2: Configure for hash operation
        $display("Test 2: Configuring for hash operation...");
        write_register(8'h02, MODE_HASH); // Set mode to hash
        write_register(8'h20, 32'h00000005); // Set cost to 5
        
        // Write key
        write_register(8'h10, 64'h123456789ABCDEF0); // Key part 0
        write_register(8'h11, 64'h123456789ABCDEF0); // Key part 1
        write_register(8'h12, 64'h123456789ABCDEF0); // Key part 2
        write_register(8'h13, 64'h123456789ABCDEF0); // Key part 3
        
        // Test 3: Start hash operation
        $display("Test 3: Starting hash operation...");
        write_register(8'h00, 64'h00000001); // Start bit set
        
        // Wait for completion
        $display("Waiting for operation completion...");
        fork
            begin
                forever begin
                    read_register(8'h01, rust_rdata); // Read status
                    if (rust_rdata & 64'h00000001) begin // Done bit
                        $display("Operation completed!");
                        disable wait_done;
                    end
                    #100;
                end
            end
            begin : wait_done
                #10000; // Timeout
                $display("Operation timeout!");
            end
        join_any
        disable fork;
        
        // Test 4: Read results
        $display("Test 4: Reading hash results...");
        read_register(8'h40, rust_rdata); // Hash part 0
        $display("Hash[0]: %h", rust_rdata);
        read_register(8'h41, rust_rdata); // Hash part 1
        $display("Hash[1]: %h", rust_rdata);
        read_register(8'h42, rust_rdata); // Hash part 2
        $display("Hash[2]: %h", rust_rdata);
        read_register(8'h43, rust_rdata); // Hash part 3
        $display("Hash[3]: %h", rust_rdata);
        
        // Test 5: Read performance counters
        $display("Test 5: Reading performance counters...");
        $display("Performance counters: %h", performance_counters);
        $display("Debug status: %h", debug_status);
        $display("Debug error: %d", debug_error);
        
        // Test 6: Test encryption mode
        $display("Test 6: Testing encryption mode...");
        write_register(8'h02, MODE_ENCRYPT); // Set mode to encrypt
        write_register(8'h30, 64'h00000000000000AA); // Input data
        write_register(8'h00, 64'h00000001); // Start operation
        
        // Wait for completion
        #1000;
        read_register(8'h31, rust_rdata); // Read encrypted data
        $display("Encrypted data: %h", rust_rdata);
        
        $display("\n=== Rust Interface Test Completed ===");
        $finish;
    end
    
endmodule
