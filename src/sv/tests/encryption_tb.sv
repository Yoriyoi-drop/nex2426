`include "nex2426_pkg.svh"

module encryption_tb;
    
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
    
    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end
    
    encryption_engine dut (
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
        .status(status)
    );
    
    initial begin
        rst_n = 0;
        start = 0;
        mode = MODE_ENCRYPT;
        key = 256'h123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0;
        cost = 1;
        bio_lock_enable = 0;
        stealth_mode = 0;
        hardware_id = 64'hDEADBEEFCAFEBABE;
        data_in = 8'hAA;
        data_valid = 0;
        #20;
        rst_n = 1;
        #10;
        
        $display("=== Encryption Test ===");
        start = 1;
        #10;
        start = 0;
        data_valid = 1;
        wait(data_ready);
        $display("Encrypted data: %h", data_out);
        data_valid = 0;
        wait(done);
        $display("Status: %d", status);
        
        #20;
        $finish;
    end
    
endmodule
