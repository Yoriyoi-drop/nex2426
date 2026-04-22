`include "nex2426_pkg.svh"

module axi_tb;
    
    import nex2426_pkg::*;
    
    logic aclk;
    logic aresetn;
    
    // AXI Lite signals
    logic awvalid, awready;
    logic [31:0] awaddr;
    logic [2:0] awprot;
    logic wvalid, wready;
    logic [63:0] wdata;
    logic [7:0] wstrb;
    logic bvalid, bready;
    logic [1:0] bresp;
    logic arvalid, arready;
    logic [31:0] araddr;
    logic [2:0] arprot;
    logic rvalid, rready;
    logic [63:0] rdata;
    logic [1:0] rresp;
    
    // Core signals
    logic core_start;
    operation_mode_t core_mode;
    logic [255:0] core_key;
    logic [31:0] core_cost;
    logic core_bio_lock;
    logic core_stealth;
    logic [63:0] core_hw_id;
    logic [7:0] core_data_in;
    logic core_data_valid;
    logic [7:0] core_data_out;
    logic core_data_ready;
    logic core_done;
    status_code_t core_status;
    logic [511:0] core_hash_out;
    
    initial begin
        aclk = 0;
        forever #5 aclk = ~aclk;
    end
    
    nex2426_axi dut (
        .aclk(aclk),
        .aresetn(aresetn),
        .awvalid(awvalid),
        .awready(awready),
        .awaddr(awaddr),
        .awprot(awprot),
        .wvalid(wvalid),
        .wready(wready),
        .wdata(wdata),
        .wstrb(wstrb),
        .bvalid(bvalid),
        .bready(bready),
        .bresp(bresp),
        .arvalid(arvalid),
        .arready(arready),
        .araddr(araddr),
        .arprot(arprot),
        .rvalid(rvalid),
        .rready(rready),
        .rdata(rdata),
        .rresp(rresp),
        .core_start(core_start),
        .core_mode(core_mode),
        .core_key(core_key),
        .core_cost(core_cost),
        .core_bio_lock(core_bio_lock),
        .core_stealth(core_stealth),
        .core_hw_id(core_hw_id),
        .core_data_in(core_data_in),
        .core_data_valid(core_data_valid),
        .core_data_out(core_data_out),
        .core_data_ready(core_data_ready),
        .core_done(core_done),
        .core_status(core_status),
        .core_hash_out(core_hash_out)
    );
    
    initial begin
        aresetn = 0;
        awvalid = 0;
        wvalid = 0;
        arvalid = 0;
        bready = 1;
        rready = 1;
        #20;
        aresetn = 1;
        #10;
        
        $display("=== AXI Interface Test ===");
        
        // Write control register
        awaddr = 12'h000;
        awvalid = 1;
        wait(awready);
        #10;
        awvalid = 0;
        
        wdata = 64'h00000001;
        wstrb = 8'hFF;
        wvalid = 1;
        wait(wready);
        #10;
        wvalid = 0;
        
        wait(bvalid);
        #10;
        bready = 0;
        #10;
        bready = 1;
        
        // Read status register
        araddr = 12'h004;
        arvalid = 1;
        wait(arready);
        #10;
        arvalid = 0;
        
        wait(rvalid);
        $display("Status: %h", rdata);
        #10;
        rready = 0;
        #10;
        rready = 1;
        
        #20;
        $finish;
    end
    
endmodule
