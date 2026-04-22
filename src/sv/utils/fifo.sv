`include "nex2426_pkg.svh"

module fifo #(
    parameter DATA_WIDTH = 8,
    parameter DEPTH = 16,
    parameter ADDR_WIDTH = $clog2(DEPTH)
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    write_en,
    input  logic                    read_en,
    input  logic [DATA_WIDTH-1:0]   write_data,
    output logic [DATA_WIDTH-1:0]   read_data,
    output logic                    full,
    output logic                    empty,
    output logic [ADDR_WIDTH:0]     count
);
    
    logic [DATA_WIDTH-1:0] mem [DEPTH-1:0];
    logic [ADDR_WIDTH-1:0] write_ptr, read_ptr;
    logic [ADDR_WIDTH:0]   write_ptr_next, read_ptr_next;
    
    assign write_ptr_next = write_ptr + (write_en & ~full);
    assign read_ptr_next = read_ptr + (read_en & ~empty);
    assign count = write_ptr_next - read_ptr_next;
    assign full = (count == DEPTH);
    assign empty = (count == 0);
    
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            write_ptr <= 0;
            read_ptr <= 0;
        end else begin
            write_ptr <= write_ptr_next[ADDR_WIDTH-1:0];
            read_ptr <= read_ptr_next[ADDR_WIDTH-1:0];
        end
    end
    
    always_ff @(posedge clk) begin
        if (write_en && !full) begin
            mem[write_ptr] <= write_data;
        end
    end
    
    assign read_data = mem[read_ptr];
    
endmodule
