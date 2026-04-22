`include "nex2426_pkg.svh"

module nex2426_wishbone #(
    parameter ADDR_WIDTH = 32,
    parameter DATA_WIDTH = 64
)(
    // Wishbone Slave Interface
    input  logic                    wb_clk_i,
    input  logic                    wb_rst_i,
    input  logic                    wb_cyc_i,
    input  logic                    wb_stb_i,
    input  logic                    wb_we_i,
    input  logic [ADDR_WIDTH-1:0]   wb_adr_i,
    input  logic [DATA_WIDTH-1:0]   wb_dat_i,
    input  logic [(DATA_WIDTH/8)-1:0] wb_sel_i,
    output logic [DATA_WIDTH-1:0]   wb_dat_o,
    output logic                    wb_ack_o,
    output logic                    wb_err_o,
    
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
    
    // Wishbone state machine
    typedef enum logic [1:0] {
        WB_IDLE,
        WB_ACCESS,
        WB_RESPONSE
    } wb_state_t;
    
    wb_state_t wb_state;
    logic [ADDR_WIDTH-1:0] current_addr;
    logic write_enable;
    
    // Wishbone interface
    assign wb_err_o = 1'b0; // No error support for simplicity
    
    always_ff @(posedge wb_clk_i or posedge wb_rst_i) begin
        if (wb_rst_i) begin
            wb_state <= WB_IDLE;
            wb_ack_o <= 1'b0;
            wb_dat_o <= 64'b0;
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
            case (wb_state)
                WB_IDLE: begin
                    if (wb_cyc_i && wb_stb_i) begin
                        wb_state <= WB_ACCESS;
                        current_addr <= wb_adr_i;
                        write_enable <= wb_we_i;
                        
                        if (wb_we_i) begin
                            // Write operation
                            case (wb_adr_i[11:0])
                                12'h000: reg_control <= wb_dat_i[31:0];
                                12'h008: reg_mode <= wb_dat_i[31:0];
                                12'h010: reg_key[31:0] <= wb_dat_i[31:0];
                                12'h014: reg_key[63:32] <= wb_dat_i[31:0];
                                12'h018: reg_key[95:64] <= wb_dat_i[31:0];
                                12'h01C: reg_key[127:96] <= wb_dat_i[31:0];
                                12'h020: reg_cost <= wb_dat_i[31:0];
                                12'h024: reg_config <= wb_dat_i[31:0];
                                12'h028: reg_hw_id[31:0] <= wb_dat_i[31:0];
                                12'h030: reg_data_in <= wb_dat_i[31:0];
                                default: ; // Read-only registers
                            endcase
                        end
                    end
                end
                
                WB_ACCESS: begin
                    wb_state <= WB_RESPONSE;
                    wb_ack_o <= 1'b1;
                    
                    // Read operation
                    if (!wb_we_i) begin
                        case (current_addr[11:0])
                            12'h000: wb_dat_o <= reg_control;
                            12'h004: wb_dat_o <= reg_status;
                            12'h008: wb_dat_o <= reg_mode;
                            12'h010: wb_dat_o <= reg_key[31:0];
                            12'h014: wb_dat_o <= reg_key[63:32];
                            12'h018: wb_dat_o <= reg_key[95:64];
                            12'h01C: wb_dat_o <= reg_key[127:96];
                            12'h020: wb_dat_o <= reg_cost;
                            12'h024: wb_dat_o <= reg_config;
                            12'h028: wb_dat_o <= reg_hw_id[31:0];
                            12'h030: wb_dat_o <= reg_data_in;
                            12'h034: wb_dat_o <= reg_data_out;
                            default: wb_dat_o <= 64'h0;
                        endcase
                    end
                end
                
                WB_RESPONSE: begin
                    wb_state <= WB_IDLE;
                    wb_ack_o <= 1'b0;
                end
                
                default: wb_state <= WB_IDLE;
            endcase
        end
    end
    
    // Core interface assignments
    assign core_start = reg_control[0];
    assign core_mode = operation_mode_t'(reg_mode[2:0]);
    assign core_key = reg_key;
    assign core_cost = reg_cost;
    assign core_bio_lock = reg_config[0];
    assign core_stealth = reg_config[1];
    assign core_hw_id = reg_hw_id;
    assign core_data_in = reg_data_in[7:0];
    assign core_data_valid = 1'b1;
    
    // Update status and output registers
    always_ff @(posedge wb_clk_i or posedge wb_rst_i) begin
        if (wb_rst_i) begin
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
