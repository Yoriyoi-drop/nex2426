`include "nex2426_pkg.svh"

module debounce #(
    parameter DEBOUNCE_TIME = 1000, // in clock cycles
    parameter SYNC_STAGES = 2
)(
    input  logic clk,
    input  rst_n,
    input  logic signal_in,
    output logic signal_out
);
    
    logic [SYNC_STAGES-1:0] sync_reg;
    logic [31:0] debounce_counter;
    logic stable_signal;
    logic debounced_signal;
    
    // Synchronize input
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            sync_reg <= 0;
        end else begin
            sync_reg <= {sync_reg[SYNC_STAGES-2:0], signal_in};
        end
    end
    
    // Debounce logic
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            debounce_counter <= 0;
            stable_signal <= 0;
            debounced_signal <= 0;
        end else begin
            if (sync_reg[SYNC_STAGES-1] != stable_signal) begin
                debounce_counter <= debounce_counter + 1;
                if (debounce_counter >= DEBOUNCE_TIME) begin
                    stable_signal <= sync_reg[SYNC_STAGES-1];
                    debounced_signal <= sync_reg[SYNC_STAGES-1];
                    debounce_counter <= 0;
                end
            end else begin
                debounce_counter <= 0;
            end
        end
    end
    
    assign signal_out = debounced_signal;
    
endmodule
