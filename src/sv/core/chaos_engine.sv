//==============================================================================
// NEX2426 Chaos Engine - Hardware Implementation
// Lorenz Attractor based Pseudo-Random Number Generator
//==============================================================================

`include "nex2426_pkg.svh"

module chaos_engine #(
    parameter SEED_WIDTH = 256,
    parameter STATE_WIDTH = 64
)(
    input  logic                    clk,
    input  logic                    rst_n,
    input  logic                    enable,
    input  logic [SEED_WIDTH-1:0]   seed,
    output logic                    ready,
    output logic [STATE_WIDTH-1:0]  entropy_out,
    output logic                    valid_out
);
    
    import nex2426_pkg::*;
    
    // Internal state
    lorenz_state_t state;
    logic signed [STATE_WIDTH-1:0] x, y, z;
    logic signed [STATE_WIDTH-1:0] dx, dy, dz;
    logic [2:0] step_count;
    
    // Fixed-point arithmetic parameters
    localparam int FP_BITS = 32;
    localparam real DT = 0.01;
    
    // Seed mapping to initial conditions
    logic [63:0] s0, s1, s2, s3;
    logic signed [63:0] x_start, y_start, z_start;
    
    assign s0 = seed[63:0];
    assign s1 = seed[127:64];
    assign s2 = seed[191:128];
    assign s3 = seed[255:192];
    
    // Map seed to initial conditions
    assign x_start = (signed'(s0 % 1000) * signed'(SCALE) / 100) + (signed'(SCALE) / 10);
    assign y_start = (signed'(s1 % 1000) * signed'(SCALE) / 100) + (signed'(SCALE) / 10);
    assign z_start = (signed'(s2 % 1000) * signed'(SCALE) / 100) + (signed'(SCALE) / 10);
    
    // Lorenz attractor equations
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state <= '{x: x_start, y: y_start, z: z_start};
            step_count <= 0;
            ready <= 1'b1;
            valid_out <= 1'b0;
        end else if (enable && ready) begin
            ready <= 1'b0;
            step_count <= 0;
        end else if (!ready && step_count < 5) begin
            // Calculate derivatives
            dx <= signed'(SIGMA * (state.y - state.x) * DT);
            dy <= signed'((state.x * (RHO - state.z) - state.y) * DT);
            dz <= signed'((state.x * state.y - BETA * state.z) * DT);
            
            // Update state
            state.x <= state.x + dx;
            state.y <= state.y + dy;
            state.z <= state.z + dz;
            
            step_count <= step_count + 1;
        end else if (!ready && step_count == 5) begin
            valid_out <= 1'b1;
            step_count <= step_count + 1;
        end else if (valid_out) begin
            ready <= 1'b1;
            valid_out <= 1'b0;
        end
    end
    
    // Extract entropy from state
    assign entropy_out = word_t'(state.x ^ state.y ^ state.z);
    
endmodule
