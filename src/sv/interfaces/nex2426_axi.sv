//==============================================================================
// NEX2426 AXI Interface - Hardware Implementation
// AXI4-Lite slave interface for NEX2426 encryption engine
//==============================================================================

`include "nex2426_pkg.svh"

module nex2426_axi #(
    parameter ADDR_WIDTH = 32,
    parameter DATA_WIDTH = 64
)(
    // Clock and reset
    input  logic                    aclk,
    input  logic                    aresetn,
    
    // AXI Lite Slave Interface - Write Address
    input  logic                    awvalid,
    output logic                    awready,
    input  logic [ADDR_WIDTH-1:0]   awaddr,
    input  logic [2:0]              awprot,
    
    // AXI Lite Slave Interface - Write Data
    input  logic                    wvalid,
    output logic                    wready,
    input  logic [DATA_WIDTH-1:0]   wdata,
    input  logic [(DATA_WIDTH/8)-1:0] wstrb,
    
    // AXI Lite Slave Interface - Write Response
    output logic                    bvalid,
    input  logic                    bready,
    output logic [1:0]              bresp,
    
    // AXI Lite Slave Interface - Read Address
    input  logic                    arvalid,
    output logic                    arready,
    input  logic [ADDR_WIDTH-1:0]   araddr,
    input  logic [2:0]              arprot,
    
    // AXI Lite Slave Interface - Read Data
    output logic                    rvalid,
    input  logic                    rready,
    output logic [DATA_WIDTH-1:0]   rdata,
    output logic [1:0]              rresp,
    
    // NEX2426 Core Interface
    output logic                    core_start,
    output operation_mode_t         core_mode,
    output logic [255:0]            core_key,
    output logic [31:0]             core_cost,
    output logic                    core_bio_lock,
    output logic                    core_stealth,
    output logic [63:0]             core_hw_id,
    output logic [7:0]              core_data_in,
    output logic                    core_data_valid,
    input  logic [7:0]              core_data_out,
    input  logic                    core_data_ready,
    input  logic                    core_done,
    input  status_code_t           core_status,
    input  logic [511:0]            core_hash_out
);
    
    import nex2426_pkg::*;
    
    // Register map
    localparam REG_CONTROL       = 12'h000;
    localparam REG_STATUS        = 12'h004;
    localparam REG_MODE          = 12'h008;
    localparam REG_KEY0          = 12'h010;
    localparam REG_KEY1          = 12'h014;
    localparam REG_KEY2          = 12'h018;
    localparam REG_KEY3          = 12'h01C;
    localparam REG_COST          = 12'h020;
    localparam REG_CONFIG        = 12'h024;
    localparam REG_HW_ID         = 12'h028;
    localparam REG_DATA_IN       = 12'h030;
    localparam REG_DATA_OUT      = 12'h034;
    localparam REG_HASH0         = 12'h040;
    localparam REG_HASH1         = 12'h044;
    localparam REG_HASH2         = 12'h048;
    localparam REG_HASH3         = 12'h04C;
    localparam REG_HASH4         = 12'h050;
    localparam REG_HASH5         = 12'h054;
    localparam REG_HASH6         = 12'h058;
    localparam REG_HASH7         = 12'h05C;
    
    // Internal registers
    logic [31:0] reg_control;
    logic [31:0] reg_status;
    logic [31:0] reg_mode;
    logic [255:0] reg_key;
    logic [31:0] reg_cost;
    logic [31:0] reg_config;
    logic [63:0] reg_hw_id;
    logic [31:0] reg_data_in;
    logic [31:0] reg_data_out;
    logic [511:0] reg_hash;
    
    // AXI state machine
    typedef enum logic [2:0] {
        AXI_IDLE,
        AXI_WRITE_ADDR,
        AXI_WRITE_DATA,
        AXI_WRITE_RESP,
        AXI_READ_ADDR,
        AXI_READ_DATA
    } axi_state_t;
    
    axi_state_t axi_state;
    logic [ADDR_WIDTH-1:0] write_addr;
    logic [ADDR_WIDTH-1:0] read_addr;
    logic write_pending;
    logic read_pending;
    
    // Control register bits
    localparam CTRL_START = 0;
    localparam CTRL_RESET = 1;
    
    // Config register bits
    localparam CFG_BIO_LOCK = 0;
    localparam CFG_STEALTH = 1;
    
    // AXI Write Interface
    assign awready = (axi_state == AXI_IDLE);
    assign wready = (axi_state == AXI_WRITE_DATA);
    
    // AXI Read Interface
    assign arready = (axi_state == AXI_IDLE);
    assign rvalid = (axi_state == AXI_READ_DATA);
    
    // AXI Response
    assign bresp = 2'b00; // OKAY response
    assign rresp = 2'b00; // OKAY response
    
    // Main AXI state machine
    always_ff @(posedge aclk or negedge aresetn) begin
        if (!aresetn) begin
            axi_state <= AXI_IDLE;
            write_pending <= 1'b0;
            read_pending <= 1'b0;
            bvalid <= 1'b0;
            reg_control <= 32'b0;
            reg_status <= 32'b0;
            reg_mode <= 32'b0;
            reg_key <= 256'b0;
            reg_cost <= 32'b0;
            reg_config <= 32'b0;
            reg_hw_id <= 64'b0;
            reg_data_in <= 32'b0;
            reg_data_out <= 32'b0;
            reg_hash <= 512'b0;
        end else begin
            case (axi_state)
                AXI_IDLE: begin
                    if (awvalid && !write_pending) begin
                        axi_state <= AXI_WRITE_ADDR;
                        write_addr <= awaddr;
                    end else if (arvalid && !read_pending) begin
                        axi_state <= AXI_READ_ADDR;
                        read_addr <= araddr;
                    end
                end
                
                AXI_WRITE_ADDR: begin
                    if (wvalid) begin
                        axi_state <= AXI_WRITE_DATA;
                        write_register(write_addr, wdata, wstrb);
                    end
                end
                
                AXI_WRITE_DATA: begin
                    axi_state <= AXI_WRITE_RESP;
                    bvalid <= 1'b1;
                end
                
                AXI_WRITE_RESP: begin
                    if (bready) begin
                        axi_state <= AXI_IDLE;
                        bvalid <= 1'b0;
                    end
                end
                
                AXI_READ_ADDR: begin
                    axi_state <= AXI_READ_DATA;
                    rdata <= read_register(read_addr);
                end
                
                AXI_READ_DATA: begin
                    if (rready) begin
                        axi_state <= AXI_IDLE;
                    end
                end
                
                default: begin
                    axi_state <= AXI_IDLE;
                end
            endcase
        end
    end
    
    // Write register task
    task write_register;
        input [ADDR_WIDTH-1:0] addr;
        input [DATA_WIDTH-1:0] data;
        input [(DATA_WIDTH/8)-1:0] strb;
        logic [31:0] word_data;
    begin
        word_data = data[31:0];
        
        case (addr[11:0])
            REG_CONTROL: begin
                if (strb[0]) reg_control[7:0] <= word_data[7:0];
                if (strb[1]) reg_control[15:8] <= word_data[15:8];
                if (strb[2]) reg_control[23:16] <= word_data[23:16];
                if (strb[3]) reg_control[31:24] <= word_data[31:24];
            end
            
            REG_MODE: begin
                if (strb[0]) reg_mode[7:0] <= word_data[7:0];
                if (strb[1]) reg_mode[15:8] <= word_data[15:8];
                if (strb[2]) reg_mode[23:16] <= word_data[23:16];
                if (strb[3]) reg_mode[31:24] <= word_data[31:24];
            end
            
            REG_KEY0: begin
                if (strb[0]) reg_key[7:0] <= word_data[7:0];
                if (strb[1]) reg_key[15:8] <= word_data[15:8];
                if (strb[2]) reg_key[23:16] <= word_data[23:16];
                if (strb[3]) reg_key[31:24] <= word_data[31:24];
            end
            
            REG_KEY1: begin
                if (strb[0]) reg_key[39:32] <= word_data[7:0];
                if (strb[1]) reg_key[47:40] <= word_data[15:8];
                if (strb[2]) reg_key[55:48] <= word_data[23:16];
                if (strb[3]) reg_key[63:56] <= word_data[31:24];
            end
            
            REG_KEY2: begin
                if (strb[0]) reg_key[71:64] <= word_data[7:0];
                if (strb[1]) reg_key[79:72] <= word_data[15:8];
                if (strb[2]) reg_key[87:80] <= word_data[23:16];
                if (strb[3]) reg_key[95:88] <= word_data[31:24];
            end
            
            REG_KEY3: begin
                if (strb[0]) reg_key[103:96] <= word_data[7:0];
                if (strb[1]) reg_key[111:104] <= word_data[15:8];
                if (strb[2]) reg_key[119:112] <= word_data[23:16];
                if (strb[3]) reg_key[127:120] <= word_data[31:24];
            end
            
            REG_COST: begin
                if (strb[0]) reg_cost[7:0] <= word_data[7:0];
                if (strb[1]) reg_cost[15:8] <= word_data[15:8];
                if (strb[2]) reg_cost[23:16] <= word_data[23:16];
                if (strb[3]) reg_cost[31:24] <= word_data[31:24];
            end
            
            REG_CONFIG: begin
                if (strb[0]) reg_config[7:0] <= word_data[7:0];
                if (strb[1]) reg_config[15:8] <= word_data[15:8];
                if (strb[2]) reg_config[23:16] <= word_data[23:16];
                if (strb[3]) reg_config[31:24] <= word_data[31:24];
            end
            
            REG_HW_ID: begin
                if (strb[0]) reg_hw_id[7:0] <= word_data[7:0];
                if (strb[1]) reg_hw_id[15:8] <= word_data[15:8];
                if (strb[2]) reg_hw_id[23:16] <= word_data[23:16];
                if (strb[3]) reg_hw_id[31:24] <= word_data[31:24];
            end
            
            REG_DATA_IN: begin
                if (strb[0]) reg_data_in[7:0] <= word_data[7:0];
                if (strb[1]) reg_data_in[15:8] <= word_data[15:8];
                if (strb[2]) reg_data_in[23:16] <= word_data[23:16];
                if (strb[3]) reg_data_in[31:24] <= word_data[31:24];
            end
            
            default: begin
                // Read-only registers or unmapped addresses
            end
        endcase
    end
    endtask
    
    // Read register function
    function logic [DATA_WIDTH-1:0] read_register;
        input [ADDR_WIDTH-1:0] addr;
        logic [DATA_WIDTH-1:0] read_data;
        logic [31:0] word_data;
    begin
        case (addr[11:0])
            REG_CONTROL: word_data = reg_control;
            REG_STATUS: word_data = reg_status;
            REG_MODE: word_data = reg_mode;
            REG_KEY0: word_data = reg_key[31:0];
            REG_KEY1: word_data = reg_key[63:32];
            REG_KEY2: word_data = reg_key[95:64];
            REG_KEY3: word_data = reg_key[127:96];
            REG_COST: word_data = reg_cost;
            REG_CONFIG: word_data = reg_config;
            REG_HW_ID: word_data = reg_hw_id[31:0];
            REG_DATA_IN: word_data = reg_data_in;
            REG_DATA_OUT: word_data = reg_data_out;
            REG_HASH0: word_data = reg_hash[31:0];
            REG_HASH1: word_data = reg_hash[63:32];
            REG_HASH2: word_data = reg_hash[95:64];
            REG_HASH3: word_data = reg_hash[127:96];
            REG_HASH4: word_data = reg_hash[159:128];
            REG_HASH5: word_data = reg_hash[191:160];
            REG_HASH6: word_data = reg_hash[223:192];
            REG_HASH7: word_data = reg_hash[255:224];
            default: word_data = 32'h0;
        endcase
        
        read_data = {32'b0, word_data};
        read_register = read_data;
    end
    endfunction
    
    // Core interface assignments
    assign core_start = reg_control[CTRL_START];
    assign core_mode = operation_mode_t'(reg_mode[2:0]);
    assign core_key = reg_key;
    assign core_cost = reg_cost;
    assign core_bio_lock = reg_config[CFG_BIO_LOCK];
    assign core_stealth = reg_config[CFG_STEALTH];
    assign core_hw_id = reg_hw_id;
    assign core_data_in = reg_data_in[7:0];
    assign core_data_valid = 1'b1; // Simplified
    
    // Update status and output registers
    always_ff @(posedge aclk or negedge aresetn) begin
        if (!aresetn) begin
            reg_status <= 32'b0;
            reg_data_out <= 32'b0;
            reg_hash <= 512'b0;
        end else begin
            reg_status[3:0] <= core_status;
            reg_status[4] <= core_done;
            
            if (core_data_ready) begin
                reg_data_out[7:0] <= core_data_out;
            end
            
            if (core_done) begin
                reg_hash <= core_hash_out;
            end
        end
    end
    
endmodule
